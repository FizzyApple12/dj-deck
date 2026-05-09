use num::{Float, complex::Complex};

use crate::linear::{
    complex_to_two_float_mut,
    fft::split_fft::{SplitFFT, SplitFFTTrait},
};

// A Real FFT which can handle multiples of 3 and 5, and can be computed in
// chunks

// template<typename Sample, bool splitComputation=false, bool
// halfBinShift=false>
pub struct RealFFT<Sample, const SPLIT_COMPUTATION: bool, const HALF_BIN_SHIFT: bool>
where
    Sample: Float,
{
    complexFft: SplitFFT<Sample, SPLIT_COMPUTATION>,

    tmpFreq: Vec<Complex<Sample>>,
    tmpTime: Vec<Complex<Sample>>,
    twiddles: Vec<Complex<Sample>>,
    halfBinTwists: Vec<Complex<Sample>>,
}

pub trait RealFFTTrait<Sample, const SPLIT_COMPUTATION: bool, const HALF_BIN_SHIFT: bool>
where
    Sample: Float + Default,
{
    const PREFERS_SPLIT: bool;

    fn fast_size_above(size: usize) -> usize;

    fn new(size: usize) -> Self;

    fn resize(&mut self, size: usize);

    fn size(&self) -> usize;
    fn steps(&self) -> usize;

    fn fft(&mut self, time: &[f32], freq: &mut [Complex<f32>]);
    fn fft_step(&mut self, step: usize, time: &[f32], freq: &mut [Complex<f32>]);
    // fn fft_split_complex(
    //     &mut self,
    //     in_r: &[f32],
    //     in_i: &[f32],
    //     out_r: &mut [f32],
    //     out_i: &mut [f32],
    // );
    // fn fft_step_split_complex(
    //     &mut self,
    //     step: usize,
    //     in_r: &[f32],
    //     in_i: &[f32],
    //     out_r: &mut [f32],
    //     out_i: &mut [f32],
    // );

    // fn ifft(&mut self, time: &[Complex<f32>], freq: &mut [Complex<f32>]);
    // fn ifft_step(&mut self, step: usize, time: &[Complex<f32>], freq: &mut
    // [Complex<f32>]); fn ifft_split_complex(
    //     &mut self,
    //     in_r: &[f32],
    //     in_i: &[f32],
    //     out_r: &mut [f32],
    //     out_i: &mut [f32],
    // );
    // fn ifft_step_split_complex(
    //     &mut self,
    //     step: usize,
    //     in_r: &[f32],
    //     in_i: &[f32],
    //     out_r: &mut [f32],
    //     out_i: &mut [f32],
    // );
}

