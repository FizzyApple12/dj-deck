use std::ops::{Index, IndexMut};

use num::{Complex, Float, integer::Roots};

use crate::dsp::fft::{FFTTrait, IndexOffsetMut, complexAddI, complexMul};

#[derive(Clone, Copy, PartialEq)]
pub enum StepType {
    Generic,
    Step2,
    Step3,
    Step4,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Step {
    step_type: StepType,
    factor: usize,
    start_index: usize,
    inner_repeats: usize,
    outer_repeats: usize,
    twiddle_index: usize,
}

#[derive(Clone, Copy, PartialEq)]
pub struct PermutationPair {
    from: usize,
    to: usize,
}

/** Floating-point FFT implementation.
It is fast for 2^a * 3^b.
*/
pub struct FFT<Sample>
where
    Sample: Float,
{
    inner_size: usize,
    working_vector: Vec<Complex<Sample>>,

    factors: Vec<usize>,
    plan: Vec<Step>,
    twiddle_vector: Vec<Complex<Sample>>,

    permutation: Vec<PermutationPair>,
}

impl FFTTrait<f32, Self::Complex, Self::Complex> for FFT<f32> {
    type Complex = Self::Complex;
    type Sample = Self::Sample;

    fn fast_size_above(mut size: usize) -> usize {
        let mut power2 = 1;

        while size >= 32 {
            size = (size - 1) / 2 + 1;

            power2 *= 2;
        }

        while size < 32 && !Self::valid_size(size) {
            size += 1;
        }

        power2 * size
    }

    fn fast_size_below(mut size: usize) -> usize {
        let mut power2 = 1;

        while size >= 32 {
            size /= 2;

            power2 *= 2;
        }

        while size > 1 && !Self::valid_size(size) {
            size -= 1;
        }

        power2 * size
    }

    fn new(mut size: usize, fast_direction: i32) -> Self {
        if fast_direction > 0 {
            size = Self::fast_size_above(size);
        }
        if fast_direction < 0 {
            size = Self::fast_size_below(size);
        }

        let mut new = Self {
            inner_size: 0,
            working_vector: Vec::new(),
            factors: Vec::new(),
            plan: Vec::new(),
            twiddle_vector: Vec::new(),
            permutation: Vec::new(),
        };

        new.set_size(size);

        new
    }

    fn set_size(&mut self, size: usize) -> usize {
        if size != self.inner_size {
            self.inner_size = size;

            self.working_vector
                .resize(size, Complex { re: 0.0, im: 0.0 });

            self.set_plan();
        }

        self.inner_size
    }

    fn set_fast_size_above(&mut self, size: usize) -> usize {
        self.set_size(Self::fast_size_above(size))
    }

    fn set_fast_size_below(&mut self, size: usize) -> usize {
        self.set_size(Self::fast_size_below(size))
    }

    fn size(&self) -> usize {
        self.inner_size
    }

    fn fft<InputBuffer, OutputBuffer>(&mut self, input: &InputBuffer, output: &mut OutputBuffer)
    where
        InputBuffer: Index<usize, Output = Self::Complex>,
        OutputBuffer: IndexMut<usize, Output = Self::Complex>,
    {
        self.run::<false, InputBuffer, OutputBuffer>(input, output)
    }

    fn ifft<InputBuffer, OutputBuffer>(&mut self, input: &InputBuffer, output: &mut OutputBuffer)
    where
        InputBuffer: Index<usize, Output = Self::Complex>,
        OutputBuffer: IndexMut<usize, Output = Self::Complex>,
    {
        self.run::<true, InputBuffer, OutputBuffer>(input, output)
    }
}

impl FFT<f32> {
    type Complex = Complex<f32>;
    type Sample = f32;

    fn add_plan_steps(
        &mut self,
        mut factor_index: usize,
        start: usize,
        length: usize,
        repeats: usize,
    ) {
        if factor_index >= self.factors.len() {
            return;
        }

        let mut factor = self.factors[factor_index];
        if factor_index + 1 < self.factors.len() {
            if self.factors[factor_index] == 2 && self.factors[factor_index + 1] == 2 {
                factor_index += 1;
                factor = 4;
            }
        }

        let sub_length = length / factor;
        let mut main_step = Step {
            step_type: StepType::Generic,
            factor,
            start_index: start,
            inner_repeats: sub_length,
            outer_repeats: repeats,
            twiddle_index: self.twiddle_vector.len(),
        };

        if factor == 2 {
            main_step.step_type = StepType::Step2;
        }
        if factor == 3 {
            main_step.step_type = StepType::Step3;
        }
        if factor == 4 {
            main_step.step_type = StepType::Step4;
        }

        // Twiddles
        let mut found_step = false;
        for existing_step in &self.plan {
            if existing_step.factor == main_step.factor
                && existing_step.inner_repeats == main_step.inner_repeats
            {
                found_step = true;
                main_step.twiddle_index = existing_step.twiddle_index;

                break;
            }
        }

        if !found_step {
            for i in 0..sub_length {
                for f in 0..factor {
                    let phase = 2.0 * std::f32::consts::PI * i as f32 * f as f32 / length as f32;

                    let twiddle = Self::Complex::new(phase.cos(), -phase.sin());

                    self.twiddle_vector.push(twiddle);
                }
            }
        }

        if repeats == 1 && std::mem::size_of::<Self::Complex>() * sub_length > 65536 {
            for i in 0..factor {
                self.add_plan_steps(factor_index + 1, start + i * sub_length, sub_length, 1);
            }
        } else {
            self.add_plan_steps(factor_index + 1, start, sub_length, repeats * factor);
        }

        self.plan.push(main_step);
    }

