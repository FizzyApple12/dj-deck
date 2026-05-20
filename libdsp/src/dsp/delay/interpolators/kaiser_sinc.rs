use std::marker::PhantomData;

use num::{Complex, Float, complex::ComplexFloat};

use crate::dsp::fft::FFTTrait;

use crate::dsp::{delay::interpolators::InterpolatorTrait, fft::fft::FFT, windows::kaiser::Kaiser};

/** Fixed-size Kaiser-windowed sinc interpolation.
If `minimumPhase` is enabled, a minimum-phase version of the kernel is used:
*/
pub struct InterpolatorKaiserSincN<Data, Sample, const N: usize, const MINIMUM_PHASE: bool>
where
    Sample: Float,
{
    sub_sample_steps: usize,
    coefficients: Vec<Sample>,
    phantom_data: PhantomData<Data>,
}

impl<Data, const N: usize, const MINIMUM_PHASE: bool>
    InterpolatorKaiserSincN<Data, f32, N, MINIMUM_PHASE>
{
    fn new() -> Self {
        Self::new_pass(0.5 - 0.45 / (N as f64).sqrt())
    }

    pub fn new_pass(pass_freq: f64) -> Self {
        Self::new_pass_stop(pass_freq, 1.0 - pass_freq)
    }

    pub fn new_pass_stop(pass_freq: f64, stop_freq: f64) -> Self {
        let sub_sample_steps = 2 * N; // Heuristic again.  Really it depends on the bandwidth as well.
        let mut kaiserBandwidth =
            (stop_freq - pass_freq) * (N as f64 + 1.0 / sub_sample_steps as f64);
        kaiserBandwidth += 1.25 / kaiserBandwidth; // We want to place the first zero, but (because using this to window a sinc essentially integrates it in the freq-domain), our ripples (and therefore zeroes) are out of phase.  This is a heuristic fix.
        let sincScale = std::f64::consts::PI * (pass_freq + stop_freq);

        let centreIndex = (N * sub_sample_steps) as f64 * 0.5;
        let scaleFactor = 1.0 / sub_sample_steps as f64;
        let windowedSinc: Vec<f32> = Vec::with_capacity(sub_sample_steps * N + 1);

        Kaiser::with_bandwidth(kaiserBandwidth, false).fill(&mut windowedSinc, windowedSinc.len());

        for i in 0..windowedSinc.len() {
            let x = (i as f64 - centreIndex) * scaleFactor;
            let intX = x.round();

            if (intX > f64::EPSILON || intX < f64::EPSILON) && (x - intX).abs() < 1e-6 {
                // Exact 0s
                windowedSinc[i] = 0.0;
            } else if x.abs() > 1e-6 {
                let p = x * sincScale;

                windowedSinc[i] *= (p.sin() / p) as f32;
            }
        }

        if MINIMUM_PHASE {
            let mut fft: FFT<f32> = FFT::<f32>::new(windowedSinc.len() * 2, 0);

            windowedSinc.resize(fft.size(), 0.0);

            let spectrum: Vec<Complex<f32>> = Vec::with_capacity(fft.size());
            let cepstrum: Vec<Complex<f32>> = Vec::with_capacity(fft.size());

            fft.fft(&windowedSinc, &mut spectrum);

            for i in 0..fft.size() {
                let complex_factor = (spectrum[i].abs() + 1e-30).ln();

                spectrum[i] = Complex::new(complex_factor, complex_factor);
            }

            fft.fft(&spectrum, &mut cepstrum);

            for i in 1..(fft.size() / 2) {
                cepstrum[i] *= 0.0;
            }

            for i in (fft.size() / 2 + 1)..fft.size() {
                cepstrum[i] *= 2.0;
            }

            let scaling = 1.0 / fft.size() as f32;

            fft.ifft(&cepstrum, &mut spectrum);

            for i in 0..fft.size() {
                let phase = spectrum[i].im * scaling;
                let mag = (spectrum[i].re * scaling).exp();

                spectrum[i] = Complex::<f32>::new(mag * phase.cos(), mag * phase.sin());
            }

            fft.ifft(&spectrum, &mut cepstrum);

            windowedSinc.resize(sub_sample_steps * N + 1, 0.0);
            windowedSinc.shrink_to_fit();

            for i in 0..windowedSinc.len() {
                windowedSinc[i] = cepstrum[i].re * scaling;
            }
        }

        // Re-order into FIR fractional-delay blocks
        let coefficients = Vec::new();

        coefficients.resize(N * (sub_sample_steps + 1), 0.0);

        for k in 0..sub_sample_steps {
            for i in 0..N {
                coefficients[k * N + i] =
                    windowedSinc[(sub_sample_steps - k) + i * sub_sample_steps];
            }
        }

        Self {
            sub_sample_steps,
            coefficients,
            phantom_data: PhantomData,
        }
    }
}