impl<const SPLIT_COMPUTATION: bool, const HALF_BIN_SHIFT: bool>
    RealFFTTrait<f32, SPLIT_COMPUTATION, HALF_BIN_SHIFT>
    for RealFFT<f32, SPLIT_COMPUTATION, HALF_BIN_SHIFT>
{
    const PREFERS_SPLIT: bool = SplitFFT::<f32, SPLIT_COMPUTATION>::PREFERS_SPLIT;

    fn fast_size_above(size: usize) -> usize {
        SplitFFT::<f32, SPLIT_COMPUTATION>::fast_size_above(size.div_ceil(2)) * 2
    }

    fn new(size: usize) -> Self {
        let mut new = Self {
            complexFft: SplitFFT::<f32, SPLIT_COMPUTATION>::new(size),

            tmpFreq: Vec::new(),
            tmpTime: Vec::new(),
            twiddles: Vec::new(),
            halfBinTwists: Vec::new(),
        };

        new.resize(size);

        new
    }

    #[allow(clippy::indexing_slicing, clippy::cast_precision_loss)]
    fn resize(&mut self, size: usize) {
        let h_size = size / 2;

        self.complexFft.resize(h_size);
        self.tmpFreq.resize(h_size, Complex { re: 0.0, im: 0.0 });
        self.tmpTime.resize(h_size, Complex { re: 0.0, im: 0.0 });

        self.twiddles
            .resize(h_size / 2 + 1, Complex { re: 0.0, im: 0.0 });

        if HALF_BIN_SHIFT {
            for i in 0..self.twiddles.len() {
                let rot_phase = (i as f32 + 0.5) * (-2.0 * std::f32::consts::PI / size as f32)
                    - std::f32::consts::PI / 2.0;

                self.twiddles[i] = Complex::from_polar(1.0, rot_phase);
            }

            self.halfBinTwists
                .resize(h_size, Complex { re: 0.0, im: 0.0 });

            for i in 0..h_size {
                let twist_phase = -2.0 * std::f32::consts::PI * i as f32 / size as f32;

                self.halfBinTwists[i] = Complex::from_polar(1.0, twist_phase);
            }
        } else {
            for i in 0..self.twiddles.len() {
                let rot_phase = i as f32 * (-2.0 * std::f32::consts::PI / size as f32)
                    - std::f32::consts::PI / 2.0; // bake rotation by (-i) into twiddles

                self.twiddles[i] = Complex::from_polar(1.0, rot_phase);
            }
        }
    }

    fn size(&self) -> usize {
        self.complexFft.size() * 2
    }

    fn steps(&self) -> usize {
        self.complexFft.steps() + if SPLIT_COMPUTATION { 3 } else { 2 }
    }

    fn fft(&mut self, time: &[f32], freq: &mut [Complex<f32>]) {
        for s in 0..self.steps() {
            self.fft_step(s, time, freq);
        }
    }

    fn fft_step(&mut self, mut step: usize, time: &[f32], freq: &mut [Complex<f32>]) {
        if Self::PREFERS_SPLIT {
            let hSize = self.complexFft.size();

            let (tmpTimeR, tmpTimeI) = complex_to_two_float_mut(&mut self.tmpTime);
            let (tmpFreqR, tmpFreqI) = complex_to_two_float_mut(&mut self.tmpFreq);

            // let tmpTimeR = self.tmpTime.data();
            // let tmpTimeI = tmpTimeR + hSize;
            // let tmpFreqR = self.tmpFreq.data();
            // let tmpFreqI = tmpFreqR + hSize;

            let step_check = step;
            step -= 1;

            if step_check == 0 {
                let hSize = self.complexFft.size();

                if HALF_BIN_SHIFT {
                    for i in 0..hSize {
                        let tr = time[2 * i];
                        let ti = time[2 * i + 1];

                        let twist = self.halfBinTwists[i];

                        tmpTimeR[i] = tr * twist.re - ti * twist.im;
                        tmpTimeI[i] = ti * twist.re + tr * twist.im;
                    }
                } else {
                    for i in 0..hSize {
                        tmpTimeR[i] = time[2 * i];
                        tmpTimeI[i] = time[2 * i + 1];
                    }
                }
            } else if step < self.complexFft.steps() {
                self.complexFft
                    .fft_step_split_complex(step, tmpTimeR, tmpTimeI, tmpFreqR, tmpFreqI);
            } else {
                if !HALF_BIN_SHIFT {
                    let bin0r = tmpFreqR[0];
                    let bin0i = tmpFreqI[0];

                    freq[0] = Complex::new(bin0r + bin0i, bin0r - bin0i);
                }

                let startI = if HALF_BIN_SHIFT { 0 } else { 1 };
                let endI = hSize / 2 + 1;

                if SPLIT_COMPUTATION {
                    // Do this last twiddle in two halves
                    if step == self.complexFft.steps() {
                        endI = (startI + endI) / 2;
                    } else {
                        startI = (startI + endI) / 2;
                    }
                }

                for i in startI..endI {
                    let conjI = if HALF_BIN_SHIFT {
                        hSize - 1 - i
                    } else {
                        hSize - i
                    };

                    let twiddle = self.twiddles[i];

                    let oddR = (tmpFreqR[i] + tmpFreqR[conjI]) * 0.5;
                    let oddI = (tmpFreqI[i] - tmpFreqI[conjI]) * 0.5;
                    let evenIR = (tmpFreqR[i] - tmpFreqR[conjI]) * 0.5;
                    let evenII = (tmpFreqI[i] + tmpFreqI[conjI]) * 0.5;

                    let evenRotMinusIR = evenIR * twiddle.re - evenII * twiddle.im;
                    let evenRotMinusII = evenII * twiddle.re + evenIR * twiddle.im;

                    freq[i] = Complex::new(oddR + evenRotMinusIR, oddI + evenRotMinusII);
                    freq[conjI] = Complex::new(oddR - evenRotMinusIR, evenRotMinusII - oddI);
                }
            }
        } else {
            let raw_time = &raw const time;

            let canUseTime = !HALF_BIN_SHIFT
                && ((raw_time as usize % std::mem::align_of::<Complex<f32>>()) == 0);

            let step_check = step;
            step -= 1;

            if step_check == 0 {
                let hSize = self.complexFft.size();

                if HALF_BIN_SHIFT {
                    for i in 0..hSize {
                        let tr = time[2 * i];
                        let ti = time[2 * i + 1];

                        let twist = self.halfBinTwists[i];

                        self.tmpTime[i] = Complex::new(
                            tr * twist.re - ti * twist.im,
                            ti * twist.re + tr * twist.im,
                        );
                    }
                } else if !canUseTime {
                    std::mem::copy(
                        &self.tmpTime,
                        time,
                        std::mem::size_of::<Complex<f32>>() * hSize,
                    );
                }
            } else if step < self.complexFft.steps() {
                self.complexFft.fft_step(
                    step,
                    if canUseTime { time } else { &self.tmpTime },
                    &mut self.tmpFreq,
                );
            } else {
                if !HALF_BIN_SHIFT {
                    let bin0 = self.tmpFreq[0];

                    freq[0] = Complex::new(
                        // pack DC & Nyquist together
                        bin0.re + bin0.im,
                        bin0.re - bin0.im,
                    );
                }

                let hSize = self.complexFft.size();

                let startI = if HALF_BIN_SHIFT { 0 } else { 1 };
                let endI = hSize / 2 + 1;

                if SPLIT_COMPUTATION {
                    // Do this last twiddle in two halves
                    if step == self.complexFft.steps() {
                        endI = (startI + endI) / 2;
                    } else {
                        startI = (startI + endI) / 2;
                    }
                }
                for i in startI..endI {
                    let conjI = if HALF_BIN_SHIFT {
                        hSize - 1 - i
                    } else {
                        hSize - i
                    };

                    let twiddle = self.twiddles[i];

                    let odd = (self.tmpFreq[i] + self.tmpFreq[conjI].conj()) * 0.5;
                    let evenI = (self.tmpFreq[i] - self.tmpFreq[conjI].conj()) * 0.5;

                    let evenRotMinusI = Complex::new(
                        // twiddle includes a factor of -i
                        evenI.re * twiddle.re - evenI.im * twiddle.im,
                        evenI.im * twiddle.re + evenI.re * twiddle.im,
                    );

                    freq[i] = odd + evenRotMinusI;
                    freq[conjI] =
                        Complex::new(odd.re - evenRotMinusI.re, evenRotMinusI.im - odd.im);
                }
            }
        }
    }
}

