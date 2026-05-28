pub mod actions;

use std::time::Duration;

use libdj::types::{
    audio_system::DeckUpdate,
    deck::{BeatLoopAdjustMode, BeatSyncMode, DeckState, PlayState},
    midi::MidiMessage,
};
use thiserror::Error;
use tokio::task::JoinHandle;

use crate::{
    controller::actions::{
        beat_jump_release, cue_press, cue_release, jog_distance, jog_release, jog_touch, play_press,
    },
    types::{controller::ControllerMessage, ui::UIMessage},
};

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
    pub fn start_hmi(
        midi_sender: tokio::sync::mpsc::UnboundedSender<MidiMessage>,
        mut midi_receiver: tokio::sync::mpsc::UnboundedReceiver<MidiMessage>,
        deck_update_sender: tokio::sync::mpsc::UnboundedSender<DeckUpdate>,
        ui_message_sender: tokio::sync::mpsc::UnboundedSender<UIMessage>,
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

                            process_midi_command(address, value, &deck_update_sender, &ui_message_sender);
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

#[allow(clippy::too_many_lines)]
fn process_midi_command(
    address: u16,
    value: u8,
    deck_update_sender: &tokio::sync::mpsc::UnboundedSender<DeckUpdate>,
    ui_message_sender: &tokio::sync::mpsc::UnboundedSender<UIMessage>,
) {
    // println!("command: {address:x} = {value}");

    match (address, value) {
        // encoder rotation
        (0xB640 | 0xB664, count) => {
            if count <= 60 {
                for _ in 0..count {
                    let _ = ui_message_sender.send(UIMessage::EncoderDown);
                }
            } else {
                for _ in 0..(128 - count) {
                    let _ = ui_message_sender.send(UIMessage::EncoderUp);
                }
            }
        }
        // encoder press
        #[allow(clippy::unnested_or_patterns)]
        (0x9646..0x9649, _) | (0x965D | 0x966D | 0x965E | 0x966F, _) => {
            let _ = ui_message_sender.send(UIMessage::EncoderSelect);
        }
        // jog
        (address, velocity)
            if address == combine_addr_channel(0xB022, 0)
                || address == combine_addr_channel(0xB023, 0)
                || address == combine_addr_channel(0xB021, 0)
                || address == combine_addr_channel(0xB026, 0) =>
        {
            deck_update_sender
                .send(Box::new(move |deck_state, _| {
                    jog_distance(deck_state, 0, false, velocity);
                }))
                .ok();
        }
        (address, velocity)
            if address == combine_addr_channel(0xB029, 0)
                || address == combine_addr_channel(0xB01F, 0) =>
        {
            deck_update_sender
                .send(Box::new(move |deck_state, _| {
                    jog_distance(deck_state, 0, true, velocity);
                }))
                .ok();
        }
        (address, 127)
            if address == combine_addr_channel(0x9036, 0)
                || address == combine_addr_channel(0x9067, 0) =>
        {
            deck_update_sender
                .send(Box::new(move |deck_state, _| {
                    jog_touch(deck_state, 0);
                }))
                .ok();
        }
        (address, 0)
            if address == combine_addr_channel(0x9036, 0)
                || address == combine_addr_channel(0x9067, 0) =>
        {
            deck_update_sender
                .send(Box::new(move |deck_state, _| {
                    jog_release(deck_state, 0);
                }))
                .ok();
        }
        (address, velocity)
            if address == combine_addr_channel(0xB022, 1)
                || address == combine_addr_channel(0xB023, 1)
                || address == combine_addr_channel(0xB021, 1)
                || address == combine_addr_channel(0xB026, 1) =>
        {
            deck_update_sender
                .send(Box::new(move |deck_state, _| {
                    jog_distance(deck_state, 1, false, velocity);
                }))
                .ok();
        }
        (address, velocity)
            if address == combine_addr_channel(0xB029, 1)
                || address == combine_addr_channel(0xB01F, 1) =>
        {
            deck_update_sender
                .send(Box::new(move |deck_state, _| {
                    jog_distance(deck_state, 1, true, velocity);
                }))
                .ok();
        }
        (address, 127)
            if address == combine_addr_channel(0x9036, 1)
                || address == combine_addr_channel(0x9067, 1) =>
        {
            deck_update_sender
                .send(Box::new(move |deck_state, _| {
                    jog_touch(deck_state, 1);
                }))
                .ok();
        }
        (address, 0)
            if address == combine_addr_channel(0x9036, 1)
                || address == combine_addr_channel(0x9067, 1) =>
        {
            deck_update_sender
                .send(Box::new(move |deck_state, _| {
                    jog_release(deck_state, 1);
                }))
                .ok();
        }
        (address, velocity)
            if address == combine_addr_channel(0xB022, 2)
                || address == combine_addr_channel(0xB023, 2)
                || address == combine_addr_channel(0xB021, 2)
                || address == combine_addr_channel(0xB026, 2) =>
        {
            deck_update_sender
                .send(Box::new(move |deck_state, _| {
                    jog_distance(deck_state, 2, false, velocity);
                }))
                .ok();
        }
        (address, velocity)
            if address == combine_addr_channel(0xB029, 2)
                || address == combine_addr_channel(0xB01F, 2) =>
        {
            deck_update_sender
                .send(Box::new(move |deck_state, _| {
                    jog_distance(deck_state, 2, true, velocity);
                }))
                .ok();
        }
        (address, 127)
            if address == combine_addr_channel(0x9036, 2)
                || address == combine_addr_channel(0x9067, 2) =>
        {
            deck_update_sender
                .send(Box::new(move |deck_state, _| {
                    jog_touch(deck_state, 2);
                }))
                .ok();
        }
        (address, 0)
            if address == combine_addr_channel(0x9036, 2)
                || address == combine_addr_channel(0x9067, 2) =>
        {
            deck_update_sender
                .send(Box::new(move |deck_state, _| {
                    jog_release(deck_state, 2);
                }))
                .ok();
        }
        (address, velocity)
            if address == combine_addr_channel(0xB022, 3)
                || address == combine_addr_channel(0xB023, 3)
                || address == combine_addr_channel(0xB021, 3)
                || address == combine_addr_channel(0xB026, 3) =>
        {
            deck_update_sender
                .send(Box::new(move |deck_state, _| {
                    jog_distance(deck_state, 3, false, velocity);
                }))
                .ok();
        }
        (address, velocity)
            if address == combine_addr_channel(0xB029, 3)
                || address == combine_addr_channel(0xB01F, 3) =>
        {
            deck_update_sender
                .send(Box::new(move |deck_state, _| {
                    jog_distance(deck_state, 3, true, velocity);
                }))
                .ok();
        }
        (address, 127)
            if address == combine_addr_channel(0x9036, 3)
                || address == combine_addr_channel(0x9067, 3) =>
        {
            deck_update_sender
                .send(Box::new(move |deck_state, _| {
                    jog_touch(deck_state, 3);
                }))
                .ok();
        }
        (address, 0)
            if address == combine_addr_channel(0x9036, 3)
                || address == combine_addr_channel(0x9067, 3) =>
        {
            deck_update_sender
                .send(Box::new(move |deck_state, _| {
                    jog_release(deck_state, 3);
                }))
                .ok();
        }
        // play
        (address, 127)
            if address == combine_addr_channel(0x900B, 0)
                || address == combine_addr_channel(0x9047, 0) =>
        {
            deck_update_sender
                .send(Box::new(|deck_state, _| play_press(deck_state, 0)))
                .ok();
        }
        (address, 127)
            if address == combine_addr_channel(0x900B, 1)
                || address == combine_addr_channel(0x9047, 1) =>
        {
            deck_update_sender
                .send(Box::new(|deck_state, _| play_press(deck_state, 1)))
                .ok();
        }
        (address, 127)
            if address == combine_addr_channel(0x900B, 2)
                || address == combine_addr_channel(0x9047, 2) =>
        {
            deck_update_sender
                .send(Box::new(|deck_state, _| play_press(deck_state, 2)))
                .ok();
        }
        (address, 127)
            if address == combine_addr_channel(0x900B, 3)
                || address == combine_addr_channel(0x9047, 3) =>
        {
            deck_update_sender
                .send(Box::new(|deck_state, _| play_press(deck_state, 3)))
                .ok();
        }
        // cue
        (address, 127) if address == combine_addr_channel(0x900C, 0) => {
            deck_update_sender
                .send(Box::new(|deck_state, _| cue_press(deck_state, 0, false)))
                .ok();
        }
        (address, 127) if address == combine_addr_channel(0x9048, 0) => {
            deck_update_sender
                .send(Box::new(|deck_state, _| cue_press(deck_state, 0, true)))
                .ok();
        }
        (address, 0)
            if address == combine_addr_channel(0x900C, 0)
                || address == combine_addr_channel(0x9048, 0) =>
        {
            deck_update_sender
                .send(Box::new(|deck_state, _| cue_release(deck_state, 0)))
                .ok();
        }
        (address, 127) if address == combine_addr_channel(0x900C, 1) => {
            deck_update_sender
                .send(Box::new(|deck_state, _| cue_press(deck_state, 1, false)))
                .ok();
        }
        (address, 127) if address == combine_addr_channel(0x9048, 1) => {
            deck_update_sender
                .send(Box::new(|deck_state, _| cue_press(deck_state, 1, true)))
                .ok();
        }
        (address, 0)
            if address == combine_addr_channel(0x900C, 1)
                || address == combine_addr_channel(0x9048, 1) =>
        {
            deck_update_sender
                .send(Box::new(|deck_state, _| cue_release(deck_state, 1)))
                .ok();
        }
        (address, 127) if address == combine_addr_channel(0x900C, 2) => {
            deck_update_sender
                .send(Box::new(|deck_state, _| cue_press(deck_state, 2, false)))
                .ok();
        }
        (address, 127) if address == combine_addr_channel(0x9048, 2) => {
            deck_update_sender
                .send(Box::new(|deck_state, _| cue_press(deck_state, 2, true)))
                .ok();
        }
        (address, 0)
            if address == combine_addr_channel(0x900C, 2)
                || address == combine_addr_channel(0x9048, 2) =>
        {
            deck_update_sender
                .send(Box::new(|deck_state, _| cue_release(deck_state, 2)))
                .ok();
        }
        (address, 127) if address == combine_addr_channel(0x900C, 3) => {
            deck_update_sender
                .send(Box::new(|deck_state, _| cue_press(deck_state, 3, false)))
                .ok();
        }
        (address, 127) if address == combine_addr_channel(0x9048, 3) => {
            deck_update_sender
                .send(Box::new(|deck_state, _| cue_press(deck_state, 3, true)))
                .ok();
        }
        (address, 0)
            if address == combine_addr_channel(0x900C, 3)
                || address == combine_addr_channel(0x9048, 3) =>
        {
            deck_update_sender
                .send(Box::new(|deck_state, _| cue_release(deck_state, 3)))
                .ok();
        }
        // beat jump
        (address, 0) if address == combine_addr_channel(0x905E, 0) => {
            deck_update_sender
                .send(Box::new(|deck_state, _| {
                    beat_jump_release(deck_state, 0, false, false);
                }))
                .ok();
        }
        (address, 0) if address == combine_addr_channel(0x905F, 0) => {
            deck_update_sender
                .send(Box::new(|deck_state, _| {
                    beat_jump_release(deck_state, 0, true, false);
                }))
                .ok();
        }
        (address, 0) if address == combine_addr_channel(0x9061, 0) => {
            deck_update_sender
                .send(Box::new(|deck_state, _| {
                    beat_jump_release(deck_state, 0, false, true);
                }))
                .ok();
        }
        (address, 0) if address == combine_addr_channel(0x9062, 0) => {
            deck_update_sender
                .send(Box::new(|deck_state, _| {
                    beat_jump_release(deck_state, 0, true, true);
                }))
                .ok();
        }
        (address, 0) if address == combine_addr_channel(0x905E, 1) => {
            deck_update_sender
                .send(Box::new(|deck_state, _| {
                    beat_jump_release(deck_state, 1, false, false);
                }))
                .ok();
        }
        (address, 0) if address == combine_addr_channel(0x905F, 1) => {
            deck_update_sender
                .send(Box::new(|deck_state, _| {
                    beat_jump_release(deck_state, 1, true, false);
                }))
                .ok();
        }
        (address, 0) if address == combine_addr_channel(0x9061, 1) => {
            deck_update_sender
                .send(Box::new(|deck_state, _| {
                    beat_jump_release(deck_state, 1, false, true);
                }))
                .ok();
        }
        (address, 0) if address == combine_addr_channel(0x9062, 1) => {
            deck_update_sender
                .send(Box::new(|deck_state, _| {
                    beat_jump_release(deck_state, 1, true, true);
                }))
                .ok();
        }
        (address, 0) if address == combine_addr_channel(0x905E, 2) => {
            deck_update_sender
                .send(Box::new(|deck_state, _| {
                    beat_jump_release(deck_state, 2, false, false);
                }))
                .ok();
        }
        (address, 0) if address == combine_addr_channel(0x905F, 2) => {
            deck_update_sender
                .send(Box::new(|deck_state, _| {
                    beat_jump_release(deck_state, 2, true, false);
                }))
                .ok();
        }
        (address, 0) if address == combine_addr_channel(0x9061, 2) => {
            deck_update_sender
                .send(Box::new(|deck_state, _| {
                    beat_jump_release(deck_state, 2, false, true);
                }))
                .ok();
        }
        (address, 0) if address == combine_addr_channel(0x9062, 2) => {
            deck_update_sender
                .send(Box::new(|deck_state, _| {
                    beat_jump_release(deck_state, 2, true, true);
                }))
                .ok();
        }
        (address, 0) if address == combine_addr_channel(0x905E, 3) => {
            deck_update_sender
                .send(Box::new(|deck_state, _| {
                    beat_jump_release(deck_state, 3, false, false);
                }))
                .ok();
        }
        (address, 0) if address == combine_addr_channel(0x905F, 3) => {
            deck_update_sender
                .send(Box::new(|deck_state, _| {
                    beat_jump_release(deck_state, 3, true, false);
                }))
                .ok();
        }
        (address, 0) if address == combine_addr_channel(0x9061, 3) => {
            deck_update_sender
                .send(Box::new(|deck_state, _| {
                    beat_jump_release(deck_state, 3, false, true);
                }))
                .ok();
        }
        (address, 0) if address == combine_addr_channel(0x9062, 3) => {
            deck_update_sender
                .send(Box::new(|deck_state, _| {
                    beat_jump_release(deck_state, 3, true, true);
                }))
                .ok();
        }
        _ => {}
    }
}

// we use the same safe code a lot here, and there's a lot of code here to
// handle, this is probably cleanupable later
#[allow(
    clippy::cast_possible_truncation,
    clippy::too_many_lines,
    clippy::indexing_slicing,
    clippy::cast_sign_loss,
    clippy::bool_to_int_with_if
)]
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
            let _ = midi_output.send(addr_value_to_midi(
                [0x9F18, 0x9F19, 0x9F1A, 0x9F1B][channel],
                if channel == master_channel { 127 } else { 0 },
            ));
        } else {
            let _ = midi_output.send(addr_value_to_midi(
                [0x9F18, 0x9F19, 0x9F1A, 0x9F1B][channel],
                0,
            ));
        }

        // vinyl
        let _ = midi_output.send(addr_channel_value_to_midi(0x9017, channel as u16, 127));

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
                        if player_state.time == cue_time {
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

        // --------- jog display stuff ---------

        // display on
        let _ = midi_output.send(addr_value_to_midi(
            [0x9F5D, 0x9F5E, 0x9F5F, 0x9F60][channel],
            0x00,
            // if player_state.current_track.is_some() {
            //     0x00
            // } else {
            //     0x7F
            // },
        ));

        // jog illumination
        let _ = midi_output.send(addr_value_to_midi(
            [0xBF09, 0xBF0A, 0xBF0B, 0xBF0C][channel],
            if player_state.is_loading {
                if flash_timers.fast { 0x01 } else { 0x00 }
            } else if player_state.current_track.is_some() {
                0x01
            } else {
                0x00
            },
        ));

        // jog position
        // 1_800_000_000 for 33.3 rpm
        let jog_position: u16 = (((player_state.time.nanoseconds.rem_euclid(1_800_000_000))
            / 1_800_000_000)
            * 0x0267) as u16;

        let _ = midi_output.send(addr_value_to_midi(
            [0xBF10, 0xBF11, 0xBF12, 0xBF13][channel],
            ((jog_position >> 8) & 0x00FF) as u8,
        ));
        let _ = midi_output.send(addr_value_to_midi(
            [0xBF30, 0xBF31, 0xBF32, 0xBF33][channel],
            (jog_position & 0x00FF) as u8,
        ));

        // cue point
        if let Some(cue_point) = player_state.cue_time {
            // 1_800_000_000 for 33.3 rpm
            let cue_point: u16 = (((cue_point.nanoseconds.rem_euclid(1_800_000_000))
                / 1_800_000_000)
                * 0x0267) as u16;

            let _ = midi_output.send(addr_value_to_midi(
                [0xBF1C, 0xBF1D, 0xBF1E, 0xBF1F][channel],
                ((cue_point >> 8) & 0x00FF) as u8,
            ));
            let _ = midi_output.send(addr_value_to_midi(
                [0xBF3C, 0xBF3D, 0xBF3E, 0xBF3F][channel],
                (cue_point & 0x00FF) as u8,
            ));
        } else {
            let _ = midi_output.send(addr_value_to_midi(
                [0xBF1C, 0xBF1D, 0xBF1E, 0xBF1F][channel],
                0x7F,
            ));
            let _ = midi_output.send(addr_value_to_midi(
                [0xBF3C, 0xBF3D, 0xBF3E, 0xBF3F][channel],
                0x7F,
            ));
        }

        // time
        let track_time_seconds = player_state.time.to_nanoseconds() / 1_000_000_000;
        let track_time_minutes = (track_time_seconds / 60).unsigned_abs() as u8;
        let track_time_seconds = (track_time_seconds % 60).unsigned_abs() as u8;

        let _ = midi_output.send(addr_value_to_midi(
            [0xBF42, 0xBF44, 0xBF46, 0xBF48][channel],
            track_time_minutes,
        ));
        let _ = midi_output.send(addr_value_to_midi(
            [0xBF43, 0xBF45, 0xBF47, 0xBF49][channel],
            track_time_seconds,
        ));

        // bpm
        if let Some(bpm) = player_state.get_current_bpm() {
            // 19983 == 0x4E0F
            let cue_point = (bpm.clamp(0.0, 999.9) * 19983.0).floor() as u16;

            let _ = midi_output.send(addr_value_to_midi(
                [0xBF14, 0xBF15, 0xBF16, 0xBF17][channel],
                ((cue_point >> 8) & 0x00FF) as u8,
            ));
            let _ = midi_output.send(addr_value_to_midi(
                [0xBF34, 0xBF35, 0xBF36, 0xBF37][channel],
                (cue_point & 0x00FF) as u8,
            ));
        } else {
            let _ = midi_output.send(addr_value_to_midi(
                [0xBF14, 0xBF15, 0xBF16, 0xBF17][channel],
                0,
            ));
            let _ = midi_output.send(addr_value_to_midi(
                [0xBF34, 0xBF35, 0xBF36, 0xBF37][channel],
                0,
            ));
        }

        // tempo percent
        // 19983 == 0x4E0F
        let tempo =
            ((player_state.tempo_percent * (19983.0 / 2.0)) + (19983.0 / 2.0)).floor() as u16;
        let _ = midi_output.send(addr_value_to_midi(
            [0xBF18, 0xBF19, 0xBF1A, 0xBF1B][channel],
            ((tempo >> 8) & 0x00FF) as u8,
        ));
        let _ = midi_output.send(addr_value_to_midi(
            [0xBF38, 0xBF39, 0xBF3A, 0xBF3B][channel],
            (tempo & 0x00FF) as u8,
        ));

        // master tempo
        let _ = midi_output.send(addr_value_to_midi(
            [0x9F20, 0x9F21, 0x9F22, 0x9F23][channel],
            if player_state.master_tempo { 127 } else { 0 },
        ));

        // beat sync
        let _ = midi_output.send(addr_value_to_midi(
            [0x9F20, 0x9F21, 0x9F22, 0x9F23][channel],
            match player_state.beat_sync {
                BeatSyncMode::Off => 0,
                BeatSyncMode::BPMSync => {
                    if flash_timers.mid {
                        127
                    } else {
                        0
                    }
                }
                BeatSyncMode::BeatSync => 127,
            },
        ));
    }
}
