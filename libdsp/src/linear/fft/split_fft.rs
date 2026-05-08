use num::{Float, complex::Complex};

use crate::linear::{
    complex_to_two_float,
    fft::pow2_fft::{Pow2FFT, Pow2FFTTrait},
};

#[derive(Clone, Copy, PartialEq)]
enum StepType {
    passthrough,
    interleaveOrder2,
    interleaveOrder3,
    interleaveOrder4,
    interleaveOrder5,
    interleaveOrderN,
    firstFFT,
    middleFFT,
    twiddles,
    finalOrder2,
    finalOrder3,
    finalOrder4,
    finalOrder5,
    finalOrderN,
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
    innerFFT: Pow2FFT<Sample, SPLIT_COMPUTATION>,

    innerSize: usize,
    outerSize: usize,
    tmpFreq: Vec<Complex<Sample>>,
    outerTwiddles: Vec<Complex<Sample>>,
    outerTwiddlesR: Vec<Sample>,
    outerTwiddlesI: Vec<Sample>,
    dftTwists: Vec<Complex<Sample>>,
    dftTmp: Vec<Complex<Sample>>,
    plan: Vec<Step>,
}

pub trait SplitFFTTrait<Sample, const SPLIT_COMPUTATION: bool>
where
    Sample: Float + Default,
{
    const MAX_SPLIT: usize;
    const MIN_INNER_SIZE: usize;
    const PREFERS_SPLIT: bool;

    fn fastSizeAbove(size: usize) -> usize;

    fn new(size: usize) -> Self;

    fn resize(&mut self, size: usize);

    fn size(&self) -> usize;
    fn steps(&self) -> usize;

    // fn fft(&mut self, time: &mut [f32], freq: &mut [Complex<f32>]);
    // fn ifft(&mut self, freq: &mut [Complex<f32>], time: &mut [f32]);
    // fn fft_split_complex(&mut self, in_r: &[f32], out_r: &mut [f32], out_i: &mut
    // [f32]); fn ifft_split_complex(&mut self, in_r: &[f32], in_i: &[f32],
    // out_r: &mut [f32]);
}

