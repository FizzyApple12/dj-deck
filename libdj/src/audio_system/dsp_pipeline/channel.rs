use std::f32;

// use cute_dsp::filters::StereoBiquad;
use crate::{
    AUDIO_CHANNELS,
    audio_system::{audio_loader::TrackAudioData, dsp_pipeline::player::PlayerDSP},
    math::fader::SingleFader,
    playback::ChannelUpdateResults,
    types::deck::ChannelState,
};

pub struct ChannelDSP {
    _target_sample_rate: u32,

    player: PlayerDSP,

    master_output_buffers: [Vec<f32>; AUDIO_CHANNELS],
    touch_cue_output_buffers: [Vec<f32>; AUDIO_CHANNELS],
    // eq_filters: (StereoBiquad<f32>, StereoBiquad<f32>, StereoBiquad<f32>),

    // filter: StereoBiquad<f32>,
}

impl ChannelDSP {
    pub fn new(sample_rate: u32) -> ChannelDSP {
        ChannelDSP {
            _target_sample_rate: sample_rate,

            player: PlayerDSP::new(sample_rate),

            master_output_buffers: Default::default(),
            touch_cue_output_buffers: Default::default(),
            // eq_filters: (
            //     StereoBiquad::new(true),
            //     StereoBiquad::new(true),
            //     StereoBiquad::new(true),
            // ),

            // filter: StereoBiquad::new(true),
        }
    }

    pub fn assign_track_data(&mut self, track_data: Option<Box<TrackAudioData>>) {
        self.reset();

        self.player.assign_track_data(track_data);
    }

    // the definitions of these types make this safe and we need max speed
    #[allow(clippy::indexing_slicing)]
    pub fn generate_samples(
        &mut self,
        channel_state: &ChannelState,
        channel_update_results: &ChannelUpdateResults,
        master_output_buffers: &mut [Vec<f32>; AUDIO_CHANNELS],
        cue_output_buffers: &mut [Vec<f32>; AUDIO_CHANNELS],
        number_samples: usize,
    ) {
        for master_output_buffer in master_output_buffers.iter_mut() {
            if master_output_buffer.len() < number_samples {
                master_output_buffer.resize(number_samples, 0.0);
            }
        }
        for cue_output_buffer in cue_output_buffers.iter_mut() {
            if cue_output_buffer.len() < number_samples {
                cue_output_buffer.resize(number_samples, 0.0);
            }
        }

        self.player.generate_samples(
            &channel_state.player,
            &channel_update_results.player,
            &mut self.master_output_buffers,
            &mut self.touch_cue_output_buffers,
            number_samples,
        );

        // let [master_output_left_buffer, master_output_right_buffer] =
        //     &mut self.master_output_buffers;

        // self.eq_filters
        //     .0
        //     .low_shelf(MIXER_EQ_LOW_CUTOFF, channel_state.eq.0);
        // self.eq_filters
        //     .0
        //     .process_buffer(master_output_left_buffer, master_output_right_buffer);

        // self.eq_filters.1.peak(
        //     (MIXER_EQ_LOW_CUTOFF * MIXER_EQ_HIGH_CUTOFF).sqrt(),
        //     2.0,
        //     channel_state.eq.1,
        // );
        // self.eq_filters
        //     .1
        //     .process_buffer(master_output_left_buffer, master_output_right_buffer);

        // self.eq_filters
        //     .2
        //     .high_shelf(MIXER_EQ_HIGH_CUTOFF, channel_state.eq.2);
        // self.eq_filters
        //     .2
        //     .process_buffer(master_output_left_buffer, master_output_right_buffer);

        // if channel_state.filter < -f32::EPSILON {
        //     // todo: what the fuck should the q factor be
        //     self.filter.lowpass(
        //         LowPassFilter::frequency(-channel_state.filter),
        //         0.5,
        //         BiquadDesign::OneSided,
        //     );

        //     self.filter
        //         .process_buffer(master_output_left_buffer,
        // master_output_right_buffer); } else if channel_state.filter >
        // f32::EPSILON {     // todo: what the fuck should the q factor be
        //     self.filter.highpass(
        //         HighPassFilter::frequency(channel_state.filter),
        //         0.5,
        //         BiquadDesign::OneSided,
        //     );

        //     self.filter
        //         .process_buffer(master_output_left_buffer,
        // master_output_right_buffer); }

        for (buffer_index, master_output_buffer) in master_output_buffers.iter_mut().enumerate() {
            for (sample_number, sample) in master_output_buffer.iter_mut().enumerate() {
                *sample = self.master_output_buffers[buffer_index][sample_number]
                    .fade(channel_state.fade);
            }
        }

        for (buffer_index, cue_output_buffer) in cue_output_buffers.iter_mut().enumerate() {
            for (sample_number, sample) in cue_output_buffer.iter_mut().enumerate() {
                *sample = self.touch_cue_output_buffers[buffer_index][sample_number]
                    + if channel_state.cue {
                        self.master_output_buffers[buffer_index][sample_number]
                    } else {
                        0.0
                    };
            }
        }
    }

    pub fn reset(&mut self) {
        // self.eq_filters.0.reset();
        // self.eq_filters.1.reset();
        // self.eq_filters.2.reset();

        // self.filter.reset();
    }
}
