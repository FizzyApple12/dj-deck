use crate::{
    MIXER_CHANNELS,
    math::{beats::closest_bpm_multiple, jog::JogRPM},
    types::{
        analysis::Beat,
        audio_system::{AudioSystemEvent, DeckUpdate},
        deck::{BeatLoopAdjustMode, BeatSyncMode, ChannelState, DeckState, PlayState, PlayerState},
        timecode::{Duration, Timecode},
    },
};

const JOG_DEADBAND: f32 = 0.75;

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
    pub fn find_new_master(&mut self, exclude_channels: &[usize]) {
        for channel_number in 0..MIXER_CHANNELS {
            if !exclude_channels.contains(&channel_number)
                && let Some(potential_channel) = self.mixer_channels.get(channel_number)
                && potential_channel.is_valid_master()
            {
                self.master_channel = Some(channel_number);

                return;
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn update(
        &mut self,
        update_receiver: &mut tokio::sync::mpsc::UnboundedReceiver<DeckUpdate>,
        event_sender: &mut tokio::sync::mpsc::UnboundedSender<AudioSystemEvent>,
        start_time: Timecode,
        end_time: Timecode,
    ) -> DeckUpdateResults {
        // we need to do this before updating the deck state to make inactivity falloff
        // work
        for channel in &mut self.mixer_channels {
            channel.update_jog(start_time, end_time);
        }

        while let Ok(deck_state_update_function) = update_receiver.try_recv() {
            deck_state_update_function(self, event_sender);
        }

        if let Some(master_channel_number) = self.master_channel
            && let Some(master_channel) = self.mixer_channels.get(master_channel_number)
            && !master_channel.is_valid_master()
        {
            self.find_new_master(&[master_channel_number]);
        }

        let mut channel_updates = [None, None, None, None];
        let mut mut_channels: [Option<&mut ChannelState>; MIXER_CHANNELS] =
            self.mixer_channels.each_mut().map(Some);

        #[allow(clippy::indexing_slicing, clippy::needless_range_loop)]
        if let Some(master_channel_number) = self.master_channel {
            let master_channel = mut_channels[master_channel_number].take().unwrap();

            channel_updates[master_channel_number] =
                Some(master_channel.update_playback(true, start_time, end_time, None, None));

            let master_track_bpm: Option<f32> = master_channel.player.get_current_bpm();

            let master_track_beat_grid: Option<&[Beat]> =
                if let Some(ref track_analysis) = master_channel.player.current_track_analysis {
                    Some(&track_analysis.beat_grid)
                } else {
                    None
                };

            for (index, channel) in mut_channels.iter_mut().enumerate() {
                if let Some(channel) = channel {
                    channel_updates[index] = Some(channel.update_playback(
                        false,
                        start_time,
                        end_time,
                        master_track_bpm,
                        master_track_beat_grid,
                    ));
                }
            }
        } else {
            for (index, channel) in mut_channels.iter_mut().enumerate() {
                if let Some(channel) = channel {
                    channel_updates[index] =
                        Some(channel.update_playback(false, start_time, end_time, None, None));
                }
            }
        }

        DeckUpdateResults {
            channels: channel_updates.map(|update| update.unwrap()),
        }
    }
}

impl ChannelState {
    pub fn is_valid_master(&self) -> bool {
        self.player.is_valid_master()
    }

    pub fn update_jog(&mut self, start_time: Timecode, end_time: Timecode) {
        self.player.update_jog(start_time, end_time);
    }

    #[allow(clippy::too_many_lines)]
    pub fn update_playback(
        &mut self,
        is_master: bool,
        start_time: Timecode,
        end_time: Timecode,
        master_track_bpm: Option<f32>,
        master_beat_grid: Option<&[Beat]>,
    ) -> ChannelUpdateResults {
        ChannelUpdateResults {
            player: self.player.update_playback(
                is_master,
                start_time,
                end_time,
                master_track_bpm,
                master_beat_grid,
            ),
        }
    }
}

impl PlayerState {
    pub fn is_valid_master(&self) -> bool {
        !(self.play_state == PlayState::Stop
            || (self.reverse_enabled && !self.slip_playing)
            || self.jog_hold
            || self.jog_wait
            || !(-f32::EPSILON..=f32::EPSILON).contains(&self.jog_velocity))
    }

    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn update_jog(&mut self, _start_time: Timecode, _end_time: Timecode) {
        self.jog_velocity *= 0.9;

        if (-JOG_DEADBAND..JOG_DEADBAND).contains(&self.jog_velocity) {
            self.jog_velocity = 0.0;
        }

        if self.jog_wait && (-f32::EPSILON..=f32::EPSILON).contains(&self.jog_velocity) {
            self.jog_wait = false;
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn update_playback(
        &mut self,
        is_master: bool,
        start_time: Timecode,
        end_time: Timecode,
        master_track_bpm: Option<f32>,
        master_beat_grid: Option<&[Beat]>,
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

        // perform bpm sync if needed

        if !is_master
            && let BeatSyncMode::BPMSync = self.beat_sync
            && let Some(master_track_bpm) = master_track_bpm
            && let Some(current_source_bpm) = self.get_current_source_bpm()
        {
            let new_tempo = closest_bpm_multiple(
                master_track_bpm,
                current_source_bpm * (self.tempo_percent + 1.0).max(0.0),
            ) / current_source_bpm;

            if (self.tempo_percent - new_tempo).abs() >= 0.1 {
                self.tempo_slider_is_accurate = false;
            }

            self.tempo_percent = new_tempo;
        }

        // calculate time deltas

        let delta_time = end_time - start_time;
        let track_time_delta = self.calculate_track_time_delta(delta_time);

        // calculate track movement

        match (self.play_state, self.jog_hold || self.jog_wait) {
            (PlayState::Stop, _) => {
                let jog_time = self.jog_velocity.jog_time_offset(delta_time);

                self.time += jog_time;
            }
            (PlayState::Play, true) => {
                self.slip_playing = self.slip;

                let jog_time = self.jog_velocity.jog_time_offset(delta_time);

                self.time += jog_time;
            }
            (PlayState::Play, false) => {
                if let BeatSyncMode::BeatSync = self.beat_sync
                    && !(-f32::EPSILON..=f32::EPSILON).contains(&self.jog_velocity)
                {
                    self.beat_sync = BeatSyncMode::BPMSync;
                }

                let pitch_time = self.jog_velocity.pitch_bend_time_offset(delta_time);

                self.slip_playing = self.slip;

                if self.reverse_enabled {
                    self.time -= track_time_delta + pitch_time;
                } else if let BeatSyncMode::BeatSync = self.beat_sync
                    && let Some(ref track_analysis) = self.current_track_analysis
                    && let Some(master_beat_grid) = master_beat_grid
                {
                    perform_beat_sync_run(
                        &mut self.time,
                        track_time_delta,
                        &track_analysis.beat_grid,
                        master_beat_grid,
                    );
                } else {
                    self.time += track_time_delta + pitch_time;
                }
            }
            (PlayState::Cue, true) => {
                self.slip_playing = false;

                let jog_time = self.jog_velocity.jog_time_offset(delta_time);

                self.time += jog_time;
            }
            (PlayState::Cue, false) => {
                if let BeatSyncMode::BeatSync = self.beat_sync
                    && !(-f32::EPSILON..=f32::EPSILON).contains(&self.jog_velocity)
                {
                    self.beat_sync = BeatSyncMode::BPMSync;
                }

                let pitch_time = self.jog_velocity.pitch_bend_time_offset(delta_time);

                self.slip_playing = false;

                if self.reverse_enabled {
                    self.time -= track_time_delta + pitch_time;
                } else if let BeatSyncMode::BeatSync = self.beat_sync
                    && let Some(ref track_analysis) = self.current_track_analysis
                    && let Some(master_beat_grid) = master_beat_grid
                {
                    perform_beat_sync_run(
                        &mut self.time,
                        track_time_delta,
                        &track_analysis.beat_grid,
                        master_beat_grid,
                    );
                } else {
                    self.time += track_time_delta + pitch_time;
                }
            }
        }

        // calculate track slip movement

        if self.slip_playing {
            match (
                self.play_state,
                self.jog_hold || self.jog_wait,
                self.reverse_enabled,
            ) {
                (PlayState::Stop, _, _) | (_, true, _) | (_, _, true) => {
                    if let BeatSyncMode::BeatSync = self.beat_sync
                        && let Some(ref track_analysis) = self.current_track_analysis
                        && let Some(master_beat_grid) = master_beat_grid
                    {
                        perform_beat_sync_run(
                            &mut self.slip_time,
                            track_time_delta,
                            &track_analysis.beat_grid,
                            master_beat_grid,
                        );
                    } else {
                        self.slip_time += track_time_delta;
                    }
                }
                (PlayState::Play | PlayState::Cue, false, false) => {
                    self.slip_time = self.time;
                }
            }
        } else {
            self.slip_time = self.time;
        }

        // calculate beat loop wrapping

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

        // run touch cue playback

        let mut touch_cue_playback_times = None;

        if let Some(ref mut touch_cue_time) = self.touch_cue_time {
            let touch_cue_start_time = *touch_cue_time;

            *touch_cue_time += track_time_delta;

            touch_cue_playback_times = Some((touch_cue_start_time, *touch_cue_time));
        }

        // send results back

        PlayerUpdateResults {
            playback_frame_start_time: player_start_time,
            playback_frame_end_time: self.time,

            playback_wrap_times,

            touch_cue_playback_times,
        }
    }
}

fn perform_beat_sync_run(
    _time: &mut Timecode,
    _delta_time: Duration,
    _local_beat_grid: &[Beat],
    _master_beat_grid: &[Beat],
) {
    // self.tempo_slider_is_accurate = false;

    // let (first_beat_time, second_beat_time)

    // todo: sync beat times and tempo with closest
    // matching beats
}
