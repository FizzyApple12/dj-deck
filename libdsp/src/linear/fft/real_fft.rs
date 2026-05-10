use num::{Float, complex::Complex};

use crate::linear::{
    complex_to_two_float_mut,
    fft::split_fft::{SplitFFT, SplitFFTTrait},
};

// A Real FFT which can handle multiples of 3 and 5, and can be computed in
// chunks
pub struct RealFFT<Sample, const SPLIT_COMPUTATION: bool, const HALF_BIN_SHIFT: bool>
where
    Sample: Float,
{
    complex_fft: SplitFFT<Sample, SPLIT_COMPUTATION>,

    tmp_freq: Vec<Complex<Sample>>,
    tmp_time: Vec<Complex<Sample>>,
    twiddles: Vec<Complex<Sample>>,
    half_bin_twists: Vec<Complex<Sample>>,
}

pub trait RealFFTTrait<Sample, const SPLIT_COMPUTATION: bool, const HALF_BIN_SHIFT: bool>
where
    Sample: Float + Default,
{
    type Complex;
    type Sample;

    const PREFERS_SPLIT: bool;

    fn fast_size_above(size: usize) -> usize;

    fn new(size: usize) -> Self;

    fn resize(&mut self, size: usize);

    fn size(&self) -> usize;
    fn steps(&self) -> usize;

    fn fft(&mut self, time: &[Self::Sample], freq: &mut [Self::Complex]);
    fn fft_step(&mut self, step: usize, time: &[Self::Sample], freq: &mut [Self::Complex]);
    fn fft_split_complex(
        &mut self,
        in_r: &[Self::Sample],
        out_r: &mut [Self::Sample],
        out_i: &mut [Self::Sample],
    );
    fn fft_step_split_complex(
        &mut self,
        step: usize,
        in_r: &[Self::Sample],
        out_r: &mut [Self::Sample],
        out_i: &mut [Self::Sample],
    );

    fn ifft(&mut self, freq: &[Self::Complex], time: &mut [Self::Sample]);
    fn ifft_step(&mut self, step: usize, freq: &[Self::Complex], time: &mut [Self::Sample]);
    fn ifft_split_complex(
        &mut self,
        in_r: &[Self::Sample],
        in_i: &[Self::Sample],
        out_r: &mut [Self::Sample],
    );
    fn ifft_step_split_complex(
        &mut self,
        step: usize,
        in_i: &[Self::Sample],
        in_i: &[Self::Sample],
        out_r: &mut [Self::Sample],
    );
}

