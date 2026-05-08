#[allow(clippy::cast_possible_truncation)]
pub fn nanoseconds_to_samples(nanoseconds: i64, sample_rate: u32) -> i64 {
    ((i128::from(nanoseconds) * i128::from(sample_rate)) / 1_000_000_000i128) as i64
}

#[allow(clippy::cast_possible_truncation)]
pub fn samples_to_nanoseconds(samples: i64, sample_rate: u32) -> i64 {
    ((i128::from(samples) * 1_000_000_000i128) / i128::from(sample_rate)) as i64
}
