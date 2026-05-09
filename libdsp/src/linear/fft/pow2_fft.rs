use num::{Float, complex::Complex};

use crate::linear::fft::simple_fft::{SimpleFFT, SimpleFFTTrait};

// A power-of-2 only FFT, specialised with platform-specific fast
pub struct Pow2FFT<Sample, const SPLIT_COMPUTATION: bool>
where
    Sample: Float,
{
    tmp: Vec<Complex<Sample>>,
    simple_fft: SimpleFFT<Sample>,
}

pub trait Pow2FFTTrait<Sample, const SPLIT_COMPUTATION: bool>
where
    Sample: Float,
{
    const PREFERS_SPLIT: bool;

    fn new(size: usize) -> Self;

    fn resize(&mut self, size: usize);

    fn fft(&mut self, time: &[Complex<f32>], freq: &mut [Complex<f32>]);
    fn ifft(&mut self, freq: &[Complex<f32>], time: &mut [Complex<f32>]);
    fn fft_split_complex(
        &mut self,
        in_r: &[Sample],
        in_i: &[Sample],
        out_r: &mut [Sample],
        out_i: &mut [Sample],
    );
    fn ifft_split_complex(
        &mut self,
        in_r: &[Sample],
        in_i: &[Sample],
        out_r: &mut [Sample],
        out_i: &mut [Sample],
    );
}

impl<const SPLIT_COMPUTATION: bool> Pow2FFTTrait<f32, SPLIT_COMPUTATION>
    for Pow2FFT<f32, SPLIT_COMPUTATION>
{
    // whether this FFT implementation is faster when given split-complex inputs
    const PREFERS_SPLIT: bool = true;

    fn new(size: usize) -> Self {
        let mut new = Self {
            tmp: Vec::new(),
            simple_fft: SimpleFFT::<f32>::new(size),
        };

        new.resize(size);

        new
    }

    fn resize(&mut self, size: usize) {
        self.simple_fft.resize(size);

        self.tmp.resize(size, Complex { re: 0.0, im: 0.0 });
    }

    fn fft(&mut self, time: &[Complex<f32>], freq: &mut [Complex<f32>]) {
        self.simple_fft.fft(time, freq);
    }

    fn ifft(&mut self, freq: &[Complex<f32>], time: &mut [Complex<f32>]) {
        self.simple_fft.ifft(freq, time);
    }

    fn fft_split_complex(
        &mut self,
        in_r: &[f32],
        in_i: &[f32],
        out_r: &mut [f32],
        out_i: &mut [f32],
    ) {
        self.simple_fft.fft_split_complex(in_r, in_i, out_r, out_i);
    }

    fn ifft_split_complex(
        &mut self,
        in_r: &[f32],
        in_i: &[f32],
        out_r: &mut [f32],
        out_i: &mut [f32],
    ) {
        self.simple_fft.ifft_split_complex(in_r, in_i, out_r, out_i);
    }
}
