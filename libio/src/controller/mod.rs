use std::{collections::BTreeMap, time::Duration};

use libdj::types::{
    bindings::DeckControlEvent,
    deck::{BeatLoopAdjustMode, BeatSyncMode, DeckState, PlayState},
};
use loop_unwrap::{ToOption, unwrap_continue};
use midir::{MidiInput, MidiInputConnection, MidiOutput};
use thiserror::Error;
use tokio::task::JoinHandle;

use crate::types::controller::ControllerMessage;

pub type MidiMessage = [u8; 3];

pub struct Controller {
    message_sender: tokio::sync::broadcast::Sender<ControllerMessage>,

    midi_inputs: Vec<MidiInputConnection<()>>,
    midi_outputs: Vec<JoinHandle<()>>,

    midi_sender: tokio::sync::mpsc::UnboundedSender<MidiMessage>,
    midi_receiver: Option<tokio::sync::mpsc::UnboundedReceiver<MidiMessage>>,

    midi_task: Option<JoinHandle<()>>,
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
    pub fn open_midi_input(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (midi_sender, midi_receiver) = tokio::sync::mpsc::unbounded_channel::<MidiMessage>();

        let midi_in = MidiInput::new("dj-deck-midi-input")?;

        let ports = midi_in.ports();

        let Some(target_port) = ports.iter().find(|port| {
            midi_in
                .port_name(port)
                .is_ok_and(|name| name.contains("DDJ-FLX10"))
        }) else {
            return Err("No port
                found"
                .into());
        };

        let port_name = midi_in.port_name(target_port)?;

        let connection = midi_in.connect(
            target_port,
            &port_name,
            move |_, message, ()| {
                if let [byte_zero, byte_one, byte_two] = *message {
                    let _ = midi_sender.send([byte_zero, byte_one, byte_two]);
                }
            },
            (),
        )?;

        self.midi_inputs.push(connection);

        self.midi_receiver = Some(midi_receiver);

        Ok(())
    }

    pub fn open_midi_output(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (midi_sender, mut midi_receiver) =
            tokio::sync::mpsc::unbounded_channel::<MidiMessage>();

        let midi_out = MidiOutput::new("dj-deck-midi-output")?;

        let ports = midi_out.ports();

        let Some(target_port) = ports.iter().find(|port| {
            midi_out
                .port_name(port)
                .is_ok_and(|name| name.contains("DDJ-FLX10"))
        }) else {
            return Err("No port found".into());
        };

        let port_name = midi_out.port_name(target_port)?;

        let mut connection = midi_out.connect(target_port, &port_name)?;

        let connection = tokio::task::spawn(async move {
            while let Some(message) = &midi_receiver.recv().await {
                if let Err(e) = connection.send(message) {
                    println!("[midi-sender] Send error: {e}");
                }
            }
        });

        self.midi_outputs.push(connection);

        self.midi_sender = midi_sender;

        Ok(())
    }

