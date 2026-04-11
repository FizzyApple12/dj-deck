use crate::{MIXER_MAX_FREQUENCY, MIXER_MIN_FREQUENCY};

pub trait LowPassFilter {
    #[must_use]
    fn frequency(self) -> Self;
}

impl LowPassFilter for f32 {
    fn frequency(self) -> f32 {
        let curve = self.abs().clamp(0.0, 1.0).powf(2.0);

        MIXER_MIN_FREQUENCY * curve + MIXER_MAX_FREQUENCY * (1.0 - curve)
    }
}

pub trait HighPassFilter {
    #[must_use]
    fn frequency(self) -> Self;
}

impl HighPassFilter for f32 {
    fn frequency(self) -> f32 {
        let curve = self.abs().clamp(0.0, 1.0).powf(2.0);

        MIXER_MIN_FREQUENCY * (1.0 - curve) + MIXER_MAX_FREQUENCY * curve
    }
}
