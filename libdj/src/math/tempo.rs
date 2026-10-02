use crate::types::deck::{PlayerState, TempoRange};

impl PlayerState {
    pub fn get_actual_slider_tempo(&self) -> f32 {
        1.0 + self.tempo_slider_position
            * match self.tempo_range {
                TempoRange::SixPercent => 0.06,
                TempoRange::TenPercent => 0.10,
                TempoRange::SixteenPercent => 0.16,
                TempoRange::OneHundredPercent => 1.0,
            }
    }
}
