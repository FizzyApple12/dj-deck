pub mod approximate_confined_gaussian;
pub mod kaiser;

use num::{Complex, Float};

use crate::linear::{
    fft::real_fft::{RealFFT, RealFFTTrait},
    stft::{approximate_confined_gaussian::ApproximateConfinedGaussian, kaiser::Kaiser},
};

// not using an enum here due to rust generics restrictions
pub const STFT_SPECTRUM_PACKED: usize = 0;
pub const STFT_SPECTRUM_MODIFIED: usize = 1;
pub const STFT_SPECTRUM_UNPACKED: usize = 2;

#[derive(Clone, Copy, PartialEq)]
pub enum WindowShape {
    Ignore,
    ACG,
    Kaiser,
}

// Input (only available so we can save/restore the input state)
#[derive(Clone)]
pub struct Input<Sample> {
    pub pos: usize,
    pub buffer: Vec<Sample>,
}

pub trait InputTrait<Sample> {
    fn swap(&mut self, other: &mut Input<Sample>);
}

impl<Sample> InputTrait<Sample> for Input<Sample> {
    fn swap(&mut self, other: &mut Input<Sample>) {
        std::mem::swap(&mut self.pos, &mut other.pos);
        std::mem::swap(&mut self.buffer, &mut other.buffer);
    }
}

// Output (only available so we can save/restore the output state)
#[derive(Clone)]
pub struct Output<Sample> {
    pub pos: usize,
    pub buffer: Vec<Sample>,
    pub window_products: Vec<Sample>,
}

pub trait OutputTrait<Sample> {
    fn swap(&mut self, other: &mut Output<Sample>);
}

impl<Sample> OutputTrait<Sample> for Output<Sample> {
    fn swap(&mut self, other: &mut Output<Sample>) {
        std::mem::swap(&mut self.pos, &mut other.pos);
        std::mem::swap(&mut self.buffer, &mut other.buffer);
        std::mem::swap(&mut self.window_products, &mut other.window_products);
    }
}

/// A self-normalising STFT, with variable position/window for output blocks
///
/// YOU NEED TO SET `HALF_BIN_SHIFT = true` WHEN `SPECTRUM_TYPE ==
/// STFT_SPECTRUM_MODIFIED`
pub struct DynamicSTFT<
    Sample,
    const SPLIT_COMPUTATION: bool,
    const SPECTRUM_TYPE: usize,
    const HALF_BIN_SHIFT: bool,
> where
    Sample: Float,
{
    pub fft: RealFFT<Sample, SPLIT_COMPUTATION, HALF_BIN_SHIFT>,

    pub input: Input<Sample>,
    pub output: Output<Sample>,

    internal_analysis_channels: usize,
    internal_synthesis_channels: usize,
    internal_input_length_samples: usize,
    internal_block_samples: usize,
    internal_fft_samples: usize,
    internal_fft_bins: usize,
    internal_default_interval: usize,

    internal_analysis_window: Vec<Sample>,
    internal_synthesis_window: Vec<Sample>,
    internal_analysis_offset: usize,
    internal_synthesis_offset: usize,

    spectrum_buffer: Vec<Complex<Sample>>,
    time_buffer: Vec<Sample>,

    internal_samples_since_synthesis: usize,
    internal_samples_since_analysis: usize,
}

pub trait DynamicSTFTTrait<
    Sample,
    const SPLIT_COMPUTATION: bool,
    const SPECTRUM_TYPE: usize,
    const HALF_BIN_SHIFT: bool,
