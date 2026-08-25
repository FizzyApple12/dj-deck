use num::{Complex, Float};

/** Filter design methods.
    These differ mostly in how they handle frequency-warping near Nyquist
*/
#[derive(Clone, Copy, PartialEq)]
pub enum BiquadDesign {
    ///< Bilinear transform, adjusting for centre frequency but not bandwidth
    Bilinear,
    ///< RBJ's "Audio EQ Cookbook".  Based on `bilinear`, adjusting bandwidth
    /// (for peak/notch/bandpass) to preserve the ratio between upper/lower
    /// boundaries.  This performs oddly near Nyquist.
    Cookbook,
    ///< Based on `bilinear`, adjusting bandwidth to preserve the lower
    /// boundary (leaving the upper one loose).
    OneSided,
    ///< From Martin Vicanek's [Matched Second Order Digital Filters](https://vicanek.de/articles/BiquadFits.pdf).  Falls back to `oneSided` for shelf and allpass filters.  This takes the poles from the impulse-invariant approach, and then picks the zeros to create a better match.  This means that Nyquist is not 0dB for peak/notch (or -Inf for lowpass), but it is a decent match to the analogue prototype.
    Vicanek,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Type {
    HighPass,
    LowPass,
    HighShelf,
    LowShelf,
    BandPass,
    Notch,
    Peak,
    AllPass,
}

pub struct FreqSpec {
    pub scaled_freq: f64,

    pub w0: f64,
    pub sin_w0: f64,
    pub cos_w0: f64,

    pub inv_2q: f64,
}

impl FreqSpec {
    pub fn new(freq: f64, design: BiquadDesign) -> Self {
        let mut scaled_freq = 1e-6.max(0.4999.min(freq));

        if design == BiquadDesign::Cookbook {
            scaled_freq = 0.45.min(scaled_freq);
        }

        let w0 = 2.0 * std::f64::consts::PI * scaled_freq;

        FreqSpec {
            scaled_freq,

            w0,
            sin_w0: w0.sin(),
            cos_w0: w0.cos(),

            inv_2q: 0.0,
        }
    }

    pub fn one_sided_comp_q(&mut self) {
        // Ratio between our (digital) lower boundary f1 and centre f0
        let f1_factor = (self.inv_2q * self.inv_2q + 1.0).sqrt() - self.inv_2q;

        // Bilinear means discrete-time freq f = continuous-time freq tan(pi*xf/pi)
        let ct_f1 = (std::f64::consts::PI * self.scaled_freq * f1_factor).tan();
        let inv_ct_f0 = (1.0 + self.cos_w0) / self.sin_w0;
        let c_f1_factor = ct_f1 * inv_ct_f0;

        self.inv_2q = 0.5 / c_f1_factor - 0.5 * c_f1_factor;
    }
}

/// A standard biquad.
///
/// This is not guaranteed to be stable if modulated at audio rate.
///
/// The default highpass/lowpass bandwidth (`defaultBandwidth`) produces a
/// Butterworth filter when bandwidth-compensation is disabled.
///
/// Bandwidth compensation defaults to `BiquadDesign::oneSided` (or
/// `BiquadDesign::cookbook` if `cookbookBandwidth` is enabled) for all filter
/// types aside from highpass/lowpass (which use `BiquadDesign::bilinear`).
#[derive(Clone, Copy)]
pub struct BiquadStatic<Sample, const COOKBOOK_BANDWIDTH: bool>
where
    Sample: Float,
{
    a1: Sample,
    a2: Sample,
    b0: Sample,
    b1: Sample,
    b2: Sample,

    x1: Sample,
    x2: Sample,
    y1: Sample,
    y2: Sample,
}

pub trait BiquadStaticTrait<Sample, const COOKBOOK_BANDWIDTH: bool> {
    const BW_DESIGN: BiquadDesign;