impl<const SPLIT_COMPUTATION: bool> SplitFFTTrait<f32, SPLIT_COMPUTATION>
    for SplitFFT<f32, SPLIT_COMPUTATION>
{
    const MAX_SPLIT: usize = if SPLIT_COMPUTATION { 4 } else { 1 };
    const MIN_INNER_SIZE: usize = 32;
    const PREFERS_SPLIT: bool = Pow2FFT::<f32, SPLIT_COMPUTATION>::PREFERS_SPLIT;

    fn fastSizeAbove(size: usize) -> usize {
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
            innerFFT: Pow2FFT::<f32, SPLIT_COMPUTATION>::new(size),

            innerSize: 1,
            outerSize: size,

            tmpFreq: Vec::new(),
            outerTwiddles: Vec::new(),
            outerTwiddlesR: Vec::new(),
            outerTwiddlesI: Vec::new(),
            dftTwists: Vec::new(),
            dftTmp: Vec::new(),
            plan: Vec::new(),
        };

        new.resize(size);

        new
    }

    #[allow(clippy::cast_precision_loss, clippy::indexing_slicing)]
    fn resize(&mut self, size: usize) {
        self.innerSize = 1;
        self.outerSize = size;

        self.dftTmp.clear();
        self.dftTwists.clear();
        self.plan.clear();

        if size == 0 {
            return;
        }

        // Inner size = largest power of 2 such that either the inner size >=
        // minInnerSize, or we have the target number of splits
        while (self.outerSize & 1 == 0)
            && (self.outerSize > SplitFFT::<f32, SPLIT_COMPUTATION>::MAX_SPLIT
                || self.innerSize < SplitFFT::<f32, SPLIT_COMPUTATION>::MIN_INNER_SIZE)
        {
            self.innerSize *= 2;
            self.outerSize /= 2;
        }
        self.tmpFreq.resize(size, Complex { re: 0.0, im: 0.0 });
        self.innerFFT.resize(self.innerSize);

        self.outerTwiddles.resize(
            self.innerSize * (self.outerSize - 1),
            Complex { re: 0.0, im: 0.0 },
        );
        self.outerTwiddlesR
            .resize(self.innerSize * (self.outerSize - 1), 0.0);
        self.outerTwiddlesI
            .resize(self.innerSize * (self.outerSize - 1), 0.0);

        for i in 0..self.innerSize {
            for s in 1..self.outerSize {
                let twiddle_phase = -2.0 * std::f32::consts::PI * i as f32 / self.innerSize as f32
                    * s as f32
                    / self.outerSize as f32;
                self.outerTwiddles[i + (s - 1) * self.innerSize] =
                    Complex::from_polar(1.0, twiddle_phase);
            }
        }

        for i in 0..self.outerTwiddles.len() {
            self.outerTwiddlesR[i] = self.outerTwiddles[i].re;
            self.outerTwiddlesI[i] = self.outerTwiddles[i].im;
        }

        let mut interleave_step = StepType::interleaveOrderN;
        let mut final_step = StepType::finalOrderN;

        if self.outerSize == 2 {
            interleave_step = StepType::interleaveOrder2;
            final_step = StepType::finalOrder2;
        }
        if self.outerSize == 3 {
            interleave_step = StepType::interleaveOrder3;
            final_step = StepType::finalOrder3;
        }
        if self.outerSize == 4 {
            interleave_step = StepType::interleaveOrder4;
            final_step = StepType::finalOrder4;
        }
        if self.outerSize == 5 {
            interleave_step = StepType::interleaveOrder5;
            final_step = StepType::finalOrder5;
        }

        if self.outerSize <= 1 {
            if size > 0 {
                self.plan.push(Step {
                    step_type: StepType::passthrough,
                    offset: 0,
                });
            }
        } else {
            self.plan.push(Step {
                step_type: interleave_step,
                offset: 0,
            });
            self.plan.push(Step {
                step_type: StepType::firstFFT,
                offset: 0,
            });
            for s in 1..self.outerSize {
                self.plan.push(Step {
                    step_type: StepType::middleFFT,
                    offset: s * self.innerSize,
                });
            }
            self.plan.push(Step {
                step_type: StepType::twiddles,
                offset: 0,
            });
            self.plan.push(Step {
                step_type: final_step,
                offset: 0,
            });

            if final_step == StepType::finalOrderN {
                self.dftTmp
                    .resize(self.outerSize, Complex { re: 0.0, im: 0.0 });
                self.dftTwists
                    .resize(self.outerSize, Complex { re: 0.0, im: 0.0 });
                for s in 0..self.outerSize {
                    let dft_phase = -2.0 * std::f32::consts::PI * s as f32 / self.outerSize as f32;

                    self.dftTwists[s] = Complex::from_polar(1.0, dft_phase);
                }
            }
        }
    }

    fn size(&self) -> usize {
        self.innerSize * self.outerSize
    }

    fn steps(&self) -> usize {
        self.plan.len()
    }

    // 	void fft(const Complex *time, Complex *freq) {
    // 		for (auto &step : plan) {
    // 			fftStep<false>(step, time, freq);
    // 		}
    // 	}
    // 	void fft(size_t step, const Complex *time, Complex *freq) {
    // 		fftStep<false>(plan[step], time, freq);
    // 	}
    // 	void fft(const Sample *inR, const Sample *inI, Sample *outR, Sample *outI) {
    // 		for (auto &step : plan) {
    // 			fftStep<false>(step, inR, inI, outR, outI);
    // 		}
    // 	}
    // 	void fft(size_t step, const Sample *inR, const Sample *inI, Sample *outR,
    // Sample *outI) { 		fftStep<false>(plan[step], inR, inI, outR, outI);
    // 	}

    // 	void ifft(const Complex *freq, Complex *time) {
    // 		for (auto &step : plan) {
    // 			fftStep<true>(step, freq, time);
    // 		}
    // 	}
    // 	void ifft(size_t step, const Complex *freq, Complex *time) {
    // 		fftStep<true>(plan[step], freq, time);
    // 	}
    // 	void ifft(const Sample *inR, const Sample *inI, Sample *outR, Sample *outI)
    // { 		for (auto &step : plan) {
    // 			fftStep<true>(step, inR, inI, outR, outI);
    // 		}
    // 	}
    // 	void ifft(size_t step, const Sample *inR, const Sample *inI, Sample *outR,
    // Sample *outI) { 		fftStep<true>(plan[step], inR, inI, outR, outI);
    // 	}
}

