use std::f32;

// use cute_dsp::stretch::SignalsmithStretch;
use crate::{
    AUDIO_CHANNELS, audio_system::audio_loader::TrackAudioData, types::timecode::Timecode,
};

pub struct TrackProcessor {
    target_sample_rate: u32,

    // stretcher: SignalsmithStretch<f32>,
    _stretch_input_buffers: [Vec<f32>; AUDIO_CHANNELS],
    stretch_output_buffers: [Vec<f32>; AUDIO_CHANNELS],
}

impl TrackProcessor {
    pub fn new(sample_rate: u32) -> TrackProcessor {
        TrackProcessor {
            target_sample_rate: sample_rate,

            // stretcher: SignalsmithStretch::<f32>::new(),
            _stretch_input_buffers: Default::default(),
            stretch_output_buffers: Default::default(),
        }
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        clippy::indexing_slicing,
        clippy::cast_lossless,
        clippy::needless_range_loop
    )]
    pub fn perform_sample_stretch_interpolate(
        &mut self,
        track_data: &TrackAudioData,
        start_time: Timecode,
        end_time: Timecode,
        wrap_times: Option<(Timecode, Timecode)>,
        output_buffers: &mut [Vec<f32>; AUDIO_CHANNELS],
        number_samples: usize,
    ) {
        for output_buffer in output_buffers.iter_mut() {
            if output_buffer.len() < number_samples {
                output_buffer.resize(number_samples, 0.0);
            }
        }

        // todo: send to stretch_input_buffers and run stretcher
        let samples_retrieved = track_data.read_samples(
            start_time,
            end_time,
            wrap_times,
            &mut self.stretch_output_buffers,
        );

        let ratio = track_data.sample_rate as f64 / self.target_sample_rate as f64;

        for buffer_index in 0..AUDIO_CHANNELS {
            for output_index in 0..number_samples {
                let pos = output_index as f64 * ratio;

                let idx_lo = pos.floor() as usize;
                let idx_hi = (idx_lo + 1).min(samples_retrieved - 1);
                let frac = (pos - pos.floor()) as f32;

                // todo: rewrite to use something like kaiser-sinc
                let sample = self.stretch_output_buffers[buffer_index][idx_lo]
                    + frac
                        * (self.stretch_output_buffers[buffer_index][idx_hi]
                            - self.stretch_output_buffers[buffer_index][idx_lo]); //
                output_buffers[buffer_index][output_index] = sample;
                if output_index >= samples_retrieved {
                    output_buffers[buffer_index][output_index] = 0.0;
                } else {
                    output_buffers[buffer_index][output_index] =
                        self.stretch_output_buffers[buffer_index][output_index];
                }
            }
        }
    }

    pub fn reset(&mut self) {
        // self.stretcher.reset();
    }

    pub fn setup(&mut self, track_data: &TrackAudioData) {
        // self.stretcher.reset();

        // #[allow(clippy::cast_precision_loss)]
        // self.stretcher
        //     .preset_default(AUDIO_CHANNELS, track_data.sample_rate as f32,
        // true);
    }
}