> where
    Sample: Float,
{
    type Complex;
    type Sample;

    const MODIFIED: bool = (SPECTRUM_TYPE == STFT_SPECTRUM_MODIFIED);
    const UNPACKED: bool = (SPECTRUM_TYPE == STFT_SPECTRUM_UNPACKED);

    const ALMOST_ZERO: Self::Sample;

    fn new() -> Self;

    fn configure(
        &mut self,
        in_channels: usize,
        out_channels: usize,
        block_samples: usize,
        extra_input_history: usize,
        interval_samples: usize,
        asymmetry: Self::Sample,
    );

    fn block_samples(&self) -> usize;
    fn fft_samples(&self) -> usize;
    fn default_interval(&self) -> usize;
    fn bands(&self) -> usize;
    fn analysis_latency(&self) -> usize;
    fn synthesis_latency(&self) -> usize;
    fn latency(&self) -> usize;

    fn bin_to_freq(&self, b: Self::Sample) -> Self::Sample;
    fn freq_to_bin(&self, f: Self::Sample) -> Self::Sample;

    fn reset(&mut self, product_weight: Self::Sample);

    fn write_input_offset(
        &mut self,
        channel: usize,
        offset: usize,
        length: usize,
        input_array: &[Self::Sample],
    );

    fn write_input(&mut self, channel: usize, length: usize, input_array: &[Self::Sample]);

    fn move_input(&mut self, samples: usize, clear_input: bool);

    fn samples_since_analysis(&self) -> usize;

    fn finish_output(&mut self, strength: Self::Sample, offset: usize);

    fn read_output_offset(
        &self,
        channel: usize,
        offset: usize,
        length: usize,
        output_array: &mut [Self::Sample],
    );

    fn read_output(&self, channel: usize, length: usize, output_array: &mut [Self::Sample]);

    fn add_output_offset(
        &mut self,
        channel: usize,
        offset: usize,
        length: usize,
        new_output_array: &[Self::Sample],
    );
    fn add_output(&mut self, channel: usize, length: usize, new_output_array: &[Self::Sample]);

    fn replace_output_offset(
        &mut self,
        channel: usize,
        offset: usize,
        length: usize,
        new_output_array: &[Self::Sample],
    );
    fn replace_output(&mut self, channel: usize, length: usize, new_output_array: &[Self::Sample]);

    fn move_output(&mut self, samples: usize);

    fn samples_since_synthesis(&self) -> usize;

    fn spectrum(&self, channel: usize) -> &[Self::Complex];
    fn spectrum_mut(&mut self, channel: usize) -> &mut [Self::Complex];

    fn analysis_window(&self) -> &[Self::Sample];
    fn analysis_window_mut(&mut self) -> &mut [Self::Sample];

    // Sets the centre index of the window
    fn set_analysis_offset(&mut self, offset: usize);
    fn get_analysis_offset(&self) -> usize;

    fn synthesis_window(&self) -> &[Self::Sample];
    fn synthesis_window_mut(&mut self) -> &mut [Self::Sample];

    // Sets the centre index of the window
    fn set_synthesis_offset(&mut self, offset: usize);
    fn get_synthesis_offset(&self) -> usize;

    fn set_interval(
        &mut self,
        default_interval: usize,
        window_shape: WindowShape,
        asymmetry: Self::Sample,
    );

    fn analyse(&mut self, samples_in_past: usize);
    fn analyse_steps(&self) -> usize;
    fn analyse_step(&mut self, step: usize, samples_in_past: usize);

    fn synthesise(&mut self);
    fn synthesise_steps(&self) -> usize;
    fn synthesise_step(&mut self, step: usize);
}

