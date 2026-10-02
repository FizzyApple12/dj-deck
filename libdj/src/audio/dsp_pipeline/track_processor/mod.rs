pub mod io;
pub mod resampler;

use std::f32;

use libdsp::{
    audio_loader::TrackAudioData,
    stretch::{SignalsmithStretch, SignalsmithStretchTrait},
};
use timecode::Timecode;

use crate::{
    AUDIO_CHANNELS,
    audio::dsp_pipeline::track_processor::{
        io::{VecIoBuffer, VecIoBufferMut},
        resampler::Resampler,
    },
};

pub struct TrackProcessor {
    target_sample_rate: u32,

    stretch_input_buffers: [Vec<f32>; AUDIO_CHANNELS],
    stretcher: SignalsmithStretch<f32>,
    stretcher_has_config: bool,

    resampler_input_buffers: [Vec<f32>; AUDIO_CHANNELS],
    resampler: Resampler,
}

impl TrackProcessor {
    pub fn new(sample_rate: u32) -> TrackProcessor {
        TrackProcessor {
            target_sample_rate: sample_rate,

            stretch_input_buffers: Default::default(),
            stretcher: SignalsmithStretch::<f32>::new(),
            stretcher_has_config: false,

            resampler_input_buffers: Default::default(),
            resampler: Resampler::default(),
        }
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        clippy::cast_possible_wrap,
        clippy::indexing_slicing,
        clippy::cast_lossless,
        clippy::needless_range_loop,
        clippy::too_many_arguments
    )]
    #[allow(clippy::println_empty_string)]
    pub fn perform_sample_stretch_interpolate(
        &mut self,
        track_data: &TrackAudioData,
        start_time: Timecode,
        end_time: Timecode,
        wrap_times: Option<(Timecode, Timecode)>,
        pitch: Option<f32>,
        output_buffers: &mut [Vec<f32>; AUDIO_CHANNELS],
        number_samples: usize,
    ) {
        // println!("");

        let resampler_input_length = if pitch.is_some() {
            let samples_retrieved = track_data.read_samples(
                start_time,
                end_time,
                wrap_times,
                &mut self.stretch_input_buffers,
            );

            let resample_ratio = track_data.sample_rate as f64 / self.target_sample_rate as f64;

            let stretch_output_samples = (number_samples as f64 * resample_ratio).ceil() as usize;

            // if let Some((wrap_start, wrap_end)) = wrap_times {
            //     println!(
            //         "[{} , {}] : {samples_retrieved} => {stretch_output_samples} =>
            // {number_samples}",         (wrap_start - start_time).nanoseconds,
            //         (end_time - wrap_end).nanoseconds
            //     );
            // } else {
            //     println!(
            //         "[{}] : {samples_retrieved} => {stretch_output_samples} =>
            // {number_samples}",         (end_time - start_time).nanoseconds
            //     );
            // }

            for input_buffer in &mut self.resampler_input_buffers {
                if input_buffer.len() < stretch_output_samples {
                    input_buffer.resize(stretch_output_samples, 0.0);
                }
            }

            // println!("stretch_input_buffers: {:?}", self.stretch_input_buffers);

            if samples_retrieved == 0 {
                for input_buffer in &mut self.resampler_input_buffers {
                    input_buffer.fill(0.0);
                }
            } else {
                self.stretcher.process(
                    &VecIoBuffer::new(&self.stretch_input_buffers),
                    samples_retrieved as i32,
                    &mut VecIoBufferMut::new(&mut self.resampler_input_buffers),
                    stretch_output_samples as i32,
                );
            }

            stretch_output_samples
        } else {
            track_data.read_samples(
                start_time,
                end_time,
                wrap_times,
                &mut self.resampler_input_buffers,
            )
        };

        for output_buffer in output_buffers.iter_mut() {
            if output_buffer.len() < number_samples {
                output_buffer.resize(number_samples, 0.0);
            }
        }

        // println!("INPUT: {:?}", self.resampler_input_buffers);

        self.resampler.process(
            &self.resampler_input_buffers,
            resampler_input_length,
            output_buffers,
            number_samples,
            pitch.is_some(),
        );

        // println!("OUTPUT: {output_buffers:?}");
    }

    pub fn reset(&mut self) {
        if self.stretcher_has_config {
            self.stretcher.reset();
        }

        self.resampler.reset();
    }

    pub fn setup(&mut self, track_data: &TrackAudioData) {
        #[allow(clippy::cast_precision_loss)]
        self.stretcher
            .preset_default(AUDIO_CHANNELS, track_data.sample_rate as f32, true);

        self.stretcher_has_config = true;

        self.reset();
    }
}
