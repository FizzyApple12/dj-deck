use std::{collections::HashMap, path::PathBuf};

pub type ArtistID = u32;
pub type ArtworkID = u32;
pub type LabelID = u32;
pub type AlbumID = u32;
pub type TrackID = u32;
pub type GenreID = u32;
pub type PlaylistTreeNodeID = u32;

#[derive(Debug, Clone)]
pub struct Library {
    pub albums: HashMap<AlbumID, Album>,
    pub artists: HashMap<ArtistID, Artist>,
    pub artworks: HashMap<ArtworkID, Artwork>,
    pub genres: HashMap<GenreID, Genre>,
    pub labels: HashMap<LabelID, Label>,

    pub tracks: HashMap<TrackID, Track>,

    pub playlist_tree: HashMap<PlaylistTreeNodeID, PlaylistTreeNode>,
}

#[derive(Debug, Clone)]
pub struct Album {
    pub id: AlbumID,
    pub artist_id: ArtistID,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Artist {
    pub id: ArtistID,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Artwork {
    pub id: ArtworkID,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Genre {
    pub id: GenreID,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Label {
    pub id: LabelID,
    pub name: String,
}

#[derive(Debug, Clone, Copy)]
pub enum ColorIndex {
    None,
    Pink,
    Red,
    Orange,
    Yellow,
    Green,
    Aqua,
    Blue,
    Purple,
}

#[derive(Debug, Clone)]
pub enum PlaylistTreeNode {
    Playlist(Playlist),
    PlaylistFolder(PlaylistFolder),
}

#[derive(Debug, Clone)]
pub struct Playlist {
    pub id: PlaylistTreeNodeID,
    pub name: String,
    pub tracks: Vec<TrackID>,
}

#[derive(Debug, Clone)]
pub struct PlaylistFolder {
    pub id: PlaylistTreeNodeID,
    pub name: String,
    pub children: Vec<PlaylistTreeNodeID>,
}

#[derive(Debug, Clone)]
pub struct Track {
    pub id: TrackID,

    pub title: String,

    pub tempo: f32,
    pub duration: f32,

    pub composer_id: ArtistID,
    pub artist_id: ArtistID,
    pub original_artist_id: ArtistID,
    pub remixer_id: ArtistID,
    pub label_id: LabelID,
    pub album_id: AlbumID,
    pub genre_id: GenreID,
    pub artwork_id: ArtworkID,

    pub audio_path: PathBuf,

    pub beat_grid: Vec<Beat>,
    pub hot_cues: Vec<HotCue>,
    pub memory_cues: Vec<MemoryCue>,
    pub tiny_preview_waveform: Vec<TinyPreviewWaveformColumn>,
    pub preview_waveform: Vec<PreviewWaveformColumn>,
    pub detail_waveform: Vec<WaveformColumn>,
}

#[derive(Debug, Clone, Copy)]
pub struct Beat {
    pub beat_number: u32,
    pub tempo: u32,
    pub time: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum CueType {
    Point,
    Loop(u32),
}

#[derive(Debug, Clone)]
pub struct MemoryCue {
    pub time: u32,
    pub comment: String,
}

#[derive(Debug, Clone)]
pub struct HotCue {
    pub cue_number: u32,
    pub cue_type: CueType,
    pub time: u32,
    pub comment: String,
    pub color_index: u8,
    pub color_rgb: (u8, u8, u8),
}

#[derive(Debug, Clone, Copy)]
pub struct TinyPreviewWaveformColumn {
    pub height: u8,
    pub color_rgb: (u8, u8, u8),
}

#[derive(Debug, Clone, Copy)]
pub struct PreviewWaveformColumn {
    pub height: u8,
    pub color_rgb: (u8, u8, u8),
}

#[derive(Debug, Clone, Copy)]
pub struct WaveformColumn {
    pub height: u8,
    pub color_rgb: (u8, u8, u8),
}
