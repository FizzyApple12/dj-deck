use crate::types::timecode::Duration;

const VINYL_RPM: f32 = 33.33333;

pub trait JogRPM {
    #[must_use]
    fn pitch_bend_time_offset(self, delta_time: Duration) -> Duration;

    #[must_use]
    fn jog_time_offset(self, delta_time: Duration) -> Duration;
}

impl JogRPM for f32 {
    fn pitch_bend_time_offset(self, delta_time: Duration) -> Duration {
        // todo: how tf is this actually calculated?
        // this is just a best guess from some testing
        (self / 400.0) * (VINYL_RPM / 60.0) * delta_time
    }

    fn jog_time_offset(self, delta_time: Duration) -> Duration {
        // todo: how tf is this actually calculated?
        // this is just a best guess from some testing
        (self / VINYL_RPM) * (VINYL_RPM / 60.0) * delta_time
    }
}
