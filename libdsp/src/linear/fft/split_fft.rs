use num::{Float, complex::Complex};

use crate::linear::{
    complex_to_two_float_mut,
    fft::{
        complex_mul, complex_mul_conj, complex_mul_conj_split_complex, complex_mul_split_complex,
        interleave_copy, interleave_copy_const_astride, interleave_copy_split_complex,
        pow2_fft::{Pow2FFT, Pow2FFTTrait},
    },
};

#[derive(Clone, Copy, PartialEq)]
enum StepType {
    Passthrough,
    InterleaveOrder2,
    InterleaveOrder3,
    InterleaveOrder4,
    InterleaveOrder5,
    InterleaveOrderN,
    FirstFFT,
    MiddleFFT,
    Twiddles,
    FinalOrder2,
    FinalOrder3,
    FinalOrder4,
    FinalOrder5,
    FinalOrderN,
}

#[derive(Clone, Copy, PartialEq)]
struct Step {
    step_type: StepType,
    offset: usize,
}

// An FFT which can handle multiples of 3 and 5, and can be computed in chunks
pub struct SplitFFT<Sample, const SPLIT_COMPUTATION: bool>
where
    Sample: Float,
{
    inner_fft: Pow2FFT<Sample, SPLIT_COMPUTATION>,

    inner_size: usize,
    outer_size: usize,
    tmp_freq: Vec<Complex<Sample>>,
    outer_twiddles: Vec<Complex<Sample>>,
    outer_twiddles_r: Vec<Sample>,
    outer_twiddles_i: Vec<Sample>,
    dft_twists: Vec<Complex<Sample>>,
    dft_tmp: Vec<Complex<Sample>>,
    plan: Vec<Step>,
}

pub trait SplitFFTTrait<Sample, const SPLIT_COMPUTATION: bool>
where
    Sample: Float + Default,
{
    type Complex;
    type Sample;

    const MAX_SPLIT: usize;
    const MIN_INNER_SIZE: usize;
    const PREFERS_SPLIT: bool;

    fn fast_size_above(size: usize) -> usize;

    fn new(size: usize) -> Self;

    fn resize(&mut self, size: usize);

    fn size(&self) -> usize;
    fn steps(&self) -> usize;

    fn fft(&mut self, time: &[Self::Complex], freq: &mut [Self::Complex]);
    fn fft_step(&mut self, step: usize, time: &[Self::Complex], freq: &mut [Self::Complex]);
    fn fft_split_complex(
        &mut self,
        in_r: &[Self::Sample],
        in_i: &[Self::Sample],
        out_r: &mut [Self::Sample],
        out_i: &mut [Self::Sample],
    );
    fn fft_step_split_complex(
        &mut self,
        step: usize,
        in_r: &[Self::Sample],
        in_i: &[Self::Sample],
        out_r: &mut [Self::Sample],
        out_i: &mut [Self::Sample],
    );

    fn ifft(&mut self, time: &[Self::Complex], freq: &mut [Self::Complex]);
    fn ifft_step(&mut self, step: usize, time: &[Self::Complex], freq: &mut [Self::Complex]);
    fn ifft_split_complex(
        &mut self,
        in_r: &[Self::Sample],
        in_i: &[Self::Sample],
        out_r: &mut [Self::Sample],
        out_i: &mut [Self::Sample],
    );
    fn ifft_step_split_complex(
        &mut self,
        step: usize,
        in_r: &[Self::Sample],
        in_i: &[Self::Sample],
        out_r: &mut [Self::Sample],
        out_i: &mut [Self::Sample],
    );
}

