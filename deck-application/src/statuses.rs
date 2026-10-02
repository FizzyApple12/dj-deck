use std::fmt::Display;

use libdj::{
    audio::{AudioPipelineCreationError, AudioPipelineRuntimeError},
    engine::DJEngineRuntimeStatus,
};

pub enum DJEngineStatus {
    AudioInitialising,

    Running,

    AudioHostNotAvailableDuringInit,
    NoAvailableAudioOutputDevicesDuringInit,
    AudioDeviceNotAvailableDuringInit,
    AudioDeviceConfigsNotAttestableDuringInit,
    AudioDeviceOutputNotSupportedDuringInit,
    NoAvailableAudioOutputConfigsDuringInit,
    AudioDeviceBusyDuringInit,
    InsufficientPermissionsDuringInit,
    UnsupportedAudioConfigurationDuringInit,
    InvalidAudioConfigurationDuringInit,
    AudioStreamInvalidatedDuringInit,

    AudioHostDisconnectedDuringRuntime,
    AudioDeviceDisconnectedDuringRuntime,
    InvalidAudioInputDuringRuntime,
    InsufficientPermissionsDuringRuntime,
    ResourcesExhaustedDuringRuntime,
    AudioStreamInvalidatedDuringRuntime,
    UnsupportedAudioConfigurationDuringRuntime,
    UnsupportedAudioOperationDuringRuntime,

    FatalAudioError,
}

impl From<DJEngineRuntimeStatus> for DJEngineStatus {
    fn from(value: DJEngineRuntimeStatus) -> Self {
        match value {
            DJEngineRuntimeStatus::AudioInitialising => DJEngineStatus::AudioInitialising,
            DJEngineRuntimeStatus::AudioInitError(audio_pipeline_creation_error) => {
                match audio_pipeline_creation_error {
                    AudioPipelineCreationError::HostNotAvailable => {
                        DJEngineStatus::AudioHostNotAvailableDuringInit
                    }
                    AudioPipelineCreationError::NoAvailableOutputDevices => {
                        DJEngineStatus::NoAvailableAudioOutputDevicesDuringInit
                    }
                    AudioPipelineCreationError::DeviceNotAvailable => {
                        DJEngineStatus::AudioDeviceNotAvailableDuringInit
                    }
                    AudioPipelineCreationError::DeviceConfigsNotAttestable => {
                        DJEngineStatus::AudioDeviceConfigsNotAttestableDuringInit
                    }
                    AudioPipelineCreationError::DeviceOutputNotSupported => {
                        DJEngineStatus::AudioDeviceOutputNotSupportedDuringInit
                    }
                    AudioPipelineCreationError::NoAvailableOutputConfigs => {
                        DJEngineStatus::NoAvailableAudioOutputConfigsDuringInit
                    }
                    AudioPipelineCreationError::DeviceBusy => {
                        DJEngineStatus::AudioDeviceBusyDuringInit
                    }
                    AudioPipelineCreationError::InsufficientPermissions => {
                        DJEngineStatus::InsufficientPermissionsDuringInit
                    }
                    AudioPipelineCreationError::UnsupportedConfiguration => {
                        DJEngineStatus::UnsupportedAudioConfigurationDuringInit
                    }
                    AudioPipelineCreationError::InvalidConfiguration => {
                        DJEngineStatus::InvalidAudioConfigurationDuringInit
                    }
                    AudioPipelineCreationError::StreamInvalidated => {
                        DJEngineStatus::AudioStreamInvalidatedDuringInit
                    }
                    AudioPipelineCreationError::Fatal => DJEngineStatus::FatalAudioError,
                }
            }
            DJEngineRuntimeStatus::Running => DJEngineStatus::Running,
            DJEngineRuntimeStatus::AudioRuntimeError(audio_pipeline_runtime_error) => {
                match audio_pipeline_runtime_error {
                    AudioPipelineRuntimeError::HostDisconnected => {
                        DJEngineStatus::AudioHostDisconnectedDuringRuntime
                    }
                    AudioPipelineRuntimeError::DeviceDisconnected => {
                        DJEngineStatus::AudioDeviceDisconnectedDuringRuntime
                    }
                    AudioPipelineRuntimeError::InvalidInput => {
                        DJEngineStatus::InvalidAudioInputDuringRuntime
                    }
                    AudioPipelineRuntimeError::InsufficientPermissions => {
                        DJEngineStatus::InsufficientPermissionsDuringRuntime
                    }
                    AudioPipelineRuntimeError::ResourcesExhausted => {
                        DJEngineStatus::ResourcesExhaustedDuringRuntime
                    }
                    AudioPipelineRuntimeError::StreamInvalidated => {
                        DJEngineStatus::AudioStreamInvalidatedDuringRuntime
                    }
                    AudioPipelineRuntimeError::UnsupportedConfiguration => {
                        DJEngineStatus::UnsupportedAudioConfigurationDuringRuntime
                    }
                    AudioPipelineRuntimeError::UnsupportedOperation => {
                        DJEngineStatus::UnsupportedAudioOperationDuringRuntime
                    }
                    AudioPipelineRuntimeError::Fatal => DJEngineStatus::FatalAudioError,
                }
            }
        }
    }
}

