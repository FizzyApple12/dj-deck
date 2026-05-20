use std::marker::PhantomData;

use num::Float;

use crate::dsp::delay::interpolators::InterpolatorTrait;

/// Spline cubic interpolator
pub struct InterpolatorCubic<Data, Sample>
where
    Sample: Float,
{
    phantom_data: PhantomData<Data>,
    phantom_sample: PhantomData<Sample>,
}

impl<Data, Sample> Default for InterpolatorCubic<Data, Sample>
where
    Sample: Float,
{
    fn default() -> Self {
        Self {
            phantom_data: PhantomData,
            phantom_sample: PhantomData,
        }
    }
}

impl InterpolatorTrait<&[f32], f32> for InterpolatorCubic<&[f32], f32> {
    const INPUT_LENGTH: usize = 4;
    const LATENCY: f32 = 1.0;

    #[allow(clippy::indexing_slicing)]
    fn fractional(&self, data: &[f32], fractional: f32) -> f32 {
        // Cubic interpolation
        let a = data[0];
        let b = data[1];
        let c = data[2];
        let d = data[3];

        let cb_diff = c - b;

        let k1 = (c - a) * 0.5;
        let k3 = k1 + (d - b) * 0.5 - cb_diff * 2.0;
        let k2 = cb_diff - k3 - k1;

        b + fractional * (k1 + fractional * (k2 + fractional * k3)) // 16 ops total, not including the indexing
    }
}