impl<const SPLIT_COMPUTATION: bool> SplitFFT<f32, SPLIT_COMPUTATION> {
    // 	template<bool inverse>
    // 	void fftStep(Step step, const Complex *time, Complex *freq) {
    // 		switch (step.type) {
    // 			case (StepType::passthrough): {
    // 				if (inverse) {
    // 					innerFFT.ifft(time, freq);
    // 				} else {
    // 					innerFFT.fft(time, freq);
    // 				}
    // 				break;
    // 			}
    // 			case (StepType::interleaveOrder2): {
    // 				_impl::interleaveCopy<2>(time, tmpFreq.data(), innerSize);
    // 				break;
    // 			}
    // 			case (StepType::interleaveOrder3): {
    // 				_impl::interleaveCopy<3>(time, tmpFreq.data(), innerSize);
    // 				break;
    // 			}
    // 			case (StepType::interleaveOrder4): {
    // 				_impl::interleaveCopy<4>(time, tmpFreq.data(), innerSize);
    // 				break;
    // 			}
    // 			case (StepType::interleaveOrder5): {
    // 				_impl::interleaveCopy<5>(time, tmpFreq.data(), innerSize);
    // 				break;
    // 			}
    // 			case (StepType::interleaveOrderN): {
    // 				_impl::interleaveCopy(time, tmpFreq.data(), outerSize, innerSize);
    // 				break;
    // 			}
    // 			case (StepType::firstFFT): {
    // 				if (inverse) {
    // 					innerFFT.ifft(tmpFreq.data(), freq);
    // 				} else {
    // 					innerFFT.fft(tmpFreq.data(), freq);
    // 				}
    // 				break;
    // 			}
    // 			case (StepType::middleFFT): {
    // 				Complex *offsetOut = freq + step.offset;
    // 				if (inverse) {
    // 					innerFFT.ifft(tmpFreq.data() + step.offset, offsetOut);
    // 				} else {
    // 					innerFFT.fft(tmpFreq.data() + step.offset, offsetOut);
    // 				}
    // 				break;
    // 			}
    // 			case (StepType::twiddles): {
    // 				if (inverse) {
    // 					_impl::complexMulConj(freq + innerSize, freq + innerSize,
    // outerTwiddles.data(), innerSize*(outerSize - 1)); 				} else {
    // 					_impl::complexMul(freq + innerSize, freq + innerSize,
    // outerTwiddles.data(), innerSize*(outerSize - 1)); 				}
    // 				break;
    // 			}
    // 			case StepType::finalOrder2:
    // 				finalPass2(freq);
    // 				break;
    // 			case StepType::finalOrder3:
    // 				finalPass3<inverse>(freq);
    // 				break;
    // 			case StepType::finalOrder4:
    // 				finalPass4<inverse>(freq);
    // 				break;
    // 			case StepType::finalOrder5:
    // 				finalPass5<inverse>(freq);
    // 				break;
    // 			case StepType::finalOrderN:
    // 				finalPassN<inverse>(freq);
    // 				break;
    // 		}
    // 	}

