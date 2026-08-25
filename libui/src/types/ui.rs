use libdj::types::{
    analysis::{PreviewWaveformColumn, TrackAnalysis, WaveformColumn},
    deck::DeckState,
    library::{Library, TrackID},
};
use libdsp::timecode::Timecode;
use rkyv::{Archive, Deserialize, Serialize};

pub const SOCKET_NAME: &str = "/tmp/lib_godot.sock";

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub enum UIMenu {
    None,
    Devices,
    Tracks,
    Playlists,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub enum UIMessage {
    DeviceConnected(u32, String),
    DeviceDisconnected(u32),

    UpdateDeckState(DeckState),

    DeviceLibrary {
        device: u32,
        library: Library,
    },

    TrackAnalysis {
        player: usize,
        analysis: TrackAnalysis,
    },

    Waveform {
        player: usize,
        waveform: Vec<WaveformColumn>,
    },
    PreviewWaveform {
        player: usize,
        waveform: Vec<PreviewWaveformColumn>,
    },

    EncoderUp,
    EncoderDown,
    EncoderSelect,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub enum UIEvent {
    LoadTrack {
        device: u32,
        id: TrackID,
        player: usize,
    },
    Eject(usize),

    GetLibrary(u32),
    EjectDevice(u32),

    GetWaveform {
        device: u32,
        id: TrackID,
        player: usize,
    },
    GetPreviewWaveform {
        device: u32,
        id: TrackID,
        player: usize,
    },
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub enum InternalUIEvent {
    LoadTrack {
        device: u32,
        id: TrackID,
        player: usize,
    },
    Eject(usize),

    GetLibrary(u32),
    EjectDevice(u32),

    GetWaveform {
        device: u32,
        id: TrackID,
        player: usize,
    },
    GetPreviewWaveform {
        device: u32,
        id: TrackID,
        player: usize,
    },

    TouchCue {
        cue_time: Option<Timecode>,
        player: usize,
    },
    BeatJump {
        beats: f32,
        player: usize,
    },
    SetBeatLoop {
        beats: f32,
        player: usize,
    },
    DoubleBeatLoop(usize),
    HalveBeatLoop(usize),
    SetKeyShift {
        player: usize,
        semitones: f32,
    },
}