impl<const SPLIT_COMPUTATION: bool> SplitFFTTrait<f32, SPLIT_COMPUTATION>
    for SplitFFT<f32, SPLIT_COMPUTATION>
{
    type Complex = Complex<f32>;
    type Sample = f32;

    const MAX_SPLIT: usize = if SPLIT_COMPUTATION { 4 } else { 1 };
    const MIN_INNER_SIZE: usize = 32;
    const PREFERS_SPLIT: bool = Pow2FFT::<f32, SPLIT_COMPUTATION>::PREFERS_SPLIT;

    fn fast_size_above(size: usize) -> usize {
        let mut pow2 = 1;

        while pow2 < 16 && pow2 < size {
            pow2 *= 2;
        }
        while pow2 * 8 < size {
            pow2 *= 2;
        }

        let mut multiple = size.div_ceil(pow2); // will be 1-8

        if multiple == 7 {
            multiple += 1;
        }

        multiple * pow2
    }

    fn new(size: usize) -> Self {
        let mut new = Self {
            inner_fft: Pow2FFT::<f32, SPLIT_COMPUTATION>::new(size),

            inner_size: 1,
            outer_size: size,

            tmp_freq: Vec::new(),
            outer_twiddles: Vec::new(),
            outer_twiddles_r: Vec::new(),
            outer_twiddles_i: Vec::new(),
            dft_twists: Vec::new(),
            dft_tmp: Vec::new(),
            plan: Vec::new(),
        };

        new.resize(size);

        new
    }

    #[allow(clippy::cast_precision_loss, clippy::indexing_slicing)]
    fn resize(&mut self, size: usize) {
        self.inner_size = 1;
        self.outer_size = size;

        self.dft_tmp.clear();
        self.dft_twists.clear();
        self.plan.clear();

        if size == 0 {
            return;
        }

        // Inner size = largest power of 2 such that either the inner size >=
        // minInnerSize, or we have the target number of splits
        while (self.outer_size & 1 == 0)
            && (self.outer_size > SplitFFT::<f32, SPLIT_COMPUTATION>::MAX_SPLIT
                || self.inner_size < SplitFFT::<f32, SPLIT_COMPUTATION>::MIN_INNER_SIZE)
        {
            self.inner_size *= 2;
            self.outer_size /= 2;
        }
        self.tmp_freq.resize(size, Complex { re: 0.0, im: 0.0 });
        self.inner_fft.resize(self.inner_size);

        self.outer_twiddles.resize(
            self.inner_size * (self.outer_size - 1),
            Complex { re: 0.0, im: 0.0 },
        );
        self.outer_twiddles_r
            .resize(self.inner_size * (self.outer_size - 1), 0.0);
        self.outer_twiddles_i
            .resize(self.inner_size * (self.outer_size - 1), 0.0);

        for i in 0..self.inner_size {
            for s in 1..self.outer_size {
                let twiddle_phase = -2.0 * std::f32::consts::PI * i as f32 / self.inner_size as f32
                    * s as f32
                    / self.outer_size as f32;
                self.outer_twiddles[i + (s - 1) * self.inner_size] =
                    Complex::<f32>::from_polar(1.0, twiddle_phase);
            }
        }

        for i in 0..self.outer_twiddles.len() {
            self.outer_twiddles_r[i] = self.outer_twiddles[i].re;
            self.outer_twiddles_i[i] = self.outer_twiddles[i].im;
        }

        let mut interleave_step = StepType::InterleaveOrderN;
        let mut final_step = StepType::FinalOrderN;

        if self.outer_size == 2 {
            interleave_step = StepType::InterleaveOrder2;
            final_step = StepType::FinalOrder2;
        }
        if self.outer_size == 3 {
            interleave_step = StepType::InterleaveOrder3;
            final_step = StepType::FinalOrder3;
        }
        if self.outer_size == 4 {
            interleave_step = StepType::InterleaveOrder4;
            final_step = StepType::FinalOrder4;
        }
        if self.outer_size == 5 {
            interleave_step = StepType::InterleaveOrder5;
            final_step = StepType::FinalOrder5;
        }

        if self.outer_size <= 1 {
            if size > 0 {
                self.plan.push(Step {
                    step_type: StepType::Passthrough,
                    offset: 0,
                });
            }
        } else {
            self.plan.push(Step {
                step_type: interleave_step,
                offset: 0,
            });
            self.plan.push(Step {
                step_type: StepType::FirstFFT,
                offset: 0,
            });
            for s in 1..self.outer_size {
                self.plan.push(Step {
                    step_type: StepType::MiddleFFT,
                    offset: s * self.inner_size,
                });
            }
            self.plan.push(Step {
                step_type: StepType::Twiddles,
                offset: 0,
            });
            self.plan.push(Step {
                step_type: final_step,
                offset: 0,
            });

            if final_step == StepType::FinalOrderN {
                self.dft_tmp
                    .resize(self.outer_size, Complex { re: 0.0, im: 0.0 });
                self.dft_twists
                    .resize(self.outer_size, Complex { re: 0.0, im: 0.0 });
                for s in 0..self.outer_size {
                    let dft_phase = -2.0 * std::f32::consts::PI * s as f32 / self.outer_size as f32;

                    self.dft_twists[s] = Complex::<f32>::from_polar(1.0, dft_phase);
                }
            }
        }
    }

    fn size(&self) -> usize {
        self.inner_size * self.outer_size
    }

    fn steps(&self) -> usize {
        self.plan.len()
    }

    #[allow(clippy::undocumented_unsafe_blocks)]
    fn fft(&mut self, time: &[Complex<f32>], freq: &mut [Complex<f32>]) {
        let plan_pointer = &raw const self.plan;

        for step in unsafe { &*plan_pointer } {
            self.fft_step_internal::<false>(step, time, freq);
        }
    }

    #[allow(clippy::undocumented_unsafe_blocks, clippy::indexing_slicing)]
    fn fft_step(&mut self, step: usize, time: &[Complex<f32>], freq: &mut [Complex<f32>]) {
        let plan_pointer = &raw const self.plan[step];

        self.fft_step_internal::<false>(unsafe { &*plan_pointer }, time, freq);
    }

    #[allow(clippy::undocumented_unsafe_blocks)]
    fn fft_split_complex(
        &mut self,
        in_r: &[f32],
        in_i: &[f32],
        out_r: &mut [f32],
        out_i: &mut [f32],
    ) {
        let plan_pointer = &raw const self.plan;

        for step in unsafe { &*plan_pointer } {
            self.fft_step_split_complex_internal::<false>(step, in_r, in_i, out_r, out_i);
        }
    }

    #[allow(clippy::undocumented_unsafe_blocks, clippy::indexing_slicing)]
    fn fft_step_split_complex(
        &mut self,
        step: usize,
        in_r: &[f32],
        in_i: &[f32],
        out_r: &mut [f32],
        out_i: &mut [f32],
    ) {
        let plan_pointer = &raw const self.plan[step];

        self.fft_step_split_complex_internal::<false>(
            unsafe { &*plan_pointer },
            in_r,
            in_i,
            out_r,
            out_i,
        );
    }

    #[allow(clippy::undocumented_unsafe_blocks)]
    fn ifft(&mut self, time: &[Complex<f32>], freq: &mut [Complex<f32>]) {
        let plan_pointer = &raw const self.plan;

        for step in unsafe { &*plan_pointer } {
            self.fft_step_internal::<true>(step, time, freq);
        }
    }

    #[allow(clippy::undocumented_unsafe_blocks, clippy::indexing_slicing)]
    fn ifft_step(&mut self, step: usize, time: &[Complex<f32>], freq: &mut [Complex<f32>]) {
        let plan_pointer = &raw const self.plan[step];

        self.fft_step_internal::<true>(unsafe { &*plan_pointer }, time, freq);
    }

    #[allow(clippy::undocumented_unsafe_blocks)]
    fn ifft_split_complex(
        &mut self,
        in_r: &[f32],
        in_i: &[f32],
        out_r: &mut [f32],
        out_i: &mut [f32],
    ) {
        let plan_pointer = &raw const self.plan;

        for step in unsafe { &*plan_pointer } {
            self.fft_step_split_complex_internal::<true>(step, in_r, in_i, out_r, out_i);
        }
    }

    #[allow(clippy::undocumented_unsafe_blocks, clippy::indexing_slicing)]
    fn ifft_step_split_complex(
        &mut self,
        step: usize,
        in_r: &[f32],
        in_i: &[f32],
        out_r: &mut [f32],
        out_i: &mut [f32],
    ) {
        let plan_pointer = &raw const self.plan[step];

        self.fft_step_split_complex_internal::<true>(
            unsafe { &*plan_pointer },
            in_r,
            in_i,
            out_r,
            out_i,
        );
    }
}