    pub fn start(
        &mut self,
        deck_control_event_sender: tokio::sync::mpsc::UnboundedSender<DeckControlEvent>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let midi_sender = self.midi_sender.clone();
        let mut midi_receiver = self
            .midi_receiver
            .take()
            .ok_or("No available midi receiver")?;

        let mut controller_message_receiver = self.message_sender.subscribe();

        let mappings = load_midi_mapping();

        let midi_task = tokio::task::spawn(async move {
            let mut current_deck_state: DeckState = DeckState::default();

            let mut controller_state: BTreeMap<(&str, usize), (u8, u8)> = BTreeMap::new();

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

                            process_midi_command(address, value, &mappings, &mut controller_state, &deck_control_event_sender);
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

        self.midi_task = Some(midi_task);

        Ok(())
    }

    pub fn send(self: &Controller, message: ControllerMessage) -> Result<(), ControllerSendError> {
        match self.message_sender.send(message) {
            Ok(_) => Ok(()),
            Err(err) => Err(ControllerSendError::ChannelError(err)),
        }
    }
}

impl Default for Controller {
    fn default() -> Self {
        let (message_sender, _) = tokio::sync::broadcast::channel(16);
        let (midi_sender, _) = tokio::sync::mpsc::unbounded_channel::<MidiMessage>();

        Self {
            message_sender,

            midi_inputs: Vec::new(),
            midi_outputs: Vec::new(),

            midi_sender,
            midi_receiver: None,

            midi_task: None,
        }
    }
}

impl Drop for Controller {
    fn drop(&mut self) {
        if let Some(midi_task) = self.midi_task.take() {
            midi_task.abort();
        }

        for midi_input in self.midi_inputs.drain(0..) {
            midi_input.close();
        }

        for midi_output in self.midi_outputs.drain(0..) {
            midi_output.abort();
        }
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

fn midi_split_pair_to_f32_range(msb: u8, lsb: u8, min: f32, max: f32) -> f32 {
    f32::from((u16::from(msb) << 7) | u16::from(lsb)) / 16383.0 * (max - min) + min
}

#[allow(clippy::cast_lossless)]
fn midi_rpm_to_f32(jog: u8) -> f32 {
    if jog >= 0x41 {
        // +
        (6.0 * jog as f32) - 390.0
    } else if jog <= 0x3F {
        // -
        (6.0 * jog as f32) - 378.0
    } else {
        0.0
    }
}

fn midi_ticks_to_relative_ticks(
    value: u8,
    clockwise_start: u8,
    counterclockwise_start: u8,
    center_threshold: u8,
) -> f32 {
    if value <= center_threshold {
        (f32::from(value) - f32::from(clockwise_start)) + 1.0
    } else {
        -((f32::from(counterclockwise_start) - f32::from(value)) + 1.0)
    }
}

const MIDI_MAPPING: &str = include_str!("../../ddj-flx10.txt");

#[allow(clippy::type_complexity)]
#[derive(Debug)]
pub struct MidiMapping {
    direct: BTreeMap<u16, Vec<(MappingTarget, DeckControlEvent)>>,
    direct_equals: BTreeMap<u16, Vec<(usize, MappingTarget, DeckControlEvent)>>,
    integrated: [BTreeMap<u16, Vec<(MappingTarget, DeckControlEvent)>>; 4],
}

#[derive(Debug)]
pub enum MappingTarget {
    Button(u8),
    NumberedButton(u8, usize),
    Absolute(BytePosition, f32, f32),
    Relative(u8, u8, u8, f32),
    Jog,
}

#[derive(Debug)]
pub enum BytePosition {
    MSB,
    LSB,
}

#[derive(Debug)]
pub enum ChannelType {
    Direct,
    DirectEquals,
    Integrated,
}

fn load_midi_mapping() -> MidiMapping {
    let mut mapping = MidiMapping {
        direct: BTreeMap::default(),
        direct_equals: BTreeMap::default(),
        integrated: [
            BTreeMap::default(),
            BTreeMap::default(),
            BTreeMap::default(),
            BTreeMap::default(),
        ],
    };

    'outer: for line in MIDI_MAPPING.lines() {
        let mut chars = line.chars();

        if unwrap_continue!(chars.next().map(|c| c != '0'), 'outer)
            || unwrap_continue!(chars.next().map(|c| c != 'x'), 'outer)
        {
            continue 'outer;
        }

        let mut number_string: String = String::with_capacity(4);
        let mut channel_type = ChannelType::Direct;
        let mut channel: usize = 0;

        for i in (0..4).rev() {
            let character = unwrap_continue!(chars.next(), 'outer);

            if character == 'n' {
                channel_type = ChannelType::Integrated;
                channel = i;

                number_string.push('0');
            } else {
                number_string.push(character);
            }
        }

        let address = unwrap_continue!(u16::from_str_radix(&number_string, 16), 'outer);

        let mut character = unwrap_continue!(chars.next(), 'outer);

        if character == '=' {
            let mut number_string: String = String::with_capacity(1);

            number_string.push(unwrap_continue!(chars.next(), 'outer));

            channel_type = ChannelType::DirectEquals;
            channel = unwrap_continue!(number_string.parse::<usize>(), 'outer);

            character = unwrap_continue!(chars.next(), 'outer);
        }

        if character != '-' || unwrap_continue!(chars.next().map(|c| c != '>'), 'outer) {
            continue 'outer;
        }

        let character = unwrap_continue!(chars.next(), 'outer);

        if unwrap_continue!(chars.next().map(|c| c != '('), 'outer) {
            continue 'outer;
        }

        let mut args: Vec<String> = vec![String::new()];

        'arg_parser: loop {
            let character = unwrap_continue!(chars.next(), 'outer);

            if character == ' ' || character == '\t' {
            } else if character == ')' {
                break 'arg_parser;
            } else if character == ',' {
                args.push(String::new());
            } else if let Some(last_arg) = args.last_mut() {
                last_arg.push(character);
            }
        }

        if let Some(last_arg) = args.last()
            && last_arg.is_empty()
        {
            args.pop();
        }

        let target = match character {
            'b' => {
                if args.len() != 1 {
                    continue 'outer;
                }

                let trigger: &String = unwrap_continue!(args.first(), 'outer);
                let trigger_radix = if trigger.starts_with("0x") { 16 } else { 10 };
                let trigger = trigger.trim_start_matches("0x");
                let trigger: u8 =
                    unwrap_continue!(u8::from_str_radix(trigger, trigger_radix), 'outer);

                MappingTarget::Button(trigger)
            }
            'n' => {
                if args.len() != 2 {
                    continue 'outer;
                }

                let trigger: &String = unwrap_continue!(args.first(), 'outer);
                let trigger_radix = if trigger.starts_with("0x") { 16 } else { 10 };
                let trigger = trigger.trim_start_matches("0x");
                let trigger: u8 =
                    unwrap_continue!(u8::from_str_radix(trigger, trigger_radix), 'outer);

                let number: &String = unwrap_continue!(args.get(1), 'outer);
                let number_radix = if number.starts_with("0x") { 16 } else { 10 };
                let number = number.trim_start_matches("0x");
                let number: usize =
                    unwrap_continue!(usize::from_str_radix(number, number_radix), 'outer);

                MappingTarget::NumberedButton(trigger, number)
            }
            'a' => {
                if args.len() != 3 {
                    continue 'outer;
                }

                let byte_position = match args.first().map(String::as_str) {
                    Some("msb") => BytePosition::MSB,
                    Some("lsb") => BytePosition::LSB,
                    _ => {
                        continue 'outer;
                    }
                };

                let min: &String = unwrap_continue!(args.get(1), 'outer);
                let min: f32 = unwrap_continue!(min.parse::<f32>(), 'outer);

                let max: &String = unwrap_continue!(args.get(2), 'outer);
                let max: f32 = unwrap_continue!(max.parse::<f32>(), 'outer);

                MappingTarget::Absolute(byte_position, min, max)
            }
            'r' => {
                if args.len() != 4 {
                    continue 'outer;
                }

                let clockwise_start: &String = unwrap_continue!(args.first(), 'outer);
                let clockwise_start_radix = if clockwise_start.starts_with("0x") {
                    16
                } else {
                    10
                };
                let clockwise_start = clockwise_start.trim_start_matches("0x");
                let clockwise_start: u8 = unwrap_continue!(u8::from_str_radix(clockwise_start, clockwise_start_radix), 'outer);

                let counter_clockwise_start: &String = unwrap_continue!(args.get(1), 'outer);
                let counter_clockwise_start_radix = if counter_clockwise_start.starts_with("0x") {
                    16
                } else {
                    10
                };
                let counter_clockwise_start = counter_clockwise_start.trim_start_matches("0x");
                let counter_clockwise_start: u8 = unwrap_continue!(u8::from_str_radix(counter_clockwise_start, counter_clockwise_start_radix), 'outer);

                let center_threshold: &String = unwrap_continue!(args.get(2), 'outer);
                let center_threshold_radix = if center_threshold.starts_with("0x") {
                    16
                } else {
                    10
                };
                let center_threshold = center_threshold.trim_start_matches("0x");
                let center_threshold: u8 = unwrap_continue!(u8::from_str_radix(center_threshold, center_threshold_radix), 'outer);

                let delta: &String = unwrap_continue!(args.get(3), 'outer);
                let delta: f32 = unwrap_continue!(delta.parse::<f32>(), 'outer);

                MappingTarget::Relative(
                    clockwise_start,
                    counter_clockwise_start,
                    center_threshold,
                    delta,
                )
            }
            'j' => {
                if !args.is_empty() {
                    continue 'outer;
                }

                MappingTarget::Jog
            }
            _ => {
                continue 'outer;
            }
        };

        if unwrap_continue!(chars.next().map(|c| c != ':'), 'outer) {
            continue 'outer;
        }

        let mut event_name: String = String::new();

        #[allow(clippy::while_let_on_iterator)]
        'name_parser: while let Some(character) = chars.next() {
            if character == ' ' || character == '\t' {
            } else if character == '#' {
                break 'name_parser;
            } else {
                event_name.push(character);
            }
        }

        let event = unwrap_continue!(DeckControlEvent::from_str(&event_name));

        match channel_type {
            ChannelType::Direct => {
                if let Some(existing_mappings) = mapping.direct.get_mut(&address) {
                    existing_mappings.push((target, event));
                } else {
                    mapping.direct.insert(address, vec![(target, event)]);
                }
            }
            ChannelType::DirectEquals => {
                if let Some(existing_mappings) = mapping.direct_equals.get_mut(&address) {
                    existing_mappings.push((channel, target, event));
                } else {
                    mapping
                        .direct_equals
                        .insert(address, vec![(channel, target, event)]);
                }
            }
            ChannelType::Integrated => {
                let map = unwrap_continue!(mapping.integrated.get_mut(channel), 'outer);

                if let Some(existing_mappings) = map.get_mut(&address) {
                    existing_mappings.push((target, event));
                } else {
                    map.insert(address, vec![(target, event)]);
                }
            }
        }
    }

    mapping
}

fn process_midi_command(
    address: u16,
    value: u8,
    mappings: &MidiMapping,
    persistence_store: &mut BTreeMap<(&str, usize), (u8, u8)>,
    deck_control_event_sender: &tokio::sync::mpsc::UnboundedSender<DeckControlEvent>,
) {
    if let Some(mappings) = mappings.direct.get(&address) {
        for (mapping_target, event) in mappings {
            if let Some(event) =
                apply_mapping_to_event(value, 0, mapping_target, *event, persistence_store)
            {
                let _ = deck_control_event_sender.send(event);
            }
        }

        return;
    }

    if let Some(mappings) = mappings.direct_equals.get(&address) {
        for (channel, mapping_target, action) in mappings {
            if let Some(event) =
                apply_mapping_to_event(value, *channel, mapping_target, *action, persistence_store)
            {
                let _ = deck_control_event_sender.send(event);
            }
        }

        return;
    }

    for index in (0..4).rev() {
        let masked_address = address & !(0xf << (index * 4));
        let channel = usize::from((address >> (index * 4)) & 0xf);

        if let Some(map) = mappings.integrated.get(index)
            && let Some(mappings) = map.get(&masked_address)
        {
            for (mapping_target, event) in mappings {
                if let Some(event) = apply_mapping_to_event(
                    value,
                    channel,
                    mapping_target,
                    *event,
                    persistence_store,
                ) {
                    let _ = deck_control_event_sender.send(event);
                }
            }

            return;
        }
    }
}

fn apply_mapping_to_event(
    value: u8,
    channel: usize,
    target: &MappingTarget,
    event: DeckControlEvent,
    persistence_store: &mut BTreeMap<(&str, usize), (u8, u8)>,
) -> Option<DeckControlEvent> {
    let mut new_position: f32 = 0.0;
    let mut new_velocity: f32 = 0.0;
    let mut new_delta: f32 = 0.0;

    let mut source_number: usize = 0;

    match target {
        MappingTarget::Button(target) => {
            if value != *target {
                return None;
            }
        }
        MappingTarget::NumberedButton(target, number) => {
            if value != *target {
                return None;
            }

            source_number = *number;
        }
        MappingTarget::Absolute(byte_position, min, max) => {
            let (msb, lsb) = if let Some((msb, lsb)) =
                persistence_store.get_mut(&(DeckControlEvent::to_str(event), channel))
            {
                match byte_position {
                    BytePosition::MSB => {
                        *msb = value;
                    }
                    BytePosition::LSB => {
                        *lsb = value;
                    }
                }

                (*msb, *lsb)
            } else {
                let bytes = match byte_position {
                    BytePosition::MSB => (value, 0),
                    BytePosition::LSB => (0, value),
                };

                persistence_store.insert((DeckControlEvent::to_str(event), channel), bytes);

                bytes
            };

            new_position = midi_split_pair_to_f32_range(msb, lsb, *min, *max);
        }
        MappingTarget::Relative(
            clockwise_start,
            counterclockwise_start,
            center_threshold,
            size,
        ) => {
            new_delta = midi_ticks_to_relative_ticks(
                value,
                *clockwise_start,
                *counterclockwise_start,
                *center_threshold,
            ) * size;
        }
        MappingTarget::Jog => {
            new_velocity = midi_rpm_to_f32(value);
        }
    }

    let new_channel = channel;

    let mut new_event = event;

    match &mut new_event {
        DeckControlEvent::BrowserEncoderPress
        | DeckControlEvent::BrowserSourcePress
        | DeckControlEvent::BrowserBrowsePress
        | DeckControlEvent::BrowserPlaylistPress
        | DeckControlEvent::BrowserBackPress
        | DeckControlEvent::MixerMasterFXEnablePress
        | DeckControlEvent::BrowserSearchPress
        | DeckControlEvent::MixerMasterCuePress
        | DeckControlEvent::MixerMasterFXSelectTouchPress
        | DeckControlEvent::MixerMasterFXSelectTouchRelease
        | DeckControlEvent::MixerMasterFXParameterSelectPress
        | DeckControlEvent::MixerMasterFXBPMSelectPress
        | DeckControlEvent::MixerChannelFXProximityPress
        | DeckControlEvent::MixerChannelFXProximityRelease
        | DeckControlEvent::MixerMasterMasterFXTargetPress => {}

        DeckControlEvent::BrowserEncoderAdjust { delta }
        | DeckControlEvent::MixerMasterFXSelect { delta }
        | DeckControlEvent::MixerMasterFXParameterAdjust { delta }
        | DeckControlEvent::MixerMasterFXBPMAdjust { delta } => {
            *delta = new_delta;
        }

        DeckControlEvent::PlayerPlayPress { channel }
        | DeckControlEvent::PlayerCuePress { channel }
        | DeckControlEvent::PlayerCueRelease { channel }
        | DeckControlEvent::PlayerAltCuePress { channel }
        | DeckControlEvent::PlayerAltCueRelease { channel }
        | DeckControlEvent::PlayerBeatJumpBackwardRelease { channel }
        | DeckControlEvent::PlayerLongBeatJumpBackwardRelease { channel }
        | DeckControlEvent::PlayerBeatJumpForwardRelease { channel }
        | DeckControlEvent::PlayerLongBeatJumpForwardRelease { channel }
        | DeckControlEvent::PlayerBeatSyncPress { channel }
        | DeckControlEvent::PlayerMasterPress { channel }
        | DeckControlEvent::PlayerTempoResetPress { channel }
        | DeckControlEvent::PlayerTempoRangePress { channel }
        | DeckControlEvent::PlayerMasterTempoPress { channel }
        | DeckControlEvent::PlayerJogTouchPress { channel }
        | DeckControlEvent::PlayerJogTouchRelease { channel }
        | DeckControlEvent::MixerChannelCuePress { channel }
        | DeckControlEvent::MixerChannelCrossfaderAssignAPress { channel }
        | DeckControlEvent::MixerChannelCrossfaderAssignNonePress { channel }
        | DeckControlEvent::MixerChannelCrossfaderAssignBPress { channel }
        | DeckControlEvent::PlayerQuantizePress { channel }
        | DeckControlEvent::PlayerSlipPress { channel }
        | DeckControlEvent::PlayerReversePress { channel }
        | DeckControlEvent::PlayerReverseRelease { channel }
        | DeckControlEvent::PlayerBPMSyncPress { channel }
        | DeckControlEvent::PlayerKeySyncPress { channel }
        | DeckControlEvent::PlayerBeatLoopInPress { channel }
        | DeckControlEvent::PlayerBeatLoopInAdjustPress { channel }
        | DeckControlEvent::PlayerBeatLoopOutPress { channel }
        | DeckControlEvent::PlayerBeatLoopOutAdjustPress { channel }
        | DeckControlEvent::PlayerReLoopPress { channel }
        | DeckControlEvent::PlayerInstantLoopPress { channel }
        | DeckControlEvent::MixerChannelMasterFXTargetPress { channel } => {
            *channel = new_channel;
        }

        DeckControlEvent::PlayerJogVelocitySet { channel, velocity }
        | DeckControlEvent::PlayerJogSearchVelocitySet { channel, velocity } => {
            *channel = new_channel;
            *velocity = new_velocity;
        }

        DeckControlEvent::MixerChannelGainSet { channel, position }
        | DeckControlEvent::MixerChannelEqHighSet { channel, position }
        | DeckControlEvent::MixerChannelEqMidSet { channel, position }
        | DeckControlEvent::MixerChannelEqLowSet { channel, position }
        | DeckControlEvent::MixerChannelFXSet { channel, position }
        | DeckControlEvent::PlayerTempoSet { channel, position }
        | DeckControlEvent::MixerChannelFaderSet { channel, position } => {
            *channel = new_channel;
            *position = new_position;
        }

        DeckControlEvent::MixerHeadphonesMixSet { position }
        | DeckControlEvent::MixerHeadphonesGainSet { position }
        | DeckControlEvent::MixerMasterFXDepthSet { position }
        | DeckControlEvent::MixerMasterGainSet { position }
        | DeckControlEvent::MixerCrossfaderSet { position } => {
            *position = new_position;
        }

        DeckControlEvent::PlayerPadPress { channel, number } => {
            *channel = new_channel;
            *number = source_number;
        }

        DeckControlEvent::MixerChannelFXPress { number } => {
            *number = source_number;
        }

        DeckControlEvent::USBEjectPress { slot } | DeckControlEvent::USBEjectRelease { slot } => {
            *slot = source_number;
        }
    }

    Some(new_event)
}

// we use the same safe code a lot here, and there's a lot of code here to
// handle, this is probably cleanupable later
#[allow(
    clippy::cast_possible_truncation,
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
                    if flash_timers.mid {
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
                let in_state = if player_state.beat_loop_adjust_mode == BeatLoopAdjustMode::In {
                    if flash_timers.fast { 127 } else { 0 }
                } else {
                    if flash_timers.mid { 127 } else { 0 }
                };
                let _ =
                    midi_output.send(addr_channel_value_to_midi(0x9010, channel as u16, in_state));
                let _ =
                    midi_output.send(addr_channel_value_to_midi(0x9012, channel as u16, in_state));
                let _ =
                    midi_output.send(addr_channel_value_to_midi(0x904C, channel as u16, in_state));

                // out
                let out_state = if player_state.beat_loop_adjust_mode == BeatLoopAdjustMode::Out {
                    if flash_timers.fast { 127 } else { 0 }
                } else {
                    if flash_timers.mid { 127 } else { 0 }
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
        let _ = midi_output.send(addr_value_to_midi(
            [0x9F20, 0x9F21, 0x9F22, 0x9F23][channel],
            if player_state.master_tempo { 127 } else { 0 },
        ));

        // --------- jog display stuff ---------

        // display on
        // let _ = midi_output.send(addr_value_to_midi(
        //     [0x9F5D, 0x9F5E, 0x9F5F, 0x9F60][channel],
        //     0x00,
        //     // if player_state.current_track.is_some() {
        //     //     0x00
        //     // } else {
        //     //     0x7F
        //     // },
        // ));

        // jog illumination
        // let _ = midi_output.send(addr_value_to_midi(
        //     [0xBF09, 0xBF0A, 0xBF0B, 0xBF0C][channel],
        //     if player_state.is_loading {
        //         if flash_timers.fast { 0x01 } else { 0x00 }
        //     } else if player_state.current_track.is_some() {
        //         0x01
        //     } else {
        //         0x00
        //     },
        // ));

        // jog position
        // 1_800_000_000 for 33.3 rpm
        // let jog_position: u16 =
        // (((player_state.time.nanoseconds.rem_euclid(1_800_000_000))
        //     / 1_800_000_000)
        //     * 0x0267) as u16;

        // let _ = midi_output.send(addr_value_to_midi(
        //     [0xBF10, 0xBF11, 0xBF12, 0xBF13][channel],
        //     ((jog_position >> 8) & 0x00FF) as u8,
        // ));
        // let _ = midi_output.send(addr_value_to_midi(
        //     [0xBF30, 0xBF31, 0xBF32, 0xBF33][channel],
        //     (jog_position & 0x00FF) as u8,
        // ));

        // cue point
        // if let Some(cue_point) = player_state.cue_time {
        //     // 1_800_000_000 for 33.3 rpm
        //     let cue_point: u16 =
        // (((cue_point.nanoseconds.rem_euclid(1_800_000_000))         /
        // 1_800_000_000)
        //         * 0x0267) as u16;

        //     let _ = midi_output.send(addr_value_to_midi(
        //         [0xBF1C, 0xBF1D, 0xBF1E, 0xBF1F][channel],
        //         ((cue_point >> 8) & 0x00FF) as u8,
        //     ));
        //     let _ = midi_output.send(addr_value_to_midi(
        //         [0xBF3C, 0xBF3D, 0xBF3E, 0xBF3F][channel],
        //         (cue_point & 0x00FF) as u8,
        //     ));
        // } else {
        //     let _ = midi_output.send(addr_value_to_midi(
        //         [0xBF1C, 0xBF1D, 0xBF1E, 0xBF1F][channel],
        //         0x7F,
        //     ));
        //     let _ = midi_output.send(addr_value_to_midi(
        //         [0xBF3C, 0xBF3D, 0xBF3E, 0xBF3F][channel],
        //         0x7F,
        //     ));
        // }

        // time
        // let track_time_seconds = player_state.time.to_nanoseconds() /
        // 1_000_000_000; let track_time_minutes = (track_time_seconds /
        // 60).unsigned_abs() as u8; let track_time_seconds =
        // (track_time_seconds % 60).unsigned_abs() as u8;

        // let _ = midi_output.send(addr_value_to_midi(
        //     [0xBF42, 0xBF44, 0xBF46, 0xBF48][channel],
        //     track_time_minutes,
        // ));
        // let _ = midi_output.send(addr_value_to_midi(
        //     [0xBF43, 0xBF45, 0xBF47, 0xBF49][channel],
        //     track_time_seconds,
        // ));

        // bpm
        // if let Some(bpm) = player_state.get_current_bpm() {
        //     // 19983 == 0x4E0F
        //     let cue_point = (bpm.clamp(0.0, 999.9) * 19983.0).floor() as u16;

        //     let _ = midi_output.send(addr_value_to_midi(
        //         [0xBF14, 0xBF15, 0xBF16, 0xBF17][channel],
        //         ((cue_point >> 8) & 0x00FF) as u8,
        //     ));
        //     let _ = midi_output.send(addr_value_to_midi(
        //         [0xBF34, 0xBF35, 0xBF36, 0xBF37][channel],
        //         (cue_point & 0x00FF) as u8,
        //     ));
        // } else {
        //     let _ = midi_output.send(addr_value_to_midi(
        //         [0xBF14, 0xBF15, 0xBF16, 0xBF17][channel],
        //         0,
        //     ));
        //     let _ = midi_output.send(addr_value_to_midi(
        //         [0xBF34, 0xBF35, 0xBF36, 0xBF37][channel],
        //         0,
        //     ));
        // }

        // tempo percent
        // 19983 == 0x4E0F
        // let tempo = (((player_state.tempo_percent - 1.0) * (19983.0 / 2.0)) +
        // (19983.0 / 2.0))     .floor() as u16;
        // let _ = midi_output.send(addr_value_to_midi(
        //     [0xBF18, 0xBF19, 0xBF1A, 0xBF1B][channel],
        //     ((tempo >> 8) & 0x00FF) as u8,
        // ));
        // let _ = midi_output.send(addr_value_to_midi(
        //     [0xBF38, 0xBF39, 0xBF3A, 0xBF3B][channel],
        //     (tempo & 0x00FF) as u8,
        // ));
    }
}