    // 	template<bool inverse>
    // 	void fftStep(Step step, const Sample *inR, const Sample *inI, Sample *outR,
    // Sample *outI) { 		Sample *tmpR = (Sample *)tmpFreq.data(), *tmpI = tmpR +
    // tmpFreq.size(); 		switch (step.type) {
    // 			case (StepType::passthrough): {
    // 				if (inverse) {
    // 					innerFFT.ifft(inR, inI, outR, outI);
    // 				} else {
    // 					innerFFT.fft(inR, inI, outR, outI);
    // 				}
    // 				break;
    // 			}
    // 			case (StepType::interleaveOrder2): {
    // 				_impl::interleaveCopy<2>(inR, tmpR, innerSize);
    // 				_impl::interleaveCopy<2>(inI, tmpI, innerSize);
    // 				break;
    // 			}
    // 			case (StepType::interleaveOrder3): {
    // 				_impl::interleaveCopy<3>(inR, tmpR, innerSize);
    // 				_impl::interleaveCopy<3>(inI, tmpI, innerSize);
    // 				break;
    // 			}
    // 			case (StepType::interleaveOrder4): {
    // 				_impl::interleaveCopy<4>(inR, tmpR, innerSize);
    // 				_impl::interleaveCopy<4>(inI, tmpI, innerSize);
    // 				break;
    // 			}
    // 			case (StepType::interleaveOrder5): {
    // 				_impl::interleaveCopy<5>(inR, tmpR, innerSize);
    // 				_impl::interleaveCopy<5>(inI, tmpI, innerSize);
    // 				break;
    // 			}
    // 			case (StepType::interleaveOrderN): {
    // 				_impl::interleaveCopy(inR, inI, tmpR, tmpI, outerSize, innerSize);
    // 				break;
    // 			}
    // 			case (StepType::firstFFT): {
    // 				if (inverse) {
    // 					innerFFT.ifft(tmpR, tmpI, outR, outI);
    // 				} else {
    // 					innerFFT.fft(tmpR, tmpI, outR, outI);
    // 				}
    // 				break;
    // 			}
    // 			case (StepType::middleFFT): {
    // 				size_t offset = step.offset;
    // 				Sample *offsetOutR = outR + offset;
    // 				Sample *offsetOutI = outI + offset;
    // 				if (inverse) {
    // 					innerFFT.ifft(tmpR + offset, tmpI + offset, offsetOutR, offsetOutI);
    // 				} else {
    // 					innerFFT.fft(tmpR + offset, tmpI + offset, offsetOutR, offsetOutI);
    // 				}
    // 				break;
    // 			}
    // 			case(StepType::twiddles): {
    // 				auto *twiddlesR = outerTwiddlesR.data();
    // 				auto *twiddlesI = outerTwiddlesI.data();
    // 				if (inverse) {
    // 					_impl::complexMulConj(outR + innerSize, outI + innerSize, outR +
    // innerSize, outI + innerSize, twiddlesR, twiddlesI, innerSize*(outerSize -
    // 1)); 				} else {
    // 					_impl::complexMul(outR + innerSize, outI + innerSize, outR + innerSize,
    // outI + innerSize, twiddlesR, twiddlesI, innerSize*(outerSize - 1)); 				}
    // 				break;
    // 			}
    // 			case StepType::finalOrder2:
    // 				finalPass2(outR, outI);
    // 				break;
    // 			case StepType::finalOrder3:
    // 				finalPass3<inverse>(outR, outI);
    // 				break;
    // 			case StepType::finalOrder4:
    // 				finalPass4<inverse>(outR, outI);
    // 				break;
    // 			case StepType::finalOrder5:
    // 				finalPass5<inverse>(outR, outI);
    // 				break;
    // 			case StepType::finalOrderN:
    // 				finalPassN<inverse>(outR, outI);
    // 				break;
    // 		}
    // 	}

    #[allow(clippy::indexing_slicing)]
    fn finalPass2(&self, f0: &mut [Complex<f32>]) {
        for i in 0..self.innerSize {
            let a = f0[i];
            let b = f0[self.innerSize + i];

            f0[i] = a + b;
            f0[self.innerSize + i] = a - b;
        }
    }

    #[allow(clippy::indexing_slicing)]
    fn finalPass2_split_complex(&self, f0r: &mut [f32], f0i: &mut [f32]) {
        for i in 0..self.innerSize {
            let ar = f0r[i];
            let ai = f0i[i];
            let br = f0r[self.innerSize + i];
            let bi = f0i[self.innerSize + i];

            f0r[i] = ar + br;
            f0i[i] = ai + bi;
            f0r[self.innerSize + i] = ar - br;
            f0i[self.innerSize + i] = ai - bi;
        }
    }

    #[allow(clippy::indexing_slicing)]
    fn finalPass3<const INVERSE: bool>(&self, f0: &mut [Complex<f32>]) {
        let tw1 = Complex::new(-0.5, -(0.75).sqrt() * (if INVERSE { -1.0 } else { 1.0 }));

        for i in 0..self.innerSize {
            let a = f0[i];
            let b = f0[self.innerSize + i];
            let c = f0[self.innerSize * 2 + i];

            let bc0 = b + c;
            let bc1 = b - c;

            f0[i] = a + bc0;
            f0[self.innerSize + i] = Complex::new(
                a.re + bc0.re * tw1.re - bc1.im * tw1.im,
                a.im + bc0.im * tw1.re + bc1.re * tw1.im,
            );
            f0[self.innerSize * 2 + i] = Complex::new(
                a.re + bc0.re * tw1.re + bc1.im * tw1.im,
                a.im + bc0.im * tw1.re - bc1.re * tw1.im,
            );
        }
    }

