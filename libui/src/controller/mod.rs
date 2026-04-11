use std::time::Duration;

use libdj::types::{
    deck::{BeatLoopAdjustMode, BeatSyncMode, DeckState, DeckUpdate, PlayState},
    midi::MidiMessage,
};
use thiserror::Error;
use tokio::task::JoinHandle;

use crate::types::controller::ControllerMessage;

#[derive(Debug)]
pub struct Controller {
    controller_message_sender: tokio::sync::broadcast::Sender<ControllerMessage>,

    midi_task: JoinHandle<()>,
}

struct ControllerLEDFlashTimers {
    slow: bool,
    mid: bool,
    fast: bool,
}

#[derive(Error, Debug)]
pub enum ControllerSendError {
    #[error("Event Loop Error: {0}")]
    ChannelError(tokio::sync::broadcast::error::SendError<ControllerMessage>),
}

impl Controller {
    #[allow(clippy::too_many_lines)]
    pub fn start_ui(
        midi_sender: tokio::sync::mpsc::UnboundedSender<MidiMessage>,
        mut midi_receiver: tokio::sync::mpsc::UnboundedReceiver<MidiMessage>,
        deck_update_sender: tokio::sync::mpsc::UnboundedSender<DeckUpdate>,
    ) -> Controller {
        let (controller_message_sender, mut controller_message_receiver) =
            tokio::sync::broadcast::channel(16);

        let midi_task = tokio::task::spawn(async move {
            let mut current_deck_state: DeckState = DeckState::default();

            let mut led_flash_timers = ControllerLEDFlashTimers {
                slow: false,
                mid: false,
                fast: false,
            };

            let mut slow_flash_interval = tokio::time::interval(Duration::from_millis(500));
            let mut mid_flash_interval = tokio::time::interval(Duration::from_millis(250));
            let mut fast_flash_interval = tokio::time::interval(Duration::from_millis(100));

            loop {
                tokio::select! {
                    _ = slow_flash_interval.tick() => {
                        led_flash_timers.slow = !led_flash_timers.slow;

                        set_deck_leds(&current_deck_state, &led_flash_timers, &midi_sender);
                    }
                    _ = mid_flash_interval.tick() => {
                        led_flash_timers.mid = !led_flash_timers.mid;

                        set_deck_leds(&current_deck_state, &led_flash_timers, &midi_sender);
                    }
                    _ = fast_flash_interval.tick() => {
                        led_flash_timers.fast = !led_flash_timers.fast;

                        set_deck_leds(&current_deck_state, &led_flash_timers, &midi_sender);
                    }
                    midi_message = midi_receiver.recv() => {
                        if let Some(midi_message) = midi_message {
                            let (address, value) = midi_to_addr_value(midi_message);

                            process_midi_command(address, value, &deck_update_sender);
                        }
                    }
                    controller_message = controller_message_receiver.recv() => {
                        if let Ok(controller_message) = controller_message {
                            match controller_message {
                                ControllerMessage::UpdateDeckState(deck_state) => {
                                    current_deck_state = deck_state;

                                    set_deck_leds(&current_deck_state, &led_flash_timers, &midi_sender);
                                },
                                ControllerMessage::UpdateCurrentSamples(samples) => {
                                    for (deck, sample) in samples.into_iter().enumerate() {
                                        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                                        let sample = f32::min(f32::abs(sample) * 127.0, 127.0) as u8;

                                        #[allow(clippy::cast_possible_truncation)]
                                        let _ = midi_sender.send(addr_channel_value_to_midi(0xB002, deck as u16, sample));
                                    }
                                },
                            }
                        }
                    }
                }
            }
        });

        Controller {
            controller_message_sender,

            midi_task,
        }
    }

    pub fn send(self: &Controller, message: ControllerMessage) -> Result<(), ControllerSendError> {
        match self.controller_message_sender.send(message) {
            Ok(_) => Ok(()),
            Err(err) => Err(ControllerSendError::ChannelError(err)),
        }
    }
}

impl Drop for Controller {
    fn drop(&mut self) {
        self.midi_task.abort();
    }
}

const fn combine_addr_channel(addr: u16, channel: u16) -> u16 {
    addr + (channel << 8)
}

