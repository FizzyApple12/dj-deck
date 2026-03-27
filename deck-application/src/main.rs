use std::time::Duration;

use libdatabase::device_manager::{DeviceManager, DeviceManagerEvent, device::OpenDeviceError};
use libdj::types::deck::{DeckState, PlayDirection, TempoPercent};
use libui::{
    gui::UI,
    types::ui::{UIEvent, UIMessage},
};
use tokio::signal;
use tokio_stream::StreamExt;

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let mut device_manager = match DeviceManager::start() {
        Ok(device_manager) => device_manager,
        Err(err) => {
            println!("failed to start device manager: {err:#?}");

            return;
        }
    };

    let mut ui = match UI::start_ui().await {
        Ok(ui) => ui,
        Err(err) => {
            println!("failed to start ui: {err:#?}");

            device_manager.stop().await;

            return;
        }
    };

    let mut temp_deck_state = DeckState::default();

    // todo: vsync?
    let mut ui_deck_update_interval = tokio::time::interval(Duration::from_millis(5));

    let mut deck_update_interval = tokio::time::interval(Duration::from_millis(1));

    loop {
        tokio::select! {
            device_event = device_manager.next() => {
                match device_event {
                    Some(DeviceManagerEvent::DeviceConnected(Ok(id))) => {
                        println!("Device {id} Connected Successfully");

                        let _ = ui.send(UIMessage::DeviceConnected(id));

                        // if let Ok(locked_device_database) = device_manager.devices.lock()
                        //     && let Some(device) = locked_device_database.get(&id) {
                        //     println!("Device Database: {:#?}", device.database.library);
                        // }
                    }
                    Some(DeviceManagerEvent::DeviceConnected(Err(err))) => {
                        if let OpenDeviceError::InvalidDeviceType = err {
                            continue;
                        }

                        println!("Device Connect Failed: {err}");
                    }
                    Some(DeviceManagerEvent::DeviceDisconnected(id)) => {
                        let _ = ui.send(UIMessage::DeviceDisconnected(id));

                        println!("Device {id} Disconnected");
                    }
                    None => {
                        println!("Device Manager Hung Up");
                        break;
                    }
                }
            }
            ui_event = ui.next() => {
                if let Some(ui_event) = ui_event {
                    match ui_event {
                        UIEvent::LoadTrack { device, id, playlist: _, deck } => {
                            if let Some(deck) = temp_deck_state.players.get_mut(deck)
                                && let Ok(locked_device_database) = device_manager.devices.lock()
                                && let Some(device) = locked_device_database.get(&device)
                                   && let Some(track) = device.database.library.tracks.get(&id){

                                deck.current_track = Some(track.clone());

                                deck.time = 0.0;
                                deck.slip_time = 0.0;
                                deck.cue_time = None;
                                deck.needle_time = None;

                                deck.beat_loop_start = None;
                                deck.beat_loop_end = None;

                                // node: this is bad, switch this when moving to libdj
                                deck.bpm = track.tempo;

                                deck.keyshift = 0;
                            }
                        },
                        UIEvent::Eject(deck) => {
                            if let Some(deck) = temp_deck_state.players.get_mut(deck) {
                               deck.current_track = None;

                               deck.time = 0.0;
                               deck.slip_time = 0.0;
                               deck.cue_time = None;
                               deck.needle_time = None;

                               deck.beat_loop_start = None;
                               deck.beat_loop_end = None;

                               deck.bpm = 0.0;

                               deck.keyshift = 0;
                            }
                        }
                        UIEvent::GetLibrary(id) => {
                            if let Ok(locked_device_database) = device_manager.devices.lock()
                                && let Some(device) = locked_device_database.get(&id) {
                                let _ = ui.send(UIMessage::DeviceLibrary{
                                    device: id,
                                    library: device.database.library.clone(),
                                });
                            }
                        },
                        UIEvent::NeedleSearch { needle_time, deck } => {
                            if let Some(deck) = temp_deck_state.players.get_mut(deck) {
                               deck.needle_time = needle_time;
                            }
                        },
                        UIEvent::BeatJump { beats, deck } => {
                            if let Some(deck) = temp_deck_state.players.get_mut(deck) {
                                deck.time += beats * (1. / deck.bpm) * 60.;
                            }
                        },
                        UIEvent::SetBeatLoop { beats, deck } => {
                            if let Some(deck) = temp_deck_state.players.get_mut(deck) {
                                deck.beat_loop_start = Some(deck.time);
                                deck.beat_loop_end = Some(deck.time + (beats * (1. / deck.bpm) * 60.));
                            }
                        },
                        UIEvent::DoubleBeatLoop(deck) => {
                            if let Some(deck) = temp_deck_state.players.get_mut(deck)
                               && let Some(beat_loop_start) = deck.beat_loop_start
                               && let Some(beat_loop_end) = deck.beat_loop_end {
                                deck.beat_loop_start = Some(deck.time);
                                deck.beat_loop_end = Some(beat_loop_end + (beat_loop_end - beat_loop_start));
                            }
                        },
                        UIEvent::HalveBeatLoop(deck) => {
                            if let Some(deck) = temp_deck_state.players.get_mut(deck)
                               && let Some(beat_loop_start) = deck.beat_loop_start
                               && let Some(beat_loop_end) = deck.beat_loop_end {
                                deck.beat_loop_start = Some(deck.time);
                                deck.beat_loop_end = Some(beat_loop_end - ((beat_loop_end - beat_loop_start) / 2.));
                            }
                        },
                        UIEvent::SetKeyShift { deck, semitones } => {
                            if let Some(deck) = temp_deck_state.players.get_mut(deck) {
                                deck.keyshift = semitones;
                            }
                        },
                    }

                    println!("UI Event: {ui_event:#?}");
                } else {
                    println!("UI Hung Up");
                    break;
                }
            }
            _ = ui_deck_update_interval.tick() => {
                let _ = ui.send(UIMessage::UpdateDeckState(temp_deck_state.clone()));
            }
            _ = deck_update_interval.tick() => {
                #[allow(clippy::explicit_iter_loop)] // i think this is more readable
                for deck in temp_deck_state.players.iter_mut() {
                    let tempo_percent = match deck.tempo_percent {
                        TempoPercent::Zero => 0.0,
                        TempoPercent::Percent(percent) => percent,
                    };

                    match deck.play_direction {
                        PlayDirection::Stop => {},
                        PlayDirection::Forward => {
                            deck.time += 0.001 * tempo_percent;
                            deck.slip_time += 0.001 * tempo_percent;
                        },
                        PlayDirection::Reverse => {
                            deck.time -= 0.001 * tempo_percent;
                            deck.slip_time -= 0.001 * tempo_percent;
                        },
                        PlayDirection::SlipReverse => {
                            deck.time -= 0.001 * tempo_percent;
                            deck.slip_time += 0.001 * tempo_percent;
                        },
                        PlayDirection::Jog => {
                            deck.slip_time = deck.time;
                        },
                        PlayDirection::SlipJog => {
                            deck.slip_time += 0.001 * tempo_percent;
                        },
                    }

                    if let Some(beat_loop_start) = deck.beat_loop_start
                        && let Some(beat_loop_end) = deck.beat_loop_end {
                        if deck.time < beat_loop_start {
                            deck.time += beat_loop_end - beat_loop_start;
                        } else if deck.time > beat_loop_end {
                            deck.time -= beat_loop_end - beat_loop_start;
                        }
                    }
                }
            }
        }
    }

    device_manager.stop().await;
}
