use std::{
    fs::File,
    path::{Path, PathBuf},
};

use binrw::BinRead;
use libdj::types::{
    analysis::{
        Beat, CueType, HotCue, MemoryCue, PreviewWaveformColumn, TrackAnalysis, WaveformColumn,
        WaveformType,
    },
    library::{
        Album, Artist, Artwork, Genre, Key, Label, Library, OriginDatabase, Playlist,
        PlaylistFolder, PlaylistTreeNode, Track,
    },
    timecode::Timecode,
};
use rekordcrate::{
    anlz::ANLZ,
    pdb::{DatabaseType, Header},
};
use thiserror::Error;

use crate::database::relativeify_path_string;

#[derive(Debug)]
pub struct RekordboxDatabase {
    device_root: PathBuf,

    pdb_file: File,
    pdb_header: Header,
}

#[derive(Error, Debug)]
pub enum OpenDatabaseError {
    #[error("PDB File Not Found")]
    PDBNotFound,
    #[error("PDB File Corrupt")]
    PDBCorrupt,
}

#[derive(Error, Debug)]
pub enum LoadBaseError {
    #[error("PDB File Not Found")]
    PDBNotFound,
    #[error("PDB File Corrupt")]
    PDBCorrupt,
}

#[derive(Error, Debug)]
pub enum LoadAnalysisError {
    #[error("Analysis File Not Found")]
    AnalysisNotFound,
    #[error("Analysis File Corrupt")]
    AnalysisCorrupt,
}

impl RekordboxDatabase {
    pub fn open(device_root: &Path) -> Result<RekordboxDatabase, OpenDatabaseError> {
        let pdb_path = device_root.join(Path::new("PIONEER/rekordbox/export.pdb"));

        let Ok(mut pdb_file) = File::open(pdb_path) else {
            return Err(OpenDatabaseError::PDBNotFound);
        };
        let Ok(pdb_header) = Header::read_args(&mut pdb_file, (DatabaseType::Plain,)) else {
            return Err(OpenDatabaseError::PDBCorrupt);
        };

        Ok(RekordboxDatabase {
            device_root: device_root.into(),

            pdb_file,
            pdb_header,
        })
    }

