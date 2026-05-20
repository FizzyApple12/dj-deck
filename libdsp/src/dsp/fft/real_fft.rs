use std::ops::{Index, IndexMut};

use num::{Complex, Float};

use crate::dsp::fft::{FFTTrait, complexMul, fft::FFT};

struct RealFFT<Sample, const HALF_FREQ_SHIFT: bool>
where
    Sample: Float,
{
    complexBuffer1: Vec<Complex<Sample>>,
    complexBuffer2: Vec<Complex<Sample>>,
    twiddlesMinusI: Vec<Complex<Sample>>,
    modifiedRotations: Vec<Complex<Sample>>,
    complexFft: FFT<Sample>,
}

impl<const HALF_FREQ_SHIFT: bool> FFTTrait<f32, Self::Sample, Self::Complex>
    for RealFFT<f32, HALF_FREQ_SHIFT>
{
    type Complex = Self::Complex;
    type Sample = Self::Sample;

    fn fast_size_above(size: usize) -> usize {
        FFT::<Self::Sample>::fast_size_above((size + 1) / 2) * 2
    }

    fn fast_size_below(size: usize) -> usize {
        FFT::<Self::Sample>::fast_size_below(size / 2) * 2
    }

    fn new(mut size: usize, fast_direction: i32) -> Self {
        if fast_direction > 0 {
            size = Self::fast_size_above(size);
        }
        if fast_direction < 0 {
            size = Self::fast_size_below(size);
        }

        let mut new = Self {
            complexBuffer1: Vec::new(),
            complexBuffer2: Vec::new(),
            twiddlesMinusI: Vec::new(),
            modifiedRotations: Vec::new(),
            complexFft: FFT::new(0, 0),
        };

        new.set_size(usize::max(size, 2));

        new
    }

    fn set_size(&mut self, size: usize) -> usize {
        self.complexBuffer1
            .resize(size / 2, Self::Complex::new(0.0, 0.0));
        self.complexBuffer2
            .resize(size / 2, Self::Complex::new(0.0, 0.0));

        let hhSize = size / 4 + 1;
        self.twiddlesMinusI
            .resize(hhSize, Self::Complex::new(0.0, 0.0));

        for i in 0..hhSize {
            let rotPhase = -2.0
                * std::f32::consts::PI
                * (if Self::modified {
                    i as f32 + 0.5
                } else {
                    i as f32
                })
                / size as f32;

            self.twiddlesMinusI[i] = Self::Complex::new(rotPhase.sin(), -rotPhase.cos());
        }

        if Self::modified {
            self.modifiedRotations
                .resize(size / 2, Self::Complex::new(0.0, 0.0));

            for i in 0..(size / 2) {
                let rotPhase = -2.0 * std::f32::consts::PI * i as f32 / size as f32;

                self.modifiedRotations[i] = Self::Complex::new(rotPhase.cos(), rotPhase.sin());
            }
        }

        self.complexFft.set_size(size / 2) * 2
    }

    fn set_fast_size_above(&mut self, size: usize) -> usize {
        self.set_size(Self::fast_size_above(size))
    }

    fn set_fast_size_below(&mut self, size: usize) -> usize {
        self.set_size(Self::fast_size_below(size))
    }

    fn size(&self) -> usize {
        self.complexFft.size() * 2
    }

    fn fft<InputBuffer, OutputBuffer>(&mut self, input: &InputBuffer, output: &mut OutputBuffer)
    where
        InputBuffer: Index<usize, Output = Self::Sample>,
        OutputBuffer: IndexMut<usize, Output = Self::Complex>,
    {
        let hSize = self.complexFft.size();

        for i in 0..hSize {
            if Self::modified {
                self.complexBuffer1[i] = complexMul::<false, Self::Sample>(
                    &Self::Complex::new(input[2 * i], input[2 * i + 1]),
                    &self.modifiedRotations[i],
                );
            } else {
                self.complexBuffer1[i] = Self::Complex::new(input[2 * i], input[2 * i + 1]);
            }
        }

        self.complexFft
            .fft(&self.complexBuffer1, &mut self.complexBuffer2);

        if !Self::modified {
            output[0] = Self::Complex::new(
                self.complexBuffer2[0].re + self.complexBuffer2[0].im,
                self.complexBuffer2[0].re - self.complexBuffer2[0].im,
            );
        }

        for i in (if Self::modified { 0 } else { 1 })..(hSize / 2 + 1) {
            let conjI = if Self::modified {
                hSize - 1 - i
            } else {
                hSize - i
            };

            let odd = (self.complexBuffer2[i] + (self.complexBuffer2[conjI]).conj()) * 0.5;
            let evenI = (self.complexBuffer2[i] - (self.complexBuffer2[conjI]).conj()) * 0.5;
            let evenRotMinusI = complexMul::<false, Self::Sample>(&evenI, &self.twiddlesMinusI[i]);

            output[i] = odd + evenRotMinusI;
            output[conjI] = (odd - evenRotMinusI).conj();
        }
    }

    fn ifft<InputBuffer, OutputBuffer>(&mut self, input: &InputBuffer, output: &mut OutputBuffer)
    where
        InputBuffer: Index<usize, Output = Self::Complex>,
        OutputBuffer: IndexMut<usize, Output = Self::Sample>,
    {
        let hSize = self.complexFft.size();

        if !Self::modified {
            self.complexBuffer1[0] =
                Self::Complex::new(input[0].re + input[0].im, input[0].re - input[0].im);
        }

        for i in (if Self::modified { 0 } else { 1 })..(hSize / 2 + 1) {
            let conjI = if Self::modified {
                hSize - 1 - i
            } else {
                hSize - i
            };
            let v = input[i];
            let v2 = input[conjI];

            let odd = v + (v2).conj();
            let evenRotMinusI = v - (v2).conj();
            let evenI = complexMul::<true, Self::Sample>(&evenRotMinusI, &self.twiddlesMinusI[i]);

            self.complexBuffer1[i] = odd + evenI;
            self.complexBuffer1[conjI] = (odd - evenI).conj();
        }

        self.complexFft
            .ifft(&self.complexBuffer1, &mut self.complexBuffer2);

        for i in 0..hSize {
            let mut v = self.complexBuffer2[i];

            if Self::modified {
                v = complexMul::<true, Self::Sample>(&v, &self.modifiedRotations[i]);
            }

            output[2 * i] = v.re;
            output[2 * i + 1] = v.im;
        }
    }
}

impl<const HALF_FREQ_SHIFT: bool> RealFFT<f32, HALF_FREQ_SHIFT> {
    type Complex = Complex<f32>;
    type Sample = f32;

    const modified: bool = HALF_FREQ_SHIFT;
}

type ModifiedRealFFT<Sample> = RealFFT<Sample, true>;
