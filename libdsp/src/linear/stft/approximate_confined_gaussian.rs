use std::ops::MulAssign;

use num::Float;

pub struct ApproximateConfinedGaussian {
    gaussian_factor: f64,
}

impl ApproximateConfinedGaussian {
    fn gaussian(&self, x: f64) -> f64 {
        (-x * x * self.gaussian_factor).exp()
    }

    fn bandwidth_to_sigma(bandwidth: f64) -> f64 {
        0.3 / bandwidth.sqrt()
    }

    pub fn new(sigma: f64) -> ApproximateConfinedGaussian {
        ApproximateConfinedGaussian {
            gaussian_factor: 0.0625 / (sigma * sigma),
        }
    }

    pub fn with_bandwidth(bandwidth: f64) -> ApproximateConfinedGaussian {
        Self::new(Self::bandwidth_to_sigma(bandwidth))
    }

    // Fills an arbitrary container
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
        let offset_scale = self.gaussian(1.0) / (self.gaussian(3.0) + self.gaussian(-1.0));
        let norm = 1.0 / (self.gaussian(0.0) - 2.0 * offset_scale * (self.gaussian(2.0)));
        let offset_i = if size & 1 == 0 {
            1.0
        } else if is_for_synthesis {
            0.0
        } else {
            2.0
        };

        for i in 0..size {
            let mut r = (2.0 * i as f64 + offset_i) * inv_size - 1.0;
            r = (r + warp) / (1.0 + r * warp);

            data[i] = Sample::from(
                norm * (self.gaussian(r)
                    - offset_scale * (self.gaussian(r - 2.0) + self.gaussian(r + 2.0))),
            )
            .unwrap();
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