impl<const SPLIT_COMPUTATION: bool, const SPECTRUM_TYPE: usize, const HALF_BIN_SHIFT: bool>
    DynamicSTFTTrait<f32, SPLIT_COMPUTATION, SPECTRUM_TYPE, HALF_BIN_SHIFT>
    for DynamicSTFT<f32, SPLIT_COMPUTATION, SPECTRUM_TYPE, HALF_BIN_SHIFT>
{
    type Complex = Complex<f32>;
    type Sample = f32;

    const ALMOST_ZERO: f32 = f32::EPSILON;

    fn new() -> Self {
        Self {
            fft: RealFFT::<f32, SPLIT_COMPUTATION, HALF_BIN_SHIFT>::new(0),
            input: Input {
                pos: 0,
                buffer: Vec::new(),
            },
            output: Output {
                pos: 0,
                buffer: Vec::new(),
                window_products: Vec::new(),
            },
            internal_analysis_channels: 0,
            internal_synthesis_channels: 0,
            internal_input_length_samples: 0,
            internal_block_samples: 0,
            internal_fft_samples: 0,
            internal_fft_bins: 0,
            internal_default_interval: 0,
            internal_analysis_window: Vec::new(),
            internal_synthesis_window: Vec::new(),
            internal_analysis_offset: 0,
            internal_synthesis_offset: 0,
            spectrum_buffer: Vec::new(),
            time_buffer: Vec::new(),
            internal_samples_since_synthesis: 0,
            internal_samples_since_analysis: 0,
        }
    }

    fn configure(
        &mut self,
        in_channels: usize,
        out_channels: usize,
        block_samples: usize,
        extra_input_history: usize,
        interval_samples: usize,
        asymmetry: f32,
    ) {
        self.internal_analysis_channels = in_channels;
        self.internal_synthesis_channels = out_channels;
        self.internal_block_samples = block_samples;
        self.internal_fft_samples =
            RealFFT::<f32, SPLIT_COMPUTATION, false>::fast_size_above(block_samples.div_ceil(2))
                * 2;
        self.fft.resize(self.internal_fft_samples);
        self.internal_fft_bins =
            self.internal_fft_samples / 2 + usize::from(SPECTRUM_TYPE == STFT_SPECTRUM_UNPACKED);

        self.internal_input_length_samples = self.internal_block_samples + extra_input_history;
        self.input.buffer.resize(
            self.internal_input_length_samples * self.internal_analysis_channels,
            0.0,
        );

        self.output.buffer.resize(
            self.internal_block_samples * self.internal_synthesis_channels,
            0.0,
        );
        self.output
            .window_products
            .resize(self.internal_block_samples, 0.0);
        self.spectrum_buffer.resize(
            self.internal_fft_bins
                * usize::max(
                    self.internal_analysis_channels,
                    self.internal_synthesis_channels,
                ),
            Complex::<f32>::new(0.0, 0.0),
        );
        self.time_buffer.resize(self.internal_fft_samples, 0.0);

        self.internal_analysis_window
            .resize(self.internal_block_samples, 0.0);
        self.internal_synthesis_window
            .resize(self.internal_block_samples, 0.0);
        self.set_interval(
            if interval_samples == 1 {
                interval_samples
            } else {
                block_samples / 4
            },
            WindowShape::ACG,
            asymmetry,
        );

        self.reset(1.0);
    }

    fn block_samples(&self) -> usize {
        self.internal_block_samples
    }

    fn fft_samples(&self) -> usize {
        self.internal_fft_samples
    }

    fn default_interval(&self) -> usize {
        self.internal_default_interval
    }

    fn bands(&self) -> usize {
        self.internal_fft_bins
    }

    fn analysis_latency(&self) -> usize {
        self.internal_block_samples - self.internal_analysis_offset
    }

    fn synthesis_latency(&self) -> usize {
        self.internal_synthesis_offset
    }

    fn latency(&self) -> usize {
        self.synthesis_latency() + self.analysis_latency()
    }

    #[allow(clippy::cast_precision_loss)]
    fn bin_to_freq(&self, b: f32) -> f32 {
        (if Self::MODIFIED { b + 0.5 } else { b }) / self.internal_fft_samples as f32
    }

    #[allow(clippy::cast_precision_loss)]
    fn freq_to_bin(&self, f: f32) -> f32 {
        if Self::MODIFIED {
            f * self.internal_fft_samples as f32 - 0.5
        } else {
            f * self.internal_fft_samples as f32
        }
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss,
        clippy::indexing_slicing
    )]
    fn reset(&mut self, product_weight: f32) {
        self.input.pos = self.internal_block_samples;
        self.output.pos = 0;

        self.input.buffer.fill(0.0);
        self.output.buffer.fill(0.0);
        for v in &mut self.spectrum_buffer {
            *v = Complex::<f32>::new(0.0, 0.0);
        }
        self.output.window_products.fill(0.0);

        self.add_window_product();

        for i in 0..(self.internal_block_samples as i32 - self.internal_default_interval as i32 - 1)
        {
            self.output.window_products[i as usize] +=
                self.output.window_products[i as usize + self.internal_default_interval];
        }

        for v in &mut self.output.window_products {
            *v = *v * product_weight + Self::ALMOST_ZERO;
        }

        self.move_output(self.internal_default_interval); // ready for first block immediately
    }

    #[allow(clippy::needless_range_loop, clippy::indexing_slicing)]
    fn write_input_offset(
        &mut self,
        channel: usize,
        offset: usize,
        length: usize,
        input_array: &[f32],
    ) {
        let offset_pos = (self.input.pos + offset) % self.internal_input_length_samples;
        let input_wrap_index = self.internal_input_length_samples - offset_pos;
        let chunk1 = usize::min(length, input_wrap_index);

        for i in 0..chunk1 {
            let i2 = offset_pos + i;

            self.input.buffer[channel * self.internal_input_length_samples + i2] = input_array[i];
        }

        for i in chunk1..length {
            let i2 = i + offset_pos - self.internal_input_length_samples;

            self.input.buffer[channel * self.internal_input_length_samples + i2] = input_array[i];
        }
    }

    fn write_input(&mut self, channel: usize, length: usize, input_array: &[f32]) {
        self.write_input_offset(channel, 0, length, input_array);
    }

    #[allow(clippy::indexing_slicing)]
    fn move_input(&mut self, samples: usize, clear_input: bool) {
        if clear_input {
            let input_wrap_index = self.internal_input_length_samples - self.input.pos;
            let chunk1 = usize::min(samples, input_wrap_index);

            for c in 0..self.internal_analysis_channels {
                for i in 0..chunk1 {
                    let i2 = self.input.pos + i;

                    self.input.buffer[c * self.internal_input_length_samples + i2] = 0.0;
                }

                for i in chunk1..samples {
                    let i2 = i + self.input.pos - self.internal_input_length_samples;

                    self.input.buffer[c * self.internal_input_length_samples + i2] = 0.0;
                }
            }
        }

        self.input.pos = (self.input.pos + samples) % self.internal_input_length_samples;
        self.internal_samples_since_analysis += samples;
    }

    fn samples_since_analysis(&self) -> usize {
        self.internal_samples_since_analysis
    }

    // When no more synthesis is expected, let output taper away to 0 based on
    // windowing.  Otherwise, the output will be scaled as if there's just a very
    // long block interval, which can exaggerate artefacts and numerical errors.
    // You still can't read more than `blockSamples()` into the future.
    #[allow(clippy::indexing_slicing)]
    fn finish_output(&mut self, strength: f32, offset: usize) {
        let mut max_window_product = 0.0;

        let chunk1 = usize::max(
            offset,
            usize::min(
                self.internal_block_samples,
                self.internal_block_samples - self.output.pos,
            ),
        );

        for i in offset..chunk1 {
            let i2 = self.output.pos + i;
            let wp = self.output.window_products[i2];

            max_window_product = f32::max(wp, max_window_product);

            self.output.window_products[i2] += (max_window_product - wp) * strength;
        }

        for i in chunk1..self.internal_block_samples {
            let i2 = i + self.output.pos - self.internal_block_samples;
            let wp = self.output.window_products[i2];

            max_window_product = f32::max(wp, max_window_product);

            self.output.window_products[i2] += (max_window_product - wp) * strength;
        }
    }

    #[allow(clippy::needless_range_loop, clippy::indexing_slicing, unused)]
    fn read_output_offset(
        &self,
        channel: usize,
        offset: usize,
        length: usize,
        mut output_array: &mut [f32],
    ) {
        let offset_pos = (self.output.pos + offset) % self.internal_block_samples;
        let output_wrap_index = self.internal_block_samples - offset_pos;
        let chunk1 = usize::min(length, output_wrap_index);

        for i in 0..chunk1 {
            let i2 = offset_pos + i;

            output_array[i] = self.output.buffer[channel * self.internal_block_samples + i2]
                / self.output.window_products[i2];
        }

        for i in chunk1..length {
            let i2 = i + offset_pos - self.internal_block_samples;

            output_array[i] = self.output.buffer[channel * self.internal_block_samples + i2]
                / self.output.window_products[i2];
        }
    }

    fn read_output(&self, channel: usize, length: usize, output_array: &mut [f32]) {
        self.read_output_offset(channel, 0, length, output_array);
    }

    #[allow(clippy::needless_range_loop, clippy::indexing_slicing)]
    fn add_output_offset(
        &mut self,
        channel: usize,
        offset: usize,
        mut length: usize,
        new_output_array: &[f32],
    ) {
        length = usize::min(self.internal_block_samples, length);

        let offset_pos = (self.output.pos + offset) % self.internal_block_samples;
        let output_wrap_index = self.internal_block_samples - offset_pos;
        let chunk1 = usize::min(length, output_wrap_index);

        for i in 0..chunk1 {
            let i2 = offset_pos + i;

            self.output.buffer[channel * self.internal_block_samples + i2] +=
                new_output_array[i] * self.output.window_products[i2];
        }

        for i in chunk1..length {
            let i2 = i + offset_pos - self.internal_block_samples;

            self.output.buffer[channel * self.internal_block_samples + i2] +=
                new_output_array[i] * self.output.window_products[i2];
        }
    }

    fn add_output(&mut self, channel: usize, length: usize, new_output_array: &[f32]) {
        self.add_output_offset(channel, 0, length, new_output_array);
    }

    #[allow(clippy::needless_range_loop, clippy::indexing_slicing)]
    fn replace_output_offset(
        &mut self,
        channel: usize,
        offset: usize,
        mut length: usize,
        new_output_array: &[f32],
    ) {
        length = usize::min(self.internal_block_samples, length);

        let offset_pos = (self.output.pos + offset) % self.internal_block_samples;
        let output_wrap_index = self.internal_block_samples - offset_pos;
        let chunk1 = usize::min(length, output_wrap_index);

        for i in 0..chunk1 {
            let i2 = offset_pos + i;

            self.output.buffer[channel * self.internal_block_samples + i2] =
                new_output_array[i] * self.output.window_products[i2];
        }

        for i in chunk1..length {
            let i2 = i + offset_pos - self.internal_block_samples;

            self.output.buffer[channel * self.internal_block_samples + i2] =
                new_output_array[i] * self.output.window_products[i2];
        }
    }

    fn replace_output(&mut self, channel: usize, length: usize, new_output_array: &[f32]) {
        self.replace_output_offset(channel, 0, length, new_output_array);
    }

    #[allow(clippy::indexing_slicing)]
    fn move_output(&mut self, samples: usize) {
        if samples == 1 {
            // avoid all the loops/chunks if we can
            for c in 0..self.internal_synthesis_channels {
                self.output.buffer[self.output.pos + c * self.internal_block_samples] = 0.0;
            }

            self.output.window_products[self.output.pos] = Self::ALMOST_ZERO;

            self.output.pos += 1;

            if self.output.pos >= self.internal_block_samples {
                self.output.pos = 0;
            }

            return;
        }

        // Zero the output buffer as we cross it
        let output_wrap_index = self.internal_block_samples - self.output.pos;
        let chunk1 = usize::min(samples, output_wrap_index);

        for c in 0..self.internal_synthesis_channels {
            for i in 0..chunk1 {
                let i2 = self.output.pos + i;

                self.output.buffer[c * self.internal_block_samples + i2] = 0.0;
            }

            for i in chunk1..samples {
                let i2 = i + self.output.pos - self.internal_block_samples;

                self.output.buffer[c * self.internal_block_samples + i2] = 0.0;
            }
        }

        for i in 0..chunk1 {
            let i2 = self.output.pos + i;

            self.output.window_products[i2] = Self::ALMOST_ZERO;
        }

        for i in chunk1..samples {
            let i2 = i + self.output.pos - self.internal_block_samples;

            self.output.window_products[i2] = Self::ALMOST_ZERO;
        }

        self.output.pos = (self.output.pos + samples) % self.internal_block_samples;
        self.internal_samples_since_synthesis += samples;
    }

    fn samples_since_synthesis(&self) -> usize {
        self.internal_samples_since_synthesis
    }

    #[allow(clippy::indexing_slicing)]
    fn spectrum(&self, channel: usize) -> &[Complex<f32>] {
        &self.spectrum_buffer[(channel * self.internal_fft_bins)..]
    }

    #[allow(clippy::indexing_slicing)]
    fn spectrum_mut(&mut self, channel: usize) -> &mut [Complex<f32>] {
        &mut self.spectrum_buffer[(channel * self.internal_fft_bins)..]
    }

    fn analysis_window(&self) -> &[f32] {
        &self.internal_analysis_window
    }

    fn analysis_window_mut(&mut self) -> &mut [f32] {
        &mut self.internal_analysis_window
    }

    // Sets the centre index of the window
    fn set_analysis_offset(&mut self, offset: usize) {
        self.internal_analysis_offset = offset;
    }

    fn get_analysis_offset(&self) -> usize {
        self.internal_analysis_offset
    }

    fn synthesis_window(&self) -> &[f32] {
        &self.internal_synthesis_window
    }

    fn synthesis_window_mut(&mut self) -> &mut [f32] {
        &mut self.internal_synthesis_window
    }

    // Sets the centre index of the window
    fn set_synthesis_offset(&mut self, offset: usize) {
        self.internal_synthesis_offset = offset;
    }

    fn get_synthesis_offset(&self) -> usize {
        self.internal_synthesis_offset
    }

    #[allow(
        clippy::indexing_slicing,
        clippy::cast_lossless,
        clippy::cast_precision_loss
    )]
    fn set_interval(&mut self, default_interval: usize, window_shape: WindowShape, asymmetry: f32) {
        self.internal_default_interval = default_interval;
        if window_shape == WindowShape::Ignore {
            return;
        }

        if window_shape == WindowShape::ACG {
            let window = ApproximateConfinedGaussian::with_bandwidth(
                self.internal_block_samples as f64 / default_interval as f64,
            );
            window.fill(
                &mut self.internal_synthesis_window,
                self.internal_block_samples,
                asymmetry as f64,
                false,
            );
        } else if window_shape == WindowShape::Kaiser {
            let window = Kaiser::with_bandwidth(
                self.internal_block_samples as f64 / default_interval as f64,
                true,
            );
            window.fill(
                &mut self.internal_synthesis_window,
                self.internal_block_samples,
                asymmetry as f64,
                true,
            );
        }

        self.internal_analysis_offset = self.internal_block_samples / 2;
        self.internal_synthesis_offset = self.internal_block_samples / 2;

        if self.internal_analysis_channels == 0 {
            self.internal_analysis_window.fill(1.0);
        } else if asymmetry == 0.0 {
            DynamicSTFT::<f32, SPLIT_COMPUTATION, SPECTRUM_TYPE, HALF_BIN_SHIFT>::force_perfect_reconstruction(
                &mut self.internal_synthesis_window,
                self.internal_block_samples,
                self.internal_default_interval,
            );
            for i in 0..self.internal_block_samples {
                self.internal_analysis_window[i] = self.internal_synthesis_window[i];
            }
        } else {
            for i in 0..self.internal_block_samples {
                self.internal_analysis_window[i] =
                    self.internal_synthesis_window[self.internal_block_samples - 1 - i];
            }
        }

        // Set offsets to peak's index
        for i in 0..self.internal_block_samples {
            if self.internal_analysis_window[i]
                > self.internal_analysis_window[self.internal_analysis_offset]
            {
                self.internal_analysis_offset = i;

                if self.internal_synthesis_window[i]
                    > self.internal_synthesis_window[self.internal_synthesis_offset]
                {
                    self.internal_synthesis_offset = i;
                }
            }
        }
    }

    fn analyse(&mut self, samples_in_past: usize) {
        for s in 0..self.analyse_steps() {
            self.analyse_step(s, samples_in_past);
        }
    }

    fn analyse_steps(&self) -> usize {
        if SPLIT_COMPUTATION {
            self.internal_analysis_channels * (self.fft.steps() + 1)
        } else {
            self.internal_analysis_channels
        }
    }

    #[allow(clippy::indexing_slicing)]
    fn analyse_step(&mut self, mut step: usize, samples_in_past: usize) {
        let fft_steps = if SPLIT_COMPUTATION {
            self.fft.steps()
        } else {
            0
        };
        let channel = step / (fft_steps + 1);
        step -= channel * (fft_steps + 1);

        let step_check = step;
        step = step.saturating_sub(1);

        if step_check == 0 {
            // extra step at start of each channel: copy windowed input into buffer
            let offset_pos = (self.internal_input_length_samples * 2 + self.input.pos
                - self.internal_block_samples
                - samples_in_past)
                % self.internal_input_length_samples;
            let input_wrap_index = self.internal_input_length_samples - offset_pos;
            let chunk1 = usize::min(self.internal_analysis_offset, input_wrap_index);
            let chunk2 = usize::max(
                self.internal_analysis_offset,
                usize::min(self.internal_block_samples, input_wrap_index),
            );

            self.internal_samples_since_analysis = samples_in_past;

            for i in 0..chunk1 {
                let w = if Self::MODIFIED {
                    -self.internal_analysis_window[i]
                } else {
                    self.internal_analysis_window[i]
                };
                let ti = i + (self.internal_fft_samples - self.internal_analysis_offset);
                let bi = offset_pos + i;

                self.time_buffer[ti] =
                    self.input.buffer[channel * self.internal_input_length_samples + bi] * w;
            }

            for i in chunk1..self.internal_analysis_offset {
                let w = if Self::MODIFIED {
                    -self.internal_analysis_window[i]
                } else {
                    self.internal_analysis_window[i]
                };
                let ti = i + (self.internal_fft_samples - self.internal_analysis_offset);
                let bi = i + offset_pos - self.internal_input_length_samples;

                self.time_buffer[ti] =
                    self.input.buffer[channel * self.internal_input_length_samples + bi] * w;
            }

            for i in self.internal_analysis_offset..chunk2 {
                let w = self.internal_analysis_window[i];
                let ti = i - self.internal_analysis_offset;
                let bi = offset_pos + i;

                self.time_buffer[ti] =
                    self.input.buffer[channel * self.internal_input_length_samples + bi] * w;
            }

            for i in chunk2..self.internal_block_samples {
                let w = self.internal_analysis_window[i];
                let ti = i - self.internal_analysis_offset;
                let bi = i + offset_pos - self.internal_input_length_samples;

                self.time_buffer[ti] =
                    self.input.buffer[channel * self.internal_input_length_samples + bi] * w;
            }

            for i in (self.internal_block_samples - self.internal_analysis_offset)
                ..(self.internal_fft_samples - self.internal_analysis_offset)
            {
                self.time_buffer[i] = 0.0;
            }

            if SPLIT_COMPUTATION {
                return;
            }
        }

        // i *should* be using this, but this has a problem with mutable borrowing, so
        // instead i'm going to expand it
        // let mut spectrumPtr = self.spectrum_mut(channel);
        // &mut self.spectrumBuffer[(channel * self._fftBins)..]

        if SPLIT_COMPUTATION {
            self.fft.fft_step(
                step,
                &self.time_buffer,
                &mut self.spectrum_buffer[(channel * self.internal_fft_bins)..],
            );

            if Self::UNPACKED && step == self.fft.steps() - 1 {
                self.spectrum_buffer
                    [(channel * self.internal_fft_bins) + (self.internal_fft_bins - 1)] =
                    Complex::<f32>::new(
                        self.spectrum_buffer[channel * self.internal_fft_bins].im,
                        0.0,
                    );

                self.spectrum_buffer[channel * self.internal_fft_bins].im = 0.0;
            }
        } else {
            self.fft.fft(
                &self.time_buffer,
                &mut self.spectrum_buffer[(channel * self.internal_fft_bins)..],
            );

            if Self::UNPACKED {
                self.spectrum_buffer
                    [(channel * self.internal_fft_bins) + (self.internal_fft_bins - 1)] =
                    Complex::<f32>::new(
                        self.spectrum_buffer[channel * self.internal_fft_bins].im,
                        0.0,
                    );

                self.spectrum_buffer[channel * self.internal_fft_bins].im = 0.0;
            }
        }
    }

    fn synthesise(&mut self) {
        for s in 0..self.synthesise_steps() {
            self.synthesise_step(s);
        }
    }

    fn synthesise_steps(&self) -> usize {
        if SPLIT_COMPUTATION {
            self.internal_synthesis_channels * (self.fft.steps() + 1) + 1
        } else {
            self.internal_synthesis_channels
        }
    }

    #[allow(clippy::indexing_slicing)]
    fn synthesise_step(&mut self, mut step: usize) {
        if step == 0 {
            // Extra first step which adds in the effective gain for a pure
            // analysis-synthesis cycle
            self.add_window_product();
            if SPLIT_COMPUTATION {
                return;
            }
        }

        if SPLIT_COMPUTATION {
            step -= 1;
        }

        let fft_steps = if SPLIT_COMPUTATION {
            self.fft.steps()
        } else {
            0
        };
        let channel = step / (fft_steps + 1);
        step -= channel * (fft_steps + 1);

        // i *should* be using this, but this has a problem with mutable borrowing, so
        // instead i'm going to expand it
        // let spectrumPtr = self.spectrum_mut(channel);
        // &mut self.spectrumBuffer[(channel * self._fftBins)..]

        if Self::UNPACKED && step == 0 {
            // re-pack
            self.spectrum_buffer[channel * self.internal_fft_bins].im = self.spectrum_buffer
                [(channel * self.internal_fft_bins) + (self.internal_fft_bins - 1)]
                .re;
        }

        if SPLIT_COMPUTATION {
            if step < fft_steps {
                self.fft.ifft_step(
                    step,
                    &self.spectrum_buffer[(channel * self.internal_fft_bins)..],
                    &mut self.time_buffer,
                );
                return;
            }
        } else {
            self.fft.ifft(
                &self.spectrum_buffer[(channel * self.internal_fft_bins)..],
                &mut self.time_buffer,
            );
        }

        // extra step after each channel's FFT
        let output_wrap_index = self.internal_block_samples - self.output.pos;
        let chunk1 = usize::min(self.internal_synthesis_offset, output_wrap_index);
        let chunk2 = usize::min(
            self.internal_block_samples,
            usize::max(self.internal_synthesis_offset, output_wrap_index),
        );

        for i in 0..chunk1 {
            let w = if Self::MODIFIED {
                -self.internal_synthesis_window[i]
            } else {
                self.internal_synthesis_window[i]
            };

            let ti = i + (self.internal_fft_samples - self.internal_synthesis_offset);
            let bi = self.output.pos + i;

            self.output.buffer[channel * self.internal_block_samples + bi] +=
                self.time_buffer[ti] * w;
        }

        for i in chunk1..self.internal_synthesis_offset {
            let w = if Self::MODIFIED {
                -self.internal_synthesis_window[i]
            } else {
                self.internal_synthesis_window[i]
            };

            let ti = i + (self.internal_fft_samples - self.internal_synthesis_offset);
            let bi = i + self.output.pos - self.internal_block_samples;

            self.output.buffer[channel * self.internal_block_samples + bi] +=
                self.time_buffer[ti] * w;
        }

        for i in self.internal_synthesis_offset..chunk2 {
            let w = self.internal_synthesis_window[i];

            let ti = i - self.internal_synthesis_offset;
            let bi = self.output.pos + i;

            self.output.buffer[channel * self.internal_block_samples + bi] +=
                self.time_buffer[ti] * w;
        }

        for i in chunk2..self.internal_block_samples {
            let w = self.internal_synthesis_window[i];

            let ti = i - self.internal_synthesis_offset;
            let bi = i + self.output.pos - self.internal_block_samples;

            self.output.buffer[channel * self.internal_block_samples + bi] +=
                self.time_buffer[ti] * w;
        }
    }
}

