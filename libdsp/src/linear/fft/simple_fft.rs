use num::{Float, complex::Complex};

use crate::linear::complex_to_two_float_mut;

// Fairly simple and very portable power-of-2 FFT
pub struct SimpleFFT<Sample>
where
    Sample: Float,
{
    twiddles: Vec<Complex<Sample>>,
    working: Vec<Complex<Sample>>,
}

pub trait SimpleFFTTrait<Sample>
where
    Sample: Float,
{
    type Complex;
    type Sample;

    fn new(size: usize) -> Self;

    fn resize(&mut self, size: usize);

    fn fft(&mut self, time: &[Self::Complex], freq: &mut [Self::Complex]);
    fn fft_split_complex(
        &mut self,
        in_r: &[Self::Sample],
        in_i: &[Self::Sample],
        out_r: &mut [Self::Sample],
        out_i: &mut [Self::Sample],
    );
    fn ifft(&mut self, freq: &[Self::Complex], time: &mut [Self::Complex]);
    fn ifft_split_complex(
        &mut self,
        in_r: &[Self::Sample],
        in_i: &[Self::Sample],
        out_r: &mut [Self::Sample],
        out_i: &mut [Self::Sample],
    );
}

impl SimpleFFTTrait<f32> for SimpleFFT<f32> {
    type Complex = Self::Complex;
    type Sample = Self::Sample;

    fn new(size: usize) -> Self {
        let mut new = Self {
            twiddles: Vec::new(),
            working: Vec::new(),
        };

        new.resize(size);

        new
    }

    #[allow(clippy::indexing_slicing, clippy::cast_precision_loss)]
    fn resize(&mut self, size: usize) {
        self.twiddles
            .resize(size * 3 / 4, Self::Complex { re: 0.0, im: 0.0 });

        for i in 0..(size * 3 / 4) {
            self.twiddles[i] = Self::Complex::from_polar(
                1.0,
                -2.0 * std::f32::consts::PI * i as Self::Sample / size as Self::Sample,
            );
        }

        self.working
            .resize(size, Self::Complex { re: 0.0, im: 0.0 });
    }

    #[allow(clippy::indexing_slicing, clippy::undocumented_unsafe_blocks)]
    fn fft(&mut self, time: &[Self::Complex], freq: &mut [Self::Complex]) {
        let size = self.working.len();

        if size < 1 {
            return;
        } else if size == 1 {
            freq[0] = time[0];

            return;
        }

        let working_pointer = &raw mut self.working;

        self.fft_pass::<false>(size, 1, time, freq, unsafe { &mut *working_pointer });
    }

    #[allow(clippy::indexing_slicing, clippy::undocumented_unsafe_blocks)]
    fn fft_split_complex(
        &mut self,
        in_r: &[Self::Sample],
        in_i: &[Self::Sample],
        out_r: &mut [Self::Sample],
        out_i: &mut [Self::Sample],
    ) {
        let size = self.working.len();

        if size < 1 {
            return;
        } else if size == 1 {
            out_r[0] = in_r[0];
            out_i[0] = in_i[0];

            return;
        }

        let working_pointer = &raw mut self.working;

        let (working_r, working_i) = complex_to_two_float_mut(unsafe { &mut *working_pointer });

        self.fft_pass_split_complex::<false>(
            size, 1, in_r, in_i, out_r, out_i, working_r, working_i,
        );
    }

    #[allow(clippy::indexing_slicing, clippy::undocumented_unsafe_blocks)]
    fn ifft(&mut self, freq: &[Self::Complex], time: &mut [Self::Complex]) {
        let size = self.working.len();

        if size < 1 {
            return;
        } else if size == 1 {
            time[0] = freq[0];

            return;
        }

        let working_pointer = &raw mut self.working;

        self.fft_pass::<true>(size, 1, freq, time, unsafe { &mut *working_pointer });
    }

    #[allow(clippy::indexing_slicing, clippy::undocumented_unsafe_blocks)]
    fn ifft_split_complex(
        &mut self,
        in_r: &[Self::Sample],
        in_i: &[Self::Sample],
        out_r: &mut [Self::Sample],
        out_i: &mut [Self::Sample],
    ) {
        let size = self.working.len();

        if size < 1 {
            return;
        } else if size == 1 {
            out_r[0] = in_r[0];
            out_i[0] = in_i[0];

            return;
        }

        let working_pointer = &raw mut self.working;

        let (working_r, working_i) = complex_to_two_float_mut(unsafe { &mut *working_pointer });

        self.fft_pass_split_complex::<false>(
            size, 1, in_r, in_i, out_r, out_i, working_r, working_i,
        );
    }
}

impl SimpleFFT<f32> {
    pub type Complex = Complex<f32>;
    pub type Sample = f32;

    fn mul<const CONJ_B: bool>(a: Self::Complex, b: Self::Complex) -> Self::Complex {
        if CONJ_B {
            Self::Complex::new(a.re * b.re + a.im * b.im, a.im * b.re - a.re * b.im)
        } else {
            Self::Complex::new(a.re * b.re - a.im * b.im, a.im * b.re + a.re * b.im)
        }
    }

    #[allow(clippy::indexing_slicing)]
    fn fft_pass<const INVERSE: bool>(
        &self,
        size: usize,
        stride: usize,
        input: &[Self::Complex],
        output: &mut [Self::Complex],
        working: &mut [Self::Complex],
    ) {
        if size / 4 > 1 {
            // Calculate four quarter-size FFTs
            self.fft_pass::<INVERSE>(size / 4, stride * 4, input, working, output);
            self.combine4::<INVERSE>(size, stride, working, output);
        } else if size == 4 {
            self.combine4::<INVERSE>(4, stride, input, output);
        } else {
            // 2-point FFT
            for s in 0..stride {
                let a = input[s];
                let b = input[s + stride];

                output[s] = a + b;
                output[s + stride] = a - b;
            }
        }
    }

