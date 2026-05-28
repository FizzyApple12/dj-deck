// use std::marker::PhantomData;

// use num::Float;

// use crate::dsp::delay::interpolators::InterpolatorTrait;

// /// Nearest-neighbour interpolator
// pub struct InterpolatorNearest<Data, Sample>
// where
//     Sample: Float,
// {
//     phantom_data: PhantomData<Data>,
//     phantom_sample: PhantomData<Sample>,
// }

// impl<Data, Sample> Default for InterpolatorNearest<Data, Sample>
// where
//     Sample: Float,
// {
//     fn default() -> Self {
//         Self {
//             phantom_data: PhantomData,
//             phantom_sample: PhantomData,
//         }
//     }
// }

// impl InterpolatorTrait<&[f32], f32> for InterpolatorNearest<&[f32], f32> {
//     const INPUT_LENGTH: usize = 1;
//     // Because we're truncating, which rounds down too often
//     const LATENCY: f32 = -0.5;

//     #[allow(clippy::indexing_slicing)]
//     fn fractional(&self, data: &[f32], _: f32) -> f32 {
//         data[0]
//     }
// }