impl<const SPLIT_COMPUTATION: bool> SplitFFT<f32, SPLIT_COMPUTATION> {
    #[allow(clippy::indexing_slicing, clippy::undocumented_unsafe_blocks)]
    fn fft_step_internal<const INVERSE: bool>(
        &mut self,
        step: &Step,
        time: &[Complex<f32>],
        freq: &mut [Complex<f32>],
    ) {
        match step.step_type {
            StepType::Passthrough => {
                if INVERSE {
                    self.inner_fft.ifft(time, freq);
                } else {
                    self.inner_fft.fft(time, freq);
                }
            }
            StepType::InterleaveOrder2 => {
                interleave_copy_const_astride::<Complex<f32>, 2>(
                    time,
                    &mut self.tmp_freq,
                    self.inner_size,
                );
            }
            StepType::InterleaveOrder3 => {
                interleave_copy_const_astride::<Complex<f32>, 3>(
                    time,
                    &mut self.tmp_freq,
                    self.inner_size,
                );
            }
            StepType::InterleaveOrder4 => {
                interleave_copy_const_astride::<Complex<f32>, 4>(
                    time,
                    &mut self.tmp_freq,
                    self.inner_size,
                );
            }
            StepType::InterleaveOrder5 => {
                interleave_copy_const_astride::<Complex<f32>, 5>(
                    time,
                    &mut self.tmp_freq,
                    self.inner_size,
                );
            }
            StepType::InterleaveOrderN => {
                interleave_copy::<Complex<f32>>(
                    time,
                    &mut self.tmp_freq,
                    self.outer_size,
                    self.inner_size,
                );
            }
            StepType::FirstFFT => {
                if INVERSE {
                    self.inner_fft.ifft(&self.tmp_freq, freq);
                } else {
                    self.inner_fft.fft(&self.tmp_freq, freq);
                }
            }
            StepType::MiddleFFT => {
                if INVERSE {
                    self.inner_fft
                        .ifft(&self.tmp_freq[step.offset..], &mut freq[step.offset..]);
                } else {
                    self.inner_fft
                        .fft(&self.tmp_freq[step.offset..], &mut freq[step.offset..]);
                }
            }
            StepType::Twiddles => {
                let freq_pointer = &raw const freq;

                let freq_2 = unsafe { &*freq_pointer };

                if INVERSE {
                    complex_mul_conj::<f32>(
                        &mut freq[self.inner_size..],
                        &freq_2[self.inner_size..],
                        &self.outer_twiddles,
                        self.inner_size * (self.outer_size - 1),
                    );
                } else {
                    complex_mul::<f32>(
                        &mut freq[self.inner_size..],
                        &freq_2[self.inner_size..],
                        &self.outer_twiddles,
                        self.inner_size * (self.outer_size - 1),
                    );
                }
            }
            StepType::FinalOrder2 => {
                self.final_pass_2(freq);
            }
            StepType::FinalOrder3 => {
                self.final_pass_3::<INVERSE>(freq);
            }
            StepType::FinalOrder4 => {
                self.final_pass_4::<INVERSE>(freq);
            }
            StepType::FinalOrder5 => {
                self.final_pass_5::<INVERSE>(freq);
            }
            StepType::FinalOrderN => {
                self.final_pass_n::<INVERSE>(freq);
            }
        }
    }