    const DEFAULT_Q: f64;
    const DEFAULT_BANDWIDTH: f64;
}

impl<Sample, const COOKBOOK_BANDWIDTH: bool> BiquadStaticTrait<Sample, COOKBOOK_BANDWIDTH>
    for BiquadStatic<Sample, COOKBOOK_BANDWIDTH>
where
    Sample: Float,
{
    const BW_DESIGN: BiquadDesign = if COOKBOOK_BANDWIDTH {
        BiquadDesign::Cookbook
    } else {
        BiquadDesign::OneSided
    };
    // sqrt(0.5)
    #[allow(clippy::unreadable_literal)]
    const DEFAULT_BANDWIDTH: f64 = 1.8999686269529916;
    #[allow(clippy::unreadable_literal, clippy::approx_constant)]
    const DEFAULT_Q: f64 = 0.7071067811865476; // equivalent to above Q
}

impl<const COOKBOOK_BANDWIDTH: bool> BiquadStatic<f32, COOKBOOK_BANDWIDTH> {
    #[inline]
    fn octave_spec(scaled_freq: f64, mut octaves: f64, design: BiquadDesign) -> FreqSpec {
        let mut spec = FreqSpec::new(scaled_freq, design);

        if design == BiquadDesign::Cookbook {
            // Approximately preserves bandwidth between halfway points
            octaves *= spec.w0 / spec.sin_w0;
        }

        spec.inv_2q = (2.0.ln() * 0.5 * octaves).sinh(); // 1/(2Q)

        if design == BiquadDesign::OneSided {
            spec.one_sided_comp_q();
        }

        spec
    }

    fn q_spec(scaled_freq: f64, q: f64, design: BiquadDesign) -> FreqSpec {
        let mut spec = FreqSpec::new(scaled_freq, design);

        spec.inv_2q = 0.5 / q;

        if design == BiquadDesign::OneSided {
            spec.one_sided_comp_q();
        }

        spec
    }

    #[inline]
    fn db_to_sqrt_gain(db: f64) -> f64 {
        10.0.powf(db * 0.025)
    }

