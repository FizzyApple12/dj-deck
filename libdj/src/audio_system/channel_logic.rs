use std::f32;

use cute_dsp::{
    filters::{BiquadDesign, StereoBiquad},
    stretch::SignalsmithStretch,
};

use crate::{
    AUDIO_CHANNELS, MIXER_CHANNELS, MIXER_EQ_HIGH_CUTOFF, MIXER_EQ_LOW_CUTOFF,
    audio_system::audio_loader::TrackAudioData,
    math::{
        fader::{CrossFader, SingleFader},
        filter::{HighPassFilter, LowPassFilter},
    },
    types::deck::{ChannelState, DeckState, PlayState, PlayerState},
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

pub struct ChannelDSP {
    _target_sample_rate: u32,

    player: PlayerDSP,

    master_output_buffers: [Vec<f32>; AUDIO_CHANNELS],
    touch_cue_output_buffers: [Vec<f32>; AUDIO_CHANNELS],

    eq_filters: (StereoBiquad<f32>, StereoBiquad<f32>, StereoBiquad<f32>),

    filter: StereoBiquad<f32>,
}

impl ChannelDSP {
    pub fn new(sample_rate: u32) -> ChannelDSP {
        ChannelDSP {
            _target_sample_rate: sample_rate,

            player: PlayerDSP::new(sample_rate),

            master_output_buffers: Default::default(),
            touch_cue_output_buffers: Default::default(),

            eq_filters: (
                StereoBiquad::new(true),
                StereoBiquad::new(true),
                StereoBiquad::new(true),
            ),

            filter: StereoBiquad::new(true),
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
                *sample = self.master_output_buffers[buffer_index][sample_number];
                //.fade(channel_state.fade);
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
        self.eq_filters.0.reset();
        self.eq_filters.1.reset();
        self.eq_filters.2.reset();

        self.filter.reset();
    }
}

pub struct PlayerDSP {
    target_sample_rate: u32,

    loaded_track: Option<TrackAudioData>,

    master_stretch_input_buffers: [Vec<f32>; AUDIO_CHANNELS],
    master_stretch_output_buffers: [Vec<f32>; AUDIO_CHANNELS],
    master_stretcher: SignalsmithStretch<f32>,

    touch_cue_stretch_input_buffers: [Vec<f32>; AUDIO_CHANNELS],
    touch_cue_stretch_output_buffers: [Vec<f32>; AUDIO_CHANNELS],
    touch_cue_stretcher: SignalsmithStretch<f32>,
}

impl PlayerDSP {
    pub fn new(sample_rate: u32) -> PlayerDSP {
        PlayerDSP {
            target_sample_rate: sample_rate,

            loaded_track: None,

            master_stretch_input_buffers: Default::default(),
            master_stretch_output_buffers: Default::default(),
            master_stretcher: SignalsmithStretch::<f32>::new(),

            touch_cue_stretch_input_buffers: Default::default(),
            touch_cue_stretch_output_buffers: Default::default(),
            touch_cue_stretcher: SignalsmithStretch::<f32>::new(),
        }
    }

    pub fn assign_track_data(&mut self, track_data: Option<Box<TrackAudioData>>) {
        self.reset();

        self.loaded_track = track_data.map(|track_data| *track_data);
    }

    // the definitions of these types make this safe and we need max speed
    #[allow(clippy::indexing_slicing)]
    pub fn generate_samples(
        &mut self,
        channel_state: &PlayerState,
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

        self.generate_master_samples(channel_state, master_output_buffers, number_samples);
        self.generate_touch_cue_samples(channel_state, touch_cue_output_buffers, number_samples);
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn load_track_data(&mut self, track_data: TrackAudioData) {
        self.loaded_track = None;

        self.reset();

        self.master_stretcher
            .preset_default(AUDIO_CHANNELS, track_data.sample_rate as f32, true);

        self.loaded_track = Some(track_data);
    }

    // so much audio conversion, i just dont wanna deal with this here
    // todo: i need to composite in jog speed and direction and account for
    // backwards playback
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        clippy::cast_possible_wrap,
        clippy::indexing_slicing
    )]
    fn generate_master_samples(
        &mut self,
        channel_state: &PlayerState,
        output_buffers: &mut [Vec<f32>; AUDIO_CHANNELS],
        number_samples: usize,
    ) {
        let Some(audio_data) = &self.loaded_track else {
            for output_buffer in output_buffers {
                for sample in output_buffer {
                    *sample = 0.0;
                }
            }

            return;
        };

        if let PlayState::Stop = channel_state.play_state {
            for output_buffer in output_buffers {
                for sample in output_buffer {
                    *sample = 0.0;
                }
            }

            return;
        }

        let resample_ratio = self.target_sample_rate as f32 / audio_data.sample_rate as f32;

        let output_samples_needed_from_source_track =
            (number_samples as f32 * resample_ratio).round() as usize;

        let input_samples_needed_from_source_track = output_samples_needed_from_source_track;
        /*(output_samples_needed_from_source_track
            as f32
            * (channel_state.tempo_percent + 1.0).max(0.0))
        .round() as usize;*/

        for master_stretch_input_buffer in &mut self.master_stretch_input_buffers {
            if master_stretch_input_buffer.len() < input_samples_needed_from_source_track {
                master_stretch_input_buffer.resize(input_samples_needed_from_source_track, 0.0);
            }
        }
        for master_stretch_output_buffer in &mut self.master_stretch_output_buffers {
            if master_stretch_output_buffer.len() < output_samples_needed_from_source_track {
                master_stretch_output_buffer.resize(output_samples_needed_from_source_track, 0.0);
            }
        }

        audio_data.read_samples(
            channel_state.time,
            input_samples_needed_from_source_track,
            &mut self.master_stretch_input_buffers,
        );

        // #[allow(clippy::cast_precision_loss)]
        // self.master_stretcher.set_transpose_factor(
        //     channel_state.keyshift,
        //     (8000 / audio_data.sample_rate) as f32,
        // );

        // self.master_stretcher.process(
        //     &self.master_stretch_input_buffers,
        //     input_samples_needed_from_source_track,
        //     &mut self.master_stretch_output_buffers,
        //     output_samples_needed_from_source_track,
        // );

        for (buffer_index, output_buffer) in output_buffers.iter_mut().enumerate() {
            for (output_sample_number, output_sample) in output_buffer.iter_mut().enumerate() {
                // todo: this
                // should either be bicubic or kaiser-sinc but i don't have time rn

                // let percent = (resample_ratio * output_sample_number as f32) % 1.0;
                // let source_sample_lower_index = ((resample_ratio * output_sample_number as
                // f32)     .floor() as usize)
                //     .clamp(0, self.master_stretch_input_buffers[buffer_index].len() - 1);
                // let source_sample_upper_index = ((resample_ratio * output_sample_number as
                // f32)     .ceil() as usize)
                //     .clamp(0, self.master_stretch_input_buffers[buffer_index].len() - 1);

                // *output_sample = (self.master_stretch_input_buffers[buffer_index]
                //     [source_sample_lower_index]
                //     * (1.0 - percent))
                //     + (self.master_stretch_input_buffers[buffer_index][source_sample_upper_index]
                //         * percent);

                *output_sample = self.master_stretch_input_buffers[buffer_index]
                    [output_sample_number
                        .clamp(0, self.master_stretch_input_buffers[buffer_index].len() - 1)];
            }
        }
    }

    // so much audio conversion, i just dont wanna deal with this here
    // todo: i need to composite in jog speed and direction and account for
    // backwards playback
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        clippy::cast_possible_wrap,
        clippy::indexing_slicing
    )]
    fn generate_touch_cue_samples(
        &mut self,
        channel_state: &PlayerState,
        output_buffers: &mut [Vec<f32>; AUDIO_CHANNELS],
        number_samples: usize,
    ) {
        let Some(touch_cue_time) = channel_state.touch_cue_time else {
            for output_buffer in output_buffers {
                for sample in output_buffer {
                    *sample = 0.0;
                }
            }

            return;
        };

        let Some(audio_data) = &self.loaded_track else {
            for output_buffer in output_buffers {
                for sample in output_buffer {
                    *sample = 0.0;
                }
            }

            return;
        };

        let resample_ratio = self.target_sample_rate as f32 / audio_data.sample_rate as f32;

        let output_samples_needed_from_source_track =
            (number_samples as f32 * resample_ratio).round() as usize;

        let input_samples_needed_from_source_track = (output_samples_needed_from_source_track
            as f32
            * (channel_state.tempo_percent + 1.0).max(0.0))
        .round() as usize;

        for touch_cue_stretch_input_buffer in &mut self.touch_cue_stretch_input_buffers {
            if touch_cue_stretch_input_buffer.len() < input_samples_needed_from_source_track {
                touch_cue_stretch_input_buffer.resize(input_samples_needed_from_source_track, 0.0);
            }
        }
        for touch_cue_stretch_output_buffer in &mut self.touch_cue_stretch_output_buffers {
            if touch_cue_stretch_output_buffer.len() < output_samples_needed_from_source_track {
                touch_cue_stretch_output_buffer
                    .resize(output_samples_needed_from_source_track, 0.0);
            }
        }

        audio_data.read_samples(
            touch_cue_time,
            input_samples_needed_from_source_track,
            &mut self.touch_cue_stretch_input_buffers,
        );

        for buffer in &mut self.touch_cue_stretch_output_buffers {
            if buffer.len() < output_samples_needed_from_source_track {
                buffer.resize(output_samples_needed_from_source_track, 0.0);
            }
        }

        #[allow(clippy::cast_precision_loss)]
        self.touch_cue_stretcher
            .set_transpose_semitones(channel_state.keyshift, (audio_data.sample_rate / 2) as f32);

        // self.touch_cue_stretcher.process(
        //     &self.touch_cue_stretch_input_buffers,
        //     input_samples_needed_from_source_track,
        //     &mut self.touch_cue_stretch_output_buffers,
        //     output_samples_needed_from_source_track,
        // );

        for (buffer_index, output_buffer) in output_buffers.iter_mut().enumerate() {
            for (output_sample_number, output_sample) in output_buffer.iter_mut().enumerate() {
                // todo: this
                // should either be bicubic or kaiser-sinc but i don't have time rn
                let percent = (resample_ratio * output_sample_number as f32) % 1.0;
                let source_sample_lower_index =
                    ((resample_ratio * output_sample_number as f32).floor() as usize).clamp(
                        0,
                        self.touch_cue_stretch_input_buffers[buffer_index].len() - 1,
                    );
                let source_sample_upper_index =
                    ((resample_ratio * output_sample_number as f32).ceil() as usize).clamp(
                        0,
                        self.touch_cue_stretch_input_buffers[buffer_index].len() - 1,
                    );

                *output_sample = (self.touch_cue_stretch_input_buffers[buffer_index]
                    [source_sample_lower_index]
                    * (1.0 - percent))
                    + (self.touch_cue_stretch_input_buffers[buffer_index]
                        [source_sample_upper_index]
                        * percent);
            }
        }
    }

    pub fn reset(&mut self) {
        self.master_stretcher.reset();

        self.touch_cue_stretcher.reset();
    }
}
