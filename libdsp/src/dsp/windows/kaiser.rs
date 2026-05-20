use std::ops::MulAssign;

use num::Float;

/** The Kaiser window (almost) maximises the energy in the main-lobe compared to the side-lobes.

Kaiser windows can be constructing using the shape-parameter (beta) or using the static `with???()` methods.*/
pub struct Kaiser {
    beta: f64,
    inv_b0: f64,
}

impl Kaiser {
    // I_0(x)=\sum_{k=0}^{N}\frac{x^{2k}}{(k!)^2\cdot4^k}
    #[inline]
    fn bessel0(x: f64) -> f64 {
        let significance_limit = 1e-4;
        let mut result = 0.0;
        let mut term = 1.0;
        let mut m = 0.0;

        while term > significance_limit {
            result += term;
            m += 1.0;
            term *= (x * x) / (4.0 * m * m);
        }

        result
    }

    fn heuristic_bandwidth(bandwidth: f64) -> f64 {
        // Good peaks
        //return bandwidth + 8/((bandwidth + 3)*(bandwidth + 3));
        // Good average
        //return bandwidth + 14/((bandwidth + 2.5)*(bandwidth + 2.5));
        // Compromise
        bandwidth
            + 8.0 / ((bandwidth + 3.0) * (bandwidth + 3.0))
            + 0.25 * f64::max(3.0 - bandwidth, 0.0)
    }

    /// Set up a Kaiser window with a given shape.  `beta` is `pi*alpha` (since
    /// there is ambiguity about shape parameters)
    pub fn new(beta: f64) -> Kaiser {
        Kaiser {
            beta,
            inv_b0: 1.0 / Self::bessel0(beta),
        }
    }

    pub fn with_bandwidth(bandwidth: f64, heuristic_optimal: bool) -> Kaiser {
        Self::new(Self::bandwidth_to_beta(bandwidth, heuristic_optimal))
    }

    /** Returns the Kaiser shape where the main lobe has the specified bandwidth (as a factor of 1/window-length).
    If `heuristicOptimal` is enabled, the main lobe width is _slightly_ wider, improving both the peak and total energy - see `bandwidthToEnergyDb()` and `bandwidthToPeakDb()`. */
    pub fn bandwidth_to_beta(mut bandwidth: f64, heuristic_optimal: bool) -> f64 {
        if heuristic_optimal {
            // Heuristic based on numerical search
            bandwidth = Self::heuristic_bandwidth(bandwidth);
        }

        bandwidth = f64::max(bandwidth, 2.0);
        let alpha = (bandwidth * bandwidth * 0.25 - 1.0).sqrt();

        alpha * std::f64::consts::PI
    }

    pub fn beta_to_bandwidth(beta: f64) -> f64 {
        let alpha = beta * (1.0 / std::f64::consts::PI);

        2.0 * (alpha * alpha + 1.0).sqrt()
    }

    /** Total energy ratio (in dB) between side-lobes and the main lobe.
        \diagram{windows-kaiser-sidelobe-energy.svg,Measured main/side lobe energy ratio.  You can see that the heuristic improves performance for all bandwidth values.}
        This function uses an approximation which is accurate to ±0.5dB for 2 ⩽ bandwidth ≤ 10, or 1 ⩽ bandwidth ≤ 10 when `heuristicOptimal`is enabled.
    */
    pub fn bandwidth_to_energy_db(mut bandwidth: f64, heuristic_optimal: bool) -> f64 {
        // Horrible heuristic fits
        if heuristic_optimal {
            if bandwidth < 3.0 {
                bandwidth += (3.0 - bandwidth) * 0.5;
            }

            return 12.9 + -3.0 / (bandwidth + 0.4) - 13.4 * bandwidth
                + if bandwidth < 3.0 {
                    -9.6 * (bandwidth - 3.0)
                } else {
                    0.0
                };
        }

        10.5 + 15.0 / (bandwidth + 0.4) - 13.25 * bandwidth
            + if bandwidth < 2.0 {
                13.0 * (bandwidth - 2.0)
            } else {
                0.0
            }
    }

    pub fn energy_db_to_bandwidth(energy_db: f64, heuristic_optimal: bool) -> f64 {
        let mut bw = 1.0;

        while bw < 20.0 && Self::bandwidth_to_energy_db(bw, heuristic_optimal) > energy_db {
            bw *= 2.0;
        }

        let mut step = bw / 2.0;

        while step > 0.0001 {
            if Self::bandwidth_to_energy_db(bw, heuristic_optimal) > energy_db {
                bw += step;
            } else {
                bw -= step;
            }

            step *= 0.5;
        }

        bw
    }

    /** Peak ratio (in dB) between side-lobes and the main lobe.
        This function uses an approximation which is accurate to ±0.5dB for 2 ⩽ bandwidth ≤ 9, or 0.5 ⩽ bandwidth ≤ 9 when `heuristicOptimal`is enabled.
    */
    pub fn bandwidth_to_peak_db(bandwidth: f64, heuristic_optimal: bool) -> f64 {
        // Horrible heuristic fits
        if heuristic_optimal {
            return 14.2 - 20.0 / (bandwidth + 1.0) - 13.0 * bandwidth
                + if bandwidth < 3.0 {
                    -6.0 * (bandwidth - 3.0)
                } else {
                    0.0
                }
                + if bandwidth < 2.25 {
                    5.8 * (bandwidth - 2.25)
                } else {
                    0.0
                };
        }

        10.0 + 8.0 / (bandwidth + 2.0) - 12.75 * bandwidth
            + if bandwidth < 2.0 {
                4.0 * (bandwidth - 2.0)
            } else {
                0.0
            }
    }

    pub fn peak_db_to_bandwidth(peak_db: f64, heuristic_optimal: bool) -> f64 {
        let mut bw = 1.0;

        while bw < 20.0 && Self::bandwidth_to_peak_db(bw, heuristic_optimal) > peak_db {
            bw *= 2.0;
        }

        let mut step = bw / 2.0;

        while step > 0.0001 {
            if Self::bandwidth_to_peak_db(bw, heuristic_optimal) > peak_db {
                bw += step;
            } else {
                bw -= step;
            }

            step *= 0.5;
        }

        bw
    }

    /** Equivalent noise bandwidth (ENBW), a measure of frequency resolution.
        This approximation is accurate to ±0.05 up to a bandwidth of 22.
    */
    #[allow(clippy::unreadable_literal)]
    pub fn bandwidth_to_enbw(mut bandwidth: f64, heuristic_optimal: bool) -> f64 {
        if heuristic_optimal {
            bandwidth = Self::heuristic_bandwidth(bandwidth);
        }

        let b2 = f64::max(bandwidth - 2.0, 0.0);

        1.0 + b2 * (0.2 + b2 * (-0.005 + b2 * (-0.000005 + b2 * 0.0000022)))
    }

    #[allow(
        clippy::indexing_slicing,
        clippy::cast_precision_loss,
        clippy::needless_range_loop
    )]
    pub fn fill<Sample>(&self, data: &mut [Sample], size: usize)
    where
        Sample: Float + MulAssign,
        f64: From<Sample>,
    {
        let inv_size = 1.0 / size as f64;

        for i in 0..size {
            let r = (2.0 * i as f64 + 1.0) * inv_size - 1.0;
            let arg = (1.0 - r * r).sqrt();

            data[i] = Sample::from(Self::bessel0(self.beta * arg) * self.inv_b0).unwrap();
        }
    }
}
