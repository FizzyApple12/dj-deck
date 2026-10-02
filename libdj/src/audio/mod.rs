pub mod dsp_pipeline;

use std::{
    cell::RefCell,
    f32,
    sync::{Arc, nonpoison::RwLock},
};

use cpal::{
    Device, ErrorKind, Host, Stream, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use libdsp::audio_loader::TrackAudioData;
use thiserror::Error;
use timecode::{Duration, Timecode};

use crate::{
    AUDIO_CHANNELS,
    audio::dsp_pipeline::deck::DeckDSP,
    types::{
        bindings::{DeckControlEvent, UIControlEvent},
        deck::DeckState,
        playback::DeckUpdate,
    },
};

pub struct AudioSystemChannels {
    pub control_event_receiver: tokio::sync::mpsc::UnboundedReceiver<DeckControlEvent>,
    pub control_event_sender: tokio::sync::mpsc::UnboundedSender<UIControlEvent>,
    pub deck_update_receiver: tokio::sync::mpsc::UnboundedReceiver<DeckUpdate>,
    pub deck_state_sender: tokio::sync::watch::Sender<DeckState>,
    pub loaded_track_receiver:
        tokio::sync::mpsc::UnboundedReceiver<(usize, Option<Box<TrackAudioData>>)>,
}

struct FullAudioPipelineStreamData {
    stream_config: StreamConfig,

    deck_dsp: Option<DeckDSP>,

    channels: Arc<RwLock<AudioSystemChannels>>,

    deck_state: DeckState,

    last_process_timecode: Timecode,

    master_output_buffers: [Vec<f32>; AUDIO_CHANNELS],
    cue_output_buffers: [Vec<f32>; AUDIO_CHANNELS],
}

#[derive(Error, Debug, Clone, Copy)]
pub enum FindDeviceError {
    #[error("Host not available")]
    HostNotAvailable,

    #[error("A fatal audio error occurred")]
    Fatal,
}

#[derive(Error, Debug, Clone, Copy)]
pub enum AudioPipelineCreationError {
    #[error("Host Not Available")]
    HostNotAvailable,

    #[error("No Available Output Devices")]
    NoAvailableOutputDevices,

    #[error("Output Device Not Available")]
    DeviceNotAvailable,

    #[error("Device Configs Not Attestable")]
    DeviceConfigsNotAttestable,

    #[error("Output Not Supported")]
    DeviceOutputNotSupported,

    #[error("No Available Output Devices")]
    NoAvailableOutputConfigs,

    #[error("DeviceBusy")]
    DeviceBusy,

    #[error("Insufficient Permissions")]
    InsufficientPermissions,

    #[error("Configuration Not Supported")]
    UnsupportedConfiguration,

    #[error("Invalid Configuration")]
    InvalidConfiguration,

    #[error("Stream Invalidated")]
    StreamInvalidated,

    #[error("A fatal audio error occurred")]
    Fatal,
}

#[derive(Error, Debug, Clone, Copy)]
pub enum AudioPipelineRuntimeError {
    #[error("Host Disconnected")]
    HostDisconnected,

    #[error("Device Disconnected")]
    DeviceDisconnected,

    #[error("Invalid Input")]
    InvalidInput,

    #[error("Insufficient Permissions")]
    InsufficientPermissions,

    #[error("Resources Exhausted")]
    ResourcesExhausted,

    #[error("Stream Invalidated")]
    StreamInvalidated,

    #[error("Configuration Not Supported")]
    UnsupportedConfiguration,

    #[error("Operation Not Supported")]
    UnsupportedOperation,

    #[error("A fatal audio error occurred")]
    Fatal,
}

#[derive(Clone, Copy)]
pub enum AudioPipelineRuntimeStatus {
    Ok,
    Err(AudioPipelineRuntimeError),
}

pub struct AudioSystem {
    host: Host,
}

pub struct AudioPipelineHandle {
    stream: Stream,

    runtime_state_receiver: tokio::sync::watch::Receiver<AudioPipelineRuntimeStatus>,
}

impl AudioPipelineHandle {
    pub async fn wait_for_new_status(&mut self) -> AudioPipelineRuntimeStatus {
        let _ = self.runtime_state_receiver.changed().await;

        *self.runtime_state_receiver.borrow_and_update()
    }
}

impl AudioSystem {
    pub fn find_output_device_by_name_substring(
        &self,
        name_substring: &str,
    ) -> Result<Option<Device>, FindDeviceError> {
        #[allow(deprecated)]
        let mut valid_devices = self
            .host
            .output_devices()
            .map_err(|err| match err.kind() {
                ErrorKind::HostUnavailable => FindDeviceError::HostNotAvailable,
                ErrorKind::BackendError => FindDeviceError::Fatal,
                _ => unreachable!(),
            })?
            .filter(|device| {
                device
                    .description()
                    .map_or(String::new(), |description| description.name().to_string())
                    .contains(name_substring)
            });

        Ok(valid_devices.next().or(None))
    }

    pub fn create_full_audio_pipeline(
        &mut self,
        channels: Arc<RwLock<AudioSystemChannels>>,
        device: Option<Device>,
    ) -> Result<AudioPipelineHandle, AudioPipelineCreationError> {
        let device = device
            .or(self.host.default_output_device())
            .ok_or(AudioPipelineCreationError::NoAvailableOutputDevices)?;

        let mut supported_configs_range =
            device
                .supported_output_configs()
                .map_err(|err| match err.kind() {
                    ErrorKind::DeviceNotAvailable => AudioPipelineCreationError::DeviceNotAvailable,
                    ErrorKind::UnsupportedConfig => {
                        AudioPipelineCreationError::DeviceConfigsNotAttestable
                    }
                    ErrorKind::UnsupportedOperation => {
                        AudioPipelineCreationError::DeviceOutputNotSupported
                    }
                    _ => unreachable!(),
                })?;
        let supported_config = supported_configs_range
            .next()
            .ok_or(AudioPipelineCreationError::NoAvailableOutputConfigs)?
            .with_max_sample_rate();

        let stream_config = supported_config.config();

        let mut deck_dsp = DeckDSP::new(stream_config.sample_rate);
        deck_dsp.reset();

        let audio_stream_data = RefCell::new(FullAudioPipelineStreamData {
            stream_config,

            deck_dsp: Some(deck_dsp),

            channels,

            deck_state: DeckState::default(),

            last_process_timecode: Timecode::zero(),

            master_output_buffers: Default::default(),
            cue_output_buffers: Default::default(),
        });

        let (runtime_state_sender, runtime_state_receiver) =
            tokio::sync::watch::channel(AudioPipelineRuntimeStatus::Ok);

        let stream = device
            .build_output_stream(
                stream_config,
                move |data: &mut [f32], _info: &cpal::OutputCallbackInfo| {
                    let mut audio_stream_data = audio_stream_data.borrow_mut();

                    let FullAudioPipelineStreamData {
                        stream_config,

                        deck_dsp,

                        channels,

                        deck_state,

                        last_process_timecode,

                        master_output_buffers,
                        cue_output_buffers,
                    } = &mut *audio_stream_data;

                    let AudioSystemChannels {
                        control_event_receiver,
                        control_event_sender,
                        deck_update_receiver,
                        deck_state_sender,
                        loaded_track_receiver,
                    } = &mut *channels.write();

                    let Some(deck_dsp) = deck_dsp else {
                        return;
                    };

                    let number_channels = stream_config.channels as usize;
                    let available_samples = data.len() / number_channels;
                    let sample_rate = stream_config.sample_rate;

                    while let Ok((channel_index, track_data)) = loaded_track_receiver.try_recv() {
                        deck_dsp.assign_track_data(channel_index, track_data);
                    }

                    #[allow(clippy::cast_possible_truncation, clippy::cast_lossless)]
                    let current_timecode = *last_process_timecode
                        + Duration::from_nanoseconds(
                            ((available_samples as i128 * 1_000_000_000) / sample_rate as i128)
                                as i64,
                        );

                    let update_results = deck_state.update(
                        control_event_receiver,
                        control_event_sender,
                        deck_update_receiver,
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
                move |err| match err.kind() {
                    ErrorKind::Xrun
                    | ErrorKind::RealtimeDenied
                    | ErrorKind::DeviceBusy
                    | ErrorKind::DeviceChanged => {}
                    ErrorKind::HostUnavailable => {
                        let _ = runtime_state_sender.send(AudioPipelineRuntimeStatus::Err(
                            AudioPipelineRuntimeError::HostDisconnected,
                        ));
                    }
                    ErrorKind::DeviceNotAvailable => {
                        let _ = runtime_state_sender.send(AudioPipelineRuntimeStatus::Err(
                            AudioPipelineRuntimeError::DeviceDisconnected,
                        ));
                    }
                    ErrorKind::InvalidInput => {
                        let _ = runtime_state_sender.send(AudioPipelineRuntimeStatus::Err(
                            AudioPipelineRuntimeError::InvalidInput,
                        ));
                    }
                    ErrorKind::PermissionDenied => {
                        let _ = runtime_state_sender.send(AudioPipelineRuntimeStatus::Err(
                            AudioPipelineRuntimeError::InsufficientPermissions,
                        ));
                    }
                    ErrorKind::ResourceExhausted => {
                        let _ = runtime_state_sender.send(AudioPipelineRuntimeStatus::Err(
                            AudioPipelineRuntimeError::ResourcesExhausted,
                        ));
                    }
                    ErrorKind::StreamInvalidated => {
                        let _ = runtime_state_sender.send(AudioPipelineRuntimeStatus::Err(
                            AudioPipelineRuntimeError::StreamInvalidated,
                        ));
                    }
                    ErrorKind::UnsupportedConfig => {
                        let _ = runtime_state_sender.send(AudioPipelineRuntimeStatus::Err(
                            AudioPipelineRuntimeError::UnsupportedConfiguration,
                        ));
                    }
                    ErrorKind::UnsupportedOperation => {
                        let _ = runtime_state_sender.send(AudioPipelineRuntimeStatus::Err(
                            AudioPipelineRuntimeError::UnsupportedOperation,
                        ));
                    }
                    ErrorKind::BackendError | ErrorKind::Other => {
                        let _ = runtime_state_sender.send(AudioPipelineRuntimeStatus::Err(
                            AudioPipelineRuntimeError::Fatal,
                        ));
                    }
                    _ => unreachable!(),
                },
                None,
            )
            .map_err(|err| match err.kind() {
                ErrorKind::UnsupportedConfig => {
                    AudioPipelineCreationError::UnsupportedConfiguration
                }
                ErrorKind::UnsupportedOperation => {
                    AudioPipelineCreationError::DeviceOutputNotSupported
                }
                ErrorKind::DeviceNotAvailable => AudioPipelineCreationError::DeviceNotAvailable,
                ErrorKind::DeviceBusy => AudioPipelineCreationError::DeviceBusy,
                ErrorKind::PermissionDenied => AudioPipelineCreationError::InsufficientPermissions,
                ErrorKind::InvalidInput => AudioPipelineCreationError::InvalidConfiguration,
                _ => unreachable!(),
            })?;

        stream.play().map_err(|err| match err.kind() {
            ErrorKind::DeviceNotAvailable => AudioPipelineCreationError::DeviceNotAvailable,
            ErrorKind::StreamInvalidated => AudioPipelineCreationError::DeviceConfigsNotAttestable,
            _ => unreachable!(),
        })?;

        Ok(AudioPipelineHandle {
            stream,
            runtime_state_receiver,
        })
    }

    pub fn destroy_full_audio_pipeline(handle: AudioPipelineHandle) {
        let _ = handle.stream.pause();

        drop(handle);
    }
}

impl Default for AudioSystem {
    fn default() -> Self {
        let host = cpal::default_host();

        AudioSystem { host }
    }
}

impl Drop for AudioSystem {
    fn drop(&mut self) {}
}