// 	void fft(const Sample *inR, Sample *outR, Sample *outI) {
// 		for (size_t s = 0; s < steps(); ++s) {
// 			fft(s, inR, outR, outI);
// 		}
// 	}
// 	void fft(size_t step, const Sample *inR, Sample *outR, Sample *outI) {
// 		size_t hSize = complexFft.size();
// 		Sample *tmpTimeR = (Sample *)tmpTime.data(), *tmpTimeI = tmpTimeR + hSize;
// 		Sample *tmpFreqR = (Sample *)tmpFreq.data(), *tmpFreqI = tmpFreqR + hSize;
// 		if (step-- == 0) {
// 			size_t hSize = complexFft.size();
// 			if (halfBinShift) {
// 				for (size_t i = 0; i < hSize; ++i) {
// 					Sample tr = inR[2*i], ti = inR[2*i + 1];
// 					Complex twist = halfBinTwists[i];
// 					tmpTimeR[i] = tr*twist.re - ti*twist.im;
// 					tmpTimeI[i] = ti*twist.re + tr*twist.im;
// 				}
// 			} else {
// 				for (size_t i = 0; i < hSize; ++i) {
// 					tmpTimeR[i] = inR[2*i];
// 					tmpTimeI[i] = inR[2*i + 1];
// 				}
// 			}
// 		} else if (step < complexFft.steps()) {
// 			complexFft.fft(step, tmpTimeR, tmpTimeI, tmpFreqR, tmpFreqI);
// 		} else {
// 			if (!halfBinShift) {
// 				Sample bin0r = tmpFreqR[0], bin0i = tmpFreqI[0];
// 				outR[0] = bin0r + bin0i;
// 				outI[0] = bin0r - bin0i;
// 			}

