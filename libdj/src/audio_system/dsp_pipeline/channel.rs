use std::f32;

use libdsp::{
    audio_loader::TrackAudioData,
    dsp::filters::{BiquadDesign, BiquadStatic},
};

use crate::{
    AUDIO_CHANNELS, MIXER_EQ_HIGH_CUTOFF, MIXER_EQ_HIGH_OCTAVES, MIXER_EQ_LOW_CUTOFF,
    MIXER_EQ_LOW_OCTAVES, MIXER_EQ_MID_CENTER, MIXER_EQ_MID_Q, MIXER_FILTER_HIGH_PASS_OCTAVES,
    MIXER_FILTER_LOW_PASS_OCTAVES, audio_system::dsp_pipeline::player::PlayerDSP,
    math::fader::SingleFader, playback::ChannelUpdateResults, types::deck::ChannelState,
};

const BASICALLY_INFINITY: f32 = 100.0;

// i disagree!
#[allow(clippy::type_complexity)]
pub struct ChannelDSP {
    target_sample_rate: u32,

    player: PlayerDSP,

    eq_coefficients: (f32, f32, f32),
    filter_coefficient: f32,

    master_output_buffers: [Vec<f32>; AUDIO_CHANNELS],
    master_eq_filters: (
        [BiquadStatic<f32, true>; AUDIO_CHANNELS],
        [BiquadStatic<f32, true>; AUDIO_CHANNELS],
        [BiquadStatic<f32, true>; AUDIO_CHANNELS],
    ),
    master_filter: (
        [BiquadStatic<f32, true>; AUDIO_CHANNELS],
        [BiquadStatic<f32, true>; AUDIO_CHANNELS],
    ),

    touch_cue_output_buffers: [Vec<f32>; AUDIO_CHANNELS],
    touch_cue_eq_filters: (
        [BiquadStatic<f32, true>; AUDIO_CHANNELS],
        [BiquadStatic<f32, true>; AUDIO_CHANNELS],
        [BiquadStatic<f32, true>; AUDIO_CHANNELS],
    ),
    touch_cue_filter: (
        [BiquadStatic<f32, true>; AUDIO_CHANNELS],
        [BiquadStatic<f32, true>; AUDIO_CHANNELS],
    ),
}