const fn addr_value_to_midi(addr: u16, command: u8) -> [u8; 3] {
    #[allow(clippy::cast_possible_truncation)]
    [(addr >> 8) as u8, (addr & 0xFF) as u8, command]
}

const fn addr_channel_value_to_midi(addr: u16, channel: u16, command: u8) -> [u8; 3] {
    let addr = combine_addr_channel(addr, channel);

    #[allow(clippy::cast_possible_truncation)]
    [(addr >> 8) as u8, (addr & 0xFF) as u8, command]
}

const fn midi_to_addr_value(midi: [u8; 3]) -> (u16, u8) {
    (((midi[0] as u16) << 8) + (midi[1] as u16), midi[2])
}

fn process_midi_command(
    address: u16,
    value: u8,
    deck_update_sender: &tokio::sync::mpsc::UnboundedSender<DeckUpdate>,
) {
    match (address, value) {
        // play
        (address, 127) if address == combine_addr_channel(0x900B, 0) => {
            deck_update_sender
                .send(Box::new(|deck_state| {
                    let play_direction = &mut deck_state.mixer_channels[0].player.play_state;

                    match play_direction {
                        PlayState::Cue | PlayState::Stop => *play_direction = PlayState::Play,
                        PlayState::Play => *play_direction = PlayState::Stop,
                    }
                }))
                .ok();
        }
        (address, 127) if address == combine_addr_channel(0x900B, 1) => {
            deck_update_sender
                .send(Box::new(|deck_state| {
                    let play_direction = &mut deck_state.mixer_channels[1].player.play_state;

                    match play_direction {
                        PlayState::Cue | PlayState::Stop => *play_direction = PlayState::Play,
                        PlayState::Play => *play_direction = PlayState::Stop,
                    }
                }))
                .ok();
        }
        (address, 127) if address == combine_addr_channel(0x900B, 2) => {
            deck_update_sender
                .send(Box::new(|deck_state| {
                    let play_direction = &mut deck_state.mixer_channels[2].player.play_state;

                    match play_direction {
                        PlayState::Cue | PlayState::Stop => *play_direction = PlayState::Play,
                        PlayState::Play => *play_direction = PlayState::Stop,
                    }
                }))
                .ok();
        }
        (address, 127) if address == combine_addr_channel(0x900B, 3) => {
            deck_update_sender
                .send(Box::new(|deck_state| {
                    let play_direction = &mut deck_state.mixer_channels[3].player.play_state;

                    match play_direction {
                        PlayState::Cue | PlayState::Stop => *play_direction = PlayState::Play,
                        PlayState::Play => *play_direction = PlayState::Stop,
                    }
                }))
                .ok();
        }
        // cue
        (address, 0) if address == combine_addr_channel(0x900C, 0) => {
            deck_update_sender
                .send(Box::new(|deck_state| {
                    let play_direction = &mut deck_state.mixer_channels[0].player.play_state;
                    let cue_time = &mut deck_state.mixer_channels[0].player.cue_time;

                    if let PlayState::Cue = play_direction {
                        *play_direction = PlayState::Stop;
                        if let Some(position) = cue_time {
                            deck_state.mixer_channels[0].player.time = *position;
                        }
                    }
                }))
                .ok();
        }
        (address, 127) if address == combine_addr_channel(0x900C, 0) => {
            deck_update_sender
                .send(Box::new(|deck_state| {
                    deck_state.mixer_channels[0].player.play_state = PlayState::Cue;
                    deck_state.mixer_channels[0].player.cue_time =
                        Some(deck_state.mixer_channels[0].player.time);
                }))
                .ok();
        }
        (address, 0) if address == combine_addr_channel(0x900C, 1) => {
            deck_update_sender
                .send(Box::new(|deck_state| {
                    let play_direction = &mut deck_state.mixer_channels[0].player.play_state;
                    let cue_time = &mut deck_state.mixer_channels[0].player.cue_time;

                    if let PlayState::Cue = play_direction {
                        *play_direction = PlayState::Stop;
                        if let Some(position) = cue_time {
                            deck_state.mixer_channels[0].player.time = *position;
                        }
                    }
                }))
                .ok();
        }
        (address, 127) if address == combine_addr_channel(0x900C, 1) => {
            deck_update_sender
                .send(Box::new(|deck_state| {
                    deck_state.mixer_channels[0].player.play_state = PlayState::Cue;
                    deck_state.mixer_channels[0].player.cue_time =
                        Some(deck_state.mixer_channels[0].player.time);
                }))
                .ok();
        }
        (address, 0) if address == combine_addr_channel(0x900C, 2) => {
            deck_update_sender
                .send(Box::new(|deck_state| {
                    let play_direction = &mut deck_state.mixer_channels[0].player.play_state;
                    let cue_time = &mut deck_state.mixer_channels[0].player.cue_time;

                    if let PlayState::Cue = play_direction {
                        *play_direction = PlayState::Stop;
                        if let Some(position) = cue_time {
                            deck_state.mixer_channels[0].player.time = *position;
                        }
                    }
                }))
                .ok();
        }
        (address, 127) if address == combine_addr_channel(0x900C, 2) => {
            deck_update_sender
                .send(Box::new(|deck_state| {
                    deck_state.mixer_channels[0].player.play_state = PlayState::Cue;
                    deck_state.mixer_channels[0].player.cue_time =
                        Some(deck_state.mixer_channels[0].player.time);
                }))
                .ok();
        }
        (address, 0) if address == combine_addr_channel(0x900C, 3) => {
            deck_update_sender
                .send(Box::new(|deck_state| {
                    let play_direction = &mut deck_state.mixer_channels[0].player.play_state;
                    let cue_time = &mut deck_state.mixer_channels[0].player.cue_time;

                    if let PlayState::Cue = play_direction {
                        *play_direction = PlayState::Stop;
                        if let Some(position) = cue_time {
                            deck_state.mixer_channels[0].player.time = *position;
                        }
                    }
                }))
                .ok();
        }
        (address, 127) if address == combine_addr_channel(0x900C, 3) => {
            deck_update_sender
                .send(Box::new(|deck_state| {
                    deck_state.mixer_channels[0].player.play_state = PlayState::Cue;
                    deck_state.mixer_channels[0].player.cue_time =
                        Some(deck_state.mixer_channels[0].player.time);
                }))
                .ok();
        }
        _ => {
            println!("unknown command: {address:x} = {value}");
        }
    }
}

