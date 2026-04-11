use std::time::Duration;

use crate::{
    math::{beats::closest_bpm_multiple, jog::JogRPM},
    types::deck::{BeatLoopAdjustMode, BeatSyncMode, DeckState, DeckUpdate, JogState, PlayState},
};

impl DeckState {
    #[allow(clippy::too_many_lines)]
    pub fn update(
        &mut self,
        update_receiver: &mut tokio::sync::mpsc::UnboundedReceiver<DeckUpdate>,
        delta_time: Duration,
    ) {
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

        for (number, mixer_channel) in self.mixer_channels.iter_mut().enumerate() {
            let player = &mut mixer_channel.player;

            let Some(ref _current_track) = player.current_track else {
                player.time = 0.0;
                player.cue_time = None;
                player.touch_cue_time = None;

                player.slip_playing = false;
                player.slip_time = 0.0;

                player.beat_loop_start = None;
                player.beat_loop_end = None;
                player.last_beat_loop = None;
                player.beat_loop_adjust_mode = BeatLoopAdjustMode::None;

                player.keyshift = 0.0;

                continue;
            };

            if let Some(master_channel_number) = self.master_channel
                && master_channel_number != number
                && let Some(master_track_bpm) = master_track_bpm
                && let Some(current_source_bpm) = player.get_current_source_bpm()
            {
                match player.beat_sync {
                    BeatSyncMode::Off => {}
                    BeatSyncMode::BPMSync => {
                        player.tempo_slider_is_accurate = false;

                        player.tempo_percent = closest_bpm_multiple(
                            master_track_bpm,
                            current_source_bpm * (player.tempo_percent + 1.0).max(0.0),
                        ) / current_source_bpm;
                    }
                    BeatSyncMode::BeatSync => {
                        player.tempo_slider_is_accurate = false;

                        // todo: sync beat times and tempo with closest matching
                        // beats
                    }
                }
            }

            let track_time_delta = player.calculate_track_time_delta(delta_time);

            match (player.play_state, player.jog_state) {
                (PlayState::Stop, JogState::Released) => {}
                (_, JogState::Jog(rpm)) | (PlayState::Stop, JogState::PitchBend(rpm)) => {
                    let jog_time = rpm.jog_time_offset(delta_time);

                    player.time += jog_time;
                }
                (PlayState::Play, jog_mode @ (JogState::Released | JogState::PitchBend(_))) => {
                    let pitch_time = match jog_mode {
                        JogState::PitchBend(rpm) => rpm.pitch_bend_time_offset(delta_time),
                        _ => 0.0,
                    };

                    player.slip_playing = player.slip;

                    if player.reverse_enabled {
                        player.time -= track_time_delta + pitch_time;
                    } else {
                        player.time += track_time_delta + pitch_time;
                    }
                }
                (PlayState::Cue, jog_mode @ (JogState::Released | JogState::PitchBend(_))) => {
                    let pitch_time = match jog_mode {
                        JogState::PitchBend(rpm) => rpm.pitch_bend_time_offset(delta_time),
                        _ => 0.0,
                    };

                    player.slip_playing = false;

                    if player.reverse_enabled {
                        player.time -= track_time_delta + pitch_time;
                    } else {
                        player.time += track_time_delta + pitch_time;
                    }
                }
            }

            if player.slip_playing {
                match (player.play_state, player.jog_state) {
                    (PlayState::Stop, _) | (_, JogState::Jog(_)) => {
                        player.slip_time += track_time_delta;
                    }
                    (
                        PlayState::Play | PlayState::Cue,
                        JogState::Released | JogState::PitchBend(_),
                    ) => {
                        player.slip_time = player.time;
                    }
                }
            } else {
                player.slip_time = player.time;
            }

            if let Some(beat_loop_start) = player.beat_loop_start
                && let Some(beat_loop_end) = player.beat_loop_end
            {
                while player.time < beat_loop_start {
                    player.time += beat_loop_end - beat_loop_start;
                }
                while player.time > beat_loop_end {
                    player.time -= beat_loop_end - beat_loop_start;
                }
            }

            if let Some(ref mut touch_cue_time) = player.touch_cue_time {
                *touch_cue_time += track_time_delta;
            }
        }
    }
}