impl<const SPLIT_COMPUTATION: bool, const SPECTRUM_TYPE: usize, const HALF_BIN_SHIFT: bool>
    DynamicSTFT<f32, SPLIT_COMPUTATION, SPECTRUM_TYPE, HALF_BIN_SHIFT>
{
    #[allow(
        clippy::indexing_slicing,
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_wrap
    )]
    fn add_window_product(&mut self) {
        self.internal_samples_since_synthesis = 0;

        let window_shift =
            self.internal_synthesis_offset as isize - self.internal_analysis_offset as isize;
        let w_min = isize::max(0, window_shift) as usize;
        let w_max = isize::min(
            self.internal_block_samples as isize,
            self.internal_block_samples as isize + window_shift,
        ) as usize;

        let output_wrap_index = self.internal_block_samples - self.output.pos;
        let chunk1 = usize::min(w_max, usize::max(w_min, output_wrap_index));

        for i in w_min..chunk1 {
            let wa = self.internal_analysis_window[i - window_shift as usize];
            let ws = self.internal_synthesis_window[i];

            let bi = self.output.pos + i;

            self.output.window_products[bi] += wa * ws * self.internal_fft_samples as f32;
        }

        for i in chunk1..w_max {
            let wa = self.internal_analysis_window[i - window_shift as usize];
            let ws = self.internal_synthesis_window[i];

            let bi = i + self.output.pos - self.internal_block_samples;

            self.output.window_products[bi] += wa * ws * self.internal_fft_samples as f32;
        }
    }

    #[allow(clippy::indexing_slicing)]
    fn force_perfect_reconstruction(data: &mut [f32], window_length: usize, interval: usize) {
        for i in 0..interval {
            let mut sum2 = 0.0;

            for index in (i..window_length).step_by(interval) {
                sum2 += data[index] * data[index];
            }

            let factor = 1.0 / sum2.sqrt();

            for index in (i..window_length).step_by(interval) {
                data[index] *= factor;
            }
        }
    }
}
