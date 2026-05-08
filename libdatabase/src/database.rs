use std::{
    collections::BTreeMap,
    fs::File,
    path::{Path, PathBuf},
};

use binrw::BinRead;
use libdj::types::{
    library::{
        Album, Artist, Artwork, Beat, CueType, Genre, HotCue, Key, Label, Library, MemoryCue,
        Playlist, PlaylistFolder, PlaylistTreeNode, PreviewWaveformColumn,
        TinyPreviewWaveformColumn, Track, WaveformColumn,
    },
    timecode::Timecode,
};
use rekordcrate::{
    anlz::ANLZ,
    pdb::{DatabaseType, Header},
};
use thiserror::Error;

#[derive(Debug)]
pub struct Database {
    pub library: Library,

    _pdb_header: Header,
}

#[derive(Error, Debug)]
pub enum OpenDatabaseError {
    #[error("PDB File Not Found")]
    PDBNotFound,
    #[error("PDB File Corrupt")]
    PDBCorrupt,
}

impl Database {
    #[allow(clippy::too_many_lines, clippy::cast_precision_loss)]
    pub fn open(device_root: &Path) -> Result<Database, OpenDatabaseError> {
        let pdb_path = device_root.join(Path::new("PIONEER/rekordbox/export.pdb"));

        let Ok(mut pdb_file) = File::open(pdb_path) else {
            return Err(OpenDatabaseError::PDBNotFound);
        };
        let Ok(pdb_header) = Header::read_args(&mut pdb_file, (DatabaseType::Plain,)) else {
            return Err(OpenDatabaseError::PDBCorrupt);
        };

        let mut album_cache = BTreeMap::new();
        let mut artist_cache = BTreeMap::new();
        let mut artwork_cache = BTreeMap::new();
        let mut genre_cache = BTreeMap::new();
        let mut label_cache = BTreeMap::new();
        let mut key_cache = BTreeMap::new();

        let mut track_cache = BTreeMap::new();

        let mut playlist_cache = BTreeMap::new();

        for table in &pdb_header.tables {
            for page in pdb_header
                .read_pages(
                    &mut pdb_file,
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
                                        album_cache.insert(
                                            album.id.0,
                                            Album {
                                                id: album.id.0,
                                                artist_id: album.artist_id.0,
                                                name: album.offsets.name.to_string(),
                                            },
                                        );
                                    }
                                    rekordcrate::pdb::PlainRow::Key(key) => {
                                        key_cache.insert(
                                            key.id.0,
                                            Key {
                                                id: key.id.0,
                                                name: key.name.to_string(),
                                            },
                                        );
                                    }
                                    rekordcrate::pdb::PlainRow::Artist(artist) => {
                                        artist_cache.insert(
                                            artist.id.0,
                                            Artist {
                                                id: artist.id.0,
                                                name: artist.offsets.name.to_string(),
                                            },
                                        );
                                    }
                                    rekordcrate::pdb::PlainRow::Artwork(artwork) => {
                                        artwork_cache.insert(
                                            artwork.id.0,
                                            Artwork {
                                                id: artwork.id.0,
                                                path: device_root.join(PathBuf::from(
                                                    relativeify_path_string(
                                                        artwork.path.to_string(),
                                                    ),
                                                )),
                                            },
                                        );
                                    }
                                    rekordcrate::pdb::PlainRow::Genre(genre) => {
                                        genre_cache.insert(
                                            genre.id.0,
                                            Genre {
                                                id: genre.id.0,
                                                name: genre.name.to_string(),
                                            },
                                        );
                                    }
                                    rekordcrate::pdb::PlainRow::Label(label) => {
                                        label_cache.insert(
                                            label.id.0,
                                            Label {
                                                id: label.id.0,
                                                name: label.name.to_string(),
                                            },
                                        );
                                    }
                                    rekordcrate::pdb::PlainRow::PlaylistTreeNode(
                                        playlist_tree_node,
                                    ) => {
                                        playlist_cache
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
                                                })
                                            } else {
                                                PlaylistTreeNode::Playlist(Playlist {
                                                    id: playlist_tree_node.id.0,
                                                    name: playlist_tree_node.name.to_string(),
                                                    tracks: Vec::new(),
                                                })
                                            });

                                        playlist_cache
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
                                                },
                                            ));
                                    }
                                    rekordcrate::pdb::PlainRow::PlaylistEntry(playlist_entry) => {
                                        if let Some(playlist) =
                                            playlist_cache.get_mut(&playlist_entry.playlist_id.0)
                                            && let PlaylistTreeNode::Playlist(Playlist {
                                                id: _,
                                                name: _,
                                                tracks,
                                            }) = playlist
                                        {
                                            tracks.push(playlist_entry.track_id.0);
                                        }
                                    }
                                    rekordcrate::pdb::PlainRow::Track(track) => {
                                        let mut beat_grid_cache = Vec::new();
                                        let mut hot_cues_cache = Vec::new();
                                        let mut memory_cues_cache = Vec::new();
                                        let mut tiny_preview_waveform_cache = Vec::new();
                                        let mut preview_waveform_cache = Vec::new();
                                        let mut detail_waveform_cache = Vec::new();

                                        let anlz_path = device_root.join(relativeify_path_string(
                                            track.offsets.analyze_path.to_string(),
                                        ));

                                        if let Ok(mut anlz_file) = File::open(anlz_path)
                                            && let Ok(anlz) = ANLZ::read(&mut anlz_file)
                                        {
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
                                                    },
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
                                                            			rekordcrate::anlz::CueType::Loop => CueType::Loop(Timecode::from_milliseconds(i64::from(cue.loop_time))),
                                                           			},
                                                        			time: Timecode::from_milliseconds(i64::from(cue.time)),
                                                        			comment: String::new(),
                                                        			color_index: 0,
                                                        			color_rgb: (255, 255, 255),
                                                      			});
                                                      		}
                                                    	}
                                                    },
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
                                                            			rekordcrate::anlz::CueType::Loop => CueType::Loop(Timecode::from_milliseconds(i64::from(cue.loop_time))),
                                                           			},
                                                        			time: Timecode::from_milliseconds(i64::from(cue.time)),
                                                        			comment: cue.comment.to_string(),
                                                        			color_index: cue.hot_cue_color_index,
                                                        			color_rgb: cue.hot_cue_color_rgb,
                                                      			});
                                                      		}
                                                    	}
                                                    },
                                                    rekordcrate::anlz::Content::TinyWaveformPreview(tiny_waveform_preview) => {
                                                    	for column in tiny_waveform_preview.data {
                                                     		tiny_preview_waveform_cache.push(TinyPreviewWaveformColumn {
                                                        		height: column.height(),
                                                        		color_rgb: (255, 255, 255),
                                                       		});
                                                    	}
                                                    },
                                                    // todo: merge in data from rekordcrate::anlz::Content::WaveformColorPreview somehow
                                                    rekordcrate::anlz::Content::WaveformPreview(waveform_preview) => {
                                                    	for column in waveform_preview.data {
                                                   			preview_waveform_cache.push(PreviewWaveformColumn {
                                                      			height: column.height(),
                                                      			color_rgb: (column.whiteness(), column.whiteness(), column.whiteness()),
                                                      		});
                                                  		}
                                                    }
                                                    rekordcrate::anlz::Content::WaveformColorDetail(waveform_color_detail) => {
                                                    	for column in waveform_color_detail.data {
                                                     		detail_waveform_cache.push(WaveformColumn {
                                                        		height: column.height(),
                                                        		color_rgb: (column.red(), column.green(), column.blue()),
                                                       		});
                                                    	}
                                                    },
                                                    _ => {},
                                                }
                                            }
                                        }

                                        track_cache.insert(
                                            track.id.0,
                                            Track {
                                                id: track.id.0,
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

                                                audio_path: device_root.join(PathBuf::from(
                                                    relativeify_path_string(
                                                        track.offsets.file_path.to_string(),
                                                    ),
                                                )),

                                                beat_grid: beat_grid_cache,
                                                hot_cues: hot_cues_cache,
                                                memory_cues: memory_cues_cache,
                                                tiny_preview_waveform: tiny_preview_waveform_cache,
                                                preview_waveform: preview_waveform_cache,
                                                detail_waveform: detail_waveform_cache,
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

        Ok(Database {
            library: Library {
                albums: album_cache,
                artists: artist_cache,
                artworks: artwork_cache,
                genres: genre_cache,
                labels: label_cache,
                keys: key_cache,

                tracks: track_cache,

                playlist_tree: playlist_cache,
            },

            _pdb_header: pdb_header,
        })
    }

    pub fn sync(self: &mut Database) {
        // todo: sync updated data back to disk (will probably need to modify
        // rekordcrate to make this happen)
    }

    pub fn close(mut self: Database) {
        self.sync();
    }
}

fn relativeify_path_string(mut path: String) -> String {
    if path.starts_with('/') {
        path.replace_first("/", "");
    }

    path
}