    fn set_plan(&mut self) {
        self.factors.clear();
        let mut size = self.inner_size;
        let mut factor = 2;

        while size > 1 {
            if size % factor == 0 {
                self.factors.push(factor);
                size /= factor;
            } else if factor > size.sqrt() {
                factor = size;
            } else {
                factor += 1;
            }
        }

        self.plan.clear();
        self.twiddle_vector.clear();
        self.add_plan_steps(0, 0, self.inner_size, 1);
        self.twiddle_vector.shrink_to_fit();

        self.permutation.clear();
        self.permutation.reserve(self.inner_size);
        self.permutation.push(PermutationPair { to: 0, from: 0 });

        let mut index_low = 0;
        let mut index_high = self.factors.len();
        let mut input_step_low = self.inner_size;
        let mut output_step_low = 1;
        let mut input_step_high = 1;
        let mut output_step_high = self.inner_size;

        while output_step_low * input_step_high < self.inner_size {
            let (f, input_step, output_step) = if output_step_low <= input_step_high {
                let f = self.factors[index_low];
                index_low += 1;

                input_step_low /= f;
                let input_step = input_step_low;
                let output_step = output_step_low;

                output_step_low *= f;

                (f, input_step, output_step)
            } else {
                index_high -= 1;
                let f = self.factors[index_high];

                let input_step = input_step_high;
                input_step_high *= f;

                output_step_high /= f;
                let output_step = output_step_high;

                (f, input_step, output_step)
            };

            let old_size = self.permutation.len();

            for i in 1..f {
                for j in 0..old_size {
                    let mut pair = self.permutation[j];

                    pair.from += i * input_step;
                    pair.to += i * output_step;

                    self.permutation.push(pair);
                }
            }
        }
    }

    fn fft_step_generic<const inverse: bool, Buffer>(&mut self, orig_data: &mut Buffer, step: &Step)
    where
        Buffer: IndexMut<usize, Output = Self::Complex>,
    {
        let stride = step.inner_repeats;

        let mut orig_data_base = 0;

        for _ in 0..step.outer_repeats {
            let mut data_base = orig_data_base;

            let mut twiddles_base = step.twiddle_index;
            let factor = step.factor;

            for _ in 0..step.inner_repeats {
                for i in 0..step.factor {
                    self.working_vector[i] = complexMul::<inverse, Self::Sample>(
                        &orig_data[data_base + i * stride],
                        &self.twiddle_vector[twiddles_base + i],
                    );
                }

                for f in 0..factor {
                    let mut sum = self.working_vector[0];

                    for i in 1..factor {
                        let phase =
                            2.0 * std::f32::consts::PI * f as f32 * i as f32 / factor as f32;

                        let twiddle = Self::Complex::new(phase.cos(), -phase.sin());

                        sum +=
                            complexMul::<inverse, Self::Sample>(&self.working_vector[i], &twiddle);
                    }

                    orig_data[data_base + f * stride] = sum;
                }

                data_base += 1;

                twiddles_base += factor;
            }

            orig_data_base += step.factor * step.inner_repeats;
        }
    }

    #[inline]
    fn fft_step_2<const inverse: bool, Buffer>(&mut self, orig_data: &mut Buffer, step: &Step)
    where
        Buffer: IndexMut<usize, Output = Self::Complex>,
    {
        let stride = step.inner_repeats;
        let orig_twiddles_base = step.twiddle_index;

        let mut orig_data_base = 0;

        for _ in 0..step.outer_repeats {
            let mut twiddles_base = orig_twiddles_base;

            for data_base in orig_data_base..(orig_data_base + stride) {
                let a = orig_data[data_base + 0];
                let b = complexMul::<inverse, Self::Sample>(
                    &orig_data[data_base + stride],
                    &self.twiddle_vector[twiddles_base + 1],
                );

                orig_data[data_base + 0] = a + b;
                orig_data[data_base + stride] = a - b;

                twiddles_base += 2;
            }

            orig_data_base += 2 * stride;
        }
    }