impl Display for DJEngineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DJEngineStatus::AudioInitialising => write!(f, "Audio Initialising"),
            DJEngineStatus::Running => write!(f, "Running"),

            DJEngineStatus::AudioHostNotAvailableDuringInit => {
                write!(f, "EA02: No Available Audio Host During Init")
            }
            DJEngineStatus::NoAvailableAudioOutputDevicesDuringInit => {
                write!(f, "EA03: No Available Audio Output Devices During Init")
            }
            DJEngineStatus::AudioDeviceNotAvailableDuringInit => {
                write!(f, "EA04: Audio Device Not Available During Init")
            }
            DJEngineStatus::AudioDeviceConfigsNotAttestableDuringInit => {
                write!(f, "EA05: Audio Device Configs Not Attestable During Init")
            }
            DJEngineStatus::AudioDeviceOutputNotSupportedDuringInit => {
                write!(f, "EA06: Audio Device Output Not Supported During Init")
            }
            DJEngineStatus::NoAvailableAudioOutputConfigsDuringInit => {
                write!(f, "EA07: No Available Audio Output Configs During Init")
            }
            DJEngineStatus::AudioDeviceBusyDuringInit => {
                write!(f, "EA08: Audio Device Busy During Init")
            }
            DJEngineStatus::InsufficientPermissionsDuringInit => {
                write!(f, "EA09: Insufficient Permissions During Init")
            }
            DJEngineStatus::UnsupportedAudioConfigurationDuringInit => {
                write!(f, "EA10: Unsupported Audio Configuration During Init")
            }
            DJEngineStatus::InvalidAudioConfigurationDuringInit => {
                write!(f, "EA11: Invalid Audio Configuration During Init")
            }
            DJEngineStatus::AudioStreamInvalidatedDuringInit => {
                write!(f, "EA12: Audio Stream InvalidatedDuring Init")
            }

            DJEngineStatus::AudioHostDisconnectedDuringRuntime => {
                write!(f, "EA13: Audio Host Disconnected During Runtime")
            }
            DJEngineStatus::AudioDeviceDisconnectedDuringRuntime => {
                write!(f, "EA14: Audio Device Disconnected During Runtime")
            }
            DJEngineStatus::InvalidAudioInputDuringRuntime => {
                write!(f, "EA15: Invalid Audio Input During Runtime")
            }
            DJEngineStatus::InsufficientPermissionsDuringRuntime => {
                write!(f, "EA16: Insufficient Permissions During Runtime")
            }
            DJEngineStatus::ResourcesExhaustedDuringRuntime => {
                write!(f, "EA17: Resources Exhausted During Runtime")
            }
            DJEngineStatus::AudioStreamInvalidatedDuringRuntime => {
                write!(f, "EA18: Audio Stream Invalidated During Runtime")
            }
            DJEngineStatus::UnsupportedAudioConfigurationDuringRuntime => {
                write!(f, "EA19: Unsupported Audio Configuration During Runtime")
            }
            DJEngineStatus::UnsupportedAudioOperationDuringRuntime => {
                write!(f, "EA20: Unsupported Audio Operation During Runtime")
            }

            DJEngineStatus::FatalAudioError => write!(f, "EA21: Fatal Audio Error"),
        }
    }
}
