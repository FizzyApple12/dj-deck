use num::{Float, complex::Complex};

// /// A default power-of-2 FFT, specialised with platform-specific fast
// implementations where available template<typename Sample>
// struct Pow2RealFFT : public SimpleRealFFT<Sample> {
// 	static constexpr bool prefersSplit = SimpleRealFFT<Sample>::prefersSplit;

// 	using SimpleRealFFT<Sample>::SimpleRealFFT;

// 	// Prevent copying, since it might be a problem for specialisations
// 	Pow2RealFFT(const Pow2RealFFT &other) = delete;
// 	// Pass move-constructor through, just to be explicit about it
// 	Pow2RealFFT(Pow2RealFFT &&other) : SimpleRealFFT<Sample>(std::move(other))
// {} };
