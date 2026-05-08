use crate::{
    MIXER_CHANNELS,
    math::{beats::closest_bpm_multiple, jog::JogRPM},
    types::{
        deck::{
            BeatLoopAdjustMode, BeatSyncMode, ChannelState, DeckState, DeckUpdate, JogState,
            PlayState, PlayerState,
        },
        timecode::{Duration, Timecode},
    },
};

pub struct DeckUpdateResults {
    pub channels: [ChannelUpdateResults; MIXER_CHANNELS],
}

pub struct ChannelUpdateResults {
    pub player: PlayerUpdateResults,
}

pub struct PlayerUpdateResults {
    pub playback_frame_start_time: Timecode,
    pub playback_frame_end_time: Timecode,

    pub playback_wrap_times: Option<(Timecode, Timecode)>,

    pub touch_cue_playback_times: Option<(Timecode, Timecode)>,
}

impl DeckState {
    #[allow(clippy::too_many_lines)]
    pub fn update(
        &mut self,
        update_receiver: &mut tokio::sync::mpsc::UnboundedReceiver<DeckUpdate>,
        start_time: Timecode,
        end_time: Timecode,
    ) -> DeckUpdateResults {
        while let Ok(deck_state_update_function) = update_receiver.try_recv() {
            deck_state_update_function(self);
        }

        let master_track_bpm: Option<f32> = if let Some(master_channel_number) = self.master_channel
            && let Some(master_channel) = self.mixer_channels.get(master_channel_number)
            && let Some(master_track_bpm) = master_channel.player.get_current_bpm()
        {
            Some(master_track_bpm)
        } else {
            None
        };

        let mut channel_updates =
            self.mixer_channels
                .iter_mut()
                .enumerate()
                .map(|(channel_number, mixer_channel)| {
                    let is_master = if let Some(master_channel_number) = self.master_channel {
                        master_channel_number == channel_number
                    } else {
                        false
                    };

                    mixer_channel.update(is_master, start_time, end_time, master_track_bpm)
                });

        DeckUpdateResults {
            channels: [
                channel_updates.next().expect("channel 0 update"),
                channel_updates.next().expect("channel 1 update"),
                channel_updates.next().expect("channel 2 update"),
                channel_updates.next().expect("channel 3 update"),
            ],
        }
    }
}

impl ChannelState {
    #[allow(clippy::too_many_lines)]
    pub fn update(
        &mut self,
        is_master: bool,
        start_time: Timecode,
        end_time: Timecode,
        master_track_bpm: Option<f32>,
    ) -> ChannelUpdateResults {
        ChannelUpdateResults {
            player: self
                .player
                .update(is_master, start_time, end_time, master_track_bpm),
        }
    }
}

impl PlayerState {
    #[allow(clippy::too_many_lines)]
    pub fn update(
        &mut self,
        is_master: bool,
        start_time: Timecode,
        end_time: Timecode,
        master_track_bpm: Option<f32>,
    ) -> PlayerUpdateResults {
        let Some(ref _current_track) = self.current_track else {
            self.time = Timecode::zero();
            self.cue_time = None;
            self.touch_cue_time = None;

            self.slip_playing = false;
            self.slip_time = Timecode::zero();

            self.beat_loop_start = None;
            self.beat_loop_end = None;
            self.last_beat_loop = None;
            self.beat_loop_adjust_mode = BeatLoopAdjustMode::None;

            self.keyshift = 0.0;

            return PlayerUpdateResults {
                playback_frame_start_time: Timecode::zero(),
                playback_frame_end_time: Timecode::zero(),

                playback_wrap_times: None,

                touch_cue_playback_times: None,
            };
        };

        let player_start_time = self.time;

        if !is_master
            && let Some(master_track_bpm) = master_track_bpm
            && let Some(current_source_bpm) = self.get_current_source_bpm()
        {
            match self.beat_sync {
                BeatSyncMode::Off => {}
                BeatSyncMode::BPMSync => {
                    self.tempo_slider_is_accurate = false;

                    self.tempo_percent = closest_bpm_multiple(
                        master_track_bpm,
                        current_source_bpm * (self.tempo_percent + 1.0).max(0.0),
                    ) / current_source_bpm;
                }
                BeatSyncMode::BeatSync => {
                    self.tempo_slider_is_accurate = false;

                    // todo: sync beat times and tempo with closest
                    // matching beats
                }
            }
        }

        let delta_time = end_time - start_time;

        let track_time_delta = self.calculate_track_time_delta(delta_time);

        let mut touch_cue_playback_times = None;

        if let Some(ref mut touch_cue_time) = self.touch_cue_time {
            let touch_cue_start_time = *touch_cue_time;

            *touch_cue_time += track_time_delta;

            touch_cue_playback_times = Some((touch_cue_start_time, *touch_cue_time));
        }

        match (self.play_state, self.jog_state) {
            (PlayState::Stop, JogState::Released) => {}
            (_, JogState::Jog(rpm)) | (PlayState::Stop, JogState::PitchBend(rpm)) => {
                let jog_time = rpm.jog_time_offset(delta_time);

                self.time += jog_time;
            }
            (PlayState::Play, jog_mode @ (JogState::Released | JogState::PitchBend(_))) => {
                let pitch_time = match jog_mode {
                    JogState::PitchBend(rpm) => rpm.pitch_bend_time_offset(delta_time),
                    _ => Duration::zero(),
                };

                self.slip_playing = self.slip;

                if self.reverse_enabled {
                    self.time -= track_time_delta + pitch_time;
                } else {
                    self.time += track_time_delta + pitch_time;
                }
            }
            (PlayState::Cue, jog_mode @ (JogState::Released | JogState::PitchBend(_))) => {
                let pitch_time = match jog_mode {
                    JogState::PitchBend(rpm) => rpm.pitch_bend_time_offset(delta_time),
                    _ => Duration::zero(),
                };

                self.slip_playing = false;

                if self.reverse_enabled {
                    self.time -= track_time_delta + pitch_time;
                } else {
                    self.time += track_time_delta + pitch_time;
                }
            }
        }

        if self.slip_playing {
            match (self.play_state, self.jog_state) {
                (PlayState::Stop, _) | (_, JogState::Jog(_)) => {
                    self.slip_time += track_time_delta;
                }
                (PlayState::Play | PlayState::Cue, JogState::Released | JogState::PitchBend(_)) => {
                    self.slip_time = self.time;
                }
            }
        } else {
            self.slip_time = self.time;
        }

        let mut playback_wrap_times = None;

        if let Some(beat_loop_start) = self.beat_loop_start
            && let Some(beat_loop_end) = self.beat_loop_end
        {
            while self.time < beat_loop_start {
                self.time += beat_loop_end - beat_loop_start;

                if playback_wrap_times.is_none() {
                    playback_wrap_times = Some((beat_loop_start, beat_loop_end));
                }
            }
            while self.time > beat_loop_end {
                self.time -= beat_loop_end - beat_loop_start;

                if playback_wrap_times.is_none() {
                    playback_wrap_times = Some((beat_loop_end, beat_loop_start));
                }
            }
        }

        PlayerUpdateResults {
            playback_frame_start_time: player_start_time,
            playback_frame_end_time: self.time,

            playback_wrap_times,

            touch_cue_playback_times,
        }
    }
}
