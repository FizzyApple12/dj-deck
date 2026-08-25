#![feature(unboxed_closures, fn_traits)]

// mod arena;
mod nesting;

use std::{cell::RefCell, path::PathBuf, rc::Rc};

use criterion::{Criterion, criterion_group, criterion_main};
use libdj::{
    playback::{ChannelUpdateResults, DeckUpdateResults},
    types::{
        analysis::{Beat, TrackAnalysis},
        deck::{ChannelState, DeckState, PlayState},
        library::{OriginDatabase, Track},
    },
};
use libdsp::{
    audio_loader::TrackAudioData,
    samples_to_nanoseconds,
    timecode::{Duration, Timecode},
};

pub struct DummyExtractorF32;

impl<Data> FnOnce<(&Data,)> for DummyExtractorF32 {
    type Output = f32;

    extern "rust-call" fn call_once(self, _: (&Data,)) -> f32 {
        panic!("Tried to call never")
    }
}

impl<Data> FnMut<(&Data,)> for DummyExtractorF32 {
    extern "rust-call" fn call_mut(&mut self, _: (&Data,)) -> f32 {
        panic!("Tried to call never")
    }
}

impl<Data> Fn<(&Data,)> for DummyExtractorF32 {
    extern "rust-call" fn call(&self, _: (&Data,)) -> f32 {
        panic!("Tried to call never")
    }
}

pub struct DummyExtractorF64;

impl<Data> FnOnce<(&Data,)> for DummyExtractorF64 {
    type Output = f64;

    extern "rust-call" fn call_once(self, _: (&Data,)) -> f64 {
        panic!("Tried to call never")
    }
}

impl<Data> FnMut<(&Data,)> for DummyExtractorF64 {
    extern "rust-call" fn call_mut(&mut self, _: (&Data,)) -> f64 {
        panic!("Tried to call never")
    }
}

impl<Data> Fn<(&Data,)> for DummyExtractorF64 {
    extern "rust-call" fn call(&self, _: (&Data,)) -> f64 {
        panic!("Tried to call never")
    }
}

pub struct PipelineDeckData<'a> {
    output_sample_rate: u32,

    track_sample_rates: [u32; 4],
    track_change_signals: [&'a std::sync::mpsc::Receiver<()>; 4],

    deck_state: &'a DeckState,
    deck_update_results: &'a DeckUpdateResults,
}

pub struct PipelineChannelData<'a> {
    output_sample_rate: u32,

    track_sample_rate: u32,
    track_change_signal: &'a std::sync::mpsc::Receiver<()>,

    channel_state: &'a ChannelState,
    channel_update_results: &'a ChannelUpdateResults,

    crossfade: f32,
}

pub const VIRTUAL_BATCH_SIZE: usize = 128;

pub const OUTPUT_SAMPLE_RATE: u32 = 48000;

pub const TEST_AUDIO_PATH: &str =
    "/home/fizzyapple12/Music/01 ABIS, Signal, Tasha Baxter - The Wall (Buunshin remix).flac";

#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::field_reassign_with_default)]
pub fn run_tests(criterion: &mut Criterion) {
    let start_time = Timecode::zero();
    let end_time = start_time
        + Duration::from_nanoseconds(samples_to_nanoseconds(
            VIRTUAL_BATCH_SIZE as i64,
            OUTPUT_SAMPLE_RATE,
        ));

    let (mut event_sender, _) = tokio::sync::mpsc::unbounded_channel();
    let (_, mut update_receiver) = tokio::sync::mpsc::unbounded_channel();

    let audio_path = PathBuf::from(TEST_AUDIO_PATH);

    let track_audio = TrackAudioData::load_from_file(&audio_path).expect("Audio file loaded");

    let mut deck_state = DeckState::default();

    deck_state.master_cue = true;

    for (index, channel) in deck_state.mixer_channels.iter_mut().enumerate() {
        channel.player.current_track = Some((
            0,
            Track {
                id: 0,
                origin: OriginDatabase::Rekordbox,
                title: "foo".to_string(),
                bpm: 120.0,
                duration: 30,
                composer_id: 0,
                artist_id: 0,
                original_artist_id: 0,
                remixer_id: 0,
                label_id: 0,
                album_id: 0,
                genre_id: 0,
                artwork_id: 0,
                key_id: 0,
                audio_path: audio_path.clone(),
                analysis_path: audio_path.clone(),
            },
        ));
        channel.player.current_track_analysis = Some(TrackAnalysis {
            beat_grid: (0..=20)
                .into_iter()
                .map(|beat_number| Beat {
                    beat_number,
                    bpm: 120.0,
                    time: Timecode::from_milliseconds(500 * i64::from(beat_number)),
                })
                .collect(),
            memory_cues: Vec::new(),
            hot_cues: Vec::new(),
            origin: OriginDatabase::Rekordbox,
        });

        channel.player.play_state = PlayState::Play;
        channel.player.time = Timecode::from_seconds(index as i64);

        channel.cue = true;
    }

    let deck_update_results = deck_state.update(
        &mut update_receiver,
        &mut event_sender,
        start_time,
        end_time,
    );

    let loaded_tracks = [
        Rc::new(RefCell::new(Some(track_audio.clone()))),
        Rc::new(RefCell::new(Some(track_audio.clone()))),
        Rc::new(RefCell::new(Some(track_audio.clone()))),
        Rc::new(RefCell::new(Some(track_audio))),
    ];

    nesting::test_nesting_inlining(criterion, &deck_state, &deck_update_results, &loaded_tracks);

    // test_arena_inlining(criterion, &deck_state, &deck_update_results,
    // &loaded_tracks);
}

criterion_group!(benches, run_tests);
criterion_main!(benches);
