use num::{Float, complex::Complex};

// /// A Real FFT which can handle multiples of 3 and 5, and can be computed in
// chunks template<typename Sample, bool splitComputation=false, bool
// halfBinShift=false> struct RealFFT {
// 	using Complex = std::complex<Sample>;
// 	static constexpr bool prefersSplit = SplitFFT<Sample,
// splitComputation>::prefersSplit;

// 	static size_t fastSizeAbove(size_t size) {
// 		return ComplexFFT::fastSizeAbove((size + 1)/2)*2;
// 	}

// 	RealFFT(size_t size=0) {
// 		resize(size);
// 	}

// 	void resize(size_t size) {
// 		size_t hSize = size/2;
// 		complexFft.resize(hSize);
// 		tmpFreq.resize(hSize);
// 		tmpTime.resize(hSize);

// 		twiddles.resize(hSize/2 + 1);

// 		if (!halfBinShift) {
// 			for (size_t i = 0; i < twiddles.size(); ++i) {
// 				Sample rotPhase = i*(-2*M_PI/size) - M_PI/2; // bake rotation by (-i)
// into twiddles 				twiddles[i] = std::polar(Sample(1), rotPhase);
// 			}
// 		} else {
// 			for (size_t i = 0; i < twiddles.size(); ++i) {
// 				Sample rotPhase = (i + 0.5)*(-2*M_PI/size) - M_PI/2;
// 				twiddles[i] = std::polar(Sample(1), rotPhase);
// 			}

// 			halfBinTwists.resize(hSize);
// 			for (size_t i = 0; i < hSize; ++i) {
// 				Sample twistPhase = -2*M_PI*i/size;
// 				halfBinTwists[i] = std::polar(Sample(1), twistPhase);
// 			}
// 		}
// 	}

// 	size_t size() const {
// 		return complexFft.size()*2;
// 	}
// 	size_t steps() const {
// 		return complexFft.steps() + (splitComputation ? 3 : 2);
// 	}

// 	void fft(const Sample *time, Complex *freq) {
// 		for (size_t s = 0; s < steps(); ++s) {
// 			fft(s, time, freq);
// 		}
// 	}
// 	void fft(size_t step, const Sample *time, Complex *freq) {
// 		if (complexPrefersSplit) {
// 			size_t hSize = complexFft.size();
// 			Sample *tmpTimeR = (Sample *)tmpTime.data(), *tmpTimeI = tmpTimeR + hSize;
// 			Sample *tmpFreqR = (Sample *)tmpFreq.data(), *tmpFreqI = tmpFreqR + hSize;
// 			if (step-- == 0) {
// 				size_t hSize = complexFft.size();
// 				if (halfBinShift) {
// 					for (size_t i = 0; i < hSize; ++i) {
// 						Sample tr = time[2*i], ti = time[2*i + 1];
// 						Complex twist = halfBinTwists[i];
// 						tmpTimeR[i] = tr*twist.re - ti*twist.im;
// 						tmpTimeI[i] = ti*twist.re + tr*twist.im;
// 					}
// 				} else {
// 					for (size_t i = 0; i < hSize; ++i) {
// 						tmpTimeR[i] = time[2*i];
// 						tmpTimeI[i] = time[2*i + 1];
// 					}
// 				}
// 			} else if (step < complexFft.steps()) {
// 				complexFft.fft(step, tmpTimeR, tmpTimeI, tmpFreqR, tmpFreqI);
// 			} else {
// 				if (!halfBinShift) {
// 					Sample bin0r = tmpFreqR[0], bin0i = tmpFreqI[0];
// 					freq[0] = {bin0r + bin0i, bin0r - bin0i};
// 				}

// 				size_t startI = halfBinShift ? 0 : 1;
// 				size_t endI = hSize/2 + 1;
// 				if (splitComputation) { // Do this last twiddle in two halves
// 					if (step == complexFft.steps()) {
// 						endI = (startI + endI)/2;
// 					} else {
// 						startI = (startI + endI)/2;
// 					}
// 				}
// 				for (size_t i = startI; i < endI; ++i) {
// 					size_t conjI = halfBinShift ? (hSize - 1 - i) : (hSize - i);
// 					Complex twiddle = twiddles[i];

