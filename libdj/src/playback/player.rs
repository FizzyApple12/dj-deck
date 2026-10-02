use timecode::{Duration, Timecode};

use crate::{
    JOG_DEADBAND,
    math::{
        beats::{closest_bpm_multiple, get_closest_beat_index, get_current_beat_index},
        jog::JogRPM,
    },
    types::{
        analysis::Beat,
        deck::{BeatLoopAdjustMode, BeatSyncMode, PlayState, PlayerState},
    },
};

pub struct PlayerUpdateResults {
    pub playback_frame_start_time: Timecode,
    pub playback_frame_end_time: Timecode,

    pub playback_wrap_times: Option<(Timecode, Timecode)>,

    pub touch_cue_playback_times: Option<(Timecode, Timecode)>,
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

    pub fn update_playback(
        &mut self,
        is_master: bool,
        start_time: Timecode,
        end_time: Timecode,
        master_track_bpm: Option<f32>,
        master_beat_sync_data: Option<(&[Beat], Timecode, f32)>,
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
            && self.beat_sync == BeatSyncMode::BPMSync
            && let Some(master_track_bpm) = master_track_bpm
            && let Some(current_source_bpm) = self.get_current_source_bpm()
        {
            let new_tempo = closest_bpm_multiple(
                master_track_bpm,
                current_source_bpm * self.tempo_percent.max(0.0),
            ) / current_source_bpm;

            if (self.tempo_percent - new_tempo).abs() >= 0.01 {
                self.tempo_slider_is_accurate = false;
            }

            self.tempo_percent = new_tempo;
        }

        // run tempo reset check

        if self.tempo_reset && (self.beat_sync == BeatSyncMode::Off || is_master) {
            if self.tempo_percent.abs() >= 0.01 {
                self.tempo_slider_is_accurate = false;
            }

            self.tempo_percent = 1.0;
        }

        // tempo slider sync and accuracy check

        let actual_slider_tempo = self.get_actual_slider_tempo();

        if self.tempo_slider_is_accurate {
            if self.beat_sync == BeatSyncMode::Off || is_master {
                self.tempo_percent = actual_slider_tempo;
            }
        } else if (self.tempo_percent - actual_slider_tempo).abs() < 0.01 {
            self.tempo_slider_is_accurate = true;
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
                if self.beat_sync == BeatSyncMode::BeatSync
                    && !(-f32::EPSILON..=f32::EPSILON).contains(&self.jog_velocity)
                {
                    self.beat_sync = BeatSyncMode::BPMSync;
                }

                let pitch_time = self.jog_velocity.pitch_bend_time_offset(delta_time);

                self.slip_playing = self.slip;

                if self.reverse_enabled {
                    self.time -= track_time_delta + pitch_time;
                } else if self.beat_sync == BeatSyncMode::BeatSync
                    && let Some(ref track_analysis) = self.current_track_analysis
                    && let Some((master_beat_grid, master_time, master_tempo_percent)) =
                        master_beat_sync_data
                {
                    if perform_beat_sync_run(
                        track_time_delta,
                        &mut self.time,
                        &mut self.tempo_percent,
                        &track_analysis.beat_grid,
                        master_time,
                        master_tempo_percent,
                        master_beat_grid,
                    ) {
                        self.tempo_slider_is_accurate = false;
                    }
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
                if self.beat_sync == BeatSyncMode::BeatSync
                    && !(-f32::EPSILON..=f32::EPSILON).contains(&self.jog_velocity)
                {
                    self.beat_sync = BeatSyncMode::BPMSync;
                }

                let pitch_time = self.jog_velocity.pitch_bend_time_offset(delta_time);

                self.slip_playing = false;

                if self.reverse_enabled {
                    self.time -= track_time_delta + pitch_time;
                } else if self.beat_sync == BeatSyncMode::BeatSync
                    && let Some(ref track_analysis) = self.current_track_analysis
                    && let Some((master_beat_grid, master_time, master_tempo_percent)) =
                        master_beat_sync_data
                {
                    if perform_beat_sync_run(
                        track_time_delta,
                        &mut self.time,
                        &mut self.tempo_percent,
                        &track_analysis.beat_grid,
                        master_time,
                        master_tempo_percent,
                        master_beat_grid,
                    ) {
                        self.tempo_slider_is_accurate = false;
                    }
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
                    if self.beat_sync == BeatSyncMode::BeatSync
                        && let Some(ref track_analysis) = self.current_track_analysis
                        && let Some((master_beat_grid, master_time, master_tempo_percent)) =
                            master_beat_sync_data
                    {
                        if perform_beat_sync_run(
                            track_time_delta,
                            &mut self.slip_time,
                            &mut self.tempo_percent,
                            &track_analysis.beat_grid,
                            master_time,
                            master_tempo_percent,
                            master_beat_grid,
                        ) {
                            self.tempo_slider_is_accurate = false;
                        }
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

#[allow(
    clippy::indexing_slicing,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]
fn perform_beat_sync_run(
    delta_time: Duration,
    local_time: &mut Timecode,
    local_tempo_percent: &mut f32,
    local_beat_grid: &[Beat],
    master_time: Timecode,
    master_tempo_percent: f32,
    master_beat_grid: &[Beat],
) -> bool {
    *local_time += delta_time;

    if (-f32::EPSILON..=f32::EPSILON).contains(&master_tempo_percent) {
        return false;
    }

    let closest_local_beat_index = get_current_beat_index(local_beat_grid, *local_time);

    let last_local_beat_index = local_beat_grid.len() - 1;

    let (local_first_beat, local_second_beat) = match closest_local_beat_index {
        None => {
            return false;
        }
        Some(closest_local_beat_index) if closest_local_beat_index == last_local_beat_index => (
            local_beat_grid[last_local_beat_index - 1],
            local_beat_grid[last_local_beat_index],
        ),
        Some(closest_local_beat_index) => (
            local_beat_grid[closest_local_beat_index],
            local_beat_grid[closest_local_beat_index + 1],
        ),
    };

    let first_closest_local_beat_index = get_closest_beat_index(
        master_beat_grid,
        map_timestamp(
            local_first_beat.time,
            *local_time,
            *local_tempo_percent,
            master_time,
            master_tempo_percent,
        ),
    );

    let second_closest_local_beat_index = get_closest_beat_index(
        master_beat_grid,
        map_timestamp(
            local_second_beat.time,
            *local_time,
            *local_tempo_percent,
            master_time,
            master_tempo_percent,
        ),
    );

    let last_master_beat_index = master_beat_grid.len() - 1;

    let (master_first_beat, master_second_beat) = match (
        first_closest_local_beat_index,
        second_closest_local_beat_index,
    ) {
        (None, _) | (_, None) => {
            return false;
        }
        (Some(first_closest_local_beat_index), Some(second_closest_local_beat_index))
            if first_closest_local_beat_index == second_closest_local_beat_index =>
        {
            if first_closest_local_beat_index == last_master_beat_index {
                (
                    master_beat_grid[first_closest_local_beat_index - 1],
                    master_beat_grid[first_closest_local_beat_index],
                )
            } else {
                (
                    master_beat_grid[first_closest_local_beat_index],
                    master_beat_grid[first_closest_local_beat_index + 1],
                )
            }
        }
        (Some(first_closest_local_beat_index), Some(second_closest_local_beat_index)) => (
            master_beat_grid[first_closest_local_beat_index],
            master_beat_grid[second_closest_local_beat_index],
        ),
    };

    let precise_tempo_percent = (local_second_beat.time.nanoseconds
        - local_first_beat.time.nanoseconds) as f64
        / (master_second_beat.time.nanoseconds - master_first_beat.time.nanoseconds) as f64;

    let new_tempo_percent = precise_tempo_percent as f32;

    let invalidate_tempo_slider = (*local_tempo_percent - new_tempo_percent).abs() >= 0.01;

    *local_tempo_percent = new_tempo_percent;

    if (-f32::EPSILON..=f32::EPSILON).contains(&new_tempo_percent) {
        return true;
    }

    // local_time.nanoseconds = -(normalised_master_first_beat_time
    //     - (local_first_beat.time.nanoseconds as f64 * precise_tempo_percent))
    //     as i64;

    local_time.nanoseconds = (local_first_beat.time.nanoseconds as f64
        - ((master_first_beat.time.nanoseconds - master_time.nanoseconds) as f64
            * (precise_tempo_percent / f64::from(master_tempo_percent))))
        as i64;

    invalidate_tempo_slider
}

fn map_timestamp(
    time: Timecode,
    from_time: Timecode,
    from_tempo_percent: f32,
    to_time: Timecode,
    to_tempo_percent: f32,
) -> Timecode {
    to_time + ((time - from_time) * (to_tempo_percent / from_tempo_percent))
}
