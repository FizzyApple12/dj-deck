use num::{Float, complex::Complex};

use crate::linear::{
    complex_to_two_float_mut,
    fft::pow2_fft::{Pow2FFT, Pow2FFTTrait},
};

pub struct SimpleRealFFT<Sample, const SPLIT_COMPUTATION: bool>
where
    Sample: Float + Default,
{
    complex_fft: Pow2FFT<Sample, SPLIT_COMPUTATION>,
    tmp_time: Vec<Complex<Sample>>,
    tmp_freq: Vec<Complex<Sample>>,
}

pub trait SimpleRealFFTTrait<Sample, const SPLIT_COMPUTATION: bool>
where
    Sample: Float + Default,
{
    type Complex;
    type Sample;

    const PREFERS_SPLIT: bool;

    fn new(size: usize) -> Self;

    fn resize(&mut self, size: usize);

    fn fft(&mut self, time: &[Self::Sample], freq: &mut [Self::Complex]);
    fn ifft(&mut self, freq: &[Self::Complex], time: &mut [Self::Sample]);
    fn fft_split_complex(
        &mut self,
        in_r: &[Self::Sample],
        out_r: &mut [Self::Sample],
        out_i: &mut [Self::Sample],
    );
    fn ifft_split_complex(
        &mut self,
        in_r: &[Self::Sample],
        in_i: &[Self::Sample],
        out_r: &mut [Self::Sample],
    );
}

// Wraps a complex FFT into a real one
impl<const SPLIT_COMPUTATION: bool> SimpleRealFFTTrait<f32, SPLIT_COMPUTATION>
    for SimpleRealFFT<f32, SPLIT_COMPUTATION>
{
    type Complex = Complex<f32>;
    type Sample = f32;

    const PREFERS_SPLIT: bool = Pow2FFT::<f32, SPLIT_COMPUTATION>::PREFERS_SPLIT;

    fn new(size: usize) -> Self {
        let mut new = Self {
            complex_fft: Pow2FFT::<f32, SPLIT_COMPUTATION>::new(size),
            tmp_time: Vec::new(),
            tmp_freq: Vec::new(),
        };

        new.resize(size);

        new
    }

    fn resize(&mut self, size: usize) {
        self.complex_fft.resize(size);

        self.tmp_time
            .resize(size, Complex::<f32> { re: 0.0, im: 0.0 });
        self.tmp_freq
            .resize(size, Complex::<f32> { re: 0.0, im: 0.0 });
    }

    #[allow(
        clippy::indexing_slicing,
        clippy::manual_memcpy,
        clippy::needless_range_loop
    )]
    fn fft(&mut self, time: &[f32], freq: &mut [Complex<f32>]) {
        for i in 0..self.tmp_time.len() {
            self.tmp_time[i] = Complex::<f32>::from(time[i]);
        }

        self.complex_fft.fft(&self.tmp_time, &mut self.tmp_freq);

        for i in 0..(self.tmp_freq.len() / 2) {
            freq[i] = self.tmp_freq[i];
        }

        freq[0] = Complex::<f32>::new(
            self.tmp_freq[0].re,
            self.tmp_freq[self.tmp_freq.len() / 2].re,
        );
    }

    #[allow(clippy::indexing_slicing, clippy::needless_range_loop)]
    fn ifft(&mut self, freq: &[Complex<f32>], time: &mut [f32]) {
        self.tmp_freq[0] = Complex::<f32>::from(freq[0].re);

        let temp_freq_len = self.tmp_freq.len();

        self.tmp_freq[temp_freq_len / 2] = Complex::<f32>::from(freq[0].im);

        for i in 1..(self.tmp_freq.len() / 2) {
            self.tmp_freq[i] = freq[i];
            self.tmp_freq[temp_freq_len - i] = freq[i].conj();
        }

        self.complex_fft.ifft(&self.tmp_freq, &mut self.tmp_time);

        for i in 0..self.tmp_time.len() {
            time[i] = self.tmp_time[i].re;
        }
    }

    #[allow(clippy::indexing_slicing, clippy::manual_memcpy)]
    fn fft_split_complex(&mut self, in_r: &[f32], out_r: &mut [f32], out_i: &mut [f32]) {
        let tmp_freq_len = self.tmp_freq.len();

        let (tmp_freq_r, tmp_freq_i) = complex_to_two_float_mut(&mut self.tmp_freq);

        let (tmp_time, _) = complex_to_two_float_mut(&mut self.tmp_time);

        for i in 0..(tmp_time.len() / 2) {
            tmp_time[i] = 0.0;
        }

        self.complex_fft
            .fft_split_complex(in_r, tmp_time, tmp_freq_r, tmp_freq_i);

        for i in 0..(self.tmp_time.len() / 2) {
            out_r[i] = tmp_freq_r[i];
            out_i[i] = tmp_freq_i[i];
        }

        out_i[0] = tmp_freq_r[tmp_freq_len / 2];
    }

    #[allow(clippy::indexing_slicing)]
    fn ifft_split_complex(&mut self, in_r: &[f32], in_i: &[f32], out_r: &mut [f32]) {
        let tmp_freq_len = self.tmp_freq.len();

        let (tmp_freq_r, tmp_freq_i) = complex_to_two_float_mut(&mut self.tmp_freq);

        let (tmp_time, _) = complex_to_two_float_mut(&mut self.tmp_time);

        tmp_freq_r[0] = in_r[0];
        tmp_freq_r[tmp_freq_len / 2] = in_i[0];

        tmp_freq_i[0] = 0.0;
        tmp_freq_i[tmp_freq_len / 2] = 0.0;

        for i in 1..(tmp_freq_len / 2) {
            tmp_freq_r[i] = in_r[i];
            tmp_freq_i[i] = in_i[i];

            tmp_freq_r[tmp_freq_len - i] = in_r[i];
            tmp_freq_i[tmp_freq_len - i] = -in_i[i];
        }

        self.complex_fft
            .ifft_split_complex(tmp_freq_r, tmp_freq_i, out_r, tmp_time);
    }
}
