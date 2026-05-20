use std::marker::PhantomData;

use num::Float;

use crate::dsp::delay::interpolators::InterpolatorTrait;

/// Linear interpolator
pub struct InterpolatorLinear<Data, Sample>
where
    Sample: Float,
{
    phantom_data: PhantomData<Data>,
    phantom_sample: PhantomData<Sample>,
}

impl<Data, Sample> Default for InterpolatorLinear<Data, Sample>
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

impl InterpolatorTrait<&[f32], f32> for InterpolatorLinear<&[f32], f32> {
    const INPUT_LENGTH: usize = 2;
    const LATENCY: f32 = 0.0;

    #[allow(clippy::indexing_slicing)]
    fn fractional(&self, data: &[f32], fractional: f32) -> f32 {
        let a = data[0];
        let b = data[1];
        a + fractional * (b - a)
    }
}
