use bytemuck::{Pod, cast_slice, cast_slice_mut};
use num::{Complex, Float};

pub mod fft;
pub mod stft;

pub fn complex_to_two_float<F>(complex: &[Complex<F>]) -> (&[F], &[F])
where
    F: Float + Pod,
{
    let original_len = complex.len();
    cast_slice::<Complex<F>, F>(complex).split_at(original_len)
}

pub fn complex_to_two_float_mut<F>(complex: &mut [Complex<F>]) -> (&mut [F], &mut [F])
where
    F: Float + Pod,
{
    let original_len = complex.len();
    cast_slice_mut::<Complex<F>, F>(complex).split_at_mut(original_len)
}
