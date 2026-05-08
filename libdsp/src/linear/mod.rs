use num::{Complex, Float};

pub mod fft;

pub fn complex_to_two_float<F>(complex: &mut [Complex<F>]) -> (&mut [F], &mut [F])
where
    F: Float,
{
    let original_len = complex.len();

    unsafe { std::mem::transmute::<&mut [Complex<F>], &mut [F]>(complex) }
        .split_at_mut(original_len)
}