impl ChannelDSP {
    pub fn new(sample_rate: u32) -> ChannelDSP {
        ChannelDSP {
            target_sample_rate: sample_rate,

            player: PlayerDSP::new(sample_rate),

            eq_coefficients: (0.0, 0.0, 0.0),
            filter_coefficient: 0.0,

            master_output_buffers: Default::default(),
            master_eq_filters: (
                [BiquadStatic::default(); AUDIO_CHANNELS],
                [BiquadStatic::default(); AUDIO_CHANNELS],
                [BiquadStatic::default(); AUDIO_CHANNELS],
            ),
            master_filter: (
                [BiquadStatic::default(); AUDIO_CHANNELS],
                [BiquadStatic::default(); AUDIO_CHANNELS],
            ),

            touch_cue_output_buffers: Default::default(),
            touch_cue_eq_filters: (
                [BiquadStatic::default(); AUDIO_CHANNELS],
                [BiquadStatic::default(); AUDIO_CHANNELS],
                [BiquadStatic::default(); AUDIO_CHANNELS],
            ),
            touch_cue_filter: (
                [BiquadStatic::default(); AUDIO_CHANNELS],
                [BiquadStatic::default(); AUDIO_CHANNELS],
            ),
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

        self.eq_coefficients = channel_state.eq;
        self.filter_coefficient = channel_state.filter;

        self.update_filters();

        for (buffer_index, master_output_buffer) in master_output_buffers.iter_mut().enumerate() {
            for (sample_number, sample) in master_output_buffer.iter_mut().enumerate() {
                let root_sample = self.master_output_buffers[buffer_index][sample_number];

                let low_filtered = self.master_eq_filters.0[buffer_index].process(root_sample);
                let mid_filtered = self.master_eq_filters.1[buffer_index].process(low_filtered);
                let high_filtered = self.master_eq_filters.2[buffer_index].process(mid_filtered);

                let filter_low_filtered = self.master_filter.0[buffer_index].process(high_filtered);
                let filter_high_filtered =
                    self.master_filter.1[buffer_index].process(high_filtered);

                let final_filtered = if channel_state.filter < -f32::EPSILON {
                    filter_low_filtered
                } else if channel_state.filter > f32::EPSILON {
                    filter_high_filtered
                } else {
                    high_filtered
                };

                *sample = final_filtered.fade(channel_state.fade);
                // *sample = low_filtered.fade(channel_state.fade);
                // *sample = root_sample.fade(channel_state.fade);
            }
        }

        for (buffer_index, cue_output_buffer) in cue_output_buffers.iter_mut().enumerate() {
            for (sample_number, sample) in cue_output_buffer.iter_mut().enumerate() {
                let root_sample = self.touch_cue_output_buffers[buffer_index][sample_number];

                let low_filtered = self.touch_cue_eq_filters.0[buffer_index].process(root_sample);
                let mid_filtered = self.touch_cue_eq_filters.1[buffer_index].process(low_filtered);
                let high_filtered = self.touch_cue_eq_filters.2[buffer_index].process(mid_filtered);

                let filter_low_filtered =
                    self.touch_cue_filter.0[buffer_index].process(high_filtered);
                let filter_high_filtered =
                    self.touch_cue_filter.1[buffer_index].process(high_filtered);

                let final_filtered = if channel_state.filter < -f32::EPSILON {
                    filter_low_filtered
                } else if channel_state.filter > f32::EPSILON {
                    filter_high_filtered
                } else {
                    high_filtered
                };

                *sample = final_filtered
                    + if channel_state.cue {
                        self.master_output_buffers[buffer_index][sample_number]
                    } else {
                        0.0
                    };
            }
        }
    }

    pub fn reset(&mut self) {
        self.master_eq_filters = (
            [BiquadStatic::default(); AUDIO_CHANNELS],
            [BiquadStatic::default(); AUDIO_CHANNELS],
            [BiquadStatic::default(); AUDIO_CHANNELS],
        );
        self.master_filter = (
            [BiquadStatic::default(); AUDIO_CHANNELS],
            [BiquadStatic::default(); AUDIO_CHANNELS],
        );

        self.touch_cue_eq_filters = (
            [BiquadStatic::default(); AUDIO_CHANNELS],
            [BiquadStatic::default(); AUDIO_CHANNELS],
            [BiquadStatic::default(); AUDIO_CHANNELS],
        );
        self.touch_cue_filter = (
            [BiquadStatic::default(); AUDIO_CHANNELS],
            [BiquadStatic::default(); AUDIO_CHANNELS],
        );
    }

    #[allow(clippy::indexing_slicing)]
    fn update_filters(&mut self) {
        for buffer_index in 0..AUDIO_CHANNELS {
            // self.master_eq_filters.0[buffer_index].low_shelf_db(
            //     MIXER_EQ_LOW_CUTOFF / f64::from(self.target_sample_rate),
            //     f64::from(amplitude_to_db(self.eq_coefficients.0 +
            // 1.0).max(-BASICALLY_INFINITY)),     MIXER_EQ_LOW_OCTAVES,
            //     BiquadDesign::OneSided,
            // );
            // self.master_eq_filters.1[buffer_index].peak_db_q(
            //     MIXER_EQ_MID_CENTER / f64::from(self.target_sample_rate),
            //     f64::from(amplitude_to_db(self.eq_coefficients.1 +
            // 1.0).max(-BASICALLY_INFINITY)),     MIXER_EQ_MID_Q,
            //     BiquadDesign::Cookbook,
            // );
            // self.master_eq_filters.2[buffer_index].high_shelf_db(
            //     MIXER_EQ_HIGH_CUTOFF / f64::from(self.target_sample_rate),
            //     f64::from(amplitude_to_db(self.eq_coefficients.2 +
            // 1.0).max(-BASICALLY_INFINITY)),     MIXER_EQ_HIGH_OCTAVES,
            //     BiquadDesign::OneSided,
            // );

            self.master_eq_filters.0[buffer_index].low_shelf(
                MIXER_EQ_LOW_CUTOFF / f64::from(self.target_sample_rate),
                f64::from(self.eq_coefficients.0 + 1.0),
                MIXER_EQ_LOW_OCTAVES,
                BiquadDesign::OneSided,
            );
            self.master_eq_filters.1[buffer_index].peak_q(
                MIXER_EQ_MID_CENTER / f64::from(self.target_sample_rate),
                f64::from(self.eq_coefficients.1 + 1.0),
                MIXER_EQ_MID_Q,
                BiquadDesign::Cookbook,
            );
            self.master_eq_filters.2[buffer_index].high_shelf(
                MIXER_EQ_HIGH_CUTOFF / f64::from(self.target_sample_rate),
                f64::from(self.eq_coefficients.2 + 1.0),
                MIXER_EQ_HIGH_OCTAVES,
                BiquadDesign::OneSided,
            );

            // self.touch_cue_eq_filters.0[buffer_index].low_shelf_db(
            //     MIXER_EQ_LOW_CUTOFF / f64::from(self.target_sample_rate),
            //     f64::from(amplitude_to_db(self.eq_coefficients.0 +
            // 1.0).max(-BASICALLY_INFINITY)),     MIXER_EQ_LOW_OCTAVES,
            //     BiquadDesign::OneSided,
            // );
            // self.touch_cue_eq_filters.1[buffer_index].peak_db_q(
            //     MIXER_EQ_MID_CENTER / f64::from(self.target_sample_rate),
            //     f64::from(amplitude_to_db(self.eq_coefficients.1 +
            // 1.0).max(-BASICALLY_INFINITY)),     MIXER_EQ_MID_Q,
            //     BiquadDesign::Cookbook,
            // );
            // self.touch_cue_eq_filters.2[buffer_index].high_shelf_db(
            //     MIXER_EQ_HIGH_CUTOFF / f64::from(self.target_sample_rate),
            //     f64::from(amplitude_to_db(self.eq_coefficients.2 +
            // 1.0).max(-BASICALLY_INFINITY)),     MIXER_EQ_HIGH_OCTAVES,
            //     BiquadDesign::OneSided,
            // );

            self.touch_cue_eq_filters.0[buffer_index].low_shelf(
                MIXER_EQ_LOW_CUTOFF / f64::from(self.target_sample_rate),
                f64::from(self.eq_coefficients.0 + 1.0),
                MIXER_EQ_LOW_OCTAVES,
                BiquadDesign::OneSided,
            );
            self.touch_cue_eq_filters.1[buffer_index].peak_q(
                MIXER_EQ_MID_CENTER / f64::from(self.target_sample_rate),
                f64::from(self.eq_coefficients.1 + 1.0),
                MIXER_EQ_MID_Q,
                BiquadDesign::Cookbook,
            );
            self.touch_cue_eq_filters.2[buffer_index].high_shelf(
                MIXER_EQ_HIGH_CUTOFF / f64::from(self.target_sample_rate),
                f64::from(self.eq_coefficients.2 + 1.0),
                MIXER_EQ_HIGH_OCTAVES,
                BiquadDesign::OneSided,
            );

            self.master_filter.0[buffer_index].lowpass(
                (-f64::from(self.filter_coefficient).min(0.0) / f64::from(self.target_sample_rate))
                    / f64::from(self.target_sample_rate),
                MIXER_FILTER_LOW_PASS_OCTAVES,
                BiquadDesign::OneSided,
            );
            self.master_filter.1[buffer_index].highpass(
                (f64::from(self.filter_coefficient).max(0.0) / f64::from(self.target_sample_rate))
                    / f64::from(self.target_sample_rate),
                MIXER_FILTER_HIGH_PASS_OCTAVES,
                BiquadDesign::OneSided,
            );

            self.touch_cue_filter.0[buffer_index].lowpass(
                (-f64::from(self.filter_coefficient).min(0.0) / f64::from(self.target_sample_rate))
                    / f64::from(self.target_sample_rate),
                MIXER_FILTER_LOW_PASS_OCTAVES,
                BiquadDesign::OneSided,
            );
            self.touch_cue_filter.1[buffer_index].highpass(
                (f64::from(self.filter_coefficient).max(0.0) / f64::from(self.target_sample_rate))
                    / f64::from(self.target_sample_rate),
                MIXER_FILTER_HIGH_PASS_OCTAVES,
                BiquadDesign::OneSided,
            );
        }
    }
}
