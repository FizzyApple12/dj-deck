use libdj::types::{
    deck::DeckState,
    library::{ArtworkID, Library, PlaylistTreeNodeID, TrackID},
};

#[derive(Debug, Clone)]
pub enum UIMessage {
    DeviceConnected(u32),
    DeviceDisconnected(u32),

    UpdateDeckState(DeckState),

    DeviceLibrary { device: u32, library: Library },
}

#[derive(Debug, Clone)]
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
