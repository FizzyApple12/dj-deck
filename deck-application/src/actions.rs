use gpui::AsyncApp;
use libdj::types::library::TrackID;
use libdsp::audio_loader::TrackAudioData;
use log::warn;
use timecode::Timecode;

use crate::GlobalUIState;

pub async fn load_track(cx: &AsyncApp, player: usize, device: usize, id: TrackID) {
    let (deck_update_sender, loaded_track_sender, device_manager) = cx
        .read_global::<GlobalUIState, _>(|state, _| {
            (
                state.deck_update_sender.clone(),
                state.loaded_track_sender.clone(),
                state.device_manager.subscribe(),
            )
        });

    let mut locked_device_database = device_manager.devices.lock().await;

    if let Some(qualified_device) = locked_device_database.get_mut(&device)
        && let Some(track) = qualified_device.database.library.tracks.get(&id)
    {
        let _ = loaded_track_sender.send((player, None));

        let _ = deck_update_sender.send(Box::new(move |deck_state| {
            if let Some(mixer_channel) = deck_state.mixer_channels.get_mut(player) {
                mixer_channel.player.current_track = None;
                mixer_channel.player.is_loading = true;

                mixer_channel.player.time = Timecode::zero();
                mixer_channel.player.slip_time = Timecode::zero();
                mixer_channel.player.cue_time = None;
                mixer_channel.player.touch_cue_time = None;

                mixer_channel.player.beat_loop_start = None;
                mixer_channel.player.beat_loop_end = None;

                mixer_channel.player.keyshift = 0.0;
            }
        }));

        let cloned_track = track.clone();

        let cloned_deck_update_sender = deck_update_sender.clone();

        std::thread::spawn(move || {
            let audio_data = match TrackAudioData::load_from_file(&cloned_track.audio_path) {
                Ok(audio_data) => audio_data,
                Err(err) => {
                    // todo: build a way to propagate errors to the ui
                    warn!("track load error: {err:?}");

                    let _ = cloned_deck_update_sender.send(Box::new(move |deck_state| {
                        if let Some(mixer_channel) = deck_state.mixer_channels.get_mut(player) {
                            mixer_channel.player.is_loading = false;
                        }
                    }));

                    return;
                }
            };

            let _ = loaded_track_sender.send((player, Some(Box::new(audio_data))));

            let _ = cloned_deck_update_sender.send(Box::new(move |deck_state| {
                if let Some(mixer_channel) = deck_state.mixer_channels.get_mut(player) {
                    mixer_channel.player.current_track = Some((device, cloned_track));
                    mixer_channel.player.is_loading = false;
                }
            }));
        });

        // todo: this should be moved to a new thread
        if let Ok(track_analysis) = qualified_device.database.load_analysis(id) {
            let cloned_track_analysis = track_analysis.clone();

            drop(locked_device_database);

            let _ = deck_update_sender.send(Box::new(move |deck_state| {
                if let Some(mixer_channel) = deck_state.mixer_channels.get_mut(player) {
                    mixer_channel.player.current_track_analysis = Some(cloned_track_analysis);
                }
            }));
        } else {
            drop(locked_device_database);

            std::thread::spawn(move || {
                // todo: perform track analysis
            });
        }
    }
}

pub fn eject(cx: &AsyncApp, player: usize) {
    let (deck_update_sender, loaded_track_sender) =
        cx.read_global::<GlobalUIState, _>(|state, _| {
            (
                state.deck_update_sender.clone(),
                state.loaded_track_sender.clone(),
            )
        });

    let _ = deck_update_sender.send(Box::new(move |deck_state| {
        if let Some(mixer_channel) = deck_state.mixer_channels.get_mut(player) {
            mixer_channel.player.current_track = None;
            mixer_channel.player.current_track_analysis = None;
            mixer_channel.player.is_loading = false;

            mixer_channel.player.time = Timecode::zero();
            mixer_channel.player.slip_time = Timecode::zero();
            mixer_channel.player.cue_time = None;
            mixer_channel.player.touch_cue_time = None;

            mixer_channel.player.beat_loop_start = None;
            mixer_channel.player.beat_loop_end = None;

            mixer_channel.player.keyshift = 0.0;
        }
    }));

    let _ = loaded_track_sender.send((player, None));
}

// UIEvent::TouchCue { cue_time, player } => {
//     let _ = deck_update_sender.send(Box::new(move |deck_state, _| {
//         if let Some(mixer_channel) =
// deck_state.mixer_channels.get_mut(player) {             
// mixer_channel.player.touch_cue_time = cue_time;         }
//     }));

//     Task::none()
// }
// UIEvent::BeatJump { beats, player } => {
//     let _ = deck_update_sender.send(Box::new(move |deck_state, _| {
//         if let Some(mixer_channel) =
// deck_state.mixer_channels.get_mut(player)             && let Some(bpm) =
// mixer_channel.player.get_current_bpm()         {
//             mixer_channel.player.time += Duration::from_nanoseconds(
//                 ((1.0 / bpm) * beats * 60_000_000_000.0) as i64,
//             );
//         }
//     }));

//     Task::none()
// }
// UIEvent::SetBeatLoop { beats, player } => {
//     let _ = deck_update_sender.send(Box::new(move |deck_state, _| {
//         if let Some(mixer_channel) =
// deck_state.mixer_channels.get_mut(player)             && let Some(bpm) =
// mixer_channel.player.get_current_bpm()         {
//             mixer_channel.player.beat_loop_start =
// Some(mixer_channel.player.time);             
// mixer_channel.player.beat_loop_end = Some(                 
// mixer_channel.player.time
//                     + Duration::from_nanoseconds( ((1.0 / bpm) * beats *
//                       60_000_000_000.0) as i64,
//                     ),
//             );
//         }
//     }));

//     Task::none()
// }
// UIEvent::DoubleBeatLoop(player) => {
//     let _ = deck_update_sender.send(Box::new(move |deck_state, _| {
//         if let Some(mixer_channel) =
// deck_state.mixer_channels.get_mut(player)             && let
// Some(beat_loop_start) = mixer_channel.player.beat_loop_start             &&
// let Some(beat_loop_end) = mixer_channel.player.beat_loop_end         {
//             mixer_channel.player.beat_loop_start =
// Some(mixer_channel.player.time);             
// mixer_channel.player.beat_loop_end =                 Some(beat_loop_end +
// (beat_loop_end - beat_loop_start));         }
//     }));

//     Task::none()
// }
// UIEvent::HalveBeatLoop(player) => {
//     let _ = deck_update_sender.send(Box::new(move |deck_state, _| {
//         if let Some(mixer_channel) =
// deck_state.mixer_channels.get_mut(player)             && let
// Some(beat_loop_start) = mixer_channel.player.beat_loop_start             &&
// let Some(beat_loop_end) = mixer_channel.player.beat_loop_end         {
//             mixer_channel.player.beat_loop_start =
// Some(mixer_channel.player.time);             
// mixer_channel.player.beat_loop_end =                 Some(beat_loop_end -
// ((beat_loop_end - beat_loop_start) / 2.));         }
//     }));

//     Task::none()
// }
// UIEvent::SetKeyShift { player, semitones } => {
//     let _ = deck_update_sender.send(Box::new(move |deck_state, _| {
//         if let Some(mixer_channel) =
// deck_state.mixer_channels.get_mut(player) {             
// mixer_channel.player.keyshift = semitones;         }
//     }));

//     Task::none()
// }
