use std::f32;

use libdsp::audio_loader::TrackAudioData;

use crate::{
    AUDIO_CHANNELS, audio_system::dsp_pipeline::track_processor::TrackProcessor,
    playback::PlayerUpdateResults, types::deck::PlayerState,
};

pub struct PlayerDSP {
    _target_sample_rate: u32,

    loaded_track: Option<TrackAudioData>,

    master_processor: TrackProcessor,
    touch_cue_processor: TrackProcessor,
}

impl PlayerDSP {
    pub fn new(sample_rate: u32) -> PlayerDSP {
        PlayerDSP {
            _target_sample_rate: sample_rate,

            loaded_track: None,

            master_processor: TrackProcessor::new(sample_rate),
            touch_cue_processor: TrackProcessor::new(sample_rate),
        }
    }

    pub fn assign_track_data(&mut self, track_data: Option<Box<TrackAudioData>>) {
        if let Some(track_data) = track_data.map(|track_data| *track_data) {
            self.loaded_track = Some(track_data);
        } else {
            self.loaded_track = None;
        }

        self.reset();
    }

    pub fn generate_samples(
        &mut self,
        player_state: &PlayerState,
        player_update_results: &PlayerUpdateResults,
        master_output_buffers: &mut [Vec<f32>; AUDIO_CHANNELS],
        touch_cue_output_buffers: &mut [Vec<f32>; AUDIO_CHANNELS],
        number_samples: usize,
    ) {
        for master_output_buffer in master_output_buffers.iter_mut() {
            if master_output_buffer.len() < number_samples {
                master_output_buffer.resize(number_samples, 0.0);
            }
        }
        for touch_cue_output_buffer in touch_cue_output_buffers.iter_mut() {
            if touch_cue_output_buffer.len() < number_samples {
                touch_cue_output_buffer.resize(number_samples, 0.0);
            }
        }

        self.generate_master_samples(
            player_state,
            player_update_results,
            master_output_buffers,
            number_samples,
        );
        self.generate_touch_cue_samples(
            player_state,
            player_update_results,
            touch_cue_output_buffers,
            number_samples,
        );
    }

    fn generate_master_samples(
        &mut self,
        channel_state: &PlayerState,
        player_update_results: &PlayerUpdateResults,
        output_buffers: &mut [Vec<f32>; AUDIO_CHANNELS],
        number_samples: usize,
    ) {
        let Some(audio_data) = &self.loaded_track else {
            for output_buffer in output_buffers {
                output_buffer.fill(0.0);
            }

            return;
        };

        self.master_processor.perform_sample_stretch_interpolate(
            audio_data,
            player_update_results.playback_frame_start_time,
            player_update_results.playback_frame_end_time,
            player_update_results.playback_wrap_times,
            if channel_state.master_tempo && !channel_state.jog_hold {
                Some(channel_state.keyshift)
            } else {
                None
            },
            output_buffers,
            number_samples,
        );
    }

    fn generate_touch_cue_samples(
        &mut self,
        channel_state: &PlayerState,
        player_update_results: &PlayerUpdateResults,
        output_buffers: &mut [Vec<f32>; AUDIO_CHANNELS],
        number_samples: usize,
    ) {
        let Some(audio_data) = &self.loaded_track else {
            for output_buffer in output_buffers {
                output_buffer.fill(0.0);
            }

            return;
        };

        let Some((touch_cue_start_time, touch_cue_end_time)) =
            player_update_results.touch_cue_playback_times
        else {
            for output_buffer in output_buffers {
                output_buffer.fill(0.0);
            }

            return;
        };

        self.touch_cue_processor.perform_sample_stretch_interpolate(
            audio_data,
            touch_cue_start_time,
            touch_cue_end_time,
            None,
            if channel_state.master_tempo {
                Some(channel_state.keyshift)
            } else {
                None
            },
            output_buffers,
            number_samples,
        );
    }

    pub fn reset(&mut self) {
        self.master_processor.reset();
        self.touch_cue_processor.reset();

        if let Some(track_data) = &self.loaded_track {
            self.master_processor.setup(track_data);
            self.touch_cue_processor.setup(track_data);
        }
    }
}
