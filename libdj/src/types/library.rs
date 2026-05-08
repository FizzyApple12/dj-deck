use std::{collections::BTreeMap, path::PathBuf};

use rkyv::{Archive, Deserialize, Serialize};

use crate::{PathBufAsString, types::timecode::Timecode};

pub type ArtistID = u32;
pub type ArtworkID = u32;
pub type LabelID = u32;
pub type AlbumID = u32;
pub type TrackID = u32;
pub type GenreID = u32;
pub type KeyID = u32;
pub type PlaylistTreeNodeID = u32;

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct Library {
    pub albums: BTreeMap<AlbumID, Album>,
    pub artists: BTreeMap<ArtistID, Artist>,
    pub artworks: BTreeMap<ArtworkID, Artwork>,
    pub genres: BTreeMap<GenreID, Genre>,
    pub labels: BTreeMap<LabelID, Label>,
    pub keys: BTreeMap<KeyID, Key>,

    pub tracks: BTreeMap<TrackID, Track>,

    pub playlist_tree: BTreeMap<PlaylistTreeNodeID, PlaylistTreeNode>,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct Album {
    pub id: AlbumID,
    pub artist_id: ArtistID,
    pub name: String,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct Artist {
    pub id: ArtistID,
    pub name: String,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct Artwork {
    pub id: ArtworkID,

    #[rkyv(with = PathBufAsString)]
    pub path: PathBuf,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct Genre {
    pub id: GenreID,
    pub name: String,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct Key {
    pub id: KeyID,
    pub name: String,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct Label {
    pub id: LabelID,
    pub name: String,
}

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub enum PlaylistTreeNode {
    Playlist(Playlist),
    PlaylistFolder(PlaylistFolder),
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct Playlist {
    pub id: PlaylistTreeNodeID,
    pub name: String,
    pub tracks: Vec<TrackID>,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct PlaylistFolder {
    pub id: PlaylistTreeNodeID,
    pub name: String,
    pub children: Vec<PlaylistTreeNodeID>,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct Track {
    pub id: TrackID,

    pub title: String,

    pub bpm: f32,
    pub duration: u32,

    pub composer_id: ArtistID,
    pub artist_id: ArtistID,
    pub original_artist_id: ArtistID,
    pub remixer_id: ArtistID,
    pub label_id: LabelID,
    pub album_id: AlbumID,
    pub genre_id: GenreID,
    pub artwork_id: ArtworkID,
    pub key_id: KeyID,

    #[rkyv(with = PathBufAsString)]
    pub audio_path: PathBuf,

    pub beat_grid: Vec<Beat>,
    pub hot_cues: Vec<HotCue>,
    pub memory_cues: Vec<MemoryCue>,
    pub tiny_preview_waveform: Vec<TinyPreviewWaveformColumn>,
    pub preview_waveform: Vec<PreviewWaveformColumn>,
    pub detail_waveform: Vec<WaveformColumn>,
}

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize)]
pub struct Beat {
    pub beat_number: u32,
    pub bpm: f32,
    pub time: Timecode,
}

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize)]
pub enum CueType {
    Point,
    Loop(Timecode),
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct MemoryCue {
    pub time: Timecode,
    pub comment: String,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct HotCue {
    pub cue_number: u32,
    pub cue_type: CueType,
    pub time: Timecode,
    pub comment: String,
    pub color_index: u8,
    pub color_rgb: (u8, u8, u8),
}

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize)]
pub struct TinyPreviewWaveformColumn {
    pub height: u8,
    pub color_rgb: (u8, u8, u8),
}

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize)]
pub struct PreviewWaveformColumn {
    pub height: u8,
    pub color_rgb: (u8, u8, u8),
}

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize)]
pub struct WaveformColumn {
    pub height: u8,
    pub color_rgb: (u8, u8, u8),
}
