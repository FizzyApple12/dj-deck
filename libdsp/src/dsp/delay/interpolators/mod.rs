use num::Float;

pub mod cubic;
pub mod kaiser_sinc;
pub mod lagrange;
pub mod linear;
pub mod nearest;

// significant translation differences are present here because the original
// implementation had an infinitely sized struct

pub trait InterpolatorTrait<Sample>
where
    Sample: Float,
{
    const INPUT_LENGTH: usize;
    const LATENCY: Sample;

    fn fractional(&self, data: &[Sample], fractional: Sample) -> Sample;
}

// Efficient Algorithms and Structures for Fractional Delay Filtering Based on
// Lagrange Interpolation Franck 2009 https://www.aes.org/e-lib/browse.cfm?elib=14647

// pub trait ProductRangeTrait<Data, Sample, const n: usize>
// where
//     Sample: Float,
// {
//     fn calculate_result_range<const LOW: usize, const HIGH: usize>(
//         &self,
//         extra_factor: Sample,
//         data: Data,
//         inv_factors: &[Sample; n + 1],
//     ) -> Sample;

//     fn calculate_result_index<const INDEX: usize>(
//         &self,
//         extra_factor: Sample,
//         data: Data,
//         inv_factors: &[Sample; n + 1],
//     ) -> Sample;
// }

// pub struct ProductRange<Data, Sample, const N: usize> {
//     phantom_data: PhantomData<Data>,
//     x: Sample,
// }

// impl<Data, const N: usize> ProductRange<Data, f32, N> {
//     pub fn new(x: f32) -> Self {
//         ProductRange::<Data, f32, N> {
//             phantom_data: PhantomData,
//             x,
//         }
//     }
// }

// impl<const N: usize> ProductRangeTrait<&[f32], f32, N> for
// ProductRange<&[f32], f32, N> {     fn calculate_result_range<const LOW:
// usize, const HIGH: usize>(         &self,
//         extra_factor: f32,
//         data: &[f32],
//         inv_factors: &[f32; N + 1],
//     ) -> f32 {
//         const MID: usize = (LOW + HIGH) / 2;

//         //totals
//         if LOW == HIGH {
//             // total: x - low as f32
//         } else {
//             // left.total + right.total
//         }

//         let right_total = 0.0; // todo: right.total
//         let left = if LOW < MID {
//             let right_total = 0.0; // todo: right.total

//             self.calculate_result_range::<LOW, { MID }>(
//                 extra_factor * right_total,
//                 data,
//                 inv_factors,
//             )
//         } else {
//             let right_total = 0.0; // todo: right.total

//             self.calculate_result_index::<LOW>(extra_factor * right_total,
// data, inv_factors)         };

//         let left_total = 0.0; // todo: left.total
//         let right = if HIGH > MID + 1 {
//             self.calculate_result_range::<{ MID + 1 }, HIGH>(
//                 extra_factor * left_total,
//                 data,
//                 inv_factors,
//             )
//         } else {
//             self.calculate_result_index::<HIGH>(extra_factor * left_total,
// data, inv_factors)         };

//         left + right
//     }

//     fn calculate_result_index<const INDEX: usize>(
//         &self,
//         extra_factor: f32,
//         data: &[f32],
//         inv_factors: &[f32; N + 1],
//     ) -> f32 {
//         extra_factor * data[INDEX] * inv_factors[INDEX]
//     }
// }
