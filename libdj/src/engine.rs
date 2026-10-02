use std::{
    sync::{Arc, nonpoison::RwLock},
    time::Duration,
};

use libdsp::audio_loader::TrackAudioData;
use thiserror::Error;

use crate::{
    audio::{
        AudioPipelineCreationError, AudioPipelineRuntimeError, AudioPipelineRuntimeStatus,
        AudioSystem, AudioSystemChannels, FindDeviceError,
    },
    types::{
        bindings::{DeckControlEvent, UIControlEvent},
        deck::DeckState,
        playback::DeckUpdate,
    },
};

#[derive(Error, Debug, Clone, Copy)]
pub enum DJEngineRuntimeError {
    #[error("Audio Host Disconnected")]
    AudioHostDisconnected,

    #[error("Audio Device Disconnected")]
    AudioDeviceDisconnected,

    #[error("Invalid Input")]
    InvalidInput,

    #[error("Insufficient Permissions")]
    InsufficientPermissions,

    #[error("Resources Exhausted")]
    ResourcesExhausted,

    #[error("Audio Stream Invalidated")]
    AudioStreamInvalidated,

    #[error("Audio Configuration Not Supported")]
    UnsupportedAudioConfiguration,

    #[error("Audio Operation Not Supported")]
    UnsupportedAudioOperation,

    #[error("A fatal audio error occurred")]
    Fatal,
}

#[derive(Clone, Copy)]
pub enum DJEngineRuntimeStatus {
    AudioInitialising,
    AudioInitError(AudioPipelineCreationError),
    Running,
    AudioRuntimeError(AudioPipelineRuntimeError),
}

pub struct DJEngine {
    runtime_state_receiver: tokio::sync::watch::Receiver<DJEngineRuntimeStatus>,
}

impl DJEngine {
    pub fn start(
        control_event_receiver: tokio::sync::mpsc::UnboundedReceiver<DeckControlEvent>,
        control_event_sender: tokio::sync::mpsc::UnboundedSender<UIControlEvent>,
        deck_update_receiver: tokio::sync::mpsc::UnboundedReceiver<DeckUpdate>,
        deck_state_sender: tokio::sync::watch::Sender<DeckState>,
        loaded_track_receiver: tokio::sync::mpsc::UnboundedReceiver<(
            usize,
            Option<Box<TrackAudioData>>,
        )>,
    ) -> Self {
        let (runtime_state_sender, runtime_state_receiver) =
            tokio::sync::watch::channel(DJEngineRuntimeStatus::AudioInitialising);

        let channels = Arc::new(RwLock::new(AudioSystemChannels {
            control_event_receiver,
            control_event_sender,
            deck_update_receiver,
            deck_state_sender,
            loaded_track_receiver,
        }));

        tokio::task::spawn(async move {
            'system_loop: loop {
                let _ = runtime_state_sender.send(DJEngineRuntimeStatus::AudioInitialising);

                let mut audio_system = AudioSystem::default();

                'pipeline_loop: loop {
                    let cloned_channels = channels.clone();

                    let device =
                        match audio_system.find_output_device_by_name_substring("DDJ-FLX10") {
                            Ok(device) => device,
                            Err(err) => {
                                match err {
                                    FindDeviceError::HostNotAvailable => {
                                        let _ = runtime_state_sender.send(
                                            DJEngineRuntimeStatus::AudioInitError(
                                                AudioPipelineCreationError::HostNotAvailable,
                                            ),
                                        );
                                    }
                                    FindDeviceError::Fatal => {
                                        let _ = runtime_state_sender.send(
                                            DJEngineRuntimeStatus::AudioInitError(
                                                AudioPipelineCreationError::Fatal,
                                            ),
                                        );
                                    }
                                }

                                drop(audio_system);

                                tokio::time::sleep(Duration::from_millis(1000)).await;

                                continue 'system_loop;
                            }
                        };

                    let mut pipeline_handle =
                        match audio_system.create_full_audio_pipeline(cloned_channels, device) {
                            Ok(pipeline_handle) => pipeline_handle,
                            Err(err) => {
                                let _ = runtime_state_sender
                                    .send(DJEngineRuntimeStatus::AudioInitError(err));

                                match err {
                                    AudioPipelineCreationError::HostNotAvailable
                                    | AudioPipelineCreationError::Fatal
                                    | AudioPipelineCreationError::InsufficientPermissions => {
                                        drop(audio_system);

                                        tokio::time::sleep(Duration::from_millis(1000)).await;

                                        continue 'system_loop;
                                    }
                                    _ => {
                                        tokio::time::sleep(Duration::from_millis(500)).await;

                                        continue 'pipeline_loop;
                                    }
                                }
                            }
                        };

                    let _ = runtime_state_sender.send(DJEngineRuntimeStatus::Running);

                    'status_loop: loop {
                        let AudioPipelineRuntimeStatus::Err(err) =
                            pipeline_handle.wait_for_new_status().await
                        else {
                            continue 'status_loop;
                        };

                        let _ = runtime_state_sender
                            .send(DJEngineRuntimeStatus::AudioRuntimeError(err));

                        match err {
                            AudioPipelineRuntimeError::HostDisconnected
                            | AudioPipelineRuntimeError::InsufficientPermissions
                            | AudioPipelineRuntimeError::ResourcesExhausted
                            | AudioPipelineRuntimeError::Fatal => {
                                drop(audio_system);

                                tokio::time::sleep(Duration::from_millis(1000)).await;

                                continue 'system_loop;
                            }
                            _ => {
                                tokio::time::sleep(Duration::from_millis(500)).await;

                                continue 'pipeline_loop;
                            }
                        }
                    }
                }
            }
        });

        DJEngine {
            runtime_state_receiver,
        }
    }

    pub async fn wait_for_new_status(&mut self) -> DJEngineRuntimeStatus {
        let _ = self.runtime_state_receiver.changed().await;

        *self.runtime_state_receiver.borrow_and_update()
    }
}
