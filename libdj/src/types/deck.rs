use rkyv::{Archive, Deserialize, Serialize};

use crate::types::library::Track;

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize)]
pub enum TempoPercent {
    Zero,
    Percent(f32),
}

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize)]
pub enum TempoRange {
    SixPercent,
    TenPercent,
    SixteenPercent,
    OneHundredPercent,
}

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize)]
pub enum PlayDirection {
    Stop,
    Forward,
    Reverse,
    SlipReverse,
    Jog,
    SlipJog,
}

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize)]
pub enum CrossfaderSide {
    A,
    B,
    None,
}

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize)]
pub enum FilterEffect {
    None,
    Space,
    DubEcho,
    Bitcrush,
    Pitch,
    Noise,
    Filter,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct DeckState {
    pub channels: [ChannelState; 4],

    pub filter_effect: FilterEffect,

    pub master_channel: Option<usize>,

    pub effects: Effects,

    pub crossfade: f32,

    pub quanitze: bool,
    pub slip: bool,

    pub master_cue: bool,
    pub master_gain: f32, // decibels
    pub booth_gain: f32,  // decibels
}

impl Default for DeckState {
    fn default() -> Self {
        Self {
            channels: Default::default(),

            filter_effect: FilterEffect::None,

            effects: Effects::default(),

            crossfade: 0.5,

            master_channel: None,

            quanitze: true,
            slip: false,

            master_cue: false,
            master_gain: 0.0,
            booth_gain: 0.0,
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct ChannelState {
    pub current_track: Option<Track>,

    pub beat_sync: bool,
    pub key_sync: bool,

    pub play_direction: PlayDirection,
    pub time: f32,
    pub slip_time: f32,
    pub cue_time: Option<f32>,
    pub needle_time: Option<f32>,

    pub beat_loop_start: Option<f32>,
    pub beat_loop_end: Option<f32>,

    // todo: remove this and do time-based interpolation + on the fly bpm calculation
    pub bpm: f32,
    pub tempo_range: TempoRange,
    pub tempo_percent: TempoPercent,
    pub master_tempo: bool,

    pub keyshift: i8, // semitones

    pub gain: f32, // decibels
    pub eq: (f32, f32, f32),
    pub filter: f32,

    pub crossfader_side: CrossfaderSide,

    pub cue: bool,
}

impl Default for ChannelState {
    fn default() -> Self {
        Self {
            current_track: None,

            beat_sync: false,
            key_sync: false,

            play_direction: PlayDirection::Stop,
            time: 0.0,
            slip_time: 0.0,
            cue_time: None,
            needle_time: None,

            beat_loop_start: None,
            beat_loop_end: None,

            bpm: 0.0,
            tempo_range: TempoRange::TenPercent,
            tempo_percent: TempoPercent::Percent(0.0),
            master_tempo: true,

            keyshift: 0,

            gain: 0.0,
            eq: (0.0, 0.0, 0.0),
            filter: 0.0,

            crossfader_side: CrossfaderSide::None,

            cue: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize)]
pub enum EffectsChannel {
    Channel(usize),
    Master,
}

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize)]
pub enum EffectsChannelEffect {
    LowCutEcho { length: f32 },
    Echo { length: f32 },
    Delay { length: f32 },
    Spiral { length: f32 },
    Reverb { percent: f32 },
    Transgate { length: f32 },
    EnigmaJet { length: f32 },
    Flanger { length: f32 },
    Phaser { length: f32 },
    Stretch { length: f32 },
    SlipRoll { length: f32 },
    Roll { length: f32 },
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct Effects {
    pub channel: EffectsChannel,

    pub effect: EffectsChannelEffect,

    pub enabled: bool,

    pub depth: f32,
    pub bpm: f32,
}

impl Default for Effects {
    fn default() -> Self {
        Self {
            channel: EffectsChannel::Master,

            effect: EffectsChannelEffect::Reverb { percent: 0.5 },

            enabled: false,

            depth: 0.0,
            bpm: 120.0,
        }
    }
}
