use std::ops::MulAssign;

use num::Float;

pub mod approximate_confined_gaussian;
pub mod kaiser;

#[allow(clippy::indexing_slicing)]
pub fn force_perfect_reconstruction<Sample>(
    data: &mut [Sample],
    window_length: usize,
    interval: usize,
) where
    Sample: Float + MulAssign,
    f64: From<Sample>,
{
    for i in 0..interval {
        let mut sum2 = 0.0;

        for index in (i..window_length).step_by(interval) {
            sum2 += f64::from(data[index]) * f64::from(data[index]);
        }

        let factor = 1.0 / sum2.sqrt();

        for index in (i..window_length).step_by(interval) {
            data[index] *= Sample::from(factor).unwrap();
        }
    }
}