    #[allow(clippy::unreadable_literal)]
    #[inline]
    fn fft_step_3<const inverse: bool, Buffer>(&mut self, orig_data: &mut Buffer, step: &Step)
    where
        Buffer: IndexMut<usize, Output = Self::Complex>,
    {
        let factor3: Self::Complex = Self::Complex::new(
            -0.5,
            if inverse {
                0.8660254037844386
            } else {
                -0.8660254037844386
            },
        );

        let stride = step.inner_repeats;
        let orig_twiddles_base = step.twiddle_index;

        let mut orig_data_base = 0;

        for _ in 0..step.outer_repeats {
            let mut twiddles_base = orig_twiddles_base;

            for data_base in orig_data_base..(orig_data_base + stride) {
                let a = orig_data[data_base + 0];
                let b = complexMul::<inverse, Self::Sample>(
                    &orig_data[data_base + stride],
                    &self.twiddle_vector[twiddles_base + 1],
                );
                let c = complexMul::<inverse, Self::Sample>(
                    &orig_data[data_base + stride * 2],
                    &self.twiddle_vector[twiddles_base + 2],
                );

                let real_sum = a + (b + c) * factor3.re;
                let imag_sum = (b - c) * factor3.im;

                orig_data[data_base + 0] = a + b + c;
                orig_data[data_base + stride] =
                    complexAddI::<false, Self::Sample>(&real_sum, &imag_sum);
                orig_data[data_base + stride * 2] =
                    complexAddI::<true, Self::Sample>(&real_sum, &imag_sum);

                twiddles_base += 3;
            }

            orig_data_base += 3 * stride;
        }
    }

    #[inline]
    fn fft_step_4<const inverse: bool, Buffer>(&mut self, orig_data: &mut Buffer, step: &Step)
    where
        Buffer: IndexMut<usize, Output = Self::Complex>,
        [(); { !inverse } as usize]:,
    {
        let stride = step.inner_repeats;
        let orig_twiddles_base = step.twiddle_index;

        let mut orig_data_base = 0;

        for _ in 0..step.outer_repeats {
            let mut twiddles_base = orig_twiddles_base;

            for data_base in orig_data_base..(orig_data_base + stride) {
                let a = orig_data[data_base + 0];
                let c = complexMul::<inverse, Self::Sample>(
                    &orig_data[data_base + stride],
                    &self.twiddle_vector[twiddles_base + 2],
                );
                let b = complexMul::<inverse, Self::Sample>(
                    &orig_data[data_base + stride * 2],
                    &self.twiddle_vector[twiddles_base + 1],
                );
                let d = complexMul::<inverse, Self::Sample>(
                    &orig_data[data_base + stride * 3],
                    &self.twiddle_vector[twiddles_base + 3],
                );

                let sum_a_c = a + c;
                let sum_b_d = b + d;
                let diff_a_c = a - c;
                let diff_b_d = b - d;

                orig_data[data_base + 0] = sum_a_c + sum_b_d;
                orig_data[data_base + stride] =
                    complexAddI::<{ !inverse }, Self::Sample>(&diff_a_c, &diff_b_d);
                orig_data[data_base + stride * 2] = sum_a_c - sum_b_d;
                orig_data[data_base + stride * 3] =
                    complexAddI::<inverse, Self::Sample>(&diff_a_c, &diff_b_d);

                twiddles_base += 4;
            }

            orig_data_base += 4 * stride;
        }
    }

    fn permute<InputBuffer, OutputBuffer, T>(&self, input: &InputBuffer, data: &mut OutputBuffer)
    where
        InputBuffer: Index<usize, Output = T>,
        OutputBuffer: IndexMut<usize, Output = T>,
        T: Copy,
    {
        for pair in &self.permutation {
            data[pair.from] = input[pair.to];
        }
    }

    fn run<const inverse: bool, InputBuffer, OutputBuffer>(
        &mut self,
        input: &InputBuffer,
        data: &mut OutputBuffer,
    ) where
        InputBuffer: Index<usize, Output = Self::Complex>,
        OutputBuffer: IndexMut<usize, Output = Self::Complex>,
        [(); { !inverse } as usize]:,
    {
        self.permute(input, data);

        let plan_pointer = &raw const self.plan;

        for step in unsafe { &*plan_pointer } {
            match step.step_type {
                StepType::Generic => self.fft_step_generic::<inverse, _>(
                    &mut IndexOffsetMut::new(data, step.start_index),
                    step,
                ),

                StepType::Step2 => self.fft_step_2::<inverse, _>(
                    &mut IndexOffsetMut::new(data, step.start_index),
                    step,
                ),

                StepType::Step3 => self.fft_step_3::<inverse, _>(
                    &mut IndexOffsetMut::new(data, step.start_index),
                    step,
                ),

                StepType::Step4 => self.fft_step_4::<inverse, _>(
                    &mut IndexOffsetMut::new(data, step.start_index),
                    step,
                ),
            }
        }
    }

    fn valid_size(size: usize) -> bool {
        const FILTER: [bool; 32] = [
            true, true, true, true, true, false, true, false, true, true, // 0-9
            false, false, true, false, false, false, true, false, true, false, // 10-19
            false, false, false, false, true, false, false, false, false, false, // 20-29
            false, false,
        ];

        FILTER[size]
    }
}