    #[allow(clippy::indexing_slicing, clippy::similar_names)]
    fn finalPass3_split_complex<const INVERSE: bool>(&self, f0r: &mut [f32], f0i: &mut [f32]) {
        let tw1r = -0.5;
        let tw1i = -(0.75).sqrt() * (if INVERSE { -1.0 } else { 1.0 });

        for i in 0..self.innerSize {
            let ar = f0r[i];
            let ai = f0i[i];
            let br = f0r[self.innerSize + i];
            let bi = f0i[self.innerSize + i];
            let cr = f0r[self.innerSize * 2 + i];
            let ci = f0i[self.innerSize * 2 + i];

            f0r[i] = ar + br + cr;
            f0i[i] = ai + bi + ci;
            f0r[self.innerSize + i] = ar + br * tw1r - bi * tw1i + cr * tw1r + ci * tw1i;
            f0i[self.innerSize + i] = ai + bi * tw1r + br * tw1i - cr * tw1i + ci * tw1r;
            f0r[self.innerSize * 2 + i] = ar + br * tw1r + bi * tw1i + cr * tw1r - ci * tw1i;
            f0i[self.innerSize * 2 + i] = ai + bi * tw1r - br * tw1i + cr * tw1i + ci * tw1r;
        }
    }

    #[allow(clippy::indexing_slicing)]
    fn finalPass4<const INVERSE: bool>(&self, f0: &mut [Complex<f32>]) {
        for i in 0..self.innerSize {
            let a = f0[i];
            let b = f0[self.innerSize + i];
            let c = f0[self.innerSize * 2 + i];
            let d = f0[self.innerSize * 3 + i];

            let ac0 = a + c;
            let ac1 = a - c;
            let bd0 = b + d;
            let bd1 = if INVERSE { b - d } else { d - b };
            let bd1i = Complex::new(-bd1.im, bd1.re);

            f0[i] = ac0 + bd0;
            f0[self.innerSize + i] = ac1 + bd1i;
            f0[self.innerSize * 2 + i] = ac0 - bd0;
            f0[self.innerSize * 3 + i] = ac1 - bd1i;
        }
    }

    #[allow(clippy::indexing_slicing, clippy::similar_names)]
    fn finalPass4_split_complex<const INVERSE: bool>(&self, f0r: &mut [f32], f0i: &mut [f32]) {
        for i in 0..self.innerSize {
            let ar = f0r[i];
            let ai = f0i[i];
            let br = f0r[self.innerSize + i];
            let bi = f0i[self.innerSize + i];
            let cr = f0r[self.innerSize * 2 + i];
            let ci = f0i[self.innerSize * 2 + i];
            let dr = f0r[self.innerSize * 3 + i];
            let di = f0i[self.innerSize * 3 + i];

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
            f0r[self.innerSize + i] = if INVERSE { ac1r - bd1i } else { ac1r + bd1i };
            f0i[self.innerSize + i] = if INVERSE { ac1i + bd1r } else { ac1i - bd1r };
            f0r[self.innerSize * 2 + i] = ac0r - bd0r;
            f0i[self.innerSize * 2 + i] = ac0i - bd0i;
            f0r[self.innerSize * 3 + i] = if INVERSE { ac1r + bd1i } else { ac1r - bd1i };
            f0i[self.innerSize * 3 + i] = if INVERSE { ac1i - bd1r } else { ac1i + bd1r };
        }
    }

