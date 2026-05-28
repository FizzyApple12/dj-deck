use std::{collections::BTreeMap, path::PathBuf};

use rkyv::{Archive, Deserialize, Serialize};

use crate::PathBufAsString;

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize)]
pub enum OriginDatabase {
    Rekordbox,
}

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

    pub origin: OriginDatabase,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct Artist {
    pub id: ArtistID,
    pub name: String,

    pub origin: OriginDatabase,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct Artwork {
    pub id: ArtworkID,

    #[rkyv(with = PathBufAsString)]
    pub path: PathBuf,

    pub origin: OriginDatabase,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct Genre {
    pub id: GenreID,
    pub name: String,

    pub origin: OriginDatabase,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct Key {
    pub id: KeyID,
    pub name: String,

    pub origin: OriginDatabase,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct Label {
    pub id: LabelID,
    pub name: String,

    pub origin: OriginDatabase,
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

    pub origin: OriginDatabase,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct PlaylistFolder {
    pub id: PlaylistTreeNodeID,
    pub name: String,
    pub children: Vec<PlaylistTreeNodeID>,

    pub origin: OriginDatabase,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct Track {
    pub id: TrackID,

    pub origin: OriginDatabase,

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

    #[rkyv(with = PathBufAsString)]
    pub analysis_path: PathBuf,
}
