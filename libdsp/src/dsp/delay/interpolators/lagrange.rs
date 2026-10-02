// use std::marker::PhantomData;

// use num::Float;

// use crate::dsp::delay::interpolators::InterpolatorTrait;

// /// Fixed-order Lagrange interpolation.
// pub struct InterpolatorLagrangeN<Data, Sample>
// where
//     Sample: Float,
// {
//     phantom_data: PhantomData<Data>,
//     phantom_sample: PhantomData<Sample>,
// }

// impl<Data, Sample> Default for InterpolatorLagrangeN<Data, Sample>
// where
//     Sample: Float,
// {
//     fn default() -> Self {
//         Self {
//             phantom_data: PhantomData,
//             phantom_sample: PhantomData,
//         }
//     }
// }

// impl InterpolatorTrait<&[f32], f32> for InterpolatorLagrangeN<&[f32], f32> {
//     const INPUT_LENGTH: usize = 1;
//     // Because we're truncating, which rounds down too often
//     const LATENCY: f32 = -0.5;

//     #[allow(clippy::indexing_slicing)]
//     fn fractional(&self, data: &[f32], _: f32) -> f32 {
//         data[0]
//     }
// }

/*

/** Fixed-order Lagrange interpolation.
\diagram{interpolator-LagrangeN.svg,aliasing and amplitude/delay errors for different sizes}
*/
template<typename Sample, int n>
struct InterpolatorLagrangeN {
    static constexpr int inputLength = n + 1;
    static constexpr int latency = (n - 1)/2;

    using Array = std::array<Sample, (n + 1)>;
    Array invDivisors;

    InterpolatorLagrangeN() {
        for (int j = 0; j <= n; ++j) {
            double divisor = 1;
            for (int k = 0; k < j; ++k) divisor *= (j - k);
            for (int k = j + 1; k <= n; ++k) divisor *= (j - k);
            invDivisors[j] = 1/divisor;
        }
    }

    template<class Data>
    Sample fractional(const Data &data, Sample fractional) const {
        constexpr int mid = n/2;
        using Left = _franck_impl::ProductRange<Sample, n, 0, mid>;
        using Right = _franck_impl::ProductRange<Sample, n, mid + 1, n>;

        Sample x = fractional + latency;

        Left left(x);
        Right right(x);

        return left.calculateResult(right.total, data, invDivisors) + right.calculateResult(left.total, data, invDivisors);
    }
};
template<typename Sample>
using InterpolatorLagrange3 = InterpolatorLagrangeN<Sample, 3>;
template<typename Sample>
using InterpolatorLagrange7 = InterpolatorLagrangeN<Sample, 7>;
template<typename Sample>
using InterpolatorLagrange19 = InterpolatorLagrangeN<Sample, 19>;

*/