    #[inline]
    #[allow(
        clippy::too_many_lines,
        clippy::cast_possible_truncation,
        clippy::cast_lossless,
        non_snake_case
    )]
    fn configure(
        &mut self,
        filter_type: Type,
        mut calc: FreqSpec,
        sqrt_gain: f64,
        design: BiquadDesign,
    ) {
        let w0 = calc.w0;

        if design == BiquadDesign::Vicanek {
            if filter_type == Type::Notch {
                // Heuristic for notches near Nyquist
                calc.inv_2q *= 1.0 - calc.scaled_freq * 0.5;
            }

            let Q = if filter_type == Type::Peak {
                0.5 * sqrt_gain
            } else {
                0.5
            } / calc.inv_2q;
            let q = if filter_type == Type::Peak {
                1.0 / sqrt_gain
            } else {
                1.0
            } * calc.inv_2q;
            let expmqw = (-q * w0).exp();
            let da1;

            if q <= 1.0 {
                da1 = -2.0 * expmqw * ((1.0 - q * q).sqrt() * w0).cos();

                self.a1 = da1 as f32;
            } else {
                da1 = -2.0 * expmqw * ((q * q - 1.0).sqrt() * w0).cosh();

                self.a1 = da1 as f32;
            }

            let da2 = expmqw * expmqw;
            self.a2 = da2 as f32;

            let sinpd2 = (w0 / 2.0).sin();

            let p0 = 1.0 - sinpd2 * sinpd2;
            let p1 = sinpd2 * sinpd2;
            let p2 = 4.0 * p0 * p1;

            let mut A0 = 1.0 + da1 + da2;
            let mut A1 = 1.0 - da1 + da2;
            let A2 = -4.0 * da2;

            A0 *= A0;
            A1 *= A1;

            if filter_type == Type::LowPass {
                let R1 = (A0 * p0 + A1 * p1 + A2 * p2) * Q * Q;
                let B0 = A0;
                let B1 = (R1 - B0 * p0) / p1;

                self.b0 = f64::midpoint(B0.sqrt(), 0.0.max(B1).sqrt()) as f32;
                self.b1 = (B0.sqrt() - self.b0 as f64) as f32;
                self.b2 = 0.0;

                return;
            } else if filter_type == Type::HighPass {
                self.b0 = ((A0 * p0 + A1 * p1 + A2 * p2).sqrt() * Q / (4.0 * p1)) as f32;
                self.b2 = self.b0;
                self.b1 = -2.0 * self.b0;

                return;
            } else if filter_type == Type::BandPass {
                let R1 = A0 * p0 + A1 * p1 + A2 * p2;
                let R2 = -A0 + A1 + 4.0 * (p0 - p1) * A2;
                let B2 = (R1 - R2 * p1) / (4.0 * p1 * p1);
                let B1 = R2 + 4.0 * (p1 - p0) * B2;

                self.b1 = (-0.5 * 0.0.max(B1).sqrt()) as f32;
                self.b0 = (0.5 * (0.0.max(B2 + 0.25 * B1).sqrt() - self.b1 as f64)) as f32;
                self.b2 = -self.b0 - self.b1;

                return;
            } else if filter_type == Type::Notch {
                // The Vicanek paper doesn't cover notches (band-stop), but we know where the
                // zeros should be:
                self.b0 = 1.0;
                let db1 = -2.0 * w0.cos(); // might be higher precision
                self.b1 = db1 as f32;
                self.b2 = 1.0;
                // Scale so that B0 == A0 to get 0dB at f=0
                let scale = A0.sqrt() / (self.b0 as f64 + db1 + self.b2 as f64);

                self.b0 *= scale as f32;
                self.b1 *= scale as f32;
                self.b2 *= scale as f32;

                return;
            } else if filter_type == Type::Peak {
                let G2 = (sqrt_gain * sqrt_gain) * (sqrt_gain * sqrt_gain);
                let R1 = (A0 * p0 + A1 * p1 + A2 * p2) * G2;
                let R2 = (-A0 + A1 + 4.0 * (p0 - p1) * A2) * G2;
                let B0 = A0;
                let B2 = (R1 - R2 * p1 - B0) / (4.0 * p1 * p1);
                let B1 = R2 + B0 + 4.0 * (p1 - p0) * B2;
                let W = f64::midpoint(B0.sqrt(), 0.0.max(B1).sqrt());

                self.b0 = f64::midpoint(W, 0.0.max(W * W + B2).sqrt()) as f32;
                self.b1 = (0.5 * (B0.sqrt() - 0.0.max(B1).sqrt())) as f32;
                self.b2 = (-B2 / (4.0 * self.b0 as f64)) as f32;

                return;
            }

            // All others fall back to `oneSided`
            // design = BiquadDesign::oneSided;
            calc.one_sided_comp_q();
        }

        let alpha = calc.sin_w0 * calc.inv_2q;
        let A = sqrt_gain;
        let sqrt_a2_alpha = 2.0 * A.sqrt() * alpha;

        let a0;

        if filter_type == Type::HighPass {
            self.b1 = (-1.0 - calc.cos_w0) as f32;
            self.b2 = f64::midpoint(1.0, calc.cos_w0) as f32;
            self.b0 = self.b2;

            a0 = 1.0 + alpha;
            self.a1 = (-2.0 * calc.cos_w0) as f32;
            self.a2 = (1.0 - alpha) as f32;
        } else if filter_type == Type::LowPass {
            self.b1 = (1.0 - calc.cos_w0) as f32;
            self.b2 = self.b1 * 0.5;
            self.b0 = self.b2;

            a0 = 1.0 + alpha;
            self.a1 = (-2.0 * calc.cos_w0) as f32;
            self.a2 = (1.0 - alpha) as f32;
        } else if filter_type == Type::HighShelf {
            self.b0 = (A * ((A + 1.0) + (A - 1.0) * calc.cos_w0 + sqrt_a2_alpha)) as f32;
            self.b2 = (A * ((A + 1.0) + (A - 1.0) * calc.cos_w0 - sqrt_a2_alpha)) as f32;
            self.b1 = (-2.0 * A * ((A - 1.0) + (A + 1.0) * calc.cos_w0)) as f32;

            a0 = (A + 1.0) - (A - 1.0) * calc.cos_w0 + sqrt_a2_alpha;
            self.a2 = ((A + 1.0) - (A - 1.0) * calc.cos_w0 - sqrt_a2_alpha) as f32;
            self.a1 = (2.0 * ((A - 1.0) - (A + 1.0) * calc.cos_w0)) as f32;
        } else if filter_type == Type::LowShelf {
            self.b0 = (A * ((A + 1.0) - (A - 1.0) * calc.cos_w0 + sqrt_a2_alpha)) as f32;
            self.b2 = (A * ((A + 1.0) - (A - 1.0) * calc.cos_w0 - sqrt_a2_alpha)) as f32;
            self.b1 = (2.0 * A * ((A - 1.0) - (A + 1.0) * calc.cos_w0)) as f32;

            a0 = (A + 1.0) + (A - 1.0) * calc.cos_w0 + sqrt_a2_alpha;
            self.a2 = ((A + 1.0) + (A - 1.0) * calc.cos_w0 - sqrt_a2_alpha) as f32;
            self.a1 = (-2.0 * ((A - 1.0) + (A + 1.0) * calc.cos_w0)) as f32;
        } else if filter_type == Type::BandPass {
            self.b0 = alpha as f32;
            self.b1 = 0.0;
            self.b2 = -alpha as f32;

            a0 = 1.0 + alpha;
            self.a1 = (-2.0 * calc.cos_w0) as f32;
            self.a2 = (1.0 - alpha) as f32;
        } else if filter_type == Type::Notch {
            self.b0 = 1.0;
            self.b1 = (-2.0 * calc.cos_w0) as f32;
            self.b2 = 1.0;

            a0 = 1.0 + alpha;
            self.a1 = self.b1;
            self.a2 = (1.0 - alpha) as f32;
        } else if filter_type == Type::Peak {
            self.b0 = (1.0 + alpha * A) as f32;
            self.b1 = (-2.0 * calc.cos_w0) as f32;
            self.b2 = (1.0 - alpha * A) as f32;

            a0 = 1.0 + alpha / A;
            self.a1 = self.b1;
            self.a2 = (1.0 - alpha / A) as f32;
        } else if filter_type == Type::AllPass {
            self.b0 = (1.0 - alpha) as f32;
            self.b1 = (-2.0 * calc.cos_w0) as f32;
            self.b2 = (1.0 + alpha) as f32;

            a0 = self.b2 as f64;
            self.a1 = self.b1;
            self.a2 = self.b0;
        } else {
            // reset to neutral
            self.b0 = 1.0;
            self.b1 = 0.0;
            self.b2 = 0.0;

            a0 = self.b0 as f64;
            self.a1 = self.b1;
            self.a2 = self.b2;
        }

        let inv_a0 = 1.0 / a0;
        self.b0 *= inv_a0 as f32;
        self.b1 *= inv_a0 as f32;
        self.b2 *= inv_a0 as f32;
        self.a1 *= inv_a0 as f32;
        self.a2 *= inv_a0 as f32;
    }

    pub fn process(&mut self, x0: f32) -> f32 {
        let y0 = x0 * self.b0 + self.x1 * self.b1 + self.x2 * self.b2
            - self.y1 * self.a1
            - self.y2 * self.a2;

        self.y2 = self.y1;
        self.y1 = y0;
        self.x2 = self.x1;
        self.x1 = x0;

        y0
    }

    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    pub fn response(&self, scaled_freq: f32) -> Complex<f32> {
        let w = scaled_freq * 2.0 * std::f32::consts::PI;

        let inv_z = Complex::<f32>::new(w.cos(), -w.sin());
        let inv_z2 = inv_z * inv_z;

        (self.b0 + inv_z * self.b1 + inv_z2 * self.b2) / (1.0 + inv_z * self.a1 + inv_z2 * self.a2)
    }

    pub fn response_db(&self, scaled_freq: f32) -> f32 {
        let w = scaled_freq * 2.0 * std::f32::consts::PI;

        let inv_z = Complex::<f32>::new(w.cos(), -w.sin());
        let inv_z2 = inv_z * inv_z;

        let energy = (self.b0 + inv_z * self.b1 + inv_z2 * self.b2).norm_sqr()
            / (1.0 + inv_z * self.a1 + inv_z2 * self.a2).norm_sqr();

        10.0 * energy.log10()
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn add_gain(&mut self, factor: f64) {
        self.b0 *= factor as f32;
        self.b1 *= factor as f32;
        self.b2 *= factor as f32;
    }

    pub fn add_gain_db(&mut self, db: f64) {
        self.add_gain((db * 0.05).powi(10));
    }

    pub fn lowpass(&mut self, scaled_freq: f64, octaves: f64, design: BiquadDesign) {
        self.configure(
            Type::LowPass,
            Self::octave_spec(scaled_freq, octaves, design),
            0.0,
            design,
        );
    }

    pub fn lowpass_q(&mut self, scaled_freq: f64, q: f64, design: BiquadDesign) {
        self.configure(
            Type::LowPass,
            Self::q_spec(scaled_freq, q, design),
            0.0,
            design,
        );
    }

    pub fn highpass(&mut self, scaled_freq: f64, octaves: f64, design: BiquadDesign) {
        self.configure(
            Type::HighPass,
            Self::octave_spec(scaled_freq, octaves, design),
            0.0,
            design,
        );
    }

    pub fn highpass_q(&mut self, scaled_freq: f64, q: f64, design: BiquadDesign) {
        self.configure(
            Type::HighPass,
            Self::q_spec(scaled_freq, q, design),
            0.0,
            design,
        );
    }

    pub fn bandpass(&mut self, scaled_freq: f64, octaves: f64, design: BiquadDesign) {
        self.configure(
            Type::BandPass,
            Self::octave_spec(scaled_freq, octaves, design),
            0.0,
            design,
        );
    }

    pub fn bandpass_q(&mut self, scaled_freq: f64, q: f64, design: BiquadDesign) {
        self.configure(
            Type::BandPass,
            Self::q_spec(scaled_freq, q, design),
            0.0,
            design,
        );
    }

    pub fn notch(&mut self, scaled_freq: f64, octaves: f64, design: BiquadDesign) {
        self.configure(
            Type::Notch,
            Self::octave_spec(scaled_freq, octaves, design),
            0.0,
            design,
        );
    }

    pub fn notch_q(&mut self, scaled_freq: f64, q: f64, design: BiquadDesign) {
        self.configure(
            Type::Notch,
            Self::q_spec(scaled_freq, q, design),
            0.0,
            design,
        );
    }

    pub fn peak(&mut self, scaled_freq: f64, gain: f64, octaves: f64, design: BiquadDesign) {
        self.configure(
            Type::Peak,
            Self::octave_spec(scaled_freq, octaves, design),
            gain.sqrt(),
            design,
        );
    }

    pub fn peak_db(&mut self, scaled_freq: f64, db: f64, octaves: f64, design: BiquadDesign) {
        self.configure(
            Type::Peak,
            Self::octave_spec(scaled_freq, octaves, design),
            Self::db_to_sqrt_gain(db),
            design,
        );
    }

    pub fn peak_q(&mut self, scaled_freq: f64, gain: f64, q: f64, design: BiquadDesign) {
        self.configure(
            Type::Peak,
            Self::q_spec(scaled_freq, q, design),
            gain.sqrt(),
            design,
        );
    }

    pub fn peak_db_q(&mut self, scaled_freq: f64, db: f64, q: f64, design: BiquadDesign) {
        self.configure(
            Type::Peak,
            Self::q_spec(scaled_freq, q, design),
            Self::db_to_sqrt_gain(db),
            design,
        );
    }

    pub fn high_shelf(&mut self, scaled_freq: f64, gain: f64, octaves: f64, design: BiquadDesign) {
        self.configure(
            Type::HighShelf,
            Self::octave_spec(scaled_freq, octaves, design),
            gain.sqrt(),
            design,
        );
    }

    pub fn high_shelf_db(&mut self, scaled_freq: f64, db: f64, octaves: f64, design: BiquadDesign) {
        self.configure(
            Type::HighShelf,
            Self::octave_spec(scaled_freq, octaves, design),
            Self::db_to_sqrt_gain(db),
            design,
        );
    }

    pub fn high_shelf_q(&mut self, scaled_freq: f64, gain: f64, q: f64, design: BiquadDesign) {
        self.configure(
            Type::HighShelf,
            Self::q_spec(scaled_freq, q, design),
            gain.sqrt(),
            design,
        );
    }

    pub fn high_shelf_db_q(&mut self, scaled_freq: f64, db: f64, q: f64, design: BiquadDesign) {
        self.configure(
            Type::HighShelf,
            Self::q_spec(scaled_freq, q, design),
            Self::db_to_sqrt_gain(db),
            design,
        );
    }

    pub fn low_shelf(&mut self, scaled_freq: f64, gain: f64, octaves: f64, design: BiquadDesign) {
        self.configure(
            Type::LowShelf,
            Self::octave_spec(scaled_freq, octaves, design),
            gain.sqrt(),
            design,
        );
    }

    pub fn low_shelf_db(&mut self, scaled_freq: f64, db: f64, octaves: f64, design: BiquadDesign) {
        self.configure(
            Type::LowShelf,
            Self::octave_spec(scaled_freq, octaves, design),
            Self::db_to_sqrt_gain(db),
            design,
        );
    }

    pub fn low_shelf_q(&mut self, scaled_freq: f64, gain: f64, q: f64, design: BiquadDesign) {
        self.configure(
            Type::LowShelf,
            Self::q_spec(scaled_freq, q, design),
            gain.sqrt(),
            design,
        );
    }

    pub fn low_shelf_db_q(&mut self, scaled_freq: f64, db: f64, q: f64, design: BiquadDesign) {
        self.configure(
            Type::LowShelf,
            Self::q_spec(scaled_freq, q, design),
            Self::db_to_sqrt_gain(db),
            design,
        );
    }

    pub fn allpass(&mut self, scaled_freq: f64, octaves: f64, design: BiquadDesign) {
        self.configure(
            Type::AllPass,
            Self::octave_spec(scaled_freq, octaves, design),
            0.0,
            design,
        );
    }

    pub fn allpass_q(&mut self, scaled_freq: f64, q: f64, design: BiquadDesign) {
        self.configure(
            Type::AllPass,
            Self::q_spec(scaled_freq, q, design),
            0.0,
            design,
        );
    }
}

impl<const COOKBOOK_BANDWIDTH: bool> Default for BiquadStatic<f32, COOKBOOK_BANDWIDTH> {
    fn default() -> Self {
        Self {
            a1: 0.0,
            a2: 0.0,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }
}
