pub mod audio_loader;
pub mod dsp_pipeline;

use std::{cell::RefCell, f32};

use cpal::{
    Device, Host, Stream, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use midir::{MidiInput, MidiInputConnection, MidiOutput};
use tokio::task::JoinHandle;

use crate::{
    AUDIO_CHANNELS,
    audio_system::{audio_loader::TrackAudioData, dsp_pipeline::deck::DeckDSP},
    types::{
        audio_system::{AudioSystemEvent, DeckUpdate},
        deck::DeckState,
        midi::MidiMessage,
        timecode::{Duration, Timecode},
    },
};

struct FullAudioPipelineStreamData {
    stream_config: StreamConfig,

    deck_dsp: Option<DeckDSP>,

    deck_state: DeckState,

    deck_update_receiver: tokio::sync::mpsc::UnboundedReceiver<DeckUpdate>,
    audio_system_event_sender: tokio::sync::mpsc::UnboundedSender<AudioSystemEvent>,
    deck_state_sender: tokio::sync::watch::Sender<DeckState>,

    last_process_timecode: Timecode,

    loaded_track_receiver:
        tokio::sync::mpsc::UnboundedReceiver<(usize, Option<Box<TrackAudioData>>)>,

    master_output_buffers: [Vec<f32>; AUDIO_CHANNELS],
    cue_output_buffers: [Vec<f32>; AUDIO_CHANNELS],
}

pub struct AudioManager {
    host: Host,

    audio_streams: Vec<Stream>,

    midi_inputs: Vec<MidiInputConnection<()>>,
    midi_outputs: Vec<JoinHandle<()>>,
}

impl AudioManager {
    pub fn new() -> Result<AudioManager, Box<dyn std::error::Error>> {
        let host = cpal::default_host();

        Ok(AudioManager {
            host,

            audio_streams: Vec::new(),

            midi_inputs: Vec::new(),
            midi_outputs: Vec::new(),
        })
    }

    pub fn start(&self) {
        for stream in &self.audio_streams {
            let _ = stream.play();
        }
    }

    pub fn find_output_device_by_name_substring(
        &self,
        name_substring: &str,
    ) -> Result<Option<Device>, Box<dyn std::error::Error>> {
        #[allow(deprecated)]
        let mut valid_devices = self.host.output_devices()?.filter(|device| {
            device
                .description()
                .map_or(String::new(), |description| description.name().to_string())
                .contains(name_substring)
        });

        Ok(valid_devices.next().or(None))
    }

    pub fn create_full_audio_pipeline(
        &mut self,
        deck_update_receiver: tokio::sync::mpsc::UnboundedReceiver<DeckUpdate>,
        audio_system_event_sender: tokio::sync::mpsc::UnboundedSender<AudioSystemEvent>,
        deck_state_sender: tokio::sync::watch::Sender<DeckState>,
        loaded_track_receiver: tokio::sync::mpsc::UnboundedReceiver<(
            usize,
            Option<Box<TrackAudioData>>,
        )>,
        device: Option<Device>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let device = device
            .or(self.host.default_output_device())
            .ok_or("No available output devices")?;

        let mut supported_configs_range = device
            .supported_output_configs()
            .expect("error while querying configs");
        let supported_config = supported_configs_range
            .next()
            .expect("no supported config?!")
            .with_max_sample_rate();

        let stream_config = supported_config.config();

        let mut deck_dsp = DeckDSP::new(stream_config.sample_rate);
        deck_dsp.reset();

        let audio_stream_data = RefCell::new(FullAudioPipelineStreamData {
            stream_config,

            deck_dsp: Some(deck_dsp),

            deck_state: DeckState::default(),

            deck_update_receiver,
            audio_system_event_sender,
            deck_state_sender,

            last_process_timecode: Timecode::zero(),

            loaded_track_receiver,

            master_output_buffers: Default::default(),
            cue_output_buffers: Default::default(),
        });

        let stream = device.build_output_stream(
            stream_config,
            move |data: &mut [f32], _info: &cpal::OutputCallbackInfo| {
                let mut audio_stream_data = audio_stream_data.borrow_mut();

                let FullAudioPipelineStreamData {
                    stream_config,

                    deck_dsp,

                    deck_state,

                    deck_update_receiver,
                    audio_system_event_sender,
                    deck_state_sender,

                    last_process_timecode,

                    loaded_track_receiver,

                    master_output_buffers,
                    cue_output_buffers,
                } = &mut *audio_stream_data;

                let Some(deck_dsp) = deck_dsp else {
                    return;
                };

                let number_channels = stream_config.channels as usize;
                let available_samples = data.len() / number_channels;
                let sample_rate = stream_config.sample_rate;

                while let Ok((channel_index, track_data)) = loaded_track_receiver.try_recv() {
                    deck_dsp.assign_track_data(channel_index, track_data);
                }

                // #[allow(clippy::cast_possible_truncation)]
                // let current_timecode =
                //     Timecode::from_nanoseconds(info.timestamp().callback.as_nanos() as i64);
                #[allow(clippy::cast_possible_truncation, clippy::cast_lossless)]
                let current_timecode = *last_process_timecode
                    + Duration::from_nanoseconds(
                        ((available_samples as i128 * 1_000_000_000) / sample_rate as i128) as i64,
                    );

                let update_results = deck_state.update(
                    deck_update_receiver,
                    audio_system_event_sender,
                    *last_process_timecode,
                    current_timecode,
                );

                *last_process_timecode = current_timecode;

                let _ = deck_state_sender.send_replace(deck_state.clone());

                deck_dsp.generate_samples(
                    deck_state,
                    &update_results,
                    master_output_buffers,
                    cue_output_buffers,
                    available_samples,
                );

                // we need max performance here
                #[allow(clippy::indexing_slicing)]
                for (sample_index, sample_buffer) in
                    data.chunks_mut(stream_config.channels as usize).enumerate()
                {
                    for (channel_index, sample) in sample_buffer.iter_mut().enumerate() {
                        *sample = match channel_index {
                            channel_number @ (0 | 1) => {
                                master_output_buffers[channel_number][sample_index]
                            }
                            channel_number @ (2 | 3) => {
                                cue_output_buffers[channel_number - 2][sample_index]
                            }
                            _ => 0.0,
                        };
                    }
                }
            },
            move |err| {
                println!("Audio Stream Error: {err}");
            },
            None,
        )?;

        self.audio_streams.push(stream);

        Ok(())
    }

    pub fn create_midi_input(
        &mut self,
    ) -> Result<tokio::sync::mpsc::UnboundedReceiver<MidiMessage>, Box<dyn std::error::Error>> {
        let (midi_sender, midi_receiver) = tokio::sync::mpsc::unbounded_channel::<MidiMessage>();

        let midi_in = MidiInput::new("dj-deck-midi-input")?;

        let ports = midi_in.ports();

        let Some(target_port) = ports.iter().find(|port| {
            midi_in
                .port_name(port)
                .is_ok_and(|name| name.contains("DDJ-FLX10"))
        }) else {
            return Err("No port
                found"
                .into());
        };

        let port_name = midi_in.port_name(target_port)?;

        let connection = midi_in.connect(
            target_port,
            &port_name,
            move |_, message, ()| {
                if let [byte_zero, byte_one, byte_two] = *message {
                    let _ = midi_sender.send([byte_zero, byte_one, byte_two]);
                }
            },
            (),
        )?;

        self.midi_inputs.push(connection);

        Ok(midi_receiver)
    }

    pub fn create_midi_output(
        &mut self,
    ) -> Result<tokio::sync::mpsc::UnboundedSender<MidiMessage>, Box<dyn std::error::Error>> {
        let (midi_sender, mut midi_receiver) =
            tokio::sync::mpsc::unbounded_channel::<MidiMessage>();

        let midi_out = MidiOutput::new("dj-deck-midi-output")?;

        let ports = midi_out.ports();

        let Some(target_port) = ports.iter().find(|port| {
            midi_out
                .port_name(port)
                .is_ok_and(|name| name.contains("DDJ-FLX10"))
        }) else {
            return Err("No port found".into());
        };

        let port_name = midi_out.port_name(target_port)?;

        let mut connection = midi_out.connect(target_port, &port_name)?;

        let connection = tokio::task::spawn(async move {
            while let Some(message) = &midi_receiver.recv().await {
                if let Err(e) = connection.send(message) {
                    println!("[midi-sender] Send error: {e}");
                }
            }
        });

        self.midi_outputs.push(connection);

        Ok(midi_sender)
    }
}

impl Drop for AudioManager {
    fn drop(&mut self) {
        for stream in self.audio_streams.drain(0..) {
            let _ = stream.pause();
        }

        for midi_input in self.midi_inputs.drain(0..) {
            midi_input.close();
        }

        for midi_output in self.midi_outputs.drain(0..) {
            midi_output.abort();
        }
    }
}
