use crate::types::{
    analysis::Beat,
    deck::PlayerState,
    timecode::{Duration, Timecode},
};

pub fn get_current_beat_index(beat_grid: &[Beat], time: Timecode) -> Option<usize> {
    if beat_grid.is_empty() {
        return None;
    }

    Some(
        beat_grid
            .partition_point(|beat| beat.time <= time)
            .saturating_sub(1),
    )
}

pub fn get_current_beat(beat_grid: &[Beat], time: Timecode) -> Option<&Beat> {
    beat_grid.get(get_current_beat_index(beat_grid, time)?)
}

#[allow(clippy::indexing_slicing)]
pub fn get_closest_beat_index(beat_grid: &[Beat], time: Timecode) -> Option<usize> {
    if beat_grid.is_empty() {
        return None;
    }

    let partition = beat_grid.partition_point(|beat| beat.time < time);

    match partition {
        0 => Some(0),
        partition if partition == beat_grid.len() => Some(partition - 1),
        partition => {
            let before = &beat_grid[partition - 1];
            let after = &beat_grid[partition];

            if time - before.time <= after.time - time {
                Some(partition - 1)
            } else {
                Some(partition)
            }
        }
    }
}

pub fn get_closest_beat(beat_grid: &[Beat], time: Timecode) -> Option<&Beat> {
    beat_grid.get(get_closest_beat_index(beat_grid, time)?)
}

impl PlayerState {
    pub fn get_current_beat(&self) -> Option<&Beat> {
        get_current_beat(&self.current_track_analysis.as_ref()?.beat_grid, self.time)
    }

    #[allow(clippy::indexing_slicing)]
    pub fn get_closest_beat(&self) -> Option<&Beat> {
        get_closest_beat(&self.current_track_analysis.as_ref()?.beat_grid, self.time)
    }

    pub fn get_current_source_bpm(&self) -> Option<f32> {
        let current_track_analysis = self.current_track_analysis.as_ref()?;

        if current_track_analysis.beat_grid.is_empty() {
            return None;
        }

        if let Some(beat) = self.get_current_beat() {
            return Some(beat.bpm);
        }

        let first_beat = current_track_analysis.beat_grid.first()?;
        if self.time < first_beat.time {
            return Some(first_beat.bpm);
        }

        let last_beat = current_track_analysis.beat_grid.last()?;
        if self.time > last_beat.time {
            return Some(last_beat.bpm);
        }

        None
    }

    pub fn get_current_bpm(&self) -> Option<f32> {
        self.get_current_source_bpm()
            .map(|bpm| bpm * self.tempo_percent.max(0.0))
    }

    pub fn calculate_track_time_delta(&self, delta_time: Duration) -> Duration {
        delta_time * self.tempo_percent.max(0.0)
    }
}

pub fn closest_bpm_multiple(source_bpm: f32, destination_bpm: f32) -> f32 {
    // todo: check all bpm multiples
    let half = f32::abs((source_bpm / 2.0) - destination_bpm);
    let full = f32::abs(source_bpm - destination_bpm);
    let double = f32::abs((source_bpm * 2.0) - destination_bpm);

    if full <= half && full <= double {
        source_bpm
    } else if half < full && half < double {
        source_bpm / 2.0
    } else if double < full && double < half {
        source_bpm * 2.0
    } else {
        source_bpm
    }
}
