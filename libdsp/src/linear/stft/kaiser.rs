use std::ops::MulAssign;

use num::Float;

// Copied from DSP library `windows.h`
pub struct Kaiser {
    beta: f64,
    inv_b0: f64,
}

impl Kaiser {
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
        bandwidth
            + 8.0 / ((bandwidth + 3.0) * (bandwidth + 3.0))
            + 0.25 * f64::max(3.0 - bandwidth, 0.0)
    }

    pub fn new(beta: f64) -> Kaiser {
        Kaiser {
            beta,
            inv_b0: 1.0 / Self::bessel0(beta),
        }
    }

    pub fn with_bandwidth(bandwidth: f64, heuristic_optimal: bool) -> Kaiser {
        Self::new(Self::bandwidth_to_beta(bandwidth, heuristic_optimal))
    }

    pub fn bandwidth_to_beta(mut bandwidth: f64, heuristic_optimal: bool) -> f64 {
        if heuristic_optimal {
            // Heuristic based on numerical search
            bandwidth = Self::heuristic_bandwidth(bandwidth);
        }

        bandwidth = f64::max(bandwidth, 2.0);
        let alpha = (bandwidth * bandwidth * 0.25 - 1.0).sqrt();

        alpha * std::f64::consts::PI
    }

    #[allow(
        clippy::indexing_slicing,
        clippy::cast_precision_loss,
        clippy::needless_range_loop
    )]
    pub fn fill<Sample>(&self, data: &mut [Sample], size: usize, warp: f64, is_for_synthesis: bool)
    where
        Sample: Float + MulAssign,
        f64: From<Sample>,
    {
        let inv_size = 1.0 / size as f64;
        let offset_i = if size & 1 == 1 {
            1.0
        } else if is_for_synthesis {
            0.0
        } else {
            2.0
        };

        for i in 0..size {
            let mut r = (2.0 * i as f64 + offset_i) * inv_size - 1.0;
            r = (r + warp) / (1.0 + r * warp);

            let arg = (1.0 - r * r).sqrt();

            data[i] = Sample::from(Self::bessel0(self.beta * arg) * self.inv_b0).unwrap();
        }

        if warp != 0.0 {
            // Warp window vertically as well, to restore some width
            for i in 0..size {
                let previous_data = f64::from(data[i]);

                data[i] *= Sample::from(
                    ((warp + 1.0) / (1.0 + warp * (2.0 * previous_data - 1.0))).sqrt(),
                )
                .unwrap();
            }
        }
    }
}
