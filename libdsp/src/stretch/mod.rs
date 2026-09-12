pub mod io;

use num::{Complex, Float};
use rand::rngs::SmallRng;
use rand_distr::{Distribution, Uniform};

use crate::{
    linear::stft::{
        DynamicSTFT, DynamicSTFTTrait, Input, InputTrait, Output, OutputTrait, WindowShape,
    },
    stretch::io::{
        BufferSplitterIoMut, IoBuffer, IoBufferMut, IoBufferOffsetIo, IoBufferOffsetIoMut, ZeroIo,
    },
};

pub fn mul<const CONJUGATE_SECOND: bool, V>(a: &Complex<V>, b: &Complex<V>) -> Complex<V>
where
    V: Float,
{
    if CONJUGATE_SECOND {
        Complex::<V>::new(b.re * a.re + b.im * a.im, b.re * a.im - b.im * a.re)
    } else {
        Complex::<V>::new(a.re * b.re - a.im * b.im, a.re * b.im + a.im * b.re)
    }
}

pub fn norm<V>(a: &Complex<V>) -> V
where
    V: Float,
{
    let r = a.re;
    let i = a.im;

    r * r + i * i
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ProcessBlock<Sample> {
    samples_since_last: usize, // = std::numeric_limits<size_t>::max();
    steps: usize,              // = 0;
    step: usize,               // = 0;

    new_spectrum: bool,       // = false;
    reanalyse_prev: bool,     // = false;
    mapped_frequencies: bool, // = false;
    process_formants: bool,   // = false;
    time_factor: Sample,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Band<Sample> {
    input: Complex<Sample>,
    prev_input: Complex<Sample>, //{0};
    output: Complex<Sample>,     //{0};
    input_energy: Sample,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Peak<Sample> {
    input: Sample,
    output: Sample,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PitchMapPoint<Sample> {
    input_bin: Sample,
    freq_grad: Sample,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Prediction<Sample> {
    energy: Sample, // = 0;
    input: Complex<Sample>,
}

pub trait PredictionTrait<Sample> {
    fn make_output(phase: Complex<Sample>, noise_floor: Sample) -> Complex<Sample>;
}

impl Prediction<f32> {
    fn make_output(&self, mut phase: Complex<f32>, noise_floor: f32) -> Complex<f32> {
        let mut phase_norm = norm(&phase);

        if phase_norm <= noise_floor {
            phase = self.input; // prediction is too weak, fall back to the input
            phase_norm = norm(&self.input) + noise_floor;
        }

        phase * (self.energy / phase_norm).sqrt()
    }
}

#[allow(clippy::struct_excessive_bools)]
pub struct SignalsmithStretch<Sample>
where
    Sample: Float,
{
    internal_split_computation: bool, // = false;
    block_process: ProcessBlock<Sample>,

    silence_counter: usize, // = 0;
    silence_first: bool,    // = true;

    freq_multiplier: Sample,     // = 1
    freq_tonality_limit: Sample, // = 0.5;
    custom_freq_map: Option<fn(Sample) -> Sample>,

    // compensate for pitch/freq change
    formant_compensation: bool,     // = false;
    formant_multiplier: Sample,     // = 1
    inv_formant_multiplier: Sample, // = 1;

    stft: DynamicSTFT<Sample, false, 1, true>, // STFT_SPECTRUM_MODIFIED
    stashed_input: Input<Sample>,
    stashed_output: Output<Sample>,

    tmp_process_buffer: Vec<Sample>,
    tmp_pre_roll_buffer: Vec<Sample>,

    channels: usize,          // = 0,
    bands: usize,             // = 0;
    prev_input_offset: i32,   // = -1;
    did_seek: bool,           // = false;
    seek_time_factor: Sample, // = 1;

    internal_channel_bands: Vec<Band<Sample>>,

    peaks: Vec<Peak<Sample>>,
    energy: Vec<Sample>,
    smoothed_energy: Vec<Sample>,
    output_map: Vec<PitchMapPoint<Sample>>,

    channel_predictions: Vec<Prediction<Sample>>,

    random_engine: SmallRng,

    process_spectrum_steps: usize, // = 0;
    smooth_energy_state: Sample,   // = 0;

    freq_estimate_weighted: Sample, // = 0;
    freq_estimate_weight: Sample,   // = 0;

    freq_estimate: Sample,

    formant_metric: Vec<Sample>,
    formant_base_freq: Sample, // = 0;
}

pub trait SignalsmithStretchTrait<Sample>
where
    Sample: Float,
{
    type Complex;
    type Sample;

    const NOISE_FLOOR: Self::Sample;
    const MAX_CLEAN_STRETCH: Self::Sample;
    // time-stretch ratio before we start randomising phases
    const SMOOTH_ENERGY_STEPS: usize = 3;
    // it's just heavy, since we're blending up to 4 different phase predictions
    const SPLIT_MAIN_PREDICTION: usize = 8;

    fn new() -> Self;

    fn input_latency(&self) -> usize;
    fn output_latency(&self) -> usize;

    fn reset(&mut self);

    // Configures using a default preset
    fn preset_default(
        &mut self,
        n_channels: usize,
        sample_rate: Self::Sample,
        split_computation: bool,
    );
    fn preset_cheaper(
        &mut self,
        n_channels: usize,
        sample_rate: Self::Sample,
        split_computation: bool,
    );

    // Manual setup
    fn configure(
        &mut self,
        n_channels: usize,
        block_samples: usize,
        interval_samples: usize,
        split_computation: bool,
    );

    fn block_samples(&self);
    fn interval_samples(&self);
    fn split_computation(&self) -> bool;

    fn set_tanspose_factor(&mut self, multiplier: Self::Sample, tonality_limit: Self::Sample);
    fn set_transpose_semitones(&mut self, semitones: Self::Sample, tonality_limit: Self::Sample);
    // Sets a custom frequency map - should be monotonically increasing
    fn set_freq_map(&mut self, input_to_output: fn(Self::Sample) -> Self::Sample);

    fn set_formant_factor(&mut self, multiplier: Self::Sample, compensate_pitch: bool);
    fn set_formant_semitones(&mut self, semitones: Self::Sample, compensate_pitch: bool);
    // Rough guesstimate of the fundamental frequency, used for formant analysis. 0
    // means attempting to detect the pitch
    fn set_formant_base(&mut self, base_freq: Self::Sample);

    // Provide previous input ("pre-roll") to smoothly change the input location
    // without interrupting the output.  This doesn't do any calculation, just
    // copies intput to a buffer. You should ideally feed it `seekLength()`
    // frames of input, unless it's directly after a `.reset()` (in which case
    // `.outputSeek()` might be a better choice)
    fn seek<Input>(&mut self, inputs: &Input, input_samples: i32, playback_rate: f64)
    where
        Input: IoBuffer<f32>;
    fn seek_length(&self) -> usize;

    // Moves the input position *and* pre-calculates some output, so that the next
    // samples returned from `.process()` are aligned to the beginning of the
    // sample. The time-stretch rate is inferred from `inputLength`, so use
    // `.outputSeekLength()` to get a correct value for that.
    fn output_seek<Input>(&mut self, inputs: &Input, input_length: i32)
    where
        Input: IoBuffer<f32>;
    fn output_seek_length(&self, playback_rate: Self::Sample) -> usize;

    fn process<Input, Output>(
        &mut self,
        inputs: &Input,
        input_samples: i32,
        outputs: &mut Output,
        output_samples: i32,
    ) where
        Input: IoBuffer<f32>,
        Output: IoBufferMut<f32>;

    fn flush<Output>(
        &mut self,
        outputs: &mut Output,
        output_samples: i32,
        playback_rate: Self::Sample,
    ) where
        Output: IoBufferMut<f32>;

    fn exact<Input, Output>(
        &mut self,
        inputs: &Input,
        input_samples: i32,
        outputs: &mut Output,
        output_samples: i32,
    ) -> bool
    where
        Input: IoBuffer<f32>,
        Output: IoBufferMut<f32>;
}

impl SignalsmithStretchTrait<f32> for SignalsmithStretch<f32> {
    type Complex = Complex<f32>;
    type Sample = f32;

    const MAX_CLEAN_STRETCH: f32 = 2.0;
    const NOISE_FLOOR: f32 = 1e-15;

    fn new() -> Self {
        let stft = DynamicSTFT::<f32, false, 1, true>::new();
        let stashed_input = stft.input.clone();
        let stashed_output = stft.output.clone();

        Self {
            internal_split_computation: false,
            block_process: ProcessBlock {
                samples_since_last: usize::MAX,
                steps: 0,
                step: 0,
                new_spectrum: false,
                reanalyse_prev: false,
                mapped_frequencies: false,
                process_formants: false,
                time_factor: 0.0,
            },
            silence_counter: 0,
            silence_first: true,
            freq_multiplier: 1.0,
            freq_tonality_limit: 0.5,
            custom_freq_map: None,
            formant_compensation: false,
            formant_multiplier: 1.0,
            inv_formant_multiplier: 1.0,
            stft,
            stashed_input,
            stashed_output,
            tmp_process_buffer: Vec::new(),
            tmp_pre_roll_buffer: Vec::new(),
            channels: 0,
            bands: 0,
            prev_input_offset: -1,
            did_seek: false,
            seek_time_factor: 1.0,
            internal_channel_bands: Vec::new(),
            peaks: Vec::new(),
            energy: Vec::new(),
            smoothed_energy: Vec::new(),
            output_map: Vec::new(),
            channel_predictions: Vec::new(),
            random_engine: rand::make_rng(),
            process_spectrum_steps: 0,
            smooth_energy_state: 0.0,
            freq_estimate_weighted: 0.0,
            freq_estimate_weight: 0.0,
            freq_estimate: 0.0,
            formant_metric: Vec::new(),
            formant_base_freq: 0.0,
        }
    }

    // The difference between the internal position (centre of a block) and the
    // input samples you're supplying
    fn input_latency(&self) -> usize {
        self.stft.analysis_latency()
    }

    fn output_latency(&self) -> usize {
        self.stft.synthesis_latency()
            + usize::from(self.internal_split_computation) * self.stft.default_interval()
    }

    fn reset(&mut self) {
        self.stft.reset(0.1);
        self.stashed_input = self.stft.input.clone();
        self.stashed_output = self.stft.output.clone();

        self.prev_input_offset = -1;
        self.internal_channel_bands.fill(Band {
            input: Complex::new(0.0, 0.0),
            prev_input: Complex::new(0.0, 0.0),
            output: Complex::new(0.0, 0.0),
            input_energy: 0.0,
        });
        self.silence_counter = 0;
        self.did_seek = false;
        self.block_process = ProcessBlock {
            samples_since_last: usize::MAX,
            steps: 0,
            step: 0,
            new_spectrum: false,
            reanalyse_prev: false,
            mapped_frequencies: false,
            process_formants: false,
            time_factor: 0.0,
        };
        self.freq_estimate_weight = 0.0;
        self.freq_estimate_weighted = 0.0;
    }

    // Configures using a default preset
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    fn preset_default(&mut self, n_channels: usize, sample_rate: f32, split_computation: bool) {
        self.configure(
            n_channels,
            (sample_rate * 0.12) as usize,
            (sample_rate * 0.03) as usize,
            split_computation,
        );
    }

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    fn preset_cheaper(&mut self, n_channels: usize, sample_rate: f32, split_computation: bool) {
        self.configure(
            n_channels,
            (sample_rate * 0.1) as usize,
            (sample_rate * 0.04) as usize,
            split_computation,
        );
    }

    // Manual setup
    fn configure(
        &mut self,
        n_channels: usize,
        block_samples: usize,
        interval_samples: usize,
        split_computation: bool,
    ) {
        self.internal_split_computation = split_computation;
        self.channels = n_channels;
        self.stft.configure(
            self.channels,
            self.channels,
            block_samples,
            interval_samples + 1,
            0,
            0.0,
        );
        self.stft
            .set_interval(interval_samples, WindowShape::Kaiser, 0.0);
        self.stft.reset(0.1);
        self.stashed_input = self.stft.input.clone();
        self.stashed_output = self.stft.output.clone();

        self.bands = self.stft.bands();
        self.internal_channel_bands.resize(
            self.bands * self.channels,
            Band {
                input: Complex::new(0.0, 0.0),
                prev_input: Complex::new(0.0, 0.0),
                output: Complex::new(0.0, 0.0),
                input_energy: 0.0,
            },
        );

        self.peaks.reserve(self.bands / 2);
        self.energy.resize(self.bands, 0.0);
        self.smoothed_energy.resize(self.bands, 0.0);
        self.output_map.resize(
            self.bands,
            PitchMapPoint {
                input_bin: 0.0,
                freq_grad: 0.0,
            },
        );
        self.channel_predictions.resize(
            self.channels * self.bands,
            Prediction {
                energy: 0.0,
                input: Complex::new(0.0, 0.0),
            },
        );

        self.block_process = ProcessBlock {
            samples_since_last: usize::MAX,
            steps: 0,
            step: 0,
            new_spectrum: false,
            reanalyse_prev: false,
            mapped_frequencies: false,
            process_formants: false,
            time_factor: 0.0,
        };
        self.formant_metric.resize(self.bands + 2, 0.0);

        self.tmp_process_buffer
            .resize(block_samples + interval_samples, 0.0);
        self.tmp_pre_roll_buffer
            .resize(self.output_latency() * self.channels, 0.0);
    }

    // For querying the existing config
    fn block_samples(&self) {
        self.stft.block_samples();
    }

    fn interval_samples(&self) {
        self.stft.default_interval();
    }

    fn split_computation(&self) -> bool {
        self.internal_split_computation
    }

    /// Frequency multiplier, and optional tonality limit (as multiple of
    /// sample-rate)
    fn set_tanspose_factor(&mut self, multiplier: f32, tonality_limit: f32) {
        self.freq_multiplier = multiplier;

        if tonality_limit > 0.0 {
            self.freq_tonality_limit = tonality_limit / multiplier.sqrt(); // compromise between input and output limits
        } else {
            self.freq_tonality_limit = 1.0;
        }

        self.custom_freq_map = None;
    }

    fn set_transpose_semitones(&mut self, semitones: f32, tonality_limit: f32) {
        self.set_tanspose_factor((semitones / 12.0).powi(2), tonality_limit);
    }

    // Sets a custom frequency map - should be monotonically increasing
    fn set_freq_map(&mut self, input_to_output: fn(f32) -> f32) {
        self.custom_freq_map = Some(input_to_output);
    }

    fn set_formant_factor(&mut self, multiplier: f32, compensate_pitch: bool) {
        self.formant_multiplier = multiplier;
        self.inv_formant_multiplier = 1.0 / multiplier;
        self.formant_compensation = compensate_pitch;
    }

    fn set_formant_semitones(&mut self, semitones: f32, compensate_pitch: bool) {
        self.set_formant_factor((semitones / 12.0).powi(2), compensate_pitch);
    }

    // Rough guesstimate of the fundamental frequency, used for formant analysis. 0
    // means attempting to detect the pitch
    fn set_formant_base(&mut self, base_freq: f32) {
        self.formant_base_freq = base_freq;
    }

    // Provide previous input ("pre-roll") to smoothly change the input location
    // without interrupting the output.  This doesn't do any calculation, just
    // copies intput to a buffer. You should ideally feed it `seekLength()`
    // frames of input, unless it's directly after a `.reset()` (in which case
    // `.outputSeek()` might be a better choice)
    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_precision_loss,
        clippy::needless_range_loop,
        clippy::indexing_slicing
    )]
    fn seek<Input>(&mut self, inputs: &Input, input_samples: i32, playback_rate: f64)
    where
        Input: IoBuffer<f32>,
    {
        self.tmp_process_buffer.clear();
        self.tmp_process_buffer.resize(
            self.stft.block_samples() + self.stft.default_interval(),
            0.0,
        );

        let start_index = i32::max(0, input_samples - self.tmp_process_buffer.len() as i32); // start position in input
        let pad_start = (self.tmp_process_buffer.len() as i32 + start_index) - input_samples; // start position in tmpProcessBuffer

        let mut total_energy = 0.0;

        for c in 0..self.channels {
            for i in start_index..input_samples {
                let s = inputs.sample(c, i as usize);

                total_energy += s * s;
                self.tmp_process_buffer[(i - start_index + pad_start) as usize] = *s;
            }

            self.stft
                .write_input(c, self.tmp_process_buffer.len(), &self.tmp_process_buffer);
        }

        self.stft.move_input(self.tmp_process_buffer.len(), false);

        if total_energy >= Self::NOISE_FLOOR {
            self.silence_counter = 0;
            self.silence_first = true;
        }

        self.did_seek = true;

        self.seek_time_factor = if playback_rate * self.stft.default_interval() as f64 > 1.0 {
            (1.0 / playback_rate) as f32
        } else {
            self.stft.default_interval() as f32
        };
    }

    fn seek_length(&self) -> usize {
        self.stft.block_samples() + self.stft.default_interval()
    }

    // Moves the input position *and* pre-calculates some output, so that the next
    // samples returned from `.process()` are aligned to the beginning of the
    // sample. The time-stretch rate is inferred from `inputLength`, so use
    // `.outputSeekLength()` to get a correct value for that.
    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_precision_loss,
        clippy::cast_lossless,
        clippy::indexing_slicing,
        clippy::undocumented_unsafe_blocks
    )]
    fn output_seek<Input>(&mut self, inputs: &Input, input_length: i32)
    where
        Input: IoBuffer<f32>,
    {
        // TODO: add fade-out parameter to avoid clicks, instead of doing a full reset
        self.reset();
        // Assume we've been handed enough surplus input to produce `outputLatency()`
        // samples of pre-roll
        let surplus_input = i32::max(input_length - self.input_latency() as i32, 0);
        let playback_rate = surplus_input as f64 / self.output_latency() as f64;

        // Move the input position to the start of the sound
        let seek_samples = input_length - surplus_input;
        self.seek(inputs, seek_samples, playback_rate);

        self.tmp_pre_roll_buffer
            .resize(self.output_latency() * self.channels, 0.0);

        let pre_roll_buffer_pointer = &raw mut self.tmp_pre_roll_buffer;
        let pre_roll_buffer = unsafe { &mut *pre_roll_buffer_pointer };

        let pre_roll_output_length = self.output_latency();
        let pre_roll_output = pre_roll_buffer;

        // Use the surplus input to produce pre-roll output
        let offset_input = IoBufferOffsetIo::new(inputs, seek_samples as usize);
        let mut pre_roll_output = BufferSplitterIoMut::new(pre_roll_output, pre_roll_output_length);

        self.process(
            &offset_input,
            surplus_input,
            &mut pre_roll_output,
            pre_roll_output_length as i32,
        );

        // Safety: pre_roll_output SHALL NOT be used beyond here

        // put the thing down, flip it and reverse it
        for v in &mut self.tmp_pre_roll_buffer {
            *v = -*v;
        }

        for c in 0..self.channels {
            self.tmp_pre_roll_buffer[(c * pre_roll_output_length)
                ..(c * pre_roll_output_length + pre_roll_output_length)]
                .reverse();

            self.stft.add_output(
                c,
                pre_roll_output_length,
                &self.tmp_pre_roll_buffer[(c * pre_roll_output_length)..],
            );
        }
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss
    )]
    fn output_seek_length(&self, playback_rate: f32) -> usize {
        self.input_latency() + (playback_rate * self.output_latency() as f32) as usize
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_precision_loss,
        clippy::needless_range_loop,
        clippy::indexing_slicing
    )]
    fn process<Input, Output>(
        &mut self,
        inputs: &Input,
        input_samples: i32,
        outputs: &mut Output,
        output_samples: i32,
    ) where
        Input: IoBuffer<f32>,
        Output: IoBufferMut<f32>,
    {
        let mut prev_copied_input = 0;

        let mut total_energy = 0.0;

        for c in 0..self.channels {
            for i in 0..input_samples {
                let s = inputs.sample(c, i as usize);

                total_energy += s * s;
            }
        }

        if total_energy < Self::NOISE_FLOOR {
            if self.silence_counter >= 2 * self.stft.block_samples() {
                if self.silence_first {
                    // first block of silence processing
                    self.silence_first = false;
                    //stft.reset();
                    self.block_process = ProcessBlock {
                        samples_since_last: usize::MAX,
                        steps: 0,
                        step: 0,
                        new_spectrum: false,
                        reanalyse_prev: false,
                        mapped_frequencies: false,
                        process_formants: false,
                        time_factor: 0.0,
                    };

                    for b in &mut self.internal_channel_bands {
                        b.input = Complex::new(0.0, 0.0);
                        b.prev_input = Complex::new(0.0, 0.0);
                        b.output = Complex::new(0.0, 0.0);
                        b.input_energy = 0.0;
                    }
                }

                if input_samples > 0 {
                    // copy from the input, wrapping around if needed
                    for output_index in 0..output_samples {
                        let input_index = output_index % input_samples;

                        for c in 0..self.channels {
                            *outputs.sample_mut(c, output_index as usize) =
                                *inputs.sample(c, input_index as usize);
                        }
                    }
                } else {
                    for c in 0..self.channels {
                        for output_index in 0..output_samples {
                            *outputs.sample_mut(c, output_index as usize) = 0.0;
                        }
                    }
                }

                // Store input in history buffer
                self.process_copy_input(inputs, input_samples, &mut prev_copied_input);

                return;
            }

            self.silence_counter += input_samples as usize;
        } else {
            self.silence_counter = 0;

            self.silence_first = true;
        }

        for output_index in 0..output_samples {
            let new_block = self.block_process.samples_since_last >= self.stft.default_interval();

            if new_block {
                self.block_process.step = 0;
                self.block_process.steps = 0; // how many processing steps this block will have
                self.block_process.samples_since_last = 0;

                // Time to process a spectrum!  Where should it come from in the input?
                let input_offset = (output_index as f32 * input_samples as f32
                    / output_samples as f32)
                    .round() as i32;
                let input_interval = input_offset - self.prev_input_offset;
                self.prev_input_offset = input_offset;

                self.process_copy_input(inputs, input_offset, &mut prev_copied_input);

                self.stashed_input = self.stft.input.clone(); // save the input state, since that's what we'll analyse later

                if self.internal_split_computation {
                    self.stashed_output = self.stft.output.clone(); // save the current output, and read from it
                    self.stft.move_output(self.stft.default_interval()); // the actual input jumps forward in time by one interval, ready for the synthesis
                }

                self.block_process.new_spectrum = self.did_seek || (input_interval > 0);
                self.block_process.mapped_frequencies = self.custom_freq_map.is_some()
                    || (self.formant_multiplier - 1.0).abs() <= f32::EPSILON;
                if self.block_process.new_spectrum {
                    // make sure the previous input is the correct distance in the past (give or
                    // take 1 sample)
                    self.block_process.reanalyse_prev = self.did_seek
                        || (input_interval - self.stft.default_interval() as i32).abs() > 1;

                    if self.block_process.reanalyse_prev {
                        self.block_process.steps += self.stft.analyse_steps() + 1;
                    }

                    // analyse a new input
                    self.block_process.steps += self.stft.analyse_steps() + 1;
                }

                self.block_process.process_formants = (self.formant_multiplier - 1.0).abs()
                    <= f32::EPSILON
                    || (self.formant_compensation && self.block_process.mapped_frequencies);

                self.block_process.time_factor = if self.did_seek {
                    self.seek_time_factor
                } else {
                    self.stft.default_interval() as f32 / (i32::max(1, input_interval)) as f32
                };
                self.did_seek = false;

                self.update_process_spectrum_steps();

                self.block_process.steps += self.process_spectrum_steps;
                self.block_process.steps += self.stft.synthesise_steps() + 1;
            }

            let mut process_to_step = if new_block {
                self.block_process.steps
            } else {
                0
            };

            if self.internal_split_computation {
                let process_ratio = (self.block_process.samples_since_last + 1) as f32
                    / self.stft.default_interval() as f32;

                process_to_step = usize::min(
                    self.block_process.steps,
                    ((self.block_process.steps as f32 + 0.999) * process_ratio) as usize,
                );
            }

            while self.block_process.step < process_to_step {
                let mut step = self.block_process.step;
                self.block_process.step += 1;

                if self.block_process.new_spectrum {
                    if self.block_process.reanalyse_prev {
                        // analyse past input
                        if step < self.stft.analyse_steps() {
                            self.stashed_input.swap(&mut self.stft.input);

                            self.stft.analyse_step(step, self.stft.default_interval());

                            self.stashed_input.swap(&mut self.stft.input);

                            continue;
                        }

                        step -= self.stft.analyse_steps();

                        if step < 1 {
                            // Copy previous analysis to our band objects
                            for c in 0..self.channels {
                                let channel_bands =
                                    &mut self.internal_channel_bands[(c * self.bands)..];
                                let spectrum_bands = self.stft.spectrum(c);

                                for b in 0..self.bands {
                                    channel_bands[b].prev_input = spectrum_bands[b];
                                }
                            }

                            continue;
                        }
                        step -= 1;
                    }

                    // Analyse latest (stashed) input
                    if step < self.stft.analyse_steps() {
                        self.stashed_input.swap(&mut self.stft.input);

                        self.stft.analyse_step(step, 0);

                        self.stashed_input.swap(&mut self.stft.input);

                        continue;
                    }

                    step -= self.stft.analyse_steps();

                    if step < 1 {
                        // Copy analysed spectrum into our band objects
                        for c in 0..self.channels {
                            let channel_bands =
                                &mut self.internal_channel_bands[(c * self.bands)..];
                            let spectrum_bands = self.stft.spectrum(c);

                            for b in 0..self.bands {
                                channel_bands[b].input = spectrum_bands[b];
                            }
                        }

                        continue;
                    }
                    step -= 1;
                }

                if step < self.process_spectrum_steps {
                    self.process_spectrum(step);

                    continue;
                }

                step -= self.process_spectrum_steps;

                if step < 1 {
                    // Copy band objects into spectrum
                    for c in 0..self.channels {
                        let channel_bands = &self.internal_channel_bands[(c * self.bands)..];

                        let spectrum_bands = self.stft.spectrum_mut(c);

                        for b in 0..self.bands {
                            spectrum_bands[b] = channel_bands[b].output;
                        }
                    }
                    continue;
                }
                step -= 1;

                if step < self.stft.synthesise_steps() {
                    self.stft.synthesise_step(step);
                }
            }

            self.block_process.samples_since_last += 1;

            if self.internal_split_computation {
                self.stashed_output.swap(&mut self.stft.output);
            }

            for c in 0..self.channels {
                let mut v = [0.0];

                self.stft.read_output(c, 1, &mut v);

                *outputs.sample_mut(c, output_index as usize) = v[0];
            }

            self.stft.move_output(1);

            if self.internal_split_computation {
                self.stashed_output.swap(&mut self.stft.output);
            }
        }

        self.process_copy_input(inputs, input_samples, &mut prev_copied_input);

        self.prev_input_offset -= input_samples;
    }

    // Read the remaining output, providing no further input.  If `outputSamples` is
    // more than one interval, it will compute additional blocks assuming a
    // zero-valued input
    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_precision_loss,
        clippy::needless_range_loop,
        clippy::indexing_slicing
    )]
    fn flush<Output>(&mut self, outputs: &mut Output, output_samples: i32, playback_rate: f32)
    where
        Output: IoBufferMut<f32>,
    {
        // If we're asked for more than an interval of extra output, then zero-pad the
        // input
        let output_block = i32::max(0, output_samples - self.stft.default_interval() as i32);
        if output_block > 0 {
            let zero_io = ZeroIo::<f32>::default();

            self.process(
                &zero_io,
                (output_block as f32 * playback_rate) as i32,
                outputs,
                output_block,
            );
        }

        let tail_samples = output_samples - output_block; // at most one interval
        self.tmp_process_buffer.resize(tail_samples as usize, 0.0);
        self.stft.finish_output(1.0, 0);

        for c in 0..self.channels {
            self.stft
                .read_output(c, tail_samples as usize, &mut self.tmp_process_buffer);

            for i in 0..tail_samples {
                *outputs.sample_mut(c, (output_block + i) as usize) =
                    self.tmp_process_buffer[i as usize];
            }

            self.stft.read_output_offset(
                c,
                tail_samples as usize,
                tail_samples as usize,
                &mut self.tmp_process_buffer,
            );

            for i in 0..tail_samples {
                *outputs.sample_mut(c, (output_block + tail_samples - 1 - i) as usize) -=
                    self.tmp_process_buffer[i as usize];
            }
        }

        self.stft.reset(0.1);

        // Reset the phase-vocoder stuff, so the next block gets a fresh start
        for c in 0..self.channels {
            for b in 0..self.bands {
                self.internal_channel_bands[(c * self.bands) + b].prev_input =
                    Complex::<f32>::new(0.0, 0.0);
                self.internal_channel_bands[(c * self.bands) + b].output =
                    Complex::<f32>::new(0.0, 0.0);
            }
        }
    }

    // Process a complete audio buffer all in one go
    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_precision_loss,
        clippy::needless_range_loop,
        clippy::indexing_slicing
    )]
    fn exact<Input, Output>(
        &mut self,
        inputs: &Input,
        input_samples: i32,
        outputs: &mut Output,
        output_samples: i32,
    ) -> bool
    where
        Input: IoBuffer<f32>,
        Output: IoBufferMut<f32>,
    {
        let playback_rate = input_samples as f32 / output_samples as f32;
        let seek_length = self.output_seek_length(playback_rate);

        if input_samples < seek_length as i32 {
            // to short for this - zero the output just to be polite
            for c in 0..self.channels {
                for i in 0..output_samples {
                    *outputs.sample_mut(c, i as usize) = 0.0;
                }
            }
            return false;
        }

        self.output_seek(inputs, seek_length as i32);

        let output_index = output_samples - (seek_length as f32 / playback_rate) as i32;

        let offset_input = IoBufferOffsetIo::new(inputs, seek_length);
        self.process(
            &offset_input,
            input_samples - seek_length as i32,
            outputs,
            output_index,
        );

        let mut offset_output = IoBufferOffsetIoMut::new(outputs, output_index as usize);
        self.flush(
            &mut offset_output,
            output_samples - output_index,
            playback_rate,
        );

        true
    }
}