    #[allow(
        clippy::indexing_slicing,
        clippy::undocumented_unsafe_blocks,
        clippy::similar_names
    )]
    fn fft_step_split_complex_internal<const INVERSE: bool>(
        &mut self,
        step: &Step,
        in_r: &[f32],
        in_i: &[f32],
        out_r: &mut [f32],
        out_i: &mut [f32],
    ) {
        let (tmp_r, tmp_i) = complex_to_two_float_mut(&mut self.tmp_freq);

        match step.step_type {
            StepType::Passthrough => {
                if INVERSE {
                    self.inner_fft.ifft_split_complex(in_r, in_i, out_r, out_i);
                } else {
                    self.inner_fft.fft_split_complex(in_r, in_i, out_r, out_i);
                }
            }
            StepType::InterleaveOrder2 => {
                interleave_copy_const_astride::<f32, 2>(in_r, tmp_r, self.inner_size);
                interleave_copy_const_astride::<f32, 2>(in_i, tmp_i, self.inner_size);
            }
            StepType::InterleaveOrder3 => {
                interleave_copy_const_astride::<f32, 3>(in_r, tmp_r, self.inner_size);
                interleave_copy_const_astride::<f32, 3>(in_i, tmp_i, self.inner_size);
            }
            StepType::InterleaveOrder4 => {
                interleave_copy_const_astride::<f32, 4>(in_r, tmp_r, self.inner_size);
                interleave_copy_const_astride::<f32, 4>(in_i, tmp_i, self.inner_size);
            }
            StepType::InterleaveOrder5 => {
                interleave_copy_const_astride::<f32, 5>(in_r, tmp_r, self.inner_size);
                interleave_copy_const_astride::<f32, 5>(in_i, tmp_i, self.inner_size);
            }
            StepType::InterleaveOrderN => {
                interleave_copy_split_complex::<f32>(
                    in_r,
                    in_i,
                    tmp_r,
                    tmp_i,
                    self.outer_size,
                    self.inner_size,
                );
            }
            StepType::FirstFFT => {
                if INVERSE {
                    self.inner_fft
                        .ifft_split_complex(tmp_r, tmp_i, out_r, out_i);
                } else {
                    self.inner_fft.fft_split_complex(tmp_r, tmp_i, out_r, out_i);
                }
            }
            StepType::MiddleFFT => {
                if INVERSE {
                    self.inner_fft.ifft_split_complex(
                        &tmp_r[step.offset..],
                        &tmp_i[step.offset..],
                        &mut out_r[step.offset..],
                        &mut out_i[step.offset..],
                    );
                } else {
                    self.inner_fft.fft_split_complex(
                        &tmp_r[step.offset..],
                        &tmp_i[step.offset..],
                        &mut out_r[step.offset..],
                        &mut out_i[step.offset..],
                    );
                }
            }
            StepType::Twiddles => {
                let out_r_pointer = &raw const out_r;
                let out_r_2 = unsafe { &*out_r_pointer };

                let out_i_pointer = &raw const out_i;
                let out_i_2 = unsafe { &*out_i_pointer };

                if INVERSE {
                    complex_mul_conj_split_complex(
                        &mut out_r[self.inner_size..],
                        &mut out_i[self.inner_size..],
                        &out_r_2[self.inner_size..],
                        &out_i_2[self.inner_size..],
                        &self.outer_twiddles_r,
                        &self.outer_twiddles_i,
                        self.inner_size * (self.outer_size - 1),
                    );
                } else {
                    complex_mul_split_complex(
                        &mut out_r[self.inner_size..],
                        &mut out_i[self.inner_size..],
                        &out_r_2[self.inner_size..],
                        &out_i_2[self.inner_size..],
                        &self.outer_twiddles_r,
                        &self.outer_twiddles_i,
                        self.inner_size * (self.outer_size - 1),
                    );
                }
            }
            StepType::FinalOrder2 => {
                self.final_pass_2_split_complex(out_r, out_i);
            }
            StepType::FinalOrder3 => {
                self.final_pass_3_split_complex::<INVERSE>(out_r, out_i);
            }
            StepType::FinalOrder4 => {
                self.final_pass_4_split_complex::<INVERSE>(out_r, out_i);
            }
            StepType::FinalOrder5 => {
                self.final_pass_5_split_complex::<INVERSE>(out_r, out_i);
            }
            StepType::FinalOrderN => {
                self.final_pass_n_split_complex::<INVERSE>(out_r, out_i);
            }
        }
    }

    #[allow(clippy::indexing_slicing)]
    fn final_pass_2(&self, f0: &mut [Complex<f32>]) {
        for i in 0..self.inner_size {
            let a = f0[i];
            let b = f0[self.inner_size + i];

            f0[i] = a + b;
            f0[self.inner_size + i] = a - b;
        }
    }

    #[allow(clippy::indexing_slicing)]
    fn final_pass_2_split_complex(&self, f0r: &mut [f32], f0i: &mut [f32]) {
        for i in 0..self.inner_size {
            let ar = f0r[i];
            let ai = f0i[i];
            let br = f0r[self.inner_size + i];
            let bi = f0i[self.inner_size + i];

            f0r[i] = ar + br;
            f0i[i] = ai + bi;
            f0r[self.inner_size + i] = ar - br;
            f0i[self.inner_size + i] = ai - bi;
        }
    }

    #[allow(clippy::indexing_slicing)]
    fn final_pass_3<const INVERSE: bool>(&self, f0: &mut [Complex<f32>]) {
        let tw1 = Complex::<f32>::new(-0.5, -(0.75).sqrt() * (if INVERSE { -1.0 } else { 1.0 }));

        for i in 0..self.inner_size {
            let a = f0[i];
            let b = f0[self.inner_size + i];
            let c = f0[self.inner_size * 2 + i];

            let bc0 = b + c;
            let bc1 = b - c;

            f0[i] = a + bc0;
            f0[self.inner_size + i] = Complex::<f32>::new(
                a.re + bc0.re * tw1.re - bc1.im * tw1.im,
                a.im + bc0.im * tw1.re + bc1.re * tw1.im,
            );
            f0[self.inner_size * 2 + i] = Complex::<f32>::new(
                a.re + bc0.re * tw1.re + bc1.im * tw1.im,
                a.im + bc0.im * tw1.re - bc1.re * tw1.im,
            );
        }
    }

    #[allow(clippy::indexing_slicing, clippy::similar_names)]
    fn final_pass_3_split_complex<const INVERSE: bool>(&self, f0r: &mut [f32], f0i: &mut [f32]) {
        let tw1r = -0.5;
        let tw1i = -(0.75).sqrt() * (if INVERSE { -1.0 } else { 1.0 });

        for i in 0..self.inner_size {
            let ar = f0r[i];
            let ai = f0i[i];
            let br = f0r[self.inner_size + i];
            let bi = f0i[self.inner_size + i];
            let cr = f0r[self.inner_size * 2 + i];
            let ci = f0i[self.inner_size * 2 + i];

            f0r[i] = ar + br + cr;
            f0i[i] = ai + bi + ci;
            f0r[self.inner_size + i] = ar + br * tw1r - bi * tw1i + cr * tw1r + ci * tw1i;
            f0i[self.inner_size + i] = ai + bi * tw1r + br * tw1i - cr * tw1i + ci * tw1r;
            f0r[self.inner_size * 2 + i] = ar + br * tw1r + bi * tw1i + cr * tw1r - ci * tw1i;
            f0i[self.inner_size * 2 + i] = ai + bi * tw1r - br * tw1i + cr * tw1i + ci * tw1r;
        }
    }

    #[allow(clippy::indexing_slicing)]
    fn final_pass_4<const INVERSE: bool>(&self, f0: &mut [Complex<f32>]) {
        for i in 0..self.inner_size {
            let a = f0[i];
            let b = f0[self.inner_size + i];
            let c = f0[self.inner_size * 2 + i];
            let d = f0[self.inner_size * 3 + i];

            let ac0 = a + c;
            let ac1 = a - c;
            let bd0 = b + d;
            let bd1 = if INVERSE { b - d } else { d - b };
            let bd1i = Complex::<f32>::new(-bd1.im, bd1.re);

            f0[i] = ac0 + bd0;
            f0[self.inner_size + i] = ac1 + bd1i;
            f0[self.inner_size * 2 + i] = ac0 - bd0;
            f0[self.inner_size * 3 + i] = ac1 - bd1i;
        }
    }

    #[allow(clippy::indexing_slicing, clippy::similar_names)]
    fn final_pass_4_split_complex<const INVERSE: bool>(&self, f0r: &mut [f32], f0i: &mut [f32]) {
        for i in 0..self.inner_size {
            let ar = f0r[i];
            let ai = f0i[i];
            let br = f0r[self.inner_size + i];
            let bi = f0i[self.inner_size + i];
            let cr = f0r[self.inner_size * 2 + i];
            let ci = f0i[self.inner_size * 2 + i];
            let dr = f0r[self.inner_size * 3 + i];
            let di = f0i[self.inner_size * 3 + i];

            let ac0r = ar + cr;
            let ac0i = ai + ci;
            let ac1r = ar - cr;
            let ac1i = ai - ci;
            let bd0r = br + dr;
            let bd0i = bi + di;
            let bd1r = br - dr;
            let bd1i = bi - di;

            f0r[i] = ac0r + bd0r;
            f0i[i] = ac0i + bd0i;
            f0r[self.inner_size + i] = if INVERSE { ac1r - bd1i } else { ac1r + bd1i };
            f0i[self.inner_size + i] = if INVERSE { ac1i + bd1r } else { ac1i - bd1r };
            f0r[self.inner_size * 2 + i] = ac0r - bd0r;
            f0i[self.inner_size * 2 + i] = ac0i - bd0i;
            f0r[self.inner_size * 3 + i] = if INVERSE { ac1r + bd1i } else { ac1r - bd1i };
            f0i[self.inner_size * 3 + i] = if INVERSE { ac1i - bd1r } else { ac1i + bd1r };
        }
    }

    #[allow(
        clippy::indexing_slicing,
        clippy::similar_names,
        clippy::excessive_precision,
        clippy::unreadable_literal,
        clippy::many_single_char_names
    )]
    fn final_pass_5<const INVERSE: bool>(&self, f0: &mut [Complex<f32>]) {
        let tw1r = 0.30901699437494745;
        let tw1i = -0.9510565162951535 * (if INVERSE { -1.0 } else { 1.0 });
        let tw2r = -0.8090169943749473;
        let tw2i = -0.5877852522924732 * (if INVERSE { -1.0 } else { 1.0 });

        for i in 0..self.inner_size {
            let a = f0[i];
            let b = f0[self.inner_size + i];
            let c = f0[self.inner_size * 2 + i];
            let d = f0[self.inner_size * 3 + i];
            let e = f0[self.inner_size * 4 + i];

            let be0 = b + e;
            let be1 = Complex::<f32>::new(e.im - b.im, b.re - e.re); //(b - e)*i
            let cd0 = c + d;
            let cd1 = Complex::<f32>::new(d.im - c.im, c.re - d.re);

            let bcde01 = be0 * tw1r + cd0 * tw2r;
            let bcde02 = be0 * tw2r + cd0 * tw1r;
            let bcde11 = be1 * tw1i + cd1 * tw2i;
            let bcde12 = be1 * tw2i - cd1 * tw1i;

            f0[i] = a + be0 + cd0;
            f0[self.inner_size + i] = a + bcde01 + bcde11;
            f0[self.inner_size * 2 + i] = a + bcde02 + bcde12;
            f0[self.inner_size * 3 + i] = a + bcde02 - bcde12;
            f0[self.inner_size * 4 + i] = a + bcde01 - bcde11;
        }
    }

    #[allow(
        clippy::indexing_slicing,
        clippy::similar_names,
        clippy::excessive_precision,
        clippy::unreadable_literal
    )]
    fn final_pass_5_split_complex<const INVERSE: bool>(&self, f0r: &mut [f32], f0i: &mut [f32]) {
        let tw1r = 0.30901699437494745;
        let tw1i = -0.9510565162951535 * (if INVERSE { -1.0 } else { 1.0 });
        let tw2r = -0.8090169943749473;
        let tw2i = -0.5877852522924732 * (if INVERSE { -1.0 } else { 1.0 });

        for i in 0..self.inner_size {
            let ar = f0r[i];
            let ai = f0i[i];
            let br = f0r[self.inner_size + i];
            let bi = f0i[self.inner_size + i];
            let cr = f0r[self.inner_size * 2 + i];
            let ci = f0i[self.inner_size * 2 + i];
            let dr = f0r[self.inner_size * 3 + i];
            let di = f0i[self.inner_size * 3 + i];
            let er = f0r[self.inner_size * 4 + i];
            let ei = f0i[self.inner_size * 4 + i];

            let be0r = br + er;
            let be0i = bi + ei;
            let be1r = ei - bi;
            let be1i = br - er;
            let cd0r = cr + dr;
            let cd0i = ci + di;
            let cd1r = di - ci;
            let cd1i = cr - dr;

            let bcde01r = be0r * tw1r + cd0r * tw2r;
            let bcde01i = be0i * tw1r + cd0i * tw2r;
            let bcde02r = be0r * tw2r + cd0r * tw1r;
            let bcde02i = be0i * tw2r + cd0i * tw1r;
            let bcde11r = be1r * tw1i + cd1r * tw2i;
            let bcde11i = be1i * tw1i + cd1i * tw2i;
            let bcde12r = be1r * tw2i - cd1r * tw1i;
            let bcde12i = be1i * tw2i - cd1i * tw1i;

            f0r[i] = ar + be0r + cd0r;
            f0i[i] = ai + be0i + cd0i;
            f0r[self.inner_size + i] = ar + bcde01r + bcde11r;
            f0i[self.inner_size + i] = ai + bcde01i + bcde11i;
            f0r[self.inner_size * 2 + i] = ar + bcde02r + bcde12r;
            f0i[self.inner_size * 2 + i] = ai + bcde02i + bcde12i;
            f0r[self.inner_size * 3 + i] = ar + bcde02r - bcde12r;
            f0i[self.inner_size * 3 + i] = ai + bcde02i - bcde12i;
            f0r[self.inner_size * 4 + i] = ar + bcde01r - bcde11r;
            f0i[self.inner_size * 4 + i] = ai + bcde01i - bcde11i;
        }
    }

    #[allow(clippy::indexing_slicing)]
    fn final_pass_n<const INVERSE: bool>(&mut self, f0: &mut [Complex<f32>]) {
        for i in 0..self.inner_size {
            let mut sum = Complex::<f32>::new(0.0, 0.0);

            for i2 in 0..self.outer_size {
                let tmp_value = f0[i + i2 * self.inner_size];

                self.dft_tmp[i2] = tmp_value;
                sum += tmp_value;
            }

            f0[i] = sum;

            for f in 1..self.outer_size {
                let mut sum = self.dft_tmp[0];

                for i2 in 1..self.outer_size {
                    let twist_index = (i2 * f) % self.outer_size;
                    let twist = if INVERSE {
                        self.dft_twists[twist_index].conj()
                    } else {
                        self.dft_twists[twist_index]
                    };

                    sum += Complex::<f32>::new(
                        self.dft_tmp[i2].re * twist.re - self.dft_tmp[i2].im * twist.im,
                        self.dft_tmp[i2].im * twist.re + self.dft_tmp[i2].re * twist.im,
                    );
                }

                f0[i + f * self.inner_size] = sum;
            }
        }
    }

    #[allow(clippy::indexing_slicing)]
    fn final_pass_n_split_complex<const INVERSE: bool>(
        &mut self,
        f0r: &mut [f32],
        f0i: &mut [f32],
    ) {
        let (tmp_r, tmp_i) = complex_to_two_float_mut(&mut self.dft_tmp);

        for i in 0..self.inner_size {
            let mut sum_r = 0.0;
            let mut sum_i = 0.0;

            for i2 in 0..self.outer_size {
                let tmp_value_r = f0r[i + i2 * self.inner_size];
                let tmp_value_i = f0i[i + i2 * self.inner_size];

                tmp_r[i2] = tmp_value_r;
                sum_r += tmp_value_r;
                tmp_i[i2] = tmp_value_i;
                sum_i += tmp_value_i;
            }

            f0r[i] = sum_r;
            f0i[i] = sum_i;

            for f in 1..self.outer_size {
                let mut sum_r = tmp_r[0];
                let mut sum_i = tmp_i[0];

                for i2 in 1..self.outer_size {
                    let twist_index = (i2 * f) % self.outer_size;

                    let twist = if INVERSE {
                        self.dft_twists[twist_index].conj()
                    } else {
                        self.dft_twists[twist_index]
                    };

                    sum_r += tmp_r[i2] * twist.re - tmp_i[i2] * twist.im;
                    sum_i += tmp_i[i2] * twist.re + tmp_r[i2] * twist.im;
                }

                f0r[i + f * self.inner_size] = sum_r;
                f0i[i + f * self.inner_size] = sum_i;
            }
        }
    }
}
