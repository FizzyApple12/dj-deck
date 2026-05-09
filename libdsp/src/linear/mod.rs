use num::{Complex, Float};

pub mod fft;

#[allow(clippy::transmute_ptr_to_ptr, clippy::undocumented_unsafe_blocks)]
pub fn complex_to_two_float<F>(complex: &[Complex<F>]) -> (&[F], &[F])
where
    F: Float,
{
    let original_len = complex.len();

    unsafe { std::mem::transmute::<&[Complex<F>], &[F]>(complex) }.split_at(original_len)
}

#[allow(clippy::transmute_ptr_to_ptr, clippy::undocumented_unsafe_blocks)]
pub fn complex_to_two_float_mut<F>(complex: &mut [Complex<F>]) -> (&mut [F], &mut [F])
where
    F: Float,
{
    let original_len = complex.len();

    unsafe { std::mem::transmute::<&mut [Complex<F>], &mut [F]>(complex) }
        .split_at_mut(original_len)
}
