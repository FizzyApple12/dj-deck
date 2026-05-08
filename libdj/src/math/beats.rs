use crate::types::{deck::PlayerState, library::Beat, timecode::Duration};

impl PlayerState {
    pub fn get_current_beat(&self) -> Option<&Beat> {
        let (_, current_track) = self.current_track.as_ref()?;

        if current_track.beat_grid.is_empty() {
            return None;
        }

        let mut low = 0;
        let mut high = current_track.beat_grid.len() - 1;
        let mut result = current_track.beat_grid.first();

        while low <= high {
            let mid = low + (high - low) / 2;

            #[allow(clippy::indexing_slicing)]
            match current_track.beat_grid[mid].time.cmp(&self.time) {
                std::cmp::Ordering::Equal => return Some(&current_track.beat_grid[mid]),
                std::cmp::Ordering::Less => {
                    result = Some(&current_track.beat_grid[mid]);

                    low = mid + 1;
                }
                std::cmp::Ordering::Greater => {
                    if mid == 0 {
                        break;
                    }

                    high = mid - 1;
                }
            }
        }

        result
    }

    pub fn get_current_source_bpm(&self) -> Option<f32> {
        let (_, current_track) = self.current_track.as_ref()?;

        if current_track.beat_grid.is_empty() {
            return None;
        }

        if let Some(beat) = self.get_current_beat() {
            return Some(beat.bpm);
        }

        let first_beat = current_track.beat_grid.first()?;
        if self.time < first_beat.time {
            return Some(first_beat.bpm);
        }

        let last_beat = current_track.beat_grid.last()?;
        if self.time > last_beat.time {
            return Some(last_beat.bpm);
        }

        None
    }

    pub fn get_current_bpm(&self) -> Option<f32> {
        self.get_current_source_bpm().map(|bpm| {
            if self.tempo_reset {
                bpm
            } else {
                bpm * (self.tempo_percent + 1.0).max(0.0)
            }
        })
    }

    pub fn calculate_track_time_delta(&self, delta_time: Duration) -> Duration {
        if self.tempo_reset {
            delta_time
        } else {
            delta_time * (self.tempo_percent + 1.0).max(0.0)
        }
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