impl SignalsmithStretch<f32> {
    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::needless_range_loop,
        clippy::indexing_slicing
    )]
    fn process_copy_input<Input>(
        &mut self,
        inputs: &Input,
        to_index: i32,
        prev_copied_input: &mut i32,
    ) where
        Input: IoBuffer<f32>,
    {
        let length = i32::min(
            (self.stft.block_samples() + self.stft.default_interval()) as i32,
            to_index - *prev_copied_input,
        );

        self.tmp_process_buffer.resize(length as usize, 0.0);

        let offset = to_index - length;

        for c in 0..self.channels {
            for i in 0..length {
                self.tmp_process_buffer[i as usize] = *inputs.sample(c, (i + offset) as usize);
            }

            self.stft
                .write_input(c, length as usize, &self.tmp_process_buffer);
        }

        self.stft.move_input(length as usize, false);

        *prev_copied_input = to_index;
    }

    fn band_to_freq(&self, b: f32) -> f32 {
        self.stft.bin_to_freq(b)
    }

    fn freq_to_band(&self, f: f32) -> f32 {
        self.stft.freq_to_bin(f)
    }

    // not implementing this due to borrow checker restrictions
    // fn bandsForChannel(&mut self, channel: i32) -> &mut Band<f32> {
    //     &mut self._channelBands[(c * self.bands)..]
    // }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::indexing_slicing
    )]
    fn get_band_input(&self, channel: usize, index: i32) -> Complex<f32> {
        if index < 0 || index >= self.bands as i32 {
            return Complex::<f32>::new(0.0, 0.0);
        }

        self.internal_channel_bands[index as usize + channel * self.bands].input
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::indexing_slicing
    )]
    fn get_band_prev_input(&self, channel: usize, index: i32) -> Complex<f32> {
        if index < 0 || index >= self.bands as i32 {
            return Complex::<f32>::new(0.0, 0.0);
        }

        self.internal_channel_bands[index as usize + channel * self.bands].prev_input
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::indexing_slicing
    )]
    fn get_band_output(&self, channel: usize, index: i32) -> Complex<f32> {
        if index < 0 || index >= self.bands as i32 {
            return Complex::<f32>::new(0.0, 0.0);
        }

        self.internal_channel_bands[index as usize + channel * self.bands].output
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::indexing_slicing
    )]
    fn get_band_input_energy(&self, channel: usize, index: i32) -> f32 {
        if index < 0 || index >= self.bands as i32 {
            return 0.0;
        }

        self.internal_channel_bands[index as usize + channel * self.bands].input_energy
    }

    fn get_split_fractional_input(
        &self,
        channel: usize,
        low_index: i32,
        fractional: f32,
    ) -> Complex<f32> {
        let low = self.get_band_input(channel, low_index);
        let high = self.get_band_input(channel, low_index + 1);

        low + (high - low) * fractional
    }

    fn get_split_fractional_prev_input(
        &self,
        channel: usize,
        low_index: i32,
        fractional: f32,
    ) -> Complex<f32> {
        let low = self.get_band_prev_input(channel, low_index);
        let high = self.get_band_prev_input(channel, low_index + 1);

        low + (high - low) * fractional
    }

    #[allow(unused)]
    fn get_split_fractional_output(
        &self,
        channel: usize,
        low_index: i32,
        fractional: f32,
    ) -> Complex<f32> {
        let low = self.get_band_output(channel, low_index);
        let high = self.get_band_output(channel, low_index + 1);

        low + (high - low) * fractional
    }

    fn get_split_fractional_input_energy(
        &self,
        channel: usize,
        low_index: i32,
        fractional: f32,
    ) -> f32 {
        let low = self.get_band_input_energy(channel, low_index);
        let high = self.get_band_input_energy(channel, low_index + 1);

        low + (high - low) * fractional
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn get_fractional_input(&self, channel: usize, input_index: f32) -> Complex<f32> {
        let low_index = f32::floor(input_index) as i32;
        let frac_index = input_index - low_index as f32;

        self.get_split_fractional_input(channel, low_index, frac_index)
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, unused)]
    fn get_fractional_prev_input(&self, channel: usize, input_index: f32) -> Complex<f32> {
        let low_index = f32::floor(input_index) as i32;
        let frac_index = input_index - low_index as f32;

        self.get_split_fractional_prev_input(channel, low_index, frac_index)
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, unused)]
    fn get_fractional_output(&self, channel: usize, input_index: f32) -> Complex<f32> {
        let low_index = f32::floor(input_index) as i32;
        let frac_index = input_index - low_index as f32;

        self.get_split_fractional_output(channel, low_index, frac_index)
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, unused)]
    fn get_fractional_input_energy(&self, channel: usize, input_index: f32) -> f32 {
        let low_index = f32::floor(input_index) as i32;
        let frac_index = input_index - low_index as f32;

        self.get_split_fractional_input_energy(channel, low_index, frac_index)
    }

    // not implementing this due to borrow checker restrictions
    // fn predictionsForChannel(&mut self, channel: i32) -> &mut
    // Prediction<f32> {     &mut self.channelPredictions[(c *
    // self.bands)..] }

    fn update_process_spectrum_steps(&mut self) {
        self.process_spectrum_steps = 0;

        if self.block_process.new_spectrum {
            self.process_spectrum_steps += self.channels;
        }

        if self.block_process.mapped_frequencies {
            self.process_spectrum_steps += Self::SMOOTH_ENERGY_STEPS;
            self.process_spectrum_steps += 1; // findPeaks
        }

        self.process_spectrum_steps += 1; // updating the output map
        self.process_spectrum_steps += self.channels; // preliminary phase-vocoder prediction
        self.process_spectrum_steps += Self::SPLIT_MAIN_PREDICTION;

        if self.block_process.new_spectrum {
            self.process_spectrum_steps += 1;
        } // .input -> .prevInput

        if self.block_process.process_formants {
            self.process_spectrum_steps += 3;
        }
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_precision_loss,
        clippy::needless_range_loop,
        clippy::indexing_slicing
    )]
    fn process_spectrum(&mut self, mut step: usize) {
        let smoothing_bins =
            (self.stft.fft_samples() as f32) / (self.stft.default_interval() as f32);
        let long_vertical_step = (smoothing_bins).round() as i32;

        let mut time_factor = self.block_process.time_factor;
        time_factor = f32::max(time_factor, 1.0 / Self::MAX_CLEAN_STRETCH);

        let random_time_factor = time_factor > Self::MAX_CLEAN_STRETCH;

        let time_factor_dist = Uniform::new(
            Self::MAX_CLEAN_STRETCH * 2.0 * if random_time_factor { 1.0 } else { 0.0 }
                - time_factor,
            time_factor,
        )
        .unwrap();

        if self.block_process.new_spectrum {
            if step < self.channels {
                let channel = step as i32;

                let mut rot = Complex::<f32>::from_polar(
                    1.0,
                    self.band_to_freq(0.0)
                        * self.stft.default_interval() as f32
                        * 2.0
                        * std::f32::consts::PI,
                );
                let freq_step = self.band_to_freq(1.0) - self.band_to_freq(0.0);
                let rot_step = Complex::<f32>::from_polar(
                    1.0,
                    freq_step * self.stft.default_interval() as f32 * 2.0 * std::f32::consts::PI,
                );

                for b in 0..self.bands {
                    let bin = &mut self.internal_channel_bands[(channel as usize * self.bands) + b];

                    bin.output = mul::<false, _>(&bin.output, &rot);
                    bin.prev_input = mul::<false, _>(&bin.prev_input, &rot);

                    rot = mul::<false, _>(&rot, &rot_step);
                }

                return;
            }

            step -= self.channels;
        }

        if self.block_process.mapped_frequencies {
            if step < Self::SMOOTH_ENERGY_STEPS {
                self.smooth_energy(step, smoothing_bins);

                return;
            }

            step -= Self::SMOOTH_ENERGY_STEPS;

            let step_check = step;
            step = step.saturating_sub(1);

            if step_check == 0 {
                self.find_peaks();

                return;
            }
        }

        let step_check = step;
        step = step.saturating_sub(1);

        if step_check == 0 {
            if self.block_process.mapped_frequencies {
                self.update_output_map();
            } else {
                // we're not pitch-shifting, so no need to find peaks etc.
                for c in 0..self.channels {
                    let bins = &mut self.internal_channel_bands[(c * self.bands)..]; //self.bandsForChannel(c);

                    for b in 0..self.bands {
                        bins[b].input_energy = norm(&bins[b].input);
                    }
                }

                for b in 0..self.bands {
                    self.output_map[b] = PitchMapPoint {
                        input_bin: b as f32,
                        freq_grad: 1.0,
                    };
                }
            }

            return;
        }

        if self.block_process.process_formants {
            if step < 3 {
                self.update_formants(step);

                return;
            }

            step -= 3;
        }

        // Preliminary output prediction from phase-vocoder
        if step < self.channels {
            let c = step as i32;

            for b in 0..self.bands {
                let map_point = self.output_map[b];
                let low_index = (map_point.input_bin).floor() as i32;
                let frac_index = map_point.input_bin - low_index as f32;

                let new_prediction_energy =
                    self.get_split_fractional_input_energy(c as usize, low_index, frac_index);
                let new_prediction_input =
                    self.get_split_fractional_input(c as usize, low_index, frac_index);
                let prev_input =
                    self.get_split_fractional_prev_input(c as usize, low_index, frac_index);

                let prediction = &mut self.channel_predictions[(c as usize * self.bands) + b];
                let prev_energy = prediction.energy;

                prediction.energy = new_prediction_energy;
                prediction.energy *= f32::max(0.0, map_point.freq_grad); // scale the energy according to local stretch factor
                prediction.input = new_prediction_input;

                let output_bin = &mut self.internal_channel_bands[(c as usize * self.bands) + b];
                let freq_twist = mul::<true, _>(&prediction.input, &prev_input);
                let phase = mul::<false, _>(&output_bin.output, &freq_twist);

                output_bin.output =
                    phase / (f32::max(prev_energy, prediction.energy) + Self::NOISE_FLOOR);
            }

            return;
        }

        step -= self.channels;

        if step < Self::SPLIT_MAIN_PREDICTION {
            // Re-predict using phase differences between frequencies
            let chunk = step;

            let start_b = (self.bands * chunk / Self::SPLIT_MAIN_PREDICTION) as i32;
            let end_b = (self.bands * (chunk + 1) / Self::SPLIT_MAIN_PREDICTION) as i32;

            for b in start_b..end_b {
                // Find maximum-energy channel and calculate that
                let mut max_channel = 0;
                let mut max_energy = self.channel_predictions[b as usize].energy; //self.predictionsForChannel(0)[b].energy;

                for c in 1..self.channels {
                    let e = self.channel_predictions[(c * self.bands) + b as usize].energy; //self.predictionsForChannel(c)[b].energy;

                    if e > max_energy {
                        max_channel = c;
                        max_energy = e;
                    }
                }

                let prediction = self.channel_predictions[(max_channel * self.bands) + b as usize];

                let mut phase = Complex::new(0.0, 0.0);
                let map_point = self.output_map[b as usize];

                // Upwards vertical steps
                if b > 0 {
                    let bin_time_factor = if random_time_factor {
                        time_factor_dist.sample(&mut self.random_engine)
                    } else {
                        time_factor
                    };
                    let down_input = self
                        .get_fractional_input(max_channel, map_point.input_bin - bin_time_factor);
                    let short_vertical_twist = mul::<true, _>(&prediction.input, &down_input);

                    let down_bin =
                        self.internal_channel_bands[(max_channel * self.bands) + (b - 1) as usize];

                    phase += mul::<false, _>(&down_bin.output, &short_vertical_twist);

                    if b >= long_vertical_step {
                        let long_down_input = self.get_fractional_input(
                            max_channel,
                            map_point.input_bin - long_vertical_step as f32 * bin_time_factor,
                        );
                        let long_vertical_twist =
                            mul::<true, _>(&prediction.input, &long_down_input);

                        let long_down_bin = self.internal_channel_bands
                            [(max_channel * self.bands) + (b - long_vertical_step) as usize];

                        phase += mul::<false, _>(&long_down_bin.output, &long_vertical_twist);
                    }
                }

                // Downwards vertical steps
                if b < self.bands as i32 - 1 {
                    let up_prediction =
                        self.channel_predictions[(max_channel * self.bands) + (b + 1) as usize];
                    let up_map_point = self.output_map[(b + 1) as usize];

                    let bin_time_factor = if random_time_factor {
                        time_factor_dist.sample(&mut self.random_engine)
                    } else {
                        time_factor
                    };
                    let down_input = self.get_fractional_input(
                        max_channel,
                        up_map_point.input_bin - bin_time_factor,
                    );
                    let short_vertical_twist = mul::<true, _>(&up_prediction.input, &down_input);

                    let up_bin =
                        self.internal_channel_bands[(max_channel * self.bands) + (b + 1) as usize];

                    phase += mul::<true, _>(&up_bin.output, &short_vertical_twist);

                    if b < self.bands as i32 - long_vertical_step {
                        let long_up_prediction = self.channel_predictions
                            [(max_channel * self.bands) + (b + long_vertical_step) as usize];
                        let long_up_map_point = self.output_map[(b + long_vertical_step) as usize];

                        let long_down_input = self.get_fractional_input(
                            max_channel,
                            long_up_map_point.input_bin
                                - long_vertical_step as f32 * bin_time_factor,
                        );
                        let long_vertical_twist =
                            mul::<true, _>(&long_up_prediction.input, &long_down_input);

                        let long_up_bin = self.internal_channel_bands
                            [(max_channel * self.bands) + (b + long_vertical_step) as usize];

                        phase += mul::<true, _>(&long_up_bin.output, &long_vertical_twist);
                    }
                }

                self.internal_channel_bands[(max_channel * self.bands) + b as usize].output =
                    prediction.make_output(phase, Self::NOISE_FLOOR);

                // All other bins are locked in phase
                for c in 0..self.channels {
                    if c != max_channel {
                        let channel_prediction =
                            self.channel_predictions[(c * self.bands) + b as usize]; //self.predictionsForChannel(c)[b];

                        let channel_twist =
                            mul::<true, _>(&channel_prediction.input, &prediction.input);
                        let channel_phase = mul::<false, _>(
                            &self.internal_channel_bands[(max_channel * self.bands) + b as usize]
                                .output,
                            &channel_twist,
                        );

                        self.internal_channel_bands[(c * self.bands) + b as usize].output =
                            channel_prediction.make_output(channel_phase, Self::NOISE_FLOOR);
                    }
                }
            }

            return;
        }

        step -= Self::SPLIT_MAIN_PREDICTION;

        if self.block_process.new_spectrum && step == 0 {
            for bin in &mut self.internal_channel_bands {
                bin.prev_input = bin.input;
            }
        }
    }

    // Produces smoothed energy across all channels
    #[allow(clippy::needless_range_loop, clippy::indexing_slicing)]
    fn smooth_energy(&mut self, step: usize, smoothing_bins: f32) {
        let smoothing_slew = 1.0 / (1.0 + smoothing_bins * 0.5);

        if step == 0 {
            self.energy.fill(0.0);

            for c in 0..self.channels {
                let bins = &mut self.internal_channel_bands[(c * self.bands)..]; //self.bandsForChannel(c);

                for b in 0..self.bands {
                    let e = norm(&bins[b].input);

                    bins[b].input_energy = e; // Used for interpolating prediction energy

                    self.energy[b] += e;
                }
            }

            for b in 0..self.bands {
                self.smoothed_energy[b] = self.energy[b];
            }

            self.smooth_energy_state = 0.0;

            return;
        }

        // The two other steps are repeated smoothing passes, down and up
        let mut e = self.smooth_energy_state;

        for b in (0..self.bands).rev() {
            e += (self.smoothed_energy[b] - e) * smoothing_slew;

            self.smoothed_energy[b] = e;
        }

        for b in 0..self.bands {
            e += (self.smoothed_energy[b] - e) * smoothing_slew;

            self.smoothed_energy[b] = e;
        }

        self.smooth_energy_state = e;
    }

    fn map_freq(&self, freq: f32) -> f32 {
        if let Some(custom_freq_map) = self.custom_freq_map {
            return custom_freq_map(freq);
        }

        if freq > self.freq_tonality_limit {
            return freq + (self.freq_multiplier - 1.0) * self.freq_tonality_limit;
        }

        freq * self.freq_multiplier
    }

    // Identifies spectral peaks using energy across all channels
    #[allow(clippy::cast_precision_loss, clippy::indexing_slicing)]
    fn find_peaks(&mut self) {
        self.peaks.clear();

        let mut start = 0;

        while start < self.bands {
            if self.energy[start] > self.smoothed_energy[start] {
                let mut end = start;

                let mut band_sum = 0.0;
                let mut energy_sum = 0.0;

                while end < self.bands && self.energy[end] > self.smoothed_energy[end] {
                    band_sum += end as f32 * self.energy[end];

                    energy_sum += self.energy[end];

                    end += 1;
                }

                let avg_band = band_sum / energy_sum;
                let avg_freq = self.band_to_freq(avg_band);

                self.peaks.push(Peak {
                    input: avg_band,
                    output: self.freq_to_band(self.map_freq(avg_freq)),
                });

                start = end;
            }

            start += 1;
        }
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_precision_loss,
        clippy::indexing_slicing
    )]
    fn update_output_map(&mut self) {
        if self.peaks.is_empty() {
            for b in 0..self.bands {
                self.output_map[b] = PitchMapPoint {
                    input_bin: b as f32,
                    freq_grad: 1.0,
                };
            }

            return;
        }

        let bottom_offset = self.peaks[0].input - self.peaks[0].output;

        for b in 0..i32::min(self.bands as i32, self.peaks[0].output.ceil() as i32) {
            self.output_map[b as usize] = PitchMapPoint {
                input_bin: b as f32 + bottom_offset,
                freq_grad: 1.0,
            };
        }

        // Interpolate between points
        for p in 1..self.peaks.len() {
            let prev = self.peaks[p - 1];
            let next = self.peaks[p];

            let range_scale = 1.0 / (next.output - prev.output);
            let out_offset = prev.input - prev.output;
            let out_scale = next.input - next.output - prev.input + prev.output;
            let grad_scale = out_scale * range_scale;
            let start_bin = i32::max(0, prev.output.ceil() as i32);
            let end_bin = i32::min(self.bands as i32, next.output.ceil() as i32);

            for b in start_bin..end_bin {
                let r = (b as f32 - prev.output) * range_scale;
                let h = r * r * (3.0 - 2.0 * r);
                let out_b = b as f32 + out_offset + h * out_scale;

                let grad_h = 6.0 * r * (1.0 - r);
                let grad_b = 1.0 + grad_h * grad_scale;

                self.output_map[b as usize] = PitchMapPoint {
                    input_bin: out_b,
                    freq_grad: grad_b,
                };
            }
        }

        let top_offset = self.peaks.last().unwrap().input - self.peaks.last().unwrap().output;

        for b in i32::max(0, self.peaks.last().unwrap().output as i32)..(self.bands as i32) {
            self.output_map[b as usize] = PitchMapPoint {
                input_bin: b as f32 + top_offset,
                freq_grad: 1.0,
            };
        }
    }

    // If we mapped formants the same way as mapFreq(), this would be the inverse
    fn inv_map_formant(&self, freq: f32) -> f32 {
        if freq * self.inv_formant_multiplier > self.freq_tonality_limit {
            return freq + (1.0 - self.formant_multiplier) * self.freq_tonality_limit;
        }

        freq * self.inv_formant_multiplier
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_precision_loss,
        clippy::needless_range_loop,
        clippy::indexing_slicing
    )]
    fn estimate_frequency(&mut self) -> f32 {
        // 3 highest peaks in the input
        let mut peak_indices: [i32; 3] = [0, 0, 0];

        for b in 1..(self.bands - 1) {
            let e = self.formant_metric[b];
            // local maxima only
            if e < self.formant_metric[b - 1] || e <= self.formant_metric[b + 1] {
                continue;
            }

            if e > self.formant_metric[peak_indices[0] as usize] {
                if e > self.formant_metric[peak_indices[1] as usize] {
                    if e > self.formant_metric[peak_indices[2] as usize] {
                        peak_indices = [peak_indices[1], peak_indices[2], b as i32];
                    } else {
                        peak_indices = [peak_indices[1], b as i32, peak_indices[2]];
                    }
                } else {
                    peak_indices[0] = b as i32;
                }
            }
        }

        // VERY rough pitch estimation
        let mut peak_estimate = peak_indices[2];

        if self.formant_metric[peak_indices[1] as usize]
            > self.formant_metric[peak_indices[2] as usize] * 0.1
        {
            let mut diff = (peak_estimate - peak_indices[1]).abs();

            if diff > peak_estimate / 8 && diff < peak_estimate * 7 / 8 {
                peak_estimate %= diff;
            }

            if self.formant_metric[peak_indices[0] as usize]
                > self.formant_metric[peak_indices[2] as usize] * 0.01
            {
                diff = (peak_estimate - peak_indices[0]).abs();

                if diff > peak_estimate / 8 && diff < peak_estimate * 7 / 8 {
                    peak_estimate %= diff;
                }
            }
        }

        let weight = self.formant_metric[peak_indices[2] as usize];

        // Smooth it out a bit
        self.freq_estimate_weighted +=
            (peak_estimate as f32 * weight - self.freq_estimate_weighted) * 0.25;
        self.freq_estimate_weight += (weight - self.freq_estimate_weight) * 0.25;

        self.freq_estimate_weighted / (self.freq_estimate_weight + 1e-30)
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::indexing_slicing
    )]
    fn update_formants_get_formant(&self, mut band: f32) -> f32 {
        if band < 0.0 {
            return 0.0;
        }

        band = f32::min(band, self.bands as f32);

        let floor_band = band.floor();
        let frac_band = band - floor_band;

        let low = self.formant_metric[floor_band.trunc() as usize];
        let high = self.formant_metric[(floor_band + 1.0).trunc() as usize];

        low + (high - low) * frac_band
    }

    #[allow(
        clippy::cast_precision_loss,
        clippy::needless_range_loop,
        clippy::indexing_slicing
    )]
    fn update_formants(&mut self, mut step: usize) {
        let step_check = step;
        step = step.saturating_sub(1);

        if step_check == 0 {
            self.formant_metric.fill(0.0);

            for c in 0..self.channels {
                let bins = &mut self.internal_channel_bands[(c * self.bands)..]; //self.bandsForChannel(c);

                for b in 0..self.bands {
                    self.formant_metric[b] += bins[b].input_energy;
                }
            }

            self.freq_estimate = self.freq_to_band(self.formant_base_freq);
            if self.formant_base_freq <= 0.0 {
                self.freq_estimate = self.estimate_frequency();
            }
        } else if step == 0 {
            let mut decay = 1.0 - 1.0 / (self.freq_estimate * 0.5 + 1.0);
            let mut e = 0.0;

            for _ in 0..2 {
                for b in (0..self.bands).rev() {
                    e = f32::max(self.formant_metric[b], e * decay);

                    self.formant_metric[b] = e;
                }

                for b in 0..self.bands {
                    e = f32::max(self.formant_metric[b], e * decay);

                    self.formant_metric[b] = e;
                }
            }

            decay = 1.0 / decay;

            for _ in 0..2 {
                for b in (0..self.bands).rev() {
                    e = f32::min(self.formant_metric[b], e * decay);

                    self.formant_metric[b] = e;
                }

                for b in 0..self.bands {
                    e = f32::min(self.formant_metric[b], e * decay);

                    self.formant_metric[b] = e;
                }
            }
        } else {
            for b in 0..self.bands {
                let input_f = self.band_to_freq(b as f32);
                let mut output_f = if self.formant_compensation {
                    self.map_freq(input_f)
                } else {
                    input_f
                };
                output_f = self.inv_map_formant(output_f);

                let input_e = self.formant_metric[b];
                let target_e = self.update_formants_get_formant(self.freq_to_band(output_f));

                let formant_ratio = target_e / (input_e + 1e-30);
                let energy_ratio = formant_ratio;

                for c in 0..self.channels {
                    let bins = &mut self.internal_channel_bands[(c * self.bands)..]; //self.bandsForChannel(c);

                    // This is what's used to decide the output energy, so this affects the output
                    bins[b].input_energy *= energy_ratio;
                }
            }
        }
    }
}
