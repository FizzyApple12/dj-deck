use libdj::types::library::TrackID;
use libdsp::audio_loader::TrackAudioData;
use log::warn;
use timecode::Timecode;

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
