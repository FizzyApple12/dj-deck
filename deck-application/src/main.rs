use std::{thread, time::Duration};

use libdatabase::device_manager::{DeviceManager, DeviceManagerEvent, device::OpenDeviceError};
use libdj::{
    audio_system::{AudioManager, audio_loader::TrackAudioData},
    types::{
        analysis::WaveformType, audio_system::DeckUpdate, deck::DeckState, library::Track,
        timecode::Timecode,
    },
};
use libui::{
    controller::Controller,
    gui::UI,
    types::{
        controller::ControllerMessage,
        ui::{UIEvent, UIMessage},
    },
};
use tokio_stream::StreamExt;

#[allow(clippy::too_many_lines)]
#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (deck_state_sender, deck_state_receiver) =
        tokio::sync::watch::channel(DeckState::default());
    let (deck_update_sender, deck_update_receiver) = tokio::sync::mpsc::unbounded_channel();

    // todo: vsync?
    let mut ui_deck_update_interval = tokio::time::interval(Duration::from_millis(10));

    let (ui_event_sender, mut ui_event_receiver) = tokio::sync::mpsc::unbounded_channel();

    let (audio_system_event_sender, mut audio_system_event_receiver) =
        tokio::sync::mpsc::unbounded_channel();

    let (loaded_track_sender, loaded_track_receiver) = tokio::sync::mpsc::unbounded_channel();

    println!("Starting Audio Manager...");

    let mut audio_manager = AudioManager::new()?;

    println!("Finding DJ Deck Audio...");

    let dj_deck_device = audio_manager.find_output_device_by_name_substring("DDJ-FLX10")?;

    println!("Creating Full Audio Pipeline...");

    audio_manager.create_full_audio_pipeline(
        deck_update_receiver,
        audio_system_event_sender,
        deck_state_sender,
        loaded_track_receiver,
        dj_deck_device,
    )?;

    println!("Creating Midi IO...");

    let midi_receiver = audio_manager.create_midi_input()?;
    let midi_sender = audio_manager.create_midi_output()?;

    println!("Starting Audio...");

    audio_manager.start();

    println!("Starting Device Manager...");

    let mut device_manager = DeviceManager::start()?;

    println!("Starting HMI...");

    let controller = Controller::start_hmi(
        midi_sender,
        midi_receiver,
        deck_update_sender.clone(),
        ui_event_sender.clone(),
    );

    println!("Starting GUI...");

    let mut ui = UI::start_ui(deck_update_sender.clone()).await?;

    println!("Ready");

    loop {
        tokio::select! {
            device_event = device_manager.next() => {
                match device_event {
                    Some(DeviceManagerEvent::DeviceConnected(Ok(id))) => {
                        println!("Device {id} Connected Successfully");

                        let name = device_manager.devices.lock().unwrap().get(&id).unwrap().name.clone();

                        let _ = ui.send(UIMessage::DeviceConnected(id, name));
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
                    println!("UI Event: {ui_event:#?}");

                    match ui_event {
                        UIEvent::LoadTrack { device, id, player } => {
                            if let Ok(mut locked_device_database) = device_manager.devices.lock()
                                && let Some(qualified_device) = locked_device_database.get_mut(&device)
                                && let Some(track) = qualified_device.database.library.tracks.get(&id) {
                                let _ = loaded_track_sender.send((player, None));

                                let _ = deck_update_sender.send(Box::new(move |deck_state, _| {
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

                                // todo: this should be moved to a new thread
                                if let Ok(track_analysis) = qualified_device.database.load_analysis(id) {
                                    let cloned_track_analysis = track_analysis.clone();

                                    let _ = deck_update_sender.send(Box::new(move |deck_state, _| {
                                        if let Some(mixer_channel) = deck_state.mixer_channels.get_mut(player) {
                                            mixer_channel.player.current_track_analysis = Some(cloned_track_analysis);
                                        }
                                    }));

                                    let _ = ui.send(UIMessage::TrackAnalysis { player, analysis: track_analysis });
                                } else {
                                    // todo: perform track analysis
                                }

                                start_load_task(player, device, track.clone(), deck_update_sender.clone(), loaded_track_sender.clone());

                                if let Ok(waveform) = qualified_device.database.load_waveform(id, WaveformType::ThreeBand) {
                                     let _ = ui.send(UIMessage::Waveform { player, waveform });
                                }
                                if let Ok(waveform) = qualified_device.database.load_preview_waveform(id, WaveformType::ThreeBand) {
                                     let _ = ui.send(UIMessage::PreviewWaveform { player, waveform });
                                }
                            }
                        },
                        UIEvent::Eject(player) => {
                            let _ = deck_update_sender.send(Box::new(move |deck_state, _| {
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
                        UIEvent::GetLibrary(id) => {
                            if let Ok(locked_device_database) = device_manager.devices.lock()
                                && let Some(device) = locked_device_database.get(&id) {
                                let _ = ui.send(UIMessage::DeviceLibrary{
                                    device: id,
                                    library: device.database.library.clone(),
                                });
                            }
                        },
                        UIEvent::EjectDevice(id) => {
                            device_manager.eject(id).await;
                        },
                        UIEvent::GetWaveform { device, id, player } =>  {
                            if let Ok(mut locked_device_database) = device_manager.devices.lock()
                                && let Some(qualified_device) = locked_device_database.get_mut(&device)
                                && let Ok(waveform) = qualified_device.database.load_waveform(id, WaveformType::ThreeBand) {
                                 let _ = ui.send(UIMessage::Waveform { player, waveform });
                            }
                        },
                        UIEvent::GetPreviewWaveform { device, id, player } =>  {
                            if let Ok(mut locked_device_database) = device_manager.devices.lock()
                                && let Some(qualified_device) = locked_device_database.get_mut(&device)
                                && let Ok(waveform) = qualified_device.database.load_preview_waveform(id, WaveformType::ThreeBand) {
                                 let _ = ui.send(UIMessage::PreviewWaveform { player, waveform });
                            }
                        },
                    }
                } else {
                    println!("UI Hung Up");
                    break;
                }
            }
            audio_system_event = audio_system_event_receiver.recv() => {
                if let Some(audio_system_event) = audio_system_event {
                    match audio_system_event {
                    }
                }
            }
            _ = ui_deck_update_interval.tick() => {
                let deck_state = deck_state_receiver.borrow();

                let _ = controller.send(ControllerMessage::UpdateDeckState(deck_state.clone()));
                let _ = controller.send(ControllerMessage::UpdateCurrentSamples([0.0, 0.0, 0.0, 0.0]));

                let _ = ui.send(UIMessage::UpdateDeckState(deck_state.clone()));

                while let Ok(event) = ui_event_receiver.try_recv() {
                    let _ = ui.send(event);
                }
            }
        }
    }

    drop(ui);
    drop(controller);

    device_manager.stop().await;

    Ok(())
}

pub fn start_load_task(
    player: usize,
    device: u32,
    track: Track,
    deck_update_sender: tokio::sync::mpsc::UnboundedSender<DeckUpdate>,
    loaded_track_sender: tokio::sync::mpsc::UnboundedSender<(usize, Option<Box<TrackAudioData>>)>,
) {
    thread::spawn(move || {
        let audio_data = match TrackAudioData::load_from_file(&track) {
            Ok(audio_data) => audio_data,
            Err(err) => {
                // todo: build a way to propagate errors to the ui
                println!("TRACK LOAD ERROR: {err:?}");

                let _ = deck_update_sender.send(Box::new(move |deck_state, _| {
                    if let Some(mixer_channel) = deck_state.mixer_channels.get_mut(player) {
                        mixer_channel.player.is_loading = false;
                    }
                }));

                return;
            }
        };

        let _ = loaded_track_sender.send((player, Some(Box::new(audio_data))));

        let _ = deck_update_sender.send(Box::new(move |deck_state, _| {
            if let Some(mixer_channel) = deck_state.mixer_channels.get_mut(player) {
                mixer_channel.player.current_track = Some((device, track));
                mixer_channel.player.is_loading = false;
            }
        }));
    });
}
