use std::ops::{Index, IndexMut};

use num::{Complex, Float};

use crate::dsp::fft::{FFTTrait, complex_mul, fft::FFT};

pub struct RealFFT<Sample, const HALF_FREQ_SHIFT: bool>
where
    Sample: Float,
{
    comple_buffer_1: Vec<Complex<Sample>>,
    complex_buffer_2: Vec<Complex<Sample>>,
    twiddles_minus_i: Vec<Complex<Sample>>,
    modified_rotations: Vec<Complex<Sample>>,
    complex_fft: FFT<Sample>,
}

impl<const HALF_FREQ_SHIFT: bool> FFTTrait<f32, f32, Complex<f32>>
    for RealFFT<f32, HALF_FREQ_SHIFT>
{
    type Complex = Complex<f32>;
    type Sample = f32;

    fn fast_size_above(size: usize) -> usize {
        FFT::<f32>::fast_size_above(size.div_ceil(2)) * 2
    }

    fn fast_size_below(size: usize) -> usize {
        FFT::<f32>::fast_size_below(size / 2) * 2
    }

    fn new(mut size: usize, fast_direction: i32) -> Self {
        if fast_direction > 0 {
            size = Self::fast_size_above(size);
        }
        if fast_direction < 0 {
            size = Self::fast_size_below(size);
        }

        let mut new = Self {
            comple_buffer_1: Vec::new(),
            complex_buffer_2: Vec::new(),
            twiddles_minus_i: Vec::new(),
            modified_rotations: Vec::new(),
            complex_fft: FFT::new(0, 0),
        };

        new.set_size(usize::max(size, 2));

        new
    }

    #[allow(clippy::cast_precision_loss, clippy::indexing_slicing)]
    fn set_size(&mut self, size: usize) -> usize {
        self.comple_buffer_1
            .resize(size / 2, Complex::<f32>::new(0.0, 0.0));
        self.complex_buffer_2
            .resize(size / 2, Complex::<f32>::new(0.0, 0.0));

        let h_h_ize = size / 4 + 1;
        self.twiddles_minus_i
            .resize(h_h_ize, Complex::<f32>::new(0.0, 0.0));

        for i in 0..h_h_ize {
            let rot_phase = -2.0
                * std::f32::consts::PI
                * (if Self::MODIFIED {
                    i as f32 + 0.5
                } else {
                    i as f32
                })
                / size as f32;

            self.twiddles_minus_i[i] = Complex::<f32>::new(rot_phase.sin(), -rot_phase.cos());
        }

        if Self::MODIFIED {
            self.modified_rotations
                .resize(size / 2, Complex::<f32>::new(0.0, 0.0));

            for i in 0..(size / 2) {
                let rot_phase = -2.0 * std::f32::consts::PI * i as f32 / size as f32;

                self.modified_rotations[i] = Complex::<f32>::new(rot_phase.cos(), rot_phase.sin());
            }
        }

        self.complex_fft.set_size(size / 2) * 2
    }

    fn set_fast_size_above(&mut self, size: usize) -> usize {
        self.set_size(Self::fast_size_above(size))
    }

    fn set_fast_size_below(&mut self, size: usize) -> usize {
        self.set_size(Self::fast_size_below(size))
    }

    fn size(&self) -> usize {
        self.complex_fft.size() * 2
    }

    #[allow(clippy::indexing_slicing, clippy::bool_to_int_with_if)]
    fn fft<InputBuffer, OutputBuffer>(&mut self, input: &InputBuffer, output: &mut OutputBuffer)
    where
        InputBuffer: Index<usize, Output = f32>,
        OutputBuffer: IndexMut<usize, Output = Complex<f32>>,
    {
        let h_size = self.complex_fft.size();

        for i in 0..h_size {
            if Self::MODIFIED {
                self.comple_buffer_1[i] = complex_mul::<false, f32>(
                    &Complex::<f32>::new(input[2 * i], input[2 * i + 1]),
                    &self.modified_rotations[i],
                );
            } else {
                self.comple_buffer_1[i] = Complex::<f32>::new(input[2 * i], input[2 * i + 1]);
            }
        }

        self.complex_fft
            .fft(&self.comple_buffer_1, &mut self.complex_buffer_2);

        if !Self::MODIFIED {
            output[0] = Complex::<f32>::new(
                self.complex_buffer_2[0].re + self.complex_buffer_2[0].im,
                self.complex_buffer_2[0].re - self.complex_buffer_2[0].im,
            );
        }

        for i in (if Self::MODIFIED { 0 } else { 1 })..=(h_size / 2) {
            let conj_i = if Self::MODIFIED {
                h_size - 1 - i
            } else {
                h_size - i
            };

            let odd = (self.complex_buffer_2[i] + (self.complex_buffer_2[conj_i]).conj()) * 0.5;
            let even_i = (self.complex_buffer_2[i] - (self.complex_buffer_2[conj_i]).conj()) * 0.5;
            let even_rot_minus_i = complex_mul::<false, f32>(&even_i, &self.twiddles_minus_i[i]);

            output[i] = odd + even_rot_minus_i;
            output[conj_i] = (odd - even_rot_minus_i).conj();
        }
    }

    #[allow(clippy::indexing_slicing, clippy::bool_to_int_with_if)]
    fn ifft<InputBuffer, OutputBuffer>(&mut self, input: &InputBuffer, output: &mut OutputBuffer)
    where
        InputBuffer: Index<usize, Output = Complex<f32>>,
        OutputBuffer: IndexMut<usize, Output = f32>,
    {
        let h_size = self.complex_fft.size();

        if !Self::MODIFIED {
            self.comple_buffer_1[0] =
                Complex::<f32>::new(input[0].re + input[0].im, input[0].re - input[0].im);
        }

        for i in (if Self::MODIFIED { 0 } else { 1 })..=(h_size / 2) {
            let conj_i = if Self::MODIFIED {
                h_size - 1 - i
            } else {
                h_size - i
            };
            let v = input[i];
            let v2 = input[conj_i];

            let odd = v + (v2).conj();
            let even_rot_minus_i = v - (v2).conj();
            let even_i = complex_mul::<true, f32>(&even_rot_minus_i, &self.twiddles_minus_i[i]);

            self.comple_buffer_1[i] = odd + even_i;
            self.comple_buffer_1[conj_i] = (odd - even_i).conj();
        }

        self.complex_fft
            .ifft(&self.comple_buffer_1, &mut self.complex_buffer_2);

        for i in 0..h_size {
            let mut v = self.complex_buffer_2[i];

            if Self::MODIFIED {
                v = complex_mul::<true, f32>(&v, &self.modified_rotations[i]);
            }

            output[2 * i] = v.re;
            output[2 * i + 1] = v.im;
        }
    }
}

impl<const HALF_FREQ_SHIFT: bool> RealFFT<f32, HALF_FREQ_SHIFT> {
    const MODIFIED: bool = HALF_FREQ_SHIFT;
}

pub type ModifiedRealFFT<Sample> = RealFFT<Sample, true>;
