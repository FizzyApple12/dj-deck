use std::f32;

use libdsp::audio_loader::TrackAudioData;

use crate::{
    AUDIO_CHANNELS, MIXER_CHANNELS, audio::dsp_pipeline::channel::ChannelDSP,
    math::fader::CrossFader, playback::deck::DeckUpdateResults, types::deck::DeckState,
};

pub struct DeckDSP {
    _target_sample_rate: u32,

    channels: [ChannelDSP; MIXER_CHANNELS],
    master_output_buffers: [[Vec<f32>; AUDIO_CHANNELS]; MIXER_CHANNELS],
    cue_output_buffers: [[Vec<f32>; AUDIO_CHANNELS]; MIXER_CHANNELS],
}

impl DeckDSP {
    pub fn new(sample_rate: u32) -> DeckDSP {
        DeckDSP {
            _target_sample_rate: sample_rate,

            channels: [
                ChannelDSP::new(sample_rate),
                ChannelDSP::new(sample_rate),
                ChannelDSP::new(sample_rate),
                ChannelDSP::new(sample_rate),
            ],
            master_output_buffers: Default::default(),
            cue_output_buffers: Default::default(),
        }
    }

    pub fn assign_track_data(&mut self, channel: usize, track_data: Option<Box<TrackAudioData>>) {
        let _ = self
            .channels
            .get_mut(channel)
            .map(|channel| channel.assign_track_data(track_data));
    }

    // the definitions of these types make this safe and we need max speed
    #[allow(clippy::indexing_slicing)]
    pub fn generate_samples(
        &mut self,
        deck_state: &DeckState,
        deck_update_results: &DeckUpdateResults,
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

        for (channel_number, channel) in self.channels.iter_mut().enumerate() {
            channel.generate_samples(
                &deck_state.mixer_channels[channel_number],
                &deck_update_results.channels[channel_number],
                &mut self.master_output_buffers[channel_number],
                &mut self.cue_output_buffers[channel_number],
                number_samples,
            );
        }

        for (buffer_index, master_output_buffer) in master_output_buffers.iter_mut().enumerate() {
            for (sample_number, sample) in master_output_buffer.iter_mut().enumerate() {
                *sample = self.master_output_buffers[0][buffer_index][sample_number].crossfade(
                    deck_state.crossfade,
                    deck_state.mixer_channels[0].cross_fader_side,
                ) + self.master_output_buffers[1][buffer_index][sample_number].crossfade(
                    deck_state.crossfade,
                    deck_state.mixer_channels[1].cross_fader_side,
                ) + self.master_output_buffers[2][buffer_index][sample_number].crossfade(
                    deck_state.crossfade,
                    deck_state.mixer_channels[2].cross_fader_side,
                ) + self.master_output_buffers[3][buffer_index][sample_number].crossfade(
                    deck_state.crossfade,
                    deck_state.mixer_channels[3].cross_fader_side,
                );
            }
        }

        for (buffer_index, cue_output_buffer) in cue_output_buffers.iter_mut().enumerate() {
            for (sample_number, sample) in cue_output_buffer.iter_mut().enumerate() {
                *sample = self.cue_output_buffers[0][buffer_index][sample_number]
                    + self.cue_output_buffers[1][buffer_index][sample_number]
                    + self.cue_output_buffers[2][buffer_index][sample_number]
                    + self.cue_output_buffers[3][buffer_index][sample_number];
            }
        }
    }

    pub fn reset(&mut self) {
        for channel in &mut self.channels {
            channel.reset();
        }
    }
}