// 			size_t startI = halfBinShift ? 0 : 1;
// 			size_t endI = hSize/2 + 1;
// 			if (splitComputation) { // Do this last twiddle in two halves
// 				if (step == complexFft.steps()) {
// 					endI = (startI + endI)/2;
// 				} else {
// 					startI = (startI + endI)/2;
// 				}
// 			}
// 			for (size_t i = startI; i < endI; ++i) {
// 				size_t conjI = halfBinShift ? (hSize - 1 - i) : (hSize - i);
// 				Complex twiddle = twiddles[i];

// 				Sample oddR = (tmpFreqR[i] + tmpFreqR[conjI])*Sample(0.5);
// 				Sample oddI = (tmpFreqI[i] - tmpFreqI[conjI])*Sample(0.5);
// 				Sample evenIR = (tmpFreqR[i] - tmpFreqR[conjI])*Sample(0.5);
// 				Sample evenII = (tmpFreqI[i] + tmpFreqI[conjI])*Sample(0.5);
// 				Sample evenRotMinusIR = evenIR*twiddle.re - evenII*twiddle.im;
// 				Sample evenRotMinusII = evenII*twiddle.re + evenIR*twiddle.im;

// 				outR[i] = oddR + evenRotMinusIR;
// 				outI[i] = oddI + evenRotMinusII;
// 				outR[conjI] = oddR - evenRotMinusIR;
// 				outI[conjI] = evenRotMinusII - oddI;
// 			}
// 		}
// 	}

// 	void ifft(const Complex *freq, Sample *time) {
// 		for (size_t s = 0; s < steps(); ++s) {
// 			ifft(s, freq, time);
// 		}
// 	}
// 	void ifft(size_t step, const Complex *freq, Sample *time) {
// 		if (complexPrefersSplit) {
// 			size_t hSize = complexFft.size();
// 			Sample *tmpTimeR = (Sample *)tmpTime.data(), *tmpTimeI = tmpTimeR + hSize;
// 			Sample *tmpFreqR = (Sample *)tmpFreq.data(), *tmpFreqI = tmpFreqR + hSize;