    #[allow(
        clippy::indexing_slicing,
        clippy::similar_names,
        clippy::excessive_precision,
        clippy::unreadable_literal,
        clippy::many_single_char_names
    )]
    fn finalPass5<const INVERSE: bool>(&self, f0: &mut [Complex<f32>]) {
        let tw1r = 0.30901699437494745;
        let tw1i = -0.9510565162951535 * (if INVERSE { -1.0 } else { 1.0 });
        let tw2r = -0.8090169943749473;
        let tw2i = -0.5877852522924732 * (if INVERSE { -1.0 } else { 1.0 });

        for i in 0..self.innerSize {
            let a = f0[i];
            let b = f0[self.innerSize + i];
            let c = f0[self.innerSize * 2 + i];
            let d = f0[self.innerSize * 3 + i];
            let e = f0[self.innerSize * 4 + i];

            let be0 = b + e;
            let be1 = Complex::new(e.im - b.im, b.re - e.re); //(b - e)*i
            let cd0 = c + d;
            let cd1 = Complex::new(d.im - c.im, c.re - d.re);

            let bcde01 = be0 * tw1r + cd0 * tw2r;
            let bcde02 = be0 * tw2r + cd0 * tw1r;
            let bcde11 = be1 * tw1i + cd1 * tw2i;
            let bcde12 = be1 * tw2i - cd1 * tw1i;

            f0[i] = a + be0 + cd0;
            f0[self.innerSize + i] = a + bcde01 + bcde11;
            f0[self.innerSize * 2 + i] = a + bcde02 + bcde12;
            f0[self.innerSize * 3 + i] = a + bcde02 - bcde12;
            f0[self.innerSize * 4 + i] = a + bcde01 - bcde11;
        }
    }

    #[allow(
        clippy::indexing_slicing,
        clippy::similar_names,
        clippy::excessive_precision,
        clippy::unreadable_literal
    )]
    fn finalPass5_split_complex<const INVERSE: bool>(&self, f0r: &mut [f32], f0i: &mut [f32]) {
        let tw1r = 0.30901699437494745;
        let tw1i = -0.9510565162951535 * (if INVERSE { -1.0 } else { 1.0 });
        let tw2r = -0.8090169943749473;
        let tw2i = -0.5877852522924732 * (if INVERSE { -1.0 } else { 1.0 });

        for i in 0..self.innerSize {
            let ar = f0r[i];
            let ai = f0i[i];
            let br = f0r[self.innerSize + i];
            let bi = f0i[self.innerSize + i];
            let cr = f0r[self.innerSize * 2 + i];
            let ci = f0i[self.innerSize * 2 + i];
            let dr = f0r[self.innerSize * 3 + i];
            let di = f0i[self.innerSize * 3 + i];
            let er = f0r[self.innerSize * 4 + i];
            let ei = f0i[self.innerSize * 4 + i];

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
            f0r[self.innerSize + i] = ar + bcde01r + bcde11r;
            f0i[self.innerSize + i] = ai + bcde01i + bcde11i;
            f0r[self.innerSize * 2 + i] = ar + bcde02r + bcde12r;
            f0i[self.innerSize * 2 + i] = ai + bcde02i + bcde12i;
            f0r[self.innerSize * 3 + i] = ar + bcde02r - bcde12r;
            f0i[self.innerSize * 3 + i] = ai + bcde02i - bcde12i;
            f0r[self.innerSize * 4 + i] = ar + bcde01r - bcde11r;
            f0i[self.innerSize * 4 + i] = ai + bcde01i - bcde11i;
        }
    }

    #[allow(clippy::indexing_slicing)]
    fn finalPassN<const INVERSE: bool>(&mut self, f0: &mut [Complex<f32>]) {
        for i in 0..self.innerSize {
            let mut sum = Complex::new(0.0, 0.0);

            for i2 in 0..self.outerSize {
                let tmp_value = f0[i + i2 * self.innerSize];

                self.dftTmp[i2] = tmp_value;
                sum += tmp_value;
            }

            f0[i] = sum;

            for f in 1..self.outerSize {
                let mut sum = self.dftTmp[0];

                for i2 in 1..self.outerSize {
                    let twist_index = (i2 * f) % self.outerSize;
                    let twist = if INVERSE {
                        self.dftTwists[twist_index].conj()
                    } else {
                        self.dftTwists[twist_index]
                    };

                    sum += Complex::new(
                        self.dftTmp[i2].re * twist.re - self.dftTmp[i2].im * twist.im,
                        self.dftTmp[i2].im * twist.re + self.dftTmp[i2].re * twist.im,
                    );
                }

                f0[i + f * self.innerSize] = sum;
            }
        }
    }

    #[allow(clippy::indexing_slicing)]
    fn finalPassN_split_complex<const INVERSE: bool>(&mut self, f0r: &mut [f32], f0i: &mut [f32]) {
        let (tmp_r, tmp_i) = complex_to_two_float(&mut self.dftTmp);

        for i in 0..self.innerSize {
            let mut sum_r = 0.0;
            let mut sum_i = 0.0;

            for i2 in 0..self.outerSize {
                let tmp_value_r = f0r[i + i2 * self.innerSize];
                let tmp_value_i = f0i[i + i2 * self.innerSize];

                tmp_r[i2] = tmp_value_r;
                sum_r += tmp_value_r;
                tmp_i[i2] = tmp_value_i;
                sum_i += tmp_value_i;
            }

            f0r[i] = sum_r;
            f0i[i] = sum_i;

            for f in 1..self.outerSize {
                let mut sum_r = tmp_r[0];
                let mut sum_i = tmp_i[0];

                for i2 in 1..self.outerSize {
                    let twist_index = (i2 * f) % self.outerSize;

                    let twist = if INVERSE {
                        self.dftTwists[twist_index].conj()
                    } else {
                        self.dftTwists[twist_index]
                    };

                    sum_r += tmp_r[i2] * twist.re - tmp_i[i2] * twist.im;
                    sum_i += tmp_i[i2] * twist.re + tmp_r[i2] * twist.im;
                }

                f0r[i + f * self.innerSize] = sum_r;
                f0i[i + f * self.innerSize] = sum_i;
            }
        }
    }
}
