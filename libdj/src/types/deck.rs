use crate::types::library::Track;

#[derive(Debug, Clone, Copy)]
pub enum TempoPercent {
    Zero,
    Percent(f32),
}

#[derive(Debug, Clone, Copy)]
pub enum TempoRange {
    SixPercent,
    TenPercent,
    SixteenPercent,
    OneHundredPercent,
}

#[derive(Debug, Clone, Copy)]
pub enum PlayDirection {
    Stop,
    Forward,
    Reverse,
    SlipReverse,
    Jog,
    SlipJog,
}

#[derive(Debug, Clone)]
pub struct DeckState {
    pub players: [PlayerState; 4],

    pub master_player: Option<usize>,

    pub quanitze: bool,
    pub slip: bool,
}

impl Default for DeckState {
    fn default() -> Self {
        Self {
            players: Default::default(),

            master_player: None,

            quanitze: true,
            slip: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlayerState {
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
}

impl Default for PlayerState {
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
        }
    }
}
