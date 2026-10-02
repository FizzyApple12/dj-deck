// use num::{Float, complex::Complex};

// A default power-of-2 FFT, specialised with platform-specific fast
// implementations where available template<typename Sample>
// struct Pow2RealFFT : public SimpleRealFFT<Sample> {
// 	static constexpr bool prefersSplit = SimpleRealFFT<Sample>::prefersSplit;

// 	using SimpleRealFFT<Sample>::SimpleRealFFT;

// 	// Prevent copying, since it might be a problem for specialisations
// 	Pow2RealFFT(const Pow2RealFFT &other) = delete;
// 	// Pass move-constructor through, just to be explicit about it
// 	Pow2RealFFT(Pow2RealFFT &&other) : SimpleRealFFT<Sample>(std::move(other))
// {} }

pub struct Pow2RealFFT {}

// // Override `Pow2FFT` / `Pow2RealFFT` templates with faster implementations
// #if defined(SIGNALSMITH_USE_PFFFT) || defined(SIGNALSMITH_USE_PFFFT_DOUBLE)
// #	if defined(SIGNALSMITH_USE_PFFFT)
// #		include "./platform/fft-pffft.h"
// #	endif
// #	if defined(SIGNALSMITH_USE_PFFFT_DOUBLE)
// #		include "./platform/fft-pffft-double.h"
// #	endif
// #elif defined(SIGNALSMITH_USE_ACCELERATE)
// #	include "./platform/fft-accelerate.h"
// #elif defined(SIGNALSMITH_USE_IPP)
// #	include "./platform/fft-ipp.h"
// #endif
