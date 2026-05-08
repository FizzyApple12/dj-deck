pub mod pow2_fft;
pub mod pow2_real_fft;
pub mod real_fft;
pub mod simple_fft;
pub mod simple_real_fft;
pub mod split_fft;

use num::{Float, complex::Complex};

// Helpers for complex arithmetic, ignoring the NaN/Inf edge-cases you get
#[allow(clippy::indexing_slicing)]
pub fn complexMul<V>(a: &mut [Complex<V>], b: &[Complex<V>], c: &[Complex<V>], size: usize)
where
    V: Float,
{
    for i in 0..size {
        let bi = &b[i];
        let ci = &c[i];

        a[i] = Complex::<V>::new(bi.re * ci.re - bi.im * ci.im, bi.im * ci.re + bi.re * ci.im);
    }
}

#[allow(clippy::indexing_slicing)]
pub fn complexMulConj<V>(a: &mut [Complex<V>], b: &[Complex<V>], c: &[Complex<V>], size: usize)
where
    V: Float,
{
    for i in 0..size {
        let bi = &b[i];
        let ci = &c[i];

        a[i] = Complex::<V>::new(bi.re * ci.re + bi.im * ci.im, bi.im * ci.re - bi.re * ci.im);
    }
}

#[allow(clippy::indexing_slicing)]
pub fn complexMul_split_complex<V>(
    ar: &mut [V],
    ai: &mut [V],
    br: &[V],
    bi: &[V],
    cr: &[V],
    ci: &[V],
    size: usize,
) where
    V: Float,
{
    for i in 0..size {
        let rr = br[i] * cr[i] - bi[i] * ci[i];
        let ri = br[i] * ci[i] + bi[i] * cr[i];

        ar[i] = rr;
        ai[i] = ri;
    }
}

#[allow(clippy::indexing_slicing)]
pub fn complexMulConj_split_complex<V>(
    ar: &mut [V],
    ai: &mut [V],
    br: &[V],
    bi: &[V],
    cr: &[V],
    ci: &[V],
) where
    V: Float,
{
    for i in 0..ar.len() {
        let rr = cr[i] * br[i] + ci[i] * bi[i];
        let ri = cr[i] * bi[i] - ci[i] * br[i];

        ar[i] = rr;
        ai[i] = ri;
    }
}

// Input: aStride elements next to each other -> output with bStride
#[allow(clippy::indexing_slicing)]
pub fn interleaveCopy_const_astride<V, const ASTRIDE: usize>(a: &[V], b: &mut [V], b_stride: usize)
where
    V: Float,
{
    for bi in 0..b_stride {
        for ai in 0..ASTRIDE {
            b[bi + ai * b_stride] = a[bi * ASTRIDE + ai];
        }
    }
}
#[allow(clippy::indexing_slicing)]
pub fn interleaveCopy<V>(a: &[V], b: &mut [V], a_stride: usize, b_stride: usize)
where
    V: Float,
{
    for bi in 0..b_stride {
        for ai in 0..a_stride {
            b[bi + ai * b_stride] = a[bi * a_stride + ai];
        }
    }
}

#[allow(clippy::indexing_slicing)]
pub fn interleaveCopy_const_astride_split_complex<V, const ASTRIDE: usize>(
    a_real: &[V],
    a_imag: &[V],
    b_real: &mut [V],
    b_imag: &mut [V],
    b_stride: usize,
) where
    V: Float,
{
    for bi in 0..b_stride {
        for ai in 0..ASTRIDE {
            b_real[bi + ai * b_stride] = a_real[bi * ASTRIDE + ai];
            b_imag[bi + ai * b_stride] = a_imag[bi * ASTRIDE + ai];
        }
    }
}
#[allow(clippy::indexing_slicing)]
pub fn interleaveCopy_split_complex<V>(
    a_real: &[V],
    a_imag: &[V],
    b_real: &mut [V],
    b_imag: &mut [V],
    a_stride: usize,
    b_stride: usize,
) where
    V: Float,
{
    for bi in 0..b_stride {
        for ai in 0..a_stride {
            b_real[bi + ai * b_stride] = a_real[bi * a_stride + ai];
            b_imag[bi + ai * b_stride] = a_imag[bi * a_stride + ai];
        }
    }
}

// template<typename Sample, bool splitComputation=false>
// using FFT = SplitFFT<Sample, splitComputation>;

// template<typename Sample, bool splitComputation=false>
// using ModifiedRealFFT = RealFFT<Sample, splitComputation, true>;

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
