use rkyv::{Archive, Deserialize, Serialize};
use timecode::Timecode;

use crate::types::library::OriginDatabase;

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct TrackAnalysis {
    pub beat_grid: Vec<Beat>,
    pub memory_cues: Vec<MemoryCue>,
    pub hot_cues: Vec<HotCue>,

    pub origin: OriginDatabase,
}

impl Default for TrackAnalysis {
    fn default() -> Self {
        Self {
            beat_grid: Vec::new(),
            memory_cues: Vec::new(),
            hot_cues: Vec::new(),
            origin: OriginDatabase::Rekordbox,
        }
    }
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
pub enum WaveformType {
    Grayscale,
    RGB,
    ThreeBand,
}

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize)]
pub enum PreviewWaveformColumn {
    Grayscale { height: u8, saturation: u8 },
    RGB { height: u8, color_rgb: (u8, u8, u8) },
    ThreeBand { height: u8, bands: (u8, u8, u8) },
}

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize)]
pub enum WaveformColumn {
    Grayscale { height: u8, saturation: u8 },
    RGB { height: u8, color_rgb: (u8, u8, u8) },
    ThreeBand { height: u8, bands: (u8, u8, u8) },
}