// 					Sample oddR = (tmpFreqR[i] + tmpFreqR[conjI])*Sample(0.5);
// 					Sample oddI = (tmpFreqI[i] - tmpFreqI[conjI])*Sample(0.5);
// 					Sample evenIR = (tmpFreqR[i] - tmpFreqR[conjI])*Sample(0.5);
// 					Sample evenII = (tmpFreqI[i] + tmpFreqI[conjI])*Sample(0.5);
// 					Sample evenRotMinusIR = evenIR*twiddle.re - evenII*twiddle.im;
// 					Sample evenRotMinusII = evenII*twiddle.re + evenIR*twiddle.im;

// 					freq[i] = {oddR + evenRotMinusIR, oddI + evenRotMinusII};
// 					freq[conjI] = {oddR - evenRotMinusIR, evenRotMinusII - oddI};
// 				}
// 			}
// 		} else {
// 			bool canUseTime = !halfBinShift && !(size_t(time)%alignof(Complex));
// 			if (step-- == 0) {
// 				size_t hSize = complexFft.size();
// 				if (halfBinShift) {
// 					for (size_t i = 0; i < hSize; ++i) {
// 						Sample tr = time[2*i], ti = time[2*i + 1];
// 						Complex twist = halfBinTwists[i];
// 						tmpTime[i] = {
// 							tr*twist.re - ti*twist.im,
// 							ti*twist.re + tr*twist.im
// 						};
// 					}
// 				} else if (!canUseTime) {
// 					std::memcpy(tmpTime.data(), time, sizeof(Complex)*hSize);
// 				}
// 			} else if (step < complexFft.steps()) {
// 				complexFft.fft(step, canUseTime ? (const Complex *)time : tmpTime.data(),
// tmpFreq.data()); 			} else {
// 				if (!halfBinShift) {
// 					Complex bin0 = tmpFreq[0];
// 					freq[0] = { // pack DC & Nyquist together
// 						bin0.re + bin0.im,
// 						bin0.re - bin0.im
// 					};
// 				}

// 				size_t hSize = complexFft.size();
// 				size_t startI = halfBinShift ? 0 : 1;
// 				size_t endI = hSize/2 + 1;
// 				if (splitComputation) { // Do this last twiddle in two halves
// 					if (step == complexFft.steps()) {
// 						endI = (startI + endI)/2;
// 					} else {
// 						startI = (startI + endI)/2;
// 					}
// 				}
// 				for (size_t i = startI; i < endI; ++i) {
// 					size_t conjI = halfBinShift ? (hSize - 1 - i) : (hSize - i);
// 					Complex twiddle = twiddles[i];

// 					Complex odd = (tmpFreq[i] + std::conj(tmpFreq[conjI]))*Sample(0.5);
// 					Complex evenI = (tmpFreq[i] - std::conj(tmpFreq[conjI]))*Sample(0.5);
// 					Complex evenRotMinusI = { // twiddle includes a factor of -i
// 						evenI.re*twiddle.re - evenI.im*twiddle.im,
// 						evenI.im*twiddle.re + evenI.re*twiddle.im
// 					};

// 					freq[i] = odd + evenRotMinusI;
// 					freq[conjI] = {odd.re - evenRotMinusI.re, evenRotMinusI.im -
// odd.im}; 				}
// 			}
// 		}
// 	}
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
// private:
// 	using ComplexFFT = SplitFFT<Sample, splitComputation>;
// 	ComplexFFT complexFft;

// 	static constexpr bool complexPrefersSplit = ComplexFFT::prefersSplit;
// 	std::vector<Complex> tmpFreq, tmpTime;
// 	std::vector<Complex> twiddles, halfBinTwists;
// };
