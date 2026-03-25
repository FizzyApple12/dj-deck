use std::{fmt, path::PathBuf};

pub type ArtistID = u32;
pub type ArtworkID = u32;
pub type LabelID = u32;
pub type AlbumID = u32;
pub type TrackID = u32;
pub type GenreID = u32;
pub type PlaylistTreeNodeID = u32;

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

#[derive(Clone)]
pub struct Track {
    pub id: TrackID,

    pub title: String,

    pub tempo: u32,
    pub duration: u16,

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

impl fmt::Debug for Track {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        #[derive(Debug)]
        #[allow(dead_code)]
        struct Track<'a> {
            id: &'a TrackID,
            title: &'a String,
            tempo: &'a u32,
            duration: &'a u16,
            composer_id: &'a ArtistID,
            artist_id: &'a ArtistID,
            original_artist_id: &'a ArtistID,
            remixer_id: &'a ArtistID,
            label_id: &'a LabelID,
            album_id: &'a AlbumID,
            genre_id: &'a GenreID,
            artwork_id: &'a ArtworkID,
            audio_path: &'a PathBuf,
        }

        let Self {
            id,
            title,
            tempo,
            duration,
            composer_id,
            artist_id,
            original_artist_id,
            remixer_id,
            label_id,
            album_id,
            genre_id,
            artwork_id,
            audio_path,
            ..
        } = self;

        fmt::Debug::fmt(
            &Track {
                id,
                title,
                tempo,
                duration,
                composer_id,
                artist_id,
                original_artist_id,
                remixer_id,
                label_id,
                album_id,
                genre_id,
                artwork_id,
                audio_path,
            },
            f,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Beat {
    pub beat_number: u16,
    pub tempo: u16,
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