    #[allow(clippy::too_many_lines)]
    pub fn load_base(
        self: &mut RekordboxDatabase,
        existing_library: &mut Library,
    ) -> Result<(), LoadBaseError> {
        for table in &self.pdb_header.tables {
            for page in self
                .pdb_header
                .read_pages(
                    &mut self.pdb_file,
                    binrw::Endian::NATIVE,
                    (&table.first_page, &table.last_page, DatabaseType::Plain),
                )
                .unwrap()
            {
                match page.content {
                    rekordcrate::pdb::PageContent::Data(data_page_content) => {
                        for row_group in data_page_content.rows {
                            match row_group.1 {
                                rekordcrate::pdb::Row::Plain(plain_row) => match plain_row {
                                    rekordcrate::pdb::PlainRow::Album(album) => {
                                        existing_library.albums.insert(
                                            album.id.0,
                                            Album {
                                                id: album.id.0,
                                                artist_id: album.artist_id.0,
                                                name: album.offsets.name.to_string(),

                                                origin: OriginDatabase::Rekordbox,
                                            },
                                        );
                                    }
                                    rekordcrate::pdb::PlainRow::Key(key) => {
                                        existing_library.keys.insert(
                                            key.id.0,
                                            Key {
                                                id: key.id.0,
                                                name: key.name.to_string(),

                                                origin: OriginDatabase::Rekordbox,
                                            },
                                        );
                                    }
                                    rekordcrate::pdb::PlainRow::Artist(artist) => {
                                        existing_library.artists.insert(
                                            artist.id.0,
                                            Artist {
                                                id: artist.id.0,
                                                name: artist.offsets.name.to_string(),

                                                origin: OriginDatabase::Rekordbox,
                                            },
                                        );
                                    }
                                    rekordcrate::pdb::PlainRow::Artwork(artwork) => {
                                        existing_library.artworks.insert(
                                            artwork.id.0,
                                            Artwork {
                                                id: artwork.id.0,
                                                path: self.device_root.join(PathBuf::from(
                                                    relativeify_path_string(
                                                        artwork.path.to_string(),
                                                    ),
                                                )),

                                                origin: OriginDatabase::Rekordbox,
                                            },
                                        );
                                    }
                                    rekordcrate::pdb::PlainRow::Genre(genre) => {
                                        existing_library.genres.insert(
                                            genre.id.0,
                                            Genre {
                                                id: genre.id.0,
                                                name: genre.name.to_string(),

                                                origin: OriginDatabase::Rekordbox,
                                            },
                                        );
                                    }
                                    rekordcrate::pdb::PlainRow::Label(label) => {
                                        existing_library.labels.insert(
                                            label.id.0,
                                            Label {
                                                id: label.id.0,
                                                name: label.name.to_string(),

                                                origin: OriginDatabase::Rekordbox,
                                            },
                                        );
                                    }
                                    rekordcrate::pdb::PlainRow::PlaylistTreeNode(
                                        playlist_tree_node,
                                    ) => {
                                        existing_library
                                            .playlist_tree
                                            .entry(playlist_tree_node.id.0)
                                            .and_modify(|entry| match entry {
                                                PlaylistTreeNode::Playlist(playlist) => {
                                                    playlist.id = playlist_tree_node.id.0;
                                                    playlist.name =
                                                        playlist_tree_node.name.to_string();
                                                }
                                                PlaylistTreeNode::PlaylistFolder(
                                                    playlist_folder,
                                                ) => {
                                                    playlist_folder.id = playlist_tree_node.id.0;
                                                    playlist_folder.name =
                                                        playlist_tree_node.name.to_string();
                                                }
                                            })
                                            .or_insert(if playlist_tree_node.is_folder() {
                                                PlaylistTreeNode::PlaylistFolder(PlaylistFolder {
                                                    id: playlist_tree_node.id.0,
                                                    name: playlist_tree_node.name.to_string(),
                                                    children: Vec::new(),

                                                    origin: OriginDatabase::Rekordbox,
                                                })
                                            } else {
                                                PlaylistTreeNode::Playlist(Playlist {
                                                    id: playlist_tree_node.id.0,
                                                    name: playlist_tree_node.name.to_string(),
                                                    tracks: Vec::new(),

                                                    origin: OriginDatabase::Rekordbox,
                                                })
                                            });

                                        existing_library
                                            .playlist_tree
                                            .entry(playlist_tree_node.parent_id.0)
                                            .and_modify(|entry| match entry {
                                                PlaylistTreeNode::Playlist(_) => {}
                                                PlaylistTreeNode::PlaylistFolder(
                                                    playlist_folder,
                                                ) => {
                                                    playlist_folder
                                                        .children
                                                        .push(playlist_tree_node.id.0);
                                                }
                                            })
                                            .or_insert(PlaylistTreeNode::PlaylistFolder(
                                                PlaylistFolder {
                                                    id: playlist_tree_node.parent_id.0,
                                                    name: String::new(),
                                                    children: vec![playlist_tree_node.id.0],

                                                    origin: OriginDatabase::Rekordbox,
                                                },
                                            ));
                                    }
                                    rekordcrate::pdb::PlainRow::PlaylistEntry(playlist_entry) => {
                                        if let Some(playlist) = existing_library
                                            .playlist_tree
                                            .get_mut(&playlist_entry.playlist_id.0)
                                            && let PlaylistTreeNode::Playlist(Playlist {
                                                id: _,
                                                name: _,
                                                tracks,

                                                origin: OriginDatabase::Rekordbox,
                                            }) = playlist
                                        {
                                            tracks.push(playlist_entry.track_id.0);
                                        }
                                    }
                                    rekordcrate::pdb::PlainRow::Track(track) => {
                                        existing_library.tracks.insert(
                                            track.id.0,
                                            Track {
                                                id: track.id.0,

                                                origin: OriginDatabase::Rekordbox,

                                                title: track.offsets.title.to_string(),

                                                #[allow(clippy::cast_precision_loss)]
                                                bpm: track.tempo as f32 / 100.0,
                                                duration: u32::from(track.duration),

                                                composer_id: track.composer_id.0,
                                                artist_id: track.artist_id.0,
                                                original_artist_id: track.orig_artist_id.0,
                                                remixer_id: track.remixer_id.0,
                                                label_id: track.label_id.0,
                                                album_id: track.album_id.0,
                                                genre_id: track.genre_id.0,
                                                artwork_id: track.artwork_id.0,
                                                key_id: track.key_id.0,

                                                audio_path: self.device_root.join(
                                                    relativeify_path_string(
                                                        track.offsets.file_path.to_string(),
                                                    ),
                                                ),

                                                analysis_path: self.device_root.join(
                                                    relativeify_path_string(
                                                        track.offsets.analyze_path.to_string(),
                                                    ),
                                                ),
                                            },
                                        );
                                    }
                                    _ => {}
                                },
                                rekordcrate::pdb::Row::Ext(_) | rekordcrate::pdb::Row::Unknown => {}
                            }
                        }
                    }
                    rekordcrate::pdb::PageContent::Index(_)
                    | rekordcrate::pdb::PageContent::Unknown => {}
                }
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    pub fn load_analysis(
        self: &RekordboxDatabase,
        track: &Track,
    ) -> Result<TrackAnalysis, LoadAnalysisError> {
        let mut beat_grid_cache = Vec::new();
        let mut hot_cues_cache = Vec::new();
        let mut memory_cues_cache = Vec::new();

        let anlz_path = track.analysis_path.clone();

        let Ok(mut anlz_file) = File::open(anlz_path) else {
            return Err(LoadAnalysisError::AnalysisNotFound);
        };

        if let Ok(anlz) = ANLZ::read(&mut anlz_file) {
            for section in anlz.sections {
                match section.content {
                    rekordcrate::anlz::Content::BeatGrid(beat_grid) => {
                        for beat in beat_grid.beats {
                            beat_grid_cache.push(Beat {
                                beat_number: u32::from(beat.beat_number),
                                bpm: f32::from(beat.tempo) / 100.0,
                                time: Timecode::from_milliseconds(i64::from(beat.time)),
                            });
                        }
                    }
                    rekordcrate::anlz::Content::CueList(cue_list) => {
                        for cue in cue_list.cues {
                            if cue.hot_cue == 0 {
                                memory_cues_cache.push(MemoryCue {
                                    time: Timecode::from_milliseconds(i64::from(cue.time)),
                                    comment: String::new(),
                                });
                            } else {
                                hot_cues_cache.push(HotCue {
                                    cue_number: cue.hot_cue,
                                    cue_type: match cue.cue_type {
                                        rekordcrate::anlz::CueType::Point => CueType::Point,
                                        rekordcrate::anlz::CueType::Loop => CueType::Loop(
                                            Timecode::from_milliseconds(i64::from(cue.loop_time)),
                                        ),
                                    },
                                    time: Timecode::from_milliseconds(i64::from(cue.time)),
                                    comment: String::new(),
                                    color_index: 0,
                                    color_rgb: (255, 255, 255),
                                });
                            }
                        }
                    }
                    rekordcrate::anlz::Content::ExtendedCueList(extended_cue_list) => {
                        for cue in extended_cue_list.cues {
                            if cue.hot_cue == 0 {
                                memory_cues_cache.push(MemoryCue {
                                    time: Timecode::from_milliseconds(i64::from(cue.time)),
                                    comment: cue.comment.to_string(),
                                });
                            } else {
                                hot_cues_cache.push(HotCue {
                                    cue_number: cue.hot_cue,
                                    cue_type: match cue.cue_type {
                                        rekordcrate::anlz::CueType::Point => CueType::Point,
                                        rekordcrate::anlz::CueType::Loop => CueType::Loop(
                                            Timecode::from_milliseconds(i64::from(cue.loop_time)),
                                        ),
                                    },
                                    time: Timecode::from_milliseconds(i64::from(cue.time)),
                                    comment: cue.comment.to_string(),
                                    color_index: cue.hot_cue_color_index,
                                    color_rgb: match cue.hot_cue_color_index {
                                        0x01 => (0x30, 0x5a, 0xff),
                                        0x02 => (0x50, 0x73, 0xff),
                                        0x03 => (0x50, 0x8c, 0xff),
                                        0x04 => (0x50, 0xa0, 0xff),
                                        0x05 => (0x50, 0xb4, 0xff),
                                        0x06 => (0x50, 0xb0, 0xf2),
                                        0x07 => (0x50, 0xae, 0xe8),
                                        0x08 => (0x45, 0xac, 0xdb),
                                        0x09 => (0x00, 0xe0, 0xff),
                                        0x0a => (0x19, 0xda, 0xf0),
                                        0x0b => (0x32, 0xd2, 0xe6),
                                        0x0c => (0x21, 0xb4, 0xb9),
                                        0x0d => (0x20, 0xaa, 0xa0),
                                        0x0e => (0x1f, 0xa3, 0x92),
                                        0x0f => (0x19, 0xa0, 0x8c),
                                        0x10 => (0x14, 0xa5, 0x84),
                                        0x11 => (0x14, 0xaa, 0x7d),
                                        0x12 => (0x10, 0xb1, 0x76),
                                        0x13 => (0x30, 0xd2, 0x6e),
                                        0x14 => (0x37, 0xde, 0x5a),
                                        0x15 => (0x3c, 0xeb, 0x50),
                                        0x16 => (0x28, 0xe2, 0x14),
                                        0x17 => (0x7d, 0xc1, 0x3d),
                                        0x18 => (0x8c, 0xc8, 0x32),
                                        0x19 => (0x9b, 0xd7, 0x23),
                                        0x1a => (0xa5, 0xe1, 0x16),
                                        0x1b => (0xa5, 0xdc, 0x0a),
                                        0x1c => (0xaa, 0xd2, 0x08),
                                        0x1d => (0xb4, 0xc8, 0x05),
                                        0x1e => (0xb4, 0xbe, 0x04),
                                        0x1f => (0xba, 0xb4, 0x04),
                                        0x20 => (0xc3, 0xaf, 0x04),
                                        0x21 => (0xe1, 0xaa, 0x00),
                                        0x22 => (0xff, 0xa0, 0x00),
                                        0x23 => (0xff, 0x96, 0x00),
                                        0x24 => (0xff, 0x8c, 0x00),
                                        0x25 => (0xff, 0x75, 0x00),
                                        0x26 => (0xe0, 0x64, 0x1b),
                                        0x27 => (0xe0, 0x46, 0x1e),
                                        0x28 => (0xe0, 0x30, 0x1e),
                                        0x29 => (0xe0, 0x28, 0x23),
                                        0x2a => (0xe6, 0x28, 0x28),
                                        0x2b => (0xff, 0x37, 0x6f),
                                        0x2c => (0xff, 0x2d, 0x6f),
                                        0x2d => (0xff, 0x12, 0x7b),
                                        0x2e => (0xf5, 0x1e, 0x8c),
                                        0x2f => (0xeb, 0x2d, 0xa0),
                                        0x30 => (0xe6, 0x37, 0xb4),
                                        0x31 => (0xde, 0x44, 0xcf),
                                        0x32 => (0xde, 0x44, 0x8d),
                                        0x33 => (0xe6, 0x30, 0xb4),
                                        0x34 => (0xe6, 0x19, 0xdc),
                                        0x35 => (0xe6, 0x00, 0xff),
                                        0x36 => (0xdc, 0x00, 0xff),
                                        0x37 => (0xcc, 0x00, 0xff),
                                        0x38 => (0xb4, 0x32, 0xff),
                                        0x39 => (0xb9, 0x3c, 0xff),
                                        0x3a => (0xc5, 0x42, 0xff),
                                        0x3b => (0xaa, 0x5a, 0xff),
                                        0x3c => (0xaa, 0x72, 0xff),
                                        0x3d => (0x82, 0x72, 0xff),
                                        0x3e => (0x64, 0x73, 0xff),
                                        _ => (0x00, 0x00, 0xff),
                                    },
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        } else {
            return Err(LoadAnalysisError::AnalysisCorrupt);
        }

        Ok(TrackAnalysis {
            beat_grid: beat_grid_cache,
            memory_cues: memory_cues_cache,
            hot_cues: hot_cues_cache,
            origin: OriginDatabase::Rekordbox,
        })
    }

    #[allow(clippy::too_many_lines)]
    pub fn load_preview_waveform(
        self: &RekordboxDatabase,
        track: &Track,
        waveform_type: WaveformType,
    ) -> Result<Vec<PreviewWaveformColumn>, LoadAnalysisError> {
        let mut anlz_path = track.analysis_path.clone();

        anlz_path.set_extension(match waveform_type {
            WaveformType::Grayscale | WaveformType::RGB => "EXT",
            WaveformType::ThreeBand => "2EX",
        });

        let Ok(mut anlz_file) = File::open(anlz_path) else {
            return Err(LoadAnalysisError::AnalysisNotFound);
        };

        let mut waveform_cache = Vec::new();

        if let Ok(anlz) = ANLZ::read(&mut anlz_file) {
            for section in anlz.sections {
                match (waveform_type, section.content) {
                    (
                        WaveformType::Grayscale,
                        rekordcrate::anlz::Content::WaveformColorPreview(waveform),
                    ) => {
                        for waveform_column in waveform.data {
                            waveform_cache.push(PreviewWaveformColumn::Grayscale {
                                height: u8::max(
                                    u8::max(
                                        waveform_column.energy_bottom_third_freq,
                                        waveform_column.energy_mid_third_freq,
                                    ),
                                    waveform_column.energy_top_third_freq,
                                ),
                                saturation: waveform_column.unknown2,
                            });
                        }
                    }
                    (
                        WaveformType::RGB,
                        rekordcrate::anlz::Content::WaveformColorPreview(waveform),
                    ) => {
                        for waveform_column in waveform.data {
                            waveform_cache.push(PreviewWaveformColumn::RGB {
                                height: u8::max(
                                    u8::max(
                                        waveform_column.energy_bottom_third_freq,
                                        waveform_column.energy_mid_third_freq,
                                    ),
                                    waveform_column.energy_top_third_freq,
                                ),
                                color_rgb: (
                                    waveform_column.energy_bottom_third_freq,
                                    waveform_column.energy_mid_third_freq,
                                    waveform_column.energy_top_third_freq,
                                ),
                            });
                        }
                    }
                    (
                        WaveformType::ThreeBand,
                        rekordcrate::anlz::Content::Waveform3BandPreview(waveform),
                    ) => {
                        for waveform_column in waveform.data {
                            waveform_cache.push(PreviewWaveformColumn::ThreeBand {
                                height: waveform_column
                                    .energy_bottom_third_freq
                                    .saturating_add(waveform_column.energy_mid_third_freq)
                                    .saturating_add(waveform_column.energy_top_third_freq),
                                bands: (
                                    waveform_column.energy_bottom_third_freq,
                                    waveform_column.energy_mid_third_freq,
                                    waveform_column.energy_top_third_freq,
                                ),
                            });
                        }
                    }
                    _ => {}
                }
            }
        } else {
            println!("d");

            return Err(LoadAnalysisError::AnalysisCorrupt);
        }

        Ok(waveform_cache)
    }

    #[allow(clippy::too_many_lines)]
    pub fn load_waveform(
        self: &RekordboxDatabase,
        track: &Track,
        waveform_type: WaveformType,
    ) -> Result<Vec<WaveformColumn>, LoadAnalysisError> {
        let mut anlz_path = track.analysis_path.clone();
        anlz_path.set_extension(match waveform_type {
            WaveformType::Grayscale | WaveformType::RGB => "EXT",
            WaveformType::ThreeBand => "2EX",
        });

        let Ok(mut anlz_file) = File::open(anlz_path) else {
            return Err(LoadAnalysisError::AnalysisNotFound);
        };

        let mut waveform_cache = Vec::new();

        if let Ok(anlz) = ANLZ::read(&mut anlz_file) {
            for section in anlz.sections {
                match (waveform_type, section.content) {
                    (
                        WaveformType::Grayscale,
                        rekordcrate::anlz::Content::WaveformDetail(waveform),
                    ) => {
                        for waveform_column in waveform.data {
                            waveform_cache.push(WaveformColumn::Grayscale {
                                height: waveform_column.height(),
                                saturation: waveform_column.whiteness(),
                            });
                        }
                    }
                    (
                        WaveformType::RGB,
                        rekordcrate::anlz::Content::WaveformColorDetail(waveform),
                    ) => {
                        for waveform_column in waveform.data {
                            // todo: swap this for the better method of colourising
                            waveform_cache.push(WaveformColumn::RGB {
                                height: waveform_column.height(),
                                color_rgb: (
                                    waveform_column.red(),
                                    waveform_column.green(),
                                    waveform_column.blue(),
                                ),
                            });
                        }
                    }
                    (
                        WaveformType::ThreeBand,
                        rekordcrate::anlz::Content::Waveform3BandDetail(waveform),
                    ) => {
                        for waveform_column in waveform.data {
                            waveform_cache.push(WaveformColumn::ThreeBand {
                                height: u8::max(
                                    u8::max(
                                        waveform_column.energy_bottom_third_freq,
                                        waveform_column.energy_mid_third_freq,
                                    ),
                                    waveform_column.energy_top_third_freq,
                                ),
                                bands: (
                                    waveform_column.energy_bottom_third_freq,
                                    // why does this work better? is the ordering of
                                    // Waveform3BandDetail wrong?
                                    waveform_column.energy_top_third_freq,
                                    waveform_column.energy_mid_third_freq,
                                ),
                            });
                        }
                    }
                    _ => {}
                }
            }
        } else {
            return Err(LoadAnalysisError::AnalysisCorrupt);
        }

        Ok(waveform_cache)
    }

    pub fn sync(self: &mut RekordboxDatabase) {
        // todo: sync updated data back to disk (will probably need to modify
        // rekordcrate to make this happen)
    }

    pub fn close(mut self: RekordboxDatabase) {
        self.sync();
    }
}