impl<const N: usize, const MINIMUM_PHASE: bool> InterpolatorTrait<&[f32], f32>
    for InterpolatorKaiserSincN<&[f32], f32, N, MINIMUM_PHASE>
{
    const INPUT_LENGTH: usize = N;
    // Because we're truncating, which rounds down too often
    const LATENCY: f32 = if MINIMUM_PHASE {
        0.0
    } else {
        N as f32 * 0.5 - 1.0
    };

    #[allow(clippy::indexing_slicing)]
    fn fractional(&self, data: &[f32], fractional: f32) -> f32 {
        let sub_sample_delay = fractional * self.sub_sample_steps as f32;
        let mut low_index = sub_sample_delay;

        if low_index >= self.sub_sample_steps as f32 {
            low_index = self.sub_sample_steps as f32 - 1.0;
        }

        let sub_sample_fractional = sub_sample_delay - low_index;
        let high_index = low_index + 1.0;

        let mut sum_low = 0.0;
        let mut sum_high = 0.0;

        for i in 0..N {
            sum_low += data[i] * self.coefficients[(low_index * N as f32 + i as f32) as usize];
            sum_high += data[i] * self.coefficients[(high_index * N as f32 + i as f32) as usize];
        }

        sum_low + (sum_high - sum_low) * sub_sample_fractional
    }
}