// 			bool splitFirst = splitComputation && (step-- == 0);
// 			if (splitFirst || step-- == 0) {
// 				Complex bin0 = freq[0];
// 				if (!halfBinShift) {
// 					tmpFreqR[0] = bin0.re + bin0.im;
// 					tmpFreqI[0] = bin0.re - bin0.im;
// 				}
// 				size_t startI = halfBinShift ? 0 : 1;
// 				size_t endI = hSize/2 + 1;
// 				if (splitComputation) { // Do this first twiddle in two halves
// 					if (splitFirst) {
// 						endI = (startI + endI)/2;
// 					} else {
// 						startI = (startI + endI)/2;
// 					}
// 				}
// 				for (size_t i = startI; i < endI; ++i) {
// 					size_t conjI = halfBinShift ? (hSize - 1 - i) : (hSize - i);
// 					Complex twiddle = twiddles[i];

// 					Complex odd = freq[i] + std::conj(freq[conjI]);
// 					Complex evenRotMinusI = freq[i] - std::conj(freq[conjI]);
// 					Complex evenI = { // Conjugate twiddle
// 						evenRotMinusI.re*twiddle.re +
// evenRotMinusI.im*twiddle.im, 						evenRotMinusI.im*twiddle.
// real() - evenRotMinusI.re*twiddle.im 					};

// 					tmpFreqR[i] = odd.re + evenI.re;
// 					tmpFreqI[i] = odd.im + evenI.im;
// 					tmpFreqR[conjI] = odd.re - evenI.re;
// 					tmpFreqI[conjI] = evenI.im - odd.im;
// 				}
// 			} else if (step < complexFft.steps()) {
// 				complexFft.ifft(step, tmpFreqR, tmpFreqI, tmpTimeR, tmpTimeI);
// 			} else {
// 				size_t hSize = complexFft.size();
// 				if (halfBinShift) {
// 					for (size_t i = 0; i < hSize; ++i) {
// 						Sample tr = tmpTimeR[i], ti = tmpTimeI[i];
// 						Complex twist = halfBinTwists[i];
// 						time[2*i] = 	tr*twist.re + ti*twist.im;
// 						time[2*i + 1] = ti*twist.re - tr*twist.im;
// 					}
// 				} else {
// 					for (size_t i = 0; i < hSize; ++i) {
// 						time[2*i] = tmpTimeR[i];
// 						time[2*i + 1] = tmpTimeI[i];
// 					}
// 				}
// 			}
// 		} else {
// 			bool canUseTime = !halfBinShift && !(size_t(time)%alignof(Complex));
// 			bool splitFirst = splitComputation && (step-- == 0);
// 			if (splitFirst || step-- == 0) {
// 				Complex bin0 = freq[0];
// 				if (!halfBinShift) {
// 					tmpFreq[0] = {
// 						bin0.re + bin0.im,
// 						bin0.re - bin0.im
// 					};
// 				}
// 				size_t hSize = complexFft.size();
// 				size_t startI = halfBinShift ? 0 : 1;
// 				size_t endI = hSize/2 + 1;
// 				if (splitComputation) { // Do this first twiddle in two halves
// 					if (splitFirst) {
// 						endI = (startI + endI)/2;
// 					} else {
// 						startI = (startI + endI)/2;
// 					}
// 				}
// 				for (size_t i = startI; i < endI; ++i) {
// 					size_t conjI = halfBinShift ? (hSize - 1 - i) : (hSize - i);
// 					Complex twiddle = twiddles[i];

// 					Complex odd = freq[i] + std::conj(freq[conjI]);
// 					Complex evenRotMinusI = freq[i] - std::conj(freq[conjI]);
// 					Complex evenI = { // Conjugate twiddle
// 						evenRotMinusI.re*twiddle.re +
// evenRotMinusI.im*twiddle.im, 						evenRotMinusI.im*twiddle.
// real() - evenRotMinusI.re*twiddle.im 					};

