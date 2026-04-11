use std::time::Duration;

const VINYL_RPM: f32 = 33.33333;

pub trait JogRPM {
    #[must_use]
    fn pitch_bend_time_offset(self, delta_time: Duration) -> Self;

    #[must_use]
    fn jog_time_offset(self, delta_time: Duration) -> Self;
}

impl JogRPM for f32 {
    fn pitch_bend_time_offset(self, delta_time: Duration) -> f32 {
        // todo: how tf is this actually calculated?
        // this is just a best guess from some testing
        (self / 10.0) * delta_time.as_secs_f32() * (60.0 / VINYL_RPM)
    }

    fn jog_time_offset(self, delta_time: Duration) -> f32 {
        (self / 60.0) * delta_time.as_secs_f32() * (60.0 / VINYL_RPM)
    }
}