impl<const SPLIT_COMPUTATION: bool, const HALF_BIN_SHIFT: bool>
    RealFFTTrait<f32, SPLIT_COMPUTATION, HALF_BIN_SHIFT>
    for RealFFT<f32, SPLIT_COMPUTATION, HALF_BIN_SHIFT>
{
    type Complex = Complex<f32>;
    type Sample = f32;

    const PREFERS_SPLIT: bool = SplitFFT::<Self::Sample, SPLIT_COMPUTATION>::PREFERS_SPLIT;

    fn fast_size_above(size: usize) -> usize {
        SplitFFT::<Self::Sample, SPLIT_COMPUTATION>::fast_size_above(size.div_ceil(2)) * 2
    }

    fn new(size: usize) -> Self {
        let mut new = Self {
            complex_fft: SplitFFT::<Self::Sample, SPLIT_COMPUTATION>::new(size),

            tmp_freq: Vec::new(),
            tmp_time: Vec::new(),
            twiddles: Vec::new(),
            half_bin_twists: Vec::new(),
        };

        new.resize(size);

        new
    }

    #[allow(clippy::indexing_slicing, clippy::cast_precision_loss)]
    fn resize(&mut self, size: usize) {
        let h_size = size / 2;

        self.complex_fft.resize(h_size);
        self.tmp_freq
            .resize(h_size, Self::Complex { re: 0.0, im: 0.0 });
        self.tmp_time
            .resize(h_size, Self::Complex { re: 0.0, im: 0.0 });

        self.twiddles
            .resize(h_size / 2 + 1, Self::Complex { re: 0.0, im: 0.0 });

        if HALF_BIN_SHIFT {
            for i in 0..self.twiddles.len() {
                let rot_phase = (i as Self::Sample + 0.5)
                    * (-2.0 * std::f32::consts::PI / size as Self::Sample)
                    - std::f32::consts::PI / 2.0;

                self.twiddles[i] = Self::Complex::from_polar(1.0, rot_phase);
            }

            self.half_bin_twists
                .resize(h_size, Self::Complex { re: 0.0, im: 0.0 });

            for i in 0..h_size {
                let twist_phase =
                    -2.0 * std::f32::consts::PI * i as Self::Sample / size as Self::Sample;

                self.half_bin_twists[i] = Self::Complex::from_polar(1.0, twist_phase);
            }
        } else {
            for i in 0..self.twiddles.len() {
                let rot_phase = i as Self::Sample
                    * (-2.0 * std::f32::consts::PI / size as Self::Sample)
                    - std::f32::consts::PI / 2.0; // bake rotation by (-i) into twiddles

                self.twiddles[i] = Self::Complex::from_polar(1.0, rot_phase);
            }
        }
    }

    fn size(&self) -> usize {
        self.complex_fft.size() * 2
    }

    fn steps(&self) -> usize {
        self.complex_fft.steps() + if SPLIT_COMPUTATION { 3 } else { 2 }
    }

    fn fft(&mut self, time: &[Self::Sample], freq: &mut [Self::Complex]) {
        for s in 0..self.steps() {
            self.fft_step(s, time, freq);
        }
    }

    #[allow(
        clippy::too_many_lines,
        clippy::indexing_slicing,
        clippy::bool_to_int_with_if,
        clippy::undocumented_unsafe_blocks
    )]
    fn fft_step(&mut self, mut step: usize, time: &[Self::Sample], freq: &mut [Self::Complex]) {
        if Self::PREFERS_SPLIT {
            let h_size = self.complex_fft.size();

            let (tmp_time_r, tmp_time_i) = complex_to_two_float_mut(&mut self.tmp_time);
            let (tmp_freq_r, tmp_freq_i) = complex_to_two_float_mut(&mut self.tmp_freq);

            let step_check = step;
            step -= 1;

            if step_check == 0 {
                let h_size = self.complex_fft.size();

                if HALF_BIN_SHIFT {
                    for i in 0..h_size {
                        let tr = time[2 * i];
                        let ti = time[2 * i + 1];

                        let twist = self.half_bin_twists[i];

                        tmp_time_r[i] = tr * twist.re - ti * twist.im;
                        tmp_time_i[i] = ti * twist.re + tr * twist.im;
                    }
                } else {
                    for i in 0..h_size {
                        tmp_time_r[i] = time[2 * i];
                        tmp_time_i[i] = time[2 * i + 1];
                    }
                }
            } else if step < self.complex_fft.steps() {
                self.complex_fft
                    .fft_step_split_complex(step, tmp_time_r, tmp_time_i, tmp_freq_r, tmp_freq_i);
            } else {
                if !HALF_BIN_SHIFT {
                    let bin0_r = tmp_freq_r[0];
                    let bin0_i = tmp_freq_i[0];

                    freq[0] = Self::Complex::new(bin0_r + bin0_i, bin0_r - bin0_i);
                }

                let mut start_i = if HALF_BIN_SHIFT { 0 } else { 1 };
                let mut end_i = h_size / 2 + 1;

                if SPLIT_COMPUTATION {
                    // Do this last twiddle in two halves
                    if step == self.complex_fft.steps() {
                        end_i = usize::midpoint(start_i, end_i);
                    } else {
                        start_i = usize::midpoint(start_i, end_i);
                    }
                }

                for i in start_i..end_i {
                    let conj_i = if HALF_BIN_SHIFT {
                        h_size - 1 - i
                    } else {
                        h_size - i
                    };

                    let twiddle = self.twiddles[i];

                    let odd_r = (tmp_freq_r[i] + tmp_freq_r[conj_i]) * 0.5;
                    let odd_i = (tmp_freq_i[i] - tmp_freq_i[conj_i]) * 0.5;
                    let even_i_r = (tmp_freq_r[i] - tmp_freq_r[conj_i]) * 0.5;
                    let even_i_i = (tmp_freq_i[i] + tmp_freq_i[conj_i]) * 0.5;

                    let even_rot_minus_i_r = even_i_r * twiddle.re - even_i_i * twiddle.im;
                    let even_rot_minus_i_i = even_i_i * twiddle.re + even_i_r * twiddle.im;

                    freq[i] =
                        Self::Complex::new(odd_r + even_rot_minus_i_r, odd_i + even_rot_minus_i_i);
                    freq[conj_i] =
                        Self::Complex::new(odd_r - even_rot_minus_i_r, even_rot_minus_i_i - odd_i);
                }
            }
        } else {
            let raw_time = &raw const time;

            let can_use_time = !HALF_BIN_SHIFT
                && (raw_time as usize).is_multiple_of(std::mem::align_of::<Self::Complex>());

            let step_check = step;
            step -= 1;

            if step_check == 0 {
                let h_size = self.complex_fft.size();

                if HALF_BIN_SHIFT {
                    for i in 0..h_size {
                        let tr = time[2 * i];
                        let ti = time[2 * i + 1];

                        let twist = self.half_bin_twists[i];

                        self.tmp_time[i] = Self::Complex::new(
                            tr * twist.re - ti * twist.im,
                            ti * twist.re + tr * twist.im,
                        );
                    }
                } else if !can_use_time {
                    let transmuted_time =
                        unsafe { &*((&raw const time).cast::<&[Self::Complex]>()) };

                    self.tmp_time.copy_from_slice(&transmuted_time[0..h_size]);
                }
            } else if step < self.complex_fft.steps() {
                let transmuted_time = unsafe { *((&raw const time).cast::<&[Self::Complex]>()) };

                self.complex_fft.fft_step(
                    step,
                    if can_use_time {
                        transmuted_time
                    } else {
                        &self.tmp_time
                    },
                    &mut self.tmp_freq,
                );
            } else {
                if !HALF_BIN_SHIFT {
                    let bin0 = self.tmp_freq[0];

                    freq[0] = Self::Complex::new(
                        // pack DC & Nyquist together
                        bin0.re + bin0.im,
                        bin0.re - bin0.im,
                    );
                }

                let h_size = self.complex_fft.size();

                let mut start_i = if HALF_BIN_SHIFT { 0 } else { 1 };
                let mut end_i = h_size / 2 + 1;

                if SPLIT_COMPUTATION {
                    // Do this last twiddle in two halves
                    if step == self.complex_fft.steps() {
                        end_i = usize::midpoint(start_i, end_i);
                    } else {
                        start_i = usize::midpoint(start_i, end_i);
                    }
                }
                for i in start_i..end_i {
                    let conj_i = if HALF_BIN_SHIFT {
                        h_size - 1 - i
                    } else {
                        h_size - i
                    };

                    let twiddle = self.twiddles[i];

                    let odd = (self.tmp_freq[i] + self.tmp_freq[conj_i].conj()) * 0.5;
                    let even_i = (self.tmp_freq[i] - self.tmp_freq[conj_i].conj()) * 0.5;

                    let even_rot_minus_i = Self::Complex::new(
                        // twiddle includes a factor of -i
                        even_i.re * twiddle.re - even_i.im * twiddle.im,
                        even_i.im * twiddle.re + even_i.re * twiddle.im,
                    );

                    freq[i] = odd + even_rot_minus_i;
                    freq[conj_i] = Self::Complex::new(
                        odd.re - even_rot_minus_i.re,
                        even_rot_minus_i.im - odd.im,
                    );
                }
            }
        }
    }

    fn fft_split_complex(
        &mut self,
        in_r: &[Self::Sample],
        out_r: &mut [Self::Sample],
        out_i: &mut [Self::Sample],
    ) {
        for s in 0..self.steps() {
            self.fft_step_split_complex(s, in_r, out_r, out_i);
        }
    }

    #[allow(
        clippy::indexing_slicing,
        clippy::bool_to_int_with_if,
        clippy::undocumented_unsafe_blocks
    )]
    fn fft_step_split_complex(
        &mut self,
        mut step: usize,
        in_r: &[Self::Sample],
        out_r: &mut [Self::Sample],
        out_i: &mut [Self::Sample],
    ) {
        let h_size = self.complex_fft.size();

        let (tmp_time_r, tmp_time_i) = complex_to_two_float_mut(&mut self.tmp_time);
        let (tmp_freq_r, tmp_freq_i) = complex_to_two_float_mut(&mut self.tmp_freq);

        let step_check = step;
        step -= 1;

        if step_check == 0 {
            let h_size = self.complex_fft.size();

            if HALF_BIN_SHIFT {
                for i in 0..h_size {
                    let tr = in_r[2 * i];
                    let ti = in_r[2 * i + 1];

                    let twist = self.half_bin_twists[i];

                    tmp_time_r[i] = tr * twist.re - ti * twist.im;
                    tmp_time_i[i] = ti * twist.re + tr * twist.im;
                }
            } else {
                for i in 0..h_size {
                    tmp_time_r[i] = in_r[2 * i];
                    tmp_time_i[i] = in_r[2 * i + 1];
                }
            }
        } else if step < self.complex_fft.steps() {
            self.complex_fft
                .fft_step_split_complex(step, tmp_time_r, tmp_time_i, tmp_freq_r, tmp_freq_i);
        } else {
            if !HALF_BIN_SHIFT {
                let bin0_r = tmp_freq_r[0];
                let bin0_i = tmp_freq_i[0];
                out_r[0] = bin0_r + bin0_i;
                out_i[0] = bin0_r - bin0_i;
            }

            let mut start_i = if HALF_BIN_SHIFT { 0 } else { 1 };
            let mut end_i = h_size / 2 + 1;

            if SPLIT_COMPUTATION {
                // Do this last twiddle in two halves
                if step == self.complex_fft.steps() {
                    end_i = usize::midpoint(start_i, end_i);
                } else {
                    start_i = usize::midpoint(start_i, end_i);
                }
            }

            for i in start_i..end_i {
                let conj_i = if HALF_BIN_SHIFT {
                    h_size - 1 - i
                } else {
                    h_size - i
                };
                let twiddle = self.twiddles[i];

                let odd_r = (tmp_freq_r[i] + tmp_freq_r[conj_i]) * 0.5;
                let odd_i = (tmp_freq_i[i] - tmp_freq_i[conj_i]) * 0.5;
                let even_i_r = (tmp_freq_r[i] - tmp_freq_r[conj_i]) * 0.5;
                let even_i_i = (tmp_freq_i[i] + tmp_freq_i[conj_i]) * 0.5;
                let even_rot_minus_i_r = even_i_r * twiddle.re - even_i_i * twiddle.im;
                let even_rot_minus_i_i = even_i_i * twiddle.re + even_i_r * twiddle.im;

                out_r[i] = odd_r + even_rot_minus_i_r;
                out_i[i] = odd_i + even_rot_minus_i_i;
                out_r[conj_i] = odd_r - even_rot_minus_i_r;
                out_i[conj_i] = even_rot_minus_i_i - odd_i;
            }
        }
    }

    fn ifft(&mut self, freq: &[Self::Complex], time: &mut [Self::Sample]) {
        for s in 0..self.steps() {
            self.ifft_step(s, freq, time);
        }
    }

    #[allow(
        clippy::too_many_lines,
        clippy::indexing_slicing,
        clippy::bool_to_int_with_if,
        clippy::undocumented_unsafe_blocks
    )]
    fn ifft_step(
        &mut self,
        mut step: usize,
        freq: &[Self::Complex],
        mut time: &mut [Self::Sample],
    ) {
        if Self::PREFERS_SPLIT {
            let h_size = self.complex_fft.size();

            let (tmp_time_r, tmp_time_i) = complex_to_two_float_mut(&mut self.tmp_time);
            let (tmp_freq_r, tmp_freq_i) = complex_to_two_float_mut(&mut self.tmp_freq);

            let step_check = step;
            step -= 1;

            let split_frst = SPLIT_COMPUTATION && (step_check == 0);

            let step_check = step;
            step -= 1;

            if split_frst || step_check == 0 {
                let bin0 = freq[0];

                if !HALF_BIN_SHIFT {
                    tmp_freq_r[0] = bin0.re + bin0.im;
                    tmp_freq_i[0] = bin0.re - bin0.im;
                }

                let mut start_i = if HALF_BIN_SHIFT { 0 } else { 1 };
                let mut end_i = h_size / 2 + 1;

                if SPLIT_COMPUTATION {
                    // Do this first twiddle in two halves
                    if split_frst {
                        end_i = usize::midpoint(start_i, end_i);
                    } else {
                        start_i = usize::midpoint(start_i, end_i);
                    }
                }

                for i in start_i..end_i {
                    let conj_i = if HALF_BIN_SHIFT {
                        h_size - 1 - i
                    } else {
                        h_size - i
                    };
                    let twiddle = self.twiddles[i];

                    let odd = freq[i] + freq[conj_i].conj();
                    let even_rot_minus_i = freq[i] - freq[conj_i].conj();

                    let even_i = Self::Complex::new(
                        // Conjugate twiddle
                        even_rot_minus_i.re * twiddle.re + even_rot_minus_i.im * twiddle.im,
                        even_rot_minus_i.im * twiddle.re - even_rot_minus_i.re * twiddle.im,
                    );

                    tmp_freq_r[i] = odd.re + even_i.re;
                    tmp_freq_i[i] = odd.im + even_i.im;
                    tmp_freq_r[conj_i] = odd.re - even_i.re;
                    tmp_freq_i[conj_i] = even_i.im - odd.im;
                }
            } else if step < self.complex_fft.steps() {
                self.complex_fft
                    .ifft_step_split_complex(step, tmp_freq_r, tmp_freq_i, tmp_time_r, tmp_time_i);
            } else {
                let h_size = self.complex_fft.size();

                if HALF_BIN_SHIFT {
                    for i in 0..h_size {
                        let tr = tmp_time_r[i];
                        let ti = tmp_time_i[i];

                        let twist = self.half_bin_twists[i];

                        time[2 * i] = tr * twist.re + ti * twist.im;
                        time[2 * i + 1] = ti * twist.re - tr * twist.im;
                    }
                } else {
                    for i in 0..h_size {
                        time[2 * i] = tmp_time_r[i];
                        time[2 * i + 1] = tmp_time_i[i];
                    }
                }
            }
        } else {
            let raw_time = &raw const time;

            let can_use_time = !HALF_BIN_SHIFT
                && (raw_time as usize).is_multiple_of(std::mem::align_of::<Self::Complex>());

            let step_check = step;
            step -= 1;

            let split_first = SPLIT_COMPUTATION && (step_check == 0);

            let step_check = step;
            step -= 1;

            if split_first || step_check == 0 {
                let bin0 = freq[0];

                if !HALF_BIN_SHIFT {
                    self.tmp_freq[0] = Self::Complex::new(bin0.re + bin0.im, bin0.re - bin0.im);
                }

                let h_size = self.complex_fft.size();

                let mut start_i = if HALF_BIN_SHIFT { 0 } else { 1 };
                let mut end_i = h_size / 2 + 1;

                if SPLIT_COMPUTATION {
                    // Do this first twiddle in two halves
                    if split_first {
                        end_i = usize::midpoint(start_i, end_i);
                    } else {
                        start_i = usize::midpoint(start_i, end_i);
                    }
                }

                for i in start_i..end_i {
                    let conj_i = if HALF_BIN_SHIFT {
                        h_size - 1 - i
                    } else {
                        h_size - i
                    };
                    let twiddle = self.twiddles[i];

                    let odd = freq[i] + freq[conj_i].conj();
                    let even_rot_minus_i = freq[i] - freq[conj_i].conj();

                    let even_i = Self::Complex::new(
                        // Conjugate twiddle
                        even_rot_minus_i.re * twiddle.re + even_rot_minus_i.im * twiddle.im,
                        even_rot_minus_i.im * twiddle.re - even_rot_minus_i.re * twiddle.im,
                    );

                    self.tmp_freq[i] = odd + even_i;
                    self.tmp_freq[conj_i] =
                        Self::Complex::new(odd.re - even_i.re, even_i.im - odd.im);
                }
            } else if step < self.complex_fft.steps() {
                let transmuted_time: &mut [Self::Complex] =
                    unsafe { *((&raw mut time).cast::<&mut [Self::Complex]>()) };

                // Can't just use time as (Complex *), since it might not be aligned properly
                self.complex_fft.ifft_step(
                    step,
                    &self.tmp_freq,
                    if can_use_time {
                        transmuted_time
                    } else {
                        &mut self.tmp_time
                    },
                );
            } else {
                let h_size = self.complex_fft.size();

                if HALF_BIN_SHIFT {
                    for i in 0..h_size {
                        let t = self.tmp_time[i];
                        let twist = self.half_bin_twists[i];

                        time[2 * i] = t.re * twist.re + t.im * twist.im;
                        time[2 * i + 1] = t.im * twist.re - t.re * twist.im;
                    }
                } else if !can_use_time {
                    let transmuted_tmp_time =
                        unsafe { &*((&raw const self.tmp_time).cast::<&[Self::Sample]>()) };

                    time.copy_from_slice(&transmuted_tmp_time[0..(h_size * 2)]);
                }
            }
        }
    }

    fn ifft_split_complex(
        &mut self,
        in_r: &[Self::Sample],
        in_i: &[Self::Sample],
        out_r: &mut [Self::Sample],
    ) {
        for s in 0..self.steps() {
            self.ifft_step_split_complex(s, in_r, in_i, out_r);
        }
    }

    #[allow(
        clippy::indexing_slicing,
        clippy::bool_to_int_with_if,
        clippy::undocumented_unsafe_blocks,
        clippy::similar_names
    )]
    fn ifft_step_split_complex(
        &mut self,
        mut step: usize,
        in_r: &[Self::Sample],
        in_i: &[Self::Sample],
        out_r: &mut [Self::Sample],
    ) {
        let h_size = self.complex_fft.size();

        let (tmp_time_r, tmp_time_i) = complex_to_two_float_mut(&mut self.tmp_time);
        let (tmp_freq_r, tmp_freq_i) = complex_to_two_float_mut(&mut self.tmp_freq);

        let step_check = step;
        step -= 1;

        let split_first = SPLIT_COMPUTATION && (step_check == 0);

        let step_check = step;
        step -= 1;

        if split_first || step_check == 0 {
            let bin0_r = in_r[0];
            let bin0_i = in_i[0];

            if !HALF_BIN_SHIFT {
                tmp_freq_r[0] = bin0_r + bin0_i;
                tmp_freq_i[0] = bin0_r - bin0_i;
            }

            let mut start_i = if HALF_BIN_SHIFT { 0 } else { 1 };
            let mut end_i = h_size / 2 + 1;

            if SPLIT_COMPUTATION {
                // Do this first twiddle in two halves
                if split_first {
                    end_i = usize::midpoint(start_i, end_i);
                } else {
                    start_i = usize::midpoint(start_i, end_i);
                }
            }

            for i in start_i..end_i {
                let conj_i = if HALF_BIN_SHIFT {
                    h_size - 1 - i
                } else {
                    h_size - i
                };

                let twiddle = self.twiddles[i];
                let fir = in_r[i];
                let fii = in_i[i];
                let fcir = in_r[conj_i];
                let fcii = in_i[conj_i];

                let odd = Self::Complex::new(fir + fcir, fii - fcii);
                let even_rot_minus_i = Self::Complex::new(fir - fcir, fii + fcii);

                let even_i = Self::Complex::new(
                    // Conjugate twiddle
                    even_rot_minus_i.re * twiddle.re + even_rot_minus_i.im * twiddle.im,
                    even_rot_minus_i.im * twiddle.re - even_rot_minus_i.re * twiddle.im,
                );

                tmp_freq_r[i] = odd.re + even_i.re;
                tmp_freq_i[i] = odd.im + even_i.im;
                tmp_freq_r[conj_i] = odd.re - even_i.re;
                tmp_freq_i[conj_i] = even_i.im - odd.im;
            }
        } else if step < self.complex_fft.steps() {
            // Can't just use time as (Complex *), since it might not be aligned properly
            self.complex_fft
                .ifft_step_split_complex(step, tmp_freq_r, tmp_freq_i, tmp_time_r, tmp_time_i);
        } else if HALF_BIN_SHIFT {
            for i in 0..h_size {
                let tr = tmp_time_r[i];
                let ti = tmp_time_i[i];
                let twist = self.half_bin_twists[i];

                out_r[2 * i] = tr * twist.re + ti * twist.im;
                out_r[2 * i + 1] = ti * twist.re - tr * twist.im;
            }
        } else {
            for i in 0..h_size {
                out_r[2 * i] = tmp_time_r[i];
                out_r[2 * i + 1] = tmp_time_i[i];
            }
        }
    }
}
