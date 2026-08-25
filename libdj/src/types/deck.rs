use libdsp::{pipeline::CrossFaderSide, timecode::Timecode};
use rkyv::{Archive, Deserialize, Serialize};

use crate::{
    MIXER_CHANNELS,
    types::{analysis::TrackAnalysis, library::Track},
};

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize, PartialEq)]
pub enum TempoRange {
    SixPercent,
    TenPercent,
    SixteenPercent,
    OneHundredPercent,
}

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize, PartialEq)]
pub enum PlayState {
    Stop,
    Play,
    Cue,
}

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize, PartialEq)]
pub enum BeatSyncMode {
    Off,
    BPMSync,
    BeatSync,
}

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize, PartialEq)]
pub enum BeatLoopAdjustMode {
    None,
    In,
    Out,
}

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize, PartialEq)]
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
    pub mixer_channels: [ChannelState; MIXER_CHANNELS],

    pub filter_effect: FilterEffect,

    pub master_channel: Option<usize>,

    pub effects: Effects,

    pub crossfade: f32,

    pub master_cue: bool,
    pub master_gain: f32, // decibels
    pub booth_gain: f32,  // decibels
}

impl Default for DeckState {
    fn default() -> Self {
        Self {
            mixer_channels: Default::default(),

            filter_effect: FilterEffect::None,

            effects: Effects::default(),

            crossfade: 0.5,

            master_channel: None,

            master_cue: false,
            master_gain: 0.0,
            booth_gain: 0.0,
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct ChannelState {
    pub player: PlayerState,

    pub fade: f32, // percent

    pub gain: f32,           // decibels
    pub eq: (f32, f32, f32), // decibels
    pub filter: f32,         // percent

    pub cross_fader_side: CrossFaderSide,

    pub cue: bool, // cue enabled
}

impl Default for ChannelState {
    fn default() -> Self {
        Self {
            player: PlayerState::default(),

            fade: 1.0,

            gain: 0.0,
            eq: (0.0, 0.0, 0.0),
            filter: 0.0,

            cross_fader_side: CrossFaderSide::None,

            cue: false,
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct PlayerState {
    pub current_track: Option<(u32, Track)>,
    pub current_track_analysis: Option<TrackAnalysis>,
    pub is_loading: bool,

    pub beat_sync: BeatSyncMode,
    pub key_sync: bool,

    pub quanitze: bool,

    pub jog_hold: bool,
    pub jog_wait: bool,
    pub jog_velocity: f32,

    pub play_state: PlayState,
    pub time: Timecode,
    pub cue_time: Option<Timecode>,       // timecode/cue not set
    pub touch_cue_time: Option<Timecode>, // timecode/touch cue not active

    pub reverse_enabled: bool,

    pub tempo_range: TempoRange,
    pub tempo_reset: bool,  // tempo reset enabled
    pub tempo_percent: f32, // tempo percent
    pub tempo_slider_position: f32,
    pub tempo_slider_is_accurate: bool,
    pub master_tempo: bool, // master tempo enabled

    pub slip: bool,         // slip enabled
    pub slip_playing: bool, // slip playing
    pub slip_time: Timecode,

    pub beat_loop_start: Option<Timecode>, // timecode/start not set
    pub beat_loop_end: Option<Timecode>,   // timecode/end not set
    pub last_beat_loop: Option<(Timecode, Timecode)>, /* (start timecode, end timecode)/no
                                            * previous loop */
    pub beat_loop_adjust_mode: BeatLoopAdjustMode,

    pub keyshift: f32, // semitones
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            current_track: None,
            current_track_analysis: None,
            is_loading: false,

            beat_sync: BeatSyncMode::Off,
            key_sync: false,

            quanitze: true,

            jog_hold: false,
            jog_wait: false,
            jog_velocity: 0.0,

            play_state: PlayState::Stop,
            time: Timecode::zero(),
            cue_time: None,
            touch_cue_time: None,
            reverse_enabled: false,

            tempo_range: TempoRange::TenPercent,
            tempo_reset: false,
            tempo_percent: 1.0,
            tempo_slider_position: 0.0,
            tempo_slider_is_accurate: true,
            master_tempo: false,

            slip: false,
            slip_playing: false,
            slip_time: Timecode::zero(),

            beat_loop_start: None,
            beat_loop_end: None,
            last_beat_loop: None,
            beat_loop_adjust_mode: BeatLoopAdjustMode::None,

            keyshift: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize, PartialEq)]
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
