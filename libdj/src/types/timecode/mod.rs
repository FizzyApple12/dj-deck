use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize)]
pub struct Timecode {
    pub nanoseconds: i64,
}

impl Timecode {
    pub const fn zero() -> Timecode {
        Timecode { nanoseconds: 0 }
    }

    pub const fn from_seconds(seconds: i64) -> Timecode {
        Timecode {
            nanoseconds: seconds * 1_000_000_000,
        }
    }

    pub const fn from_milliseconds(milliseconds: i64) -> Timecode {
        Timecode {
            nanoseconds: milliseconds * 1_000_000,
        }
    }

    pub const fn from_microseconds(microseconds: i64) -> Timecode {
        Timecode {
            nanoseconds: microseconds * 1_000,
        }
    }

    pub const fn from_nanoseconds(nanoseconds: i64) -> Timecode {
        Timecode { nanoseconds }
    }

    pub fn to_nanoseconds(self) -> i64 {
        self.nanoseconds
    }
}

impl PartialEq for Timecode {
    fn eq(&self, other: &Self) -> bool {
        self.nanoseconds == other.nanoseconds
    }
}

impl Eq for Timecode {}

impl PartialOrd for Timecode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Timecode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.nanoseconds.cmp(&other.nanoseconds)
    }
}

impl Add<Duration> for Timecode {
    type Output = Timecode;

    fn add(self, rhs: Duration) -> Self::Output {
        Timecode {
            nanoseconds: self.nanoseconds + rhs.nanoseconds,
        }
    }
}

impl AddAssign<Duration> for Timecode {
    fn add_assign(&mut self, rhs: Duration) {
        self.nanoseconds += rhs.nanoseconds;
    }
}

impl Sub for Timecode {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        Duration {
            nanoseconds: self.nanoseconds - rhs.nanoseconds,
        }
    }
}

impl Sub<Duration> for Timecode {
    type Output = Timecode;

    fn sub(self, rhs: Duration) -> Self::Output {
        Timecode {
            nanoseconds: self.nanoseconds - rhs.nanoseconds,
        }
    }
}

impl SubAssign<Duration> for Timecode {
    fn sub_assign(&mut self, rhs: Duration) {
        self.nanoseconds -= rhs.nanoseconds;
    }
}

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize)]
pub struct Duration {
    pub nanoseconds: i64,
}

impl Duration {
    pub const fn zero() -> Duration {
        Duration { nanoseconds: 0 }
    }

    pub const fn from_seconds(seconds: i64) -> Duration {
        Duration {
            nanoseconds: seconds * 1_000_000_000,
        }
    }

    pub const fn from_milliseconds(milliseconds: i64) -> Duration {
        Duration {
            nanoseconds: milliseconds * 1_000_000,
        }
    }

    pub const fn from_microseconds(microseconds: i64) -> Duration {
        Duration {
            nanoseconds: microseconds * 1_000,
        }
    }

    pub const fn from_nanoseconds(nanoseconds: i64) -> Duration {
        Duration { nanoseconds }
    }

    pub fn to_nanoseconds(self) -> i64 {
        self.nanoseconds
    }
}

impl PartialEq for Duration {
    fn eq(&self, other: &Self) -> bool {
        self.nanoseconds == other.nanoseconds
    }
}

impl Eq for Duration {}

impl PartialOrd for Duration {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Duration {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.nanoseconds.cmp(&other.nanoseconds)
    }
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Duration) -> Self::Output {
        Duration {
            nanoseconds: self.nanoseconds + rhs.nanoseconds,
        }
    }
}

impl Sub for Duration {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        Duration {
            nanoseconds: self.nanoseconds - rhs.nanoseconds,
        }
    }
}

impl Mul<f32> for Duration {
    type Output = Duration;

    fn mul(self, rhs: f32) -> Self::Output {
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        Duration {
            nanoseconds: ((self.nanoseconds as f32) * rhs) as i64,
        }
    }
}

impl Mul<Duration> for f32 {
    type Output = Duration;

    fn mul(self, rhs: Duration) -> Self::Output {
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        Duration {
            nanoseconds: (self * (rhs.nanoseconds as f32)) as i64,
        }
    }
}

impl Div<f32> for Duration {
    type Output = Duration;

    fn div(self, rhs: f32) -> Self::Output {
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        Duration {
            nanoseconds: ((self.nanoseconds as f32) / rhs) as i64,
        }
    }
}

impl Div<Duration> for f32 {
    type Output = Duration;

    fn div(self, rhs: Duration) -> Self::Output {
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        Duration {
            nanoseconds: (self / (rhs.nanoseconds as f32)) as i64,
        }
    }
}
