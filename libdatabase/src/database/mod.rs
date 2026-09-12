pub mod rekordbox;

use std::{
    collections::BTreeMap,
    error::Error,
    fs::{self},
    path::Path,
};

use libdj::types::{
    analysis::{PreviewWaveformColumn, TrackAnalysis, WaveformColumn, WaveformType},
    library::{Library, OriginDatabase, TrackID},
};
use log::{debug, info, warn};
use thiserror::Error;

use crate::database::rekordbox::RekordboxDatabase;

#[derive(Debug)]
pub struct Database {
    pub library: Library,

    rekordbox: Option<RekordboxDatabase>,
}

#[derive(Error, Debug)]
pub enum OpenDatabaseError {
    #[error("Path Found")]
    PathNotFound,
}

#[derive(Error, Debug)]
pub enum LoadAnalysisError {
    #[error("Track Not Found")]
    TrackNotFound,

    #[error("Track From Invalid Origin")]
    InvalidOrigin,

    #[error("Load Error: {0}")]
    LoadError(Box<dyn Error>),
}

#[derive(Error, Debug)]
pub enum LoadWaveformError {
    #[error("Track Not Found")]
    TrackNotFound,

    #[error("Track From Invalid Origin")]
    InvalidOrigin,

    #[error("Load Error: {0}")]
    LoadError(Box<dyn Error>),
}

impl Database {
    #[allow(clippy::cast_precision_loss)]
    pub fn open(device_root: &Path) -> Result<Database, OpenDatabaseError> {
        if match fs::exists(device_root) {
            Ok(exists) => !exists,
            Err(_) => true,
        } {
            warn!(target: "libdatabase::database", "Path {} not found", device_root.display());

            return Err(OpenDatabaseError::PathNotFound);
        }

        info!(target: "libdatabase::database", "Opening database on {}", device_root.display());

        let mut library = Library {
            albums: BTreeMap::new(),
            artists: BTreeMap::new(),
            artworks: BTreeMap::new(),
            genres: BTreeMap::new(),
            labels: BTreeMap::new(),
            keys: BTreeMap::new(),

            tracks: BTreeMap::new(),

            playlist_tree: BTreeMap::new(),
        };

        let rekordbox = if let Ok(mut rekordbox) = RekordboxDatabase::open(device_root) {
            if let Err(error) = rekordbox.load_base(&mut library) {
                warn!(target: "libdatabase::database", "Failed to load rekordbox database on device: {error}");

                None
            } else {
                Some(rekordbox)
            }
        } else {
            None
        };

        Ok(Database { library, rekordbox })
    }

    pub fn load_analysis(
        self: &Database,
        track_id: TrackID,
    ) -> Result<TrackAnalysis, LoadAnalysisError> {
        let Some(track) = self.library.tracks.get(&track_id) else {
            warn!(target: "libdatabase::database", "Analysis for Track {track_id} not found");

            return Err(LoadAnalysisError::TrackNotFound);
        };

        info!(target: "libdatabase::database", "Loading analysis for Track {track_id}");

        match track.origin {
            OriginDatabase::Rekordbox => {
                if let Some(ref rekordbox) = self.rekordbox {
                    debug!(target: "libdatabase::database", "Pulling from rekordbox");

                    rekordbox
                        .load_analysis(track)
                        .map_err(|err| LoadAnalysisError::LoadError(err.into()))
                } else {
                    Err(LoadAnalysisError::InvalidOrigin)
                }
            }
        }
    }

    pub fn load_preview_waveform(
        self: &Database,
        track_id: TrackID,
        waveform_type: WaveformType,
    ) -> Result<Vec<PreviewWaveformColumn>, LoadAnalysisError> {
        let Some(track) = self.library.tracks.get(&track_id) else {
            warn!(target: "libdatabase::database", "Preview waveform for Track {track_id} not found");

            return Err(LoadAnalysisError::TrackNotFound);
        };

        info!(target: "libdatabase::database", "Loading preview waveform for Track {track_id}");

        match track.origin {
            OriginDatabase::Rekordbox => {
                if let Some(ref rekordbox) = self.rekordbox {
                    debug!(target: "libdatabase::database", "Pulling from rekordbox");

                    rekordbox
                        .load_preview_waveform(track, waveform_type)
                        .map_err(|err| LoadAnalysisError::LoadError(err.into()))
                } else {
                    Err(LoadAnalysisError::InvalidOrigin)
                }
            }
        }
    }

    pub fn load_waveform(
        self: &Database,
        track_id: TrackID,
        waveform_type: WaveformType,
    ) -> Result<Vec<WaveformColumn>, LoadAnalysisError> {
        let Some(track) = self.library.tracks.get(&track_id) else {
            warn!(target: "libdatabase::database", "Waveform for Track {track_id} not found");

            return Err(LoadAnalysisError::TrackNotFound);
        };

        info!(target: "libdatabase::database", "Loading waveform for Track {track_id}");

        match track.origin {
            OriginDatabase::Rekordbox => {
                if let Some(ref rekordbox) = self.rekordbox {
                    debug!(target: "libdatabase::database", "Pulling from rekordbox");

                    rekordbox
                        .load_waveform(track, waveform_type)
                        .map_err(|err| LoadAnalysisError::LoadError(err.into()))
                } else {
                    Err(LoadAnalysisError::InvalidOrigin)
                }
            }
        }
    }

    pub fn sync(self: &mut Database) {
        info!(target: "libdatabase::database", "Syncing database to disk");

        if let Some(ref mut rekordbox) = self.rekordbox {
            debug!(target: "libdatabase::database", "Syncing rekordbox");

            rekordbox.sync();
        }
    }

    pub fn close(mut self: Database) {
        info!(target: "libdatabase::database", "Closing database");

        self.sync();

        if let Some(rekordbox) = self.rekordbox.take() {
            debug!(target: "libdatabase::database", "Closing rekordbox");

            rekordbox.close();
        }
    }
}

pub fn relativeify_path_string(mut path: String) -> String {
    if path.starts_with('/') {
        path.replace_first("/", "");
    }

    path
}
