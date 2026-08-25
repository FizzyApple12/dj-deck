use libdsp::pipeline::CrossFaderSide;

pub trait SingleFader {
    #[must_use]
    fn fade(self, fade_percent: f32) -> Self;
}

impl SingleFader for f32 {
    fn fade(self, fade_percent: f32) -> f32 {
        self * fade_percent
    }
}

pub trait CrossFader {
    #[must_use]
    fn crossfade(self, fade_percent: f32, fader_side: CrossFaderSide) -> Self;
}

impl CrossFader for f32 {
    fn crossfade(self, fade_percent: f32, fader_side: CrossFaderSide) -> f32 {
        match fader_side {
            CrossFaderSide::A => self * (1.0 - fade_percent).clamp(0.0, 1.0),
            CrossFaderSide::B => self * (1.0 + fade_percent).clamp(0.0, 1.0),
            CrossFaderSide::None => self,
        }
    }
}