// 					tmpFreq[i] = odd + evenI;
// 					tmpFreq[conjI] = {odd.re - evenI.re, evenI.im - odd.im};
// 				}
// 			} else if (step < complexFft.steps()) {
// 				// Can't just use time as (Complex *), since it might not be aligned
// properly 				complexFft.ifft(step, tmpFreq.data(), canUseTime ? (Complex
// *)time : tmpTime.data()); 			} else {
// 				size_t hSize = complexFft.size();
// 				if (halfBinShift) {
// 					for (size_t i = 0; i < hSize; ++i) {
// 						Complex t = tmpTime[i];
// 						Complex twist = halfBinTwists[i];
// 						time[2*i] = 	t.re*twist.re + t.im*twist.im;
// 						time[2*i + 1] = t.im*twist.re - t.re*twist.im;
// 					}
// 				} else if (!canUseTime) {
// 					std::memcpy(time, tmpTime.data(), sizeof(Complex)*hSize);
// 				}
// 			}
// 		}
// 	}
// 	void ifft(const Sample *inR, const Sample *inI, Sample *outR) {
// 		for (size_t s = 0; s < steps(); ++s) {
// 			ifft(s, inR, inI, outR);
// 		}
// 	}
// 	void ifft(size_t step, const Sample *inR, const Sample *inI, Sample *outR) {
// 		size_t hSize = complexFft.size();
// 		Sample *tmpTimeR = (Sample *)tmpTime.data(), *tmpTimeI = tmpTimeR + hSize;
// 		Sample *tmpFreqR = (Sample *)tmpFreq.data(), *tmpFreqI = tmpFreqR + hSize;

// 		bool splitFirst = splitComputation && (step-- == 0);
// 		if (splitFirst || step-- == 0) {
// 			Sample bin0r = inR[0], bin0i = inI[0];
// 			if (!halfBinShift) {
// 				tmpFreqR[0] = bin0r + bin0i;
// 				tmpFreqI[0] = bin0r - bin0i;
// 			}
// 			size_t startI = halfBinShift ? 0 : 1;
// 			size_t endI = hSize/2 + 1;
// 			if (splitComputation) { // Do this first twiddle in two halves
// 				if (splitFirst) {
// 					endI = (startI + endI)/2;
// 				} else {
// 					startI = (startI + endI)/2;
// 				}
// 			}
// 			for (size_t i = startI; i < endI; ++i) {
// 				size_t conjI = halfBinShift ? (hSize - 1 - i) : (hSize - i);
// 				Complex twiddle = twiddles[i];
// 				Sample fir = inR[i], fii = inI[i];
// 				Sample fcir = inR[conjI], fcii = inI[conjI];

// 				Complex odd = {fir + fcir, fii - fcii};
// 				Complex evenRotMinusI = {fir - fcir, fii + fcii};
// 				Complex evenI = { // Conjugate twiddle
// 					evenRotMinusI.re*twiddle.re +
// evenRotMinusI.im*twiddle.im, 					evenRotMinusI.im*twiddle.re
// - evenRotMinusI.re*twiddle.im 				};

// 				tmpFreqR[i] = odd.re + evenI.re;
// 				tmpFreqI[i] = odd.im + evenI.im;
// 				tmpFreqR[conjI] = odd.re - evenI.re;
// 				tmpFreqI[conjI] = evenI.im - odd.im;
// 			}
// 		} else if (step < complexFft.steps()) {
// 			// Can't just use time as (Complex *), since it might not be aligned
// properly 			complexFft.ifft(step, tmpFreqR, tmpFreqI, tmpTimeR, tmpTimeI);
// 		} else {
// 			if (halfBinShift) {
// 				for (size_t i = 0; i < hSize; ++i) {
// 					Sample tr = tmpTimeR[i], ti = tmpTimeI[i];
// 					Complex twist = halfBinTwists[i];
// 					outR[2*i] = 	tr*twist.re + ti*twist.im;
// 					outR[2*i + 1] = ti*twist.re - tr*twist.im;
// 				}
// 			} else {
// 				for (size_t i = 0; i < hSize; ++i) {
// 					outR[2*i] = tmpTimeR[i];
// 					outR[2*i + 1] = tmpTimeI[i];
// 				}
// 			}
// 		}
// 	}
// };