/*

template<typename Sample, int n, bool minimumPhase=false>
struct InterpolatorKaiserSincN {
    static constexpr int inputLength = n;
    static constexpr Sample latency = minimumPhase ? 0 : (n*Sample(0.5) - 1);

    int subSampleSteps;
    std::vector<Sample> coefficients;

    InterpolatorKaiserSincN() : InterpolatorKaiserSincN(0.5 - 0.45/std::sqrt(n)) {}
    InterpolatorKaiserSincN(double passFreq) : InterpolatorKaiserSincN(passFreq, 1 - passFreq) {}
    InterpolatorKaiserSincN(double passFreq, double stopFreq) {
        subSampleSteps = 2*n; // Heuristic again.  Really it depends on the bandwidth as well.
        double kaiserBandwidth = (stopFreq - passFreq)*(n + 1.0/subSampleSteps);
        kaiserBandwidth += 1.25/kaiserBandwidth; // We want to place the first zero, but (because using this to window a sinc essentially integrates it in the freq-domain), our ripples (and therefore zeroes) are out of phase.  This is a heuristic fix.
        double sincScale = M_PI*(passFreq + stopFreq);

        double centreIndex = n*subSampleSteps*0.5, scaleFactor = 1.0/subSampleSteps;
        std::vector<Sample> windowedSinc(subSampleSteps*n + 1);

        ::signalsmith::windows::Kaiser::withBandwidth(kaiserBandwidth, false).fill(windowedSinc, windowedSinc.size());

        for (size_t i = 0; i < windowedSinc.size(); ++i) {
            double x = (i - centreIndex)*scaleFactor;
            int intX = std::round(x);
            if (intX != 0 && std::abs(x - intX) < 1e-6) {
                // Exact 0s
                windowedSinc[i] = 0;
            } else if (std::abs(x) > 1e-6) {
                double p = x*sincScale;
                windowedSinc[i] *= std::sin(p)/p;
            }
        }

        if (minimumPhase) {
            signalsmith::fft::FFT<Sample> fft(windowedSinc.size()*2, 1);
            windowedSinc.resize(fft.size(), 0);
            std::vector<std::complex<Sample>> spectrum(fft.size());
            std::vector<std::complex<Sample>> cepstrum(fft.size());
            fft.fft(windowedSinc, spectrum);
            for (size_t i = 0; i < fft.size(); ++i) {
                spectrum[i] = std::log(std::abs(spectrum[i]) + 1e-30);
            }
            fft.fft(spectrum, cepstrum);
            for (size_t i = 1; i < fft.size()/2; ++i) {
                cepstrum[i] *= 0;
            }
            for (size_t i = fft.size()/2 + 1; i < fft.size(); ++i) {
                cepstrum[i] *= 2;
            }
            Sample scaling = Sample(1)/fft.size();
            fft.ifft(cepstrum, spectrum);

            for (size_t i = 0; i < fft.size(); ++i) {
                Sample phase = spectrum[i].imag()*scaling;
                Sample mag = std::exp(spectrum[i].real()*scaling);
                spectrum[i] = {mag*std::cos(phase), mag*std::sin(phase)};
            }
            fft.ifft(spectrum, cepstrum);
            windowedSinc.resize(subSampleSteps*n + 1);
            windowedSinc.shrink_to_fit();
            for (size_t i = 0; i < windowedSinc.size(); ++i) {
                windowedSinc[i] = cepstrum[i].real()*scaling;
            }
        }

        // Re-order into FIR fractional-delay blocks
        coefficients.resize(n*(subSampleSteps + 1));
        for (int k = 0; k <= subSampleSteps; ++k) {
            for (int i = 0; i < n; ++i) {
                coefficients[k*n + i] = windowedSinc[(subSampleSteps - k) + i*subSampleSteps];
            }
        }
    }

    template<class Data>
    Sample fractional(const Data &data, Sample fractional) const {
        Sample subSampleDelay = fractional*subSampleSteps;
        int lowIndex = subSampleDelay;
        if (lowIndex >= subSampleSteps) lowIndex = subSampleSteps - 1;
        Sample subSampleFractional = subSampleDelay - lowIndex;
        int highIndex = lowIndex + 1;

        Sample sumLow = 0, sumHigh = 0;
        const Sample *coeffLow = coefficients.data() + lowIndex*n;
        const Sample *coeffHigh = coefficients.data() + highIndex*n;
        for (int i = 0; i < n; ++i) {
            sumLow += data[i]*coeffLow[i];
            sumHigh += data[i]*coeffHigh[i];
        }
        return sumLow + (sumHigh - sumLow)*subSampleFractional;
    }
};

template<typename Sample>
using InterpolatorKaiserSinc20 = InterpolatorKaiserSincN<Sample, 20>;
template<typename Sample>
using InterpolatorKaiserSinc8 = InterpolatorKaiserSincN<Sample, 8>;
template<typename Sample>
using InterpolatorKaiserSinc4 = InterpolatorKaiserSincN<Sample, 4>;

template<typename Sample>
using InterpolatorKaiserSinc20Min = InterpolatorKaiserSincN<Sample, 20, true>;
template<typename Sample>
using InterpolatorKaiserSinc8Min = InterpolatorKaiserSincN<Sample, 8, true>;
template<typename Sample>
using InterpolatorKaiserSinc4Min = InterpolatorKaiserSincN<Sample, 4, true>;
///  @}

*/
