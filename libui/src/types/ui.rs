use libdj::types::{
    deck::DeckState,
    library::{Library, PlaylistTreeNodeID, TrackID},
};
use rkyv::{Archive, Deserialize, Serialize};

pub const SOCKET_NAME: &str = "/tmp/lib_godot.sock";

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub enum UIMessage {
    DeviceConnected(u32),
    DeviceDisconnected(u32),

    UpdateDeckState(DeckState),

    DeviceLibrary { device: u32, library: Library },
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub enum UIEvent {
    LoadTrack {
        device: u32,
        id: TrackID,
        playlist: Option<PlaylistTreeNodeID>,
        deck: usize,
    },
    Eject(usize),

    GetLibrary(u32),

    NeedleSearch {
        needle_time: Option<f32>,
        deck: usize,
    },
    BeatJump {
        beats: f32,
        deck: usize,
    },
    SetBeatLoop {
        beats: f32,
        deck: usize,
    },
    DoubleBeatLoop(usize),
    HalveBeatLoop(usize),
    SetKeyShift {
        deck: usize,
        semitones: i8,
    },
}
