use rkyv::{Archive, Deserialize, Serialize};
use timecode::Timecode;

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
pub enum CrossFaderSide {
    A,
    B,
    None,
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
pub enum ChannelFXEffect {
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

    pub master_channel: Option<usize>,

    pub master_fx: MasterFX,

    pub crossfade: f32,

    pub master_cue: bool,
    pub master_gain: f32, // decibels

    pub channel_fx_effect: ChannelFXEffect,
}

impl Default for DeckState {
    fn default() -> Self {
        Self {
            mixer_channels: Default::default(),

            master_fx: MasterFX::default(),

            crossfade: 0.5,

            master_channel: None,

            master_cue: false,
            master_gain: 0.0,

            channel_fx_effect: ChannelFXEffect::None,
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct ChannelState {
    pub player: PlayerState,

    pub gain: f32,           // decibels
    pub eq: (f32, f32, f32), // decibels
    pub fx: f32,             // percent

    pub cue: bool, // cue enabled

    pub fade: f32, // percent

    pub cross_fader_side: CrossFaderSide,
}

impl Default for ChannelState {
    fn default() -> Self {
        Self {
            player: PlayerState::default(),

            gain: 0.0,
            eq: (0.0, 0.0, 0.0),
            fx: 0.0,

            cue: false,

            fade: 1.0,

            cross_fader_side: CrossFaderSide::None,
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct PlayerState {
    pub current_track: Option<(usize, Track)>,
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
pub enum MasterFXChannel {
    Channel(usize),
    Master,
}

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize)]
pub enum MasterFXEffect {
    LowCutEcho,
    Echo,
    Delay,
    Spiral,
    Reverb,
    Transgate,
    EnigmaJet,
    Flanger,
    Phaser,
    Stretch,
    SlipRoll,
    Roll,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct MasterFX {
    pub channel: MasterFXChannel,

    pub effect: MasterFXEffect,

    pub enabled: bool,

    pub length: f32,
    pub percent: f32,

    pub depth: f32,
    pub bpm: f32,
    pub auto_bpm: bool,
}

impl Default for MasterFX {
    fn default() -> Self {
        Self {
            channel: MasterFXChannel::Master,

            effect: MasterFXEffect::Reverb,

            enabled: false,

            length: 0.0,
            percent: 0.0,

            depth: 0.0,
            bpm: 120.0,
            auto_bpm: true,
        }
    }
}