    // Combine interleaved results into a single spectrum
    #[allow(clippy::indexing_slicing)]
    fn combine4<const INVERSE: bool>(
        &self,
        size: usize,
        stride: usize,
        input: &[Self::Complex],
        output: &mut [Self::Complex],
    ) {
        let twiddle_step = self.working.len() / size;

        for i in 0..(size / 4) {
            let twiddle_b = self.twiddles[i * twiddle_step];
            let twiddle_c = self.twiddles[i * 2 * twiddle_step];
            let twiddle_d = self.twiddles[i * 3 * twiddle_step];

            for s in 0..stride {
                let a = input[4 * i * stride + s];
                let b = Self::mul::<INVERSE>(input[(4 * i + 1) * stride + s], twiddle_b);
                let c = Self::mul::<INVERSE>(input[(4 * i + 2) * stride + s], twiddle_c);
                let d = Self::mul::<INVERSE>(input[(4 * i + 3) * stride + s], twiddle_d);

                let ac0 = a + c;
                let ac1 = a - c;
                let bd0 = b + d;
                let bd1 = if INVERSE { b - d } else { d - b };

                let bd1i = Self::Complex::new(-bd1.im, bd1.re);

                output[i * stride + s] = ac0 + bd0;
                output[(i + size / 4) * stride + s] = ac1 + bd1i;
                output[(i + size / 4 * 2) * stride + s] = ac0 - bd0;
                output[(i + size / 4 * 3) * stride + s] = ac1 - bd1i;
            }
        }
    }

    // The same thing, but translated for split-complex input/output
    #[allow(clippy::indexing_slicing, clippy::too_many_arguments)]
    fn fft_pass_split_complex<const INVERSE: bool>(
        &self,
        size: usize,
        stride: usize,
        input_r: &[Self::Sample],
        input_i: &[Self::Sample],
        output_r: &mut [Self::Sample],
        output_i: &mut [Self::Sample],
        working_r: &mut [Self::Sample],
        working_i: &mut [Self::Sample],
    ) {
        if size / 4 > 1 {
            // Calculate four quarter-size FFTs
            self.fft_pass_split_complex::<INVERSE>(
                size / 4,
                stride * 4,
                input_r,
                input_i,
                working_r,
                working_i,
                output_r,
                output_i,
            );
            self.combine4_split_complex::<INVERSE>(
                size, stride, working_r, working_i, output_r, output_i,
            );
        } else if size == 4 {
            self.combine4_split_complex::<INVERSE>(4, stride, input_r, input_i, output_r, output_i);
        } else {
            // 2-point FFT
            for s in 0..stride {
                let ar = input_r[s];
                let ai = input_i[s];
                let br = input_r[s + stride];
                let bi = input_i[s + stride];

                output_r[s] = ar + br;
                output_i[s] = ai + bi;
                output_r[s + stride] = ar - br;
                output_i[s + stride] = ai - bi;
            }
        }
    }

    // Combine interleaved results into a single spectrum
    #[allow(clippy::indexing_slicing)]
    fn combine4_split_complex<const INVERSE: bool>(
        &self,
        size: usize,
        stride: usize,
        input_r: &[Self::Sample],
        input_i: &[Self::Sample],
        output_r: &mut [Self::Sample],
        output_i: &mut [Self::Sample],
    ) {
        let twiddle_step = self.working.len() / size;

        for i in 0..(size / 4) {
            let twiddle_b = self.twiddles[i * twiddle_step];
            let twiddle_c = self.twiddles[i * 2 * twiddle_step];
            let twiddle_d = self.twiddles[i * 3 * twiddle_step];

            for s in 0..stride {
                let a =
                    Self::Complex::new(input_r[4 * i * stride + s], input_i[4 * i * stride + s]);
                let b = Self::mul::<INVERSE>(
                    Self::Complex::new(
                        input_r[(4 * i + 1) * stride + s],
                        input_i[(4 * i + 1) * stride + s],
                    ),
                    twiddle_b,
                );
                let c = Self::mul::<INVERSE>(
                    Self::Complex::new(
                        input_r[(4 * i + 2) * stride + s],
                        input_i[(4 * i + 2) * stride + s],
                    ),
                    twiddle_c,
                );
                let d = Self::mul::<INVERSE>(
                    Self::Complex::new(
                        input_r[(4 * i + 3) * stride + s],
                        input_i[(4 * i + 3) * stride + s],
                    ),
                    twiddle_d,
                );

                let ac0 = a + c;
                let ac1 = a - c;
                let bd0 = b + d;
                let bd1 = if INVERSE { b - d } else { d - b };

                let bd1i = Self::Complex::new(-bd1.im, bd1.re);

                output_r[i * stride + s] = ac0.re + bd0.re;
                output_i[i * stride + s] = ac0.im + bd0.im;
                output_r[(i + size / 4) * stride + s] = ac1.re + bd1i.re;
                output_i[(i + size / 4) * stride + s] = ac1.im + bd1i.im;
                output_r[(i + size / 4 * 2) * stride + s] = ac0.re - bd0.re;
                output_i[(i + size / 4 * 2) * stride + s] = ac0.im - bd0.im;
                output_r[(i + size / 4 * 3) * stride + s] = ac1.re - bd1i.re;
                output_i[(i + size / 4 * 3) * stride + s] = ac1.im - bd1i.im;
            }
        }
    }
}
