use num::{Complex, Float, complex::ComplexFloat};

use crate::dsp::{
    delay::interpolators::InterpolatorTrait,
    fft::{FFTTrait, fft::FFT},
    windows::kaiser::Kaiser,
};

/** Fixed-size Kaiser-windowed sinc interpolation.
If `minimumPhase` is enabled, a minimum-phase version of the kernel is used:
*/
pub struct InterpolatorKaiserSincN<Sample, const N: usize, const MINIMUM_PHASE: bool>
where
    Sample: Float,
{
    sub_sample_steps: usize,
    coefficients: Vec<Sample>,
}

impl<const N: usize, const MINIMUM_PHASE: bool> Default
    for InterpolatorKaiserSincN<f32, N, MINIMUM_PHASE>
{
    #[allow(clippy::cast_precision_loss)]
    fn default() -> Self {
        Self::new_pass(0.5 - 0.45 / (N as f64).sqrt())
    }
}

impl<const N: usize, const MINIMUM_PHASE: bool> InterpolatorKaiserSincN<f32, N, MINIMUM_PHASE> {
    pub fn new_pass(pass_freq: f64) -> Self {
        Self::new_pass_stop(pass_freq, 1.0 - pass_freq)
    }

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::needless_range_loop,
        clippy::indexing_slicing
    )]
    pub fn new_pass_stop(pass_freq: f64, stop_freq: f64) -> Self {
        let sub_sample_steps = 2 * N; // Heuristic again.  Really it depends on the bandwidth as well.
        let mut kaiser_bandwidth =
            (stop_freq - pass_freq) * (N as f64 + 1.0 / sub_sample_steps as f64);
        kaiser_bandwidth += 1.25 / kaiser_bandwidth; // We want to place the first zero, but (because using this to window a sinc essentially integrates it in the freq-domain), our ripples (and therefore zeroes) are out of phase.  This is a heuristic fix.
        let sinc_scale = std::f64::consts::PI * (pass_freq + stop_freq);

        let centre_index = (N * sub_sample_steps) as f64 * 0.5;
        let scale_factor = 1.0 / sub_sample_steps as f64;
        let mut windowed_sinc: Vec<f32> = Vec::with_capacity(sub_sample_steps * N + 1);
        let windowed_sinc_len = windowed_sinc.len();

        Kaiser::with_bandwidth(kaiser_bandwidth, false).fill(&mut windowed_sinc, windowed_sinc_len);

        for i in 0..windowed_sinc.len() {
            let x = (i as f64 - centre_index) * scale_factor;
            let int_x = x.round();

            if !(-f64::EPSILON..=f64::EPSILON).contains(&int_x) && (x - int_x).abs() < 1e-6 {
                // Exact 0s
                windowed_sinc[i] = 0.0;
            } else if x.abs() > 1e-6 {
                let p = x * sinc_scale;

                windowed_sinc[i] *= (p.sin() / p) as f32;
            }
        }

        if MINIMUM_PHASE {
            let mut fft: FFT<f32> = FFT::<f32>::new(windowed_sinc.len() * 2, 0);

            windowed_sinc.resize(fft.size(), 0.0);

            let mut spectrum: Vec<Complex<f32>> = Vec::with_capacity(fft.size());
            let mut cepstrum: Vec<Complex<f32>> = Vec::with_capacity(fft.size());

            let windowed_sinc_complex: Vec<Complex<f32>> = windowed_sinc
                .iter()
                .map(|&x| Complex::new(x, 0.0))
                .collect();

            fft.fft(&windowed_sinc_complex, &mut spectrum);

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

            windowed_sinc.resize(sub_sample_steps * N + 1, 0.0);
            windowed_sinc.shrink_to_fit();

            for i in 0..windowed_sinc.len() {
                windowed_sinc[i] = cepstrum[i].re * scaling;
            }
        }

        // Re-order into FIR fractional-delay blocks
        let mut coefficients = Vec::new();

        coefficients.resize(N * (sub_sample_steps + 1), 0.0);

        for k in 0..sub_sample_steps {
            for i in 0..N {
                coefficients[k * N + i] =
                    windowed_sinc[(sub_sample_steps - k) + i * sub_sample_steps];
            }
        }

        Self {
            sub_sample_steps,
            coefficients,
        }
    }
}

impl<const N: usize, const MINIMUM_PHASE: bool> InterpolatorTrait<f32>
    for InterpolatorKaiserSincN<f32, N, MINIMUM_PHASE>
{
    const INPUT_LENGTH: usize = N;
    // Because we're truncating, which rounds down too often
    #[allow(clippy::cast_precision_loss)]
    const LATENCY: f32 = if MINIMUM_PHASE {
        0.0
    } else {
        N as f32 * 0.5 - 1.0
    };

    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::needless_range_loop,
        clippy::indexing_slicing
    )]
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

pub type InterpolatorKaiserSinc20<Sample> = InterpolatorKaiserSincN<Sample, 20, false>;
pub type InterpolatorKaiserSinc8<Sample> = InterpolatorKaiserSincN<Sample, 8, false>;
pub type InterpolatorKaiserSinc4<Sample> = InterpolatorKaiserSincN<Sample, 4, false>;

pub type InterpolatorKaiserSinc20Min<Sample> = InterpolatorKaiserSincN<Sample, 20, true>;
pub type InterpolatorKaiserSinc8Min<Sample> = InterpolatorKaiserSincN<Sample, 8, true>;
pub type InterpolatorKaiserSinc4Min<Sample> = InterpolatorKaiserSincN<Sample, 4, true>;
