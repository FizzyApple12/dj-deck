// #![feature(stdarch_arm_feature_detection)]
#![feature(sort_floats)]
#![feature(trait_alias)]
#![feature(slice_shift)]

// this library heavily inspired by:
// https://github.com/Signalsmith-Audio/linear
// https://signalsmith-audio.co.uk/code/dsp/
// https://signalsmith-audio.co.uk/code/stretch/
// thank you signalsmith for your excellent work, you've made great reference
// material

pub mod audio_loader;
pub mod dsp;
pub mod linear;
pub mod pipeline;
pub mod stretch;
pub mod timecode;

pub fn db_to_amplitude(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

pub fn amplitude_to_db(amplitude: f32) -> f32 {
    20.0 * amplitude.log10()
}

#[allow(clippy::cast_precision_loss)]
pub fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let mean_sq = samples.iter().map(|&s| s * s).sum::<f32>() / samples.len() as f32;

    mean_sq.sqrt()
}

#[allow(clippy::cast_possible_truncation)]
pub fn nanoseconds_to_samples(nanoseconds: i64, sample_rate: u32) -> i64 {
    ((i128::from(nanoseconds) * i128::from(sample_rate)) / 1_000_000_000i128) as i64
}

#[allow(clippy::cast_possible_truncation)]
pub fn samples_to_nanoseconds(samples: i64, sample_rate: u32) -> i64 {
    ((i128::from(samples) * 1_000_000_000i128) / i128::from(sample_rate)) as i64
}