// we use the same safe code a lot here, and there's a lot of code here to
// handle, this is probably cleanupable later
#[allow(clippy::cast_possible_truncation, clippy::too_many_lines)]
fn set_deck_leds(
    deck_state: &DeckState,
    flash_timers: &ControllerLEDFlashTimers,
    midi_output: &tokio::sync::mpsc::UnboundedSender<MidiMessage>,
) {
    // master cue
    let _ = midi_output.send(addr_value_to_midi(
        0x9662,
        if deck_state.master_cue { 127 } else { 0 },
    ));
    let _ = midi_output.send(addr_value_to_midi(
        0x9663,
        if deck_state.master_cue { 127 } else { 0 },
    ));

    for (channel, channel_state) in deck_state.mixer_channels.iter().enumerate() {
        let player_state = &channel_state.player;

        // cue
        #[allow(clippy::indexing_slicing)]
        let _ = midi_output.send(addr_channel_value_to_midi(
            0x9054,
            channel as u16,
            if channel_state.cue || player_state.touch_cue_time.is_some() {
                127
            } else {
                0
            },
        ));

        let is_loaded = player_state.current_track.is_some();

        // master channel led
        if let Some(master_channel) = deck_state.master_channel {
            let _ = midi_output.send(addr_channel_value_to_midi(
                0xB002,
                channel as u16,
                if channel == master_channel { 127 } else { 0 },
            ));
        } else {
            let _ = midi_output.send(addr_channel_value_to_midi(0xB002, channel as u16, 0));
        }

        // vinyl
        let _ = midi_output.send(addr_channel_value_to_midi(0x9017, channel as u16, 127));

        // todo: load indicator

        // beat sync
        let _ = midi_output.send(addr_channel_value_to_midi(
            0x9058,
            channel as u16,
            match player_state.beat_sync {
                BeatSyncMode::Off => 0,
                BeatSyncMode::BPMSync => {
                    if flash_timers.fast {
                        127
                    } else {
                        0
                    }
                }
                BeatSyncMode::BeatSync => 127,
            },
        ));

        // key sync
        let _ = midi_output.send(addr_channel_value_to_midi(
            0x9065,
            channel as u16,
            if player_state.key_sync { 127 } else { 0 },
        ));

        // quantize
        let _ = midi_output.send(addr_channel_value_to_midi(
            0x9035,
            channel as u16,
            if player_state.quanitze { 127 } else { 0 },
        ));

        // slip
        let _ = midi_output.send(addr_channel_value_to_midi(
            0x9040,
            channel as u16,
            if player_state.slip { 127 } else { 0 },
        ));

        // reverse
        let _ = midi_output.send(addr_channel_value_to_midi(
            0x9038,
            channel as u16,
            if player_state.reverse_enabled { 127 } else { 0 },
        ));

        if is_loaded {
            // play
            match player_state.play_state {
                PlayState::Play => {
                    let _ =
                        midi_output.send(addr_channel_value_to_midi(0x900B, channel as u16, 127));
                    let _ =
                        midi_output.send(addr_channel_value_to_midi(0x9047, channel as u16, 127));
                }
                PlayState::Cue | PlayState::Stop => {
                    let _ = midi_output.send(addr_channel_value_to_midi(
                        0x900B,
                        channel as u16,
                        if flash_timers.slow { 127 } else { 0 },
                    ));
                    let _ = midi_output.send(addr_channel_value_to_midi(
                        0x9047,
                        channel as u16,
                        if flash_timers.slow { 127 } else { 0 },
                    ));
                }
            }

            // cue
            match player_state.play_state {
                PlayState::Play | PlayState::Cue => {
                    let _ =
                        midi_output.send(addr_channel_value_to_midi(0x900C, channel as u16, 127));
                    let _ =
                        midi_output.send(addr_channel_value_to_midi(0x9048, channel as u16, 127));
                }
                PlayState::Stop => {
                    if let Some(cue_time) = player_state.cue_time {
                        if f32::abs(player_state.time - cue_time) < f32::EPSILON {
                            let _ = midi_output.send(addr_channel_value_to_midi(
                                0x900C,
                                channel as u16,
                                127,
                            ));
                            let _ = midi_output.send(addr_channel_value_to_midi(
                                0x9048,
                                channel as u16,
                                127,
                            ));
                        } else {
                            let _ = midi_output.send(addr_channel_value_to_midi(
                                0x900C,
                                channel as u16,
                                if flash_timers.mid { 127 } else { 0 },
                            ));
                            let _ = midi_output.send(addr_channel_value_to_midi(
                                0x9048,
                                channel as u16,
                                if flash_timers.mid { 127 } else { 0 },
                            ));
                        }
                    } else {
                        let _ = midi_output.send(addr_channel_value_to_midi(
                            0x900C,
                            channel as u16,
                            if flash_timers.mid { 127 } else { 0 },
                        ));
                        let _ = midi_output.send(addr_channel_value_to_midi(
                            0x9048,
                            channel as u16,
                            if flash_timers.mid { 127 } else { 0 },
                        ));
                    }
                }
            }
        } else {
            let _ = midi_output.send(addr_channel_value_to_midi(0x900B, channel as u16, 0));
            let _ = midi_output.send(addr_channel_value_to_midi(0x9047, channel as u16, 0));

            let _ = midi_output.send(addr_channel_value_to_midi(0x900C, channel as u16, 0));
            let _ = midi_output.send(addr_channel_value_to_midi(0x9048, channel as u16, 0));
        }

        // beat loop
        match (player_state.beat_loop_start, player_state.beat_loop_end) {
            (None, None) => {
                // in
                let _ = midi_output.send(addr_channel_value_to_midi(0x9010, channel as u16, 127));
                let _ = midi_output.send(addr_channel_value_to_midi(0x9012, channel as u16, 127));
                let _ = midi_output.send(addr_channel_value_to_midi(0x904C, channel as u16, 0));

                // out
                let _ = midi_output.send(addr_channel_value_to_midi(0x9011, channel as u16, 127));
                let _ = midi_output.send(addr_channel_value_to_midi(0x9013, channel as u16, 127));
                let _ = midi_output.send(addr_channel_value_to_midi(
                    0x904D,
                    channel as u16,
                    if player_state.last_beat_loop.is_some() {
                        127
                    } else {
                        0
                    },
                ));

                // 4 beat
                let _ = midi_output.send(addr_channel_value_to_midi(0x9014, channel as u16, 0));
                let _ = midi_output.send(addr_channel_value_to_midi(0x9050, channel as u16, 0));
            }
            (Some(_), None) => {
                // in
                let in_state = if flash_timers.fast { 127 } else { 0 };
                let _ =
                    midi_output.send(addr_channel_value_to_midi(0x9010, channel as u16, in_state));
                let _ =
                    midi_output.send(addr_channel_value_to_midi(0x9012, channel as u16, in_state));
                let _ =
                    midi_output.send(addr_channel_value_to_midi(0x904C, channel as u16, in_state));

                // out
                let _ = midi_output.send(addr_channel_value_to_midi(0x9011, channel as u16, 127));
                let _ = midi_output.send(addr_channel_value_to_midi(0x9013, channel as u16, 127));
                let _ = midi_output.send(addr_channel_value_to_midi(0x904D, channel as u16, 127));

                // 4 beat
                let _ = midi_output.send(addr_channel_value_to_midi(0x9014, channel as u16, 0));
                let _ = midi_output.send(addr_channel_value_to_midi(0x9050, channel as u16, 0));
            }
            (None, Some(_)) => {
                // in
                let _ = midi_output.send(addr_channel_value_to_midi(0x9010, channel as u16, 127));
                let _ = midi_output.send(addr_channel_value_to_midi(0x9012, channel as u16, 127));
                let _ = midi_output.send(addr_channel_value_to_midi(0x904C, channel as u16, 127));

                // out
                let out_state = if flash_timers.fast { 127 } else { 0 };
                let _ = midi_output.send(addr_channel_value_to_midi(
                    0x9011,
                    channel as u16,
                    out_state,
                ));
                let _ = midi_output.send(addr_channel_value_to_midi(
                    0x9013,
                    channel as u16,
                    out_state,
                ));
                let _ = midi_output.send(addr_channel_value_to_midi(
                    0x904D,
                    channel as u16,
                    out_state,
                ));

                // 4 beat
                let _ = midi_output.send(addr_channel_value_to_midi(0x9014, channel as u16, 0));
                let _ = midi_output.send(addr_channel_value_to_midi(0x9050, channel as u16, 0));
            }
            (Some(_), Some(_)) => {
                // in
                let in_state = if let BeatLoopAdjustMode::In = player_state.beat_loop_adjust_mode {
                    if flash_timers.fast { 127 } else { 0 }
                } else {
                    if flash_timers.fast { 127 } else { 0 }
                };
                let _ =
                    midi_output.send(addr_channel_value_to_midi(0x9010, channel as u16, in_state));
                let _ =
                    midi_output.send(addr_channel_value_to_midi(0x9012, channel as u16, in_state));
                let _ =
                    midi_output.send(addr_channel_value_to_midi(0x904C, channel as u16, in_state));

                // out
                let out_state = if let BeatLoopAdjustMode::Out = player_state.beat_loop_adjust_mode
                {
                    if flash_timers.fast { 127 } else { 0 }
                } else {
                    if flash_timers.fast { 127 } else { 0 }
                };
                let _ = midi_output.send(addr_channel_value_to_midi(
                    0x9011,
                    channel as u16,
                    out_state,
                ));
                let _ = midi_output.send(addr_channel_value_to_midi(
                    0x9013,
                    channel as u16,
                    out_state,
                ));
                let _ = midi_output.send(addr_channel_value_to_midi(
                    0x904D,
                    channel as u16,
                    out_state,
                ));

                // 4 beat
                let _ = midi_output.send(addr_channel_value_to_midi(0x9014, channel as u16, 127));
                let _ = midi_output.send(addr_channel_value_to_midi(0x9050, channel as u16, 127));
            }
        }

        // tempo reset
        let _ = midi_output.send(addr_channel_value_to_midi(
            0x9041,
            channel as u16,
            if player_state.tempo_reset { 127 } else { 0 },
        ));

        // master tempo
        let _ = midi_output.send(addr_channel_value_to_midi(
            0x901A,
            channel as u16,
            if player_state.master_tempo { 127 } else { 0 },
        ));
    }
}
