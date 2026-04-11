pub mod audio_loader;
pub mod channel_logic;

use std::{
    cell::Cell,
    f32, mem,
    rc::Rc,
    time::{Duration, Instant},
};

use midir::{MidiInput, MidiInputConnection, MidiOutput};
use pipewire::{
    context::ContextRc,
    core::CoreRc,
    keys::{self},
    properties::properties,
    spa::{
        self,
        param::{
            ParamType,
            audio::AudioInfoRaw,
            format::{MediaSubtype, MediaType},
            format_utils,
        },
        pod::{Object, Pod, Value, serialize::PodSerializer},
        utils::{Direction, SpaTypes},
    },
    stream::{StreamFlags, StreamListener, StreamRc},
    thread_loop::ThreadLoopRc,
};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    AUDIO_CHANNELS, MAX_BUFFER_SIZE, PLAYBACK_SAMPLE_RATE,
    audio_system::{audio_loader::TrackAudioData, channel_logic::DeckDSP},
    types::{
        deck::{DeckState, DeckUpdate},
        midi::MidiMessage,
    },
};

struct FullAudioPipelineStreamData {
    format: spa::param::audio::AudioInfoRaw,

    deck_dsp: Option<DeckDSP>,

    deck_state: DeckState,

    deck_update_receiver: UnboundedReceiver<DeckUpdate>,
    deck_state_sender: tokio::sync::watch::Sender<DeckState>,

    last_process_instant: Instant,

    loaded_track_receiver:
        tokio::sync::mpsc::UnboundedReceiver<(usize, Option<Box<TrackAudioData>>)>,

    master_output_buffers: [Vec<f32>; AUDIO_CHANNELS],
    cue_output_buffers: [Vec<f32>; AUDIO_CHANNELS],
}

pub struct AudioManager {
    pipewire_thread: ThreadLoopRc,
    pipewire_core: CoreRc,

    streams: Vec<StreamRc>,

    full_output_listeners: Vec<StreamListener<FullAudioPipelineStreamData>>,

    midi_inputs: Vec<MidiInputConnection<()>>,
}

impl AudioManager {
    pub fn new() -> Result<AudioManager, Box<dyn std::error::Error>> {
        pipewire::init();

        // i *think* the only unsafe thing inside this function is getting a pointer to
        // the name of the thread :skull:
        #[allow(clippy::undocumented_unsafe_blocks)]
        let pipewire_thread = match unsafe { ThreadLoopRc::new(Some("pipewire main loop"), None) } {
            Ok(mainloop) => mainloop,
            Err(err) => {
                return Err(Box::new(err));
            }
        };
        let pipewire_context = match ContextRc::new(&pipewire_thread, None) {
            Ok(context) => context,
            Err(err) => {
                return Err(Box::new(err));
            }
        };
        let pipewire_core = match pipewire_context.connect_rc(None) {
            Ok(core) => core,
            Err(err) => {
                return Err(Box::new(err));
            }
        };

        Ok(AudioManager {
            pipewire_thread,
            pipewire_core,

            streams: Vec::new(),

            full_output_listeners: Vec::new(),

            midi_inputs: Vec::new(),
        })
    }

    pub fn start(&self) {
        self.pipewire_thread.start();
    }

    pub fn find_node_by_name_substring(
        &self,
        name_substring: &str,
    ) -> Result<Option<u32>, Box<dyn std::error::Error>> {
        let target_node_id: Rc<Cell<Option<u32>>> = Rc::new(Cell::new(None));
        let done_received: Rc<Cell<bool>> = Rc::new(Cell::new(false));

        let registry = self.pipewire_core.get_registry_rc()?;

        let target_clone = target_node_id.clone();
        let name_substring_owned = name_substring.to_owned();

        let registry_listener = registry
            .add_listener_local()
            .global(move |global_object| {
                if global_object.type_ == pipewire::types::ObjectType::Node
                    && let Some(global_object_properties) = &global_object.props
                    && let Some(node_name) = global_object_properties.get("node.name")
                    && node_name.contains(&name_substring_owned)
                {
                    target_clone.set(Some(global_object.id));
                }
            })
            .register();

        let pending_seq = self.pipewire_core.sync(0)?;

        let done_clone = done_received.clone();
        let core_listener = self
            .pipewire_core
            .add_listener_local()
            .done(move |_id, seq| {
                if seq == pending_seq {
                    done_clone.set(true);
                }
            })
            .register();

        self.pipewire_thread.start();

        let lock = self.pipewire_thread.lock();

        let timeout = Duration::from_secs(3);
        let deadline = std::time::Instant::now() + timeout;

        while !done_received.get() {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());

            if remaining.is_zero() {
                drop(lock);

                self.pipewire_thread.stop();

                return Err("timed out waiting for pipewire registry sync".into());
            }

            self.pipewire_thread.timed_wait(remaining);
        }

        drop(lock);

        drop(core_listener);
        drop(registry_listener);

        self.pipewire_thread.stop();

        Ok(target_node_id.get())
    }

    // pipewire has a complex initialisation pipeline
    #[allow(clippy::too_many_lines)]
    pub fn create_full_audio_pipeline(
        &mut self,
        deck_update_receiver: UnboundedReceiver<DeckUpdate>,
        deck_state_sender: tokio::sync::watch::Sender<DeckState>,
        loaded_track_receiver: tokio::sync::mpsc::UnboundedReceiver<(
            usize,
            Option<Box<TrackAudioData>>,
        )>,
        connected_node: Option<u32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let audio_stream = match StreamRc::new(
            self.pipewire_core.clone(),
            "dj-deck-audio",
            properties! {
                *keys::MEDIA_TYPE => "Audio",
                *keys::MEDIA_CATEGORY => "Playback",
                *keys::MEDIA_ROLE => "Production",
                *keys::AUDIO_CHANNELS => "4",
                *keys::AUDIO_FORMAT => "F32LE",
                *keys::NODE_LATENCY => format!("{MAX_BUFFER_SIZE}/{PLAYBACK_SAMPLE_RATE}"),
                *keys::NODE_MAX_LATENCY => format!("{MAX_BUFFER_SIZE}/{PLAYBACK_SAMPLE_RATE}"),
                *keys::STREAM_LATENCY_MIN => format!("{MAX_BUFFER_SIZE}"),
                *keys::STREAM_LATENCY_MAX => format!("{MAX_BUFFER_SIZE}"),
            },
        ) {
            Ok(core) => core,
            Err(err) => {
                return Err(Box::new(err));
            }
        };

        let audio_stream_data = FullAudioPipelineStreamData {
            format: AudioInfoRaw::default(),

            deck_dsp: None,

            deck_state: DeckState::default(),

            deck_update_receiver,
            deck_state_sender,

            last_process_instant: Instant::now(),

            loaded_track_receiver,

            master_output_buffers: Default::default(),
            cue_output_buffers: Default::default(),
        };

        let full_output_data = match audio_stream
            .add_local_listener_with_user_data(audio_stream_data)
            .param_changed(|_, audio_stream_data, id, params| {
                let Some(param) = params else {
                    audio_stream_data.deck_dsp = None;

                    return;
                };

                if id != ParamType::Format.as_raw() {
                    audio_stream_data.deck_dsp = None;

                    return;
                }

                let Ok((media_type, media_subtype)) = format_utils::parse_format(param) else {
                    audio_stream_data.deck_dsp = None;

                    return;
                };

                if media_type != MediaType::Audio || media_subtype != MediaSubtype::Raw {
                    audio_stream_data.deck_dsp = None;

                    return;
                }

                if audio_stream_data.format.parse(param).is_err() {
                    audio_stream_data.deck_dsp = None;

                    return;
                }

                #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
                if audio_stream_data.format.channels() != 2 * AUDIO_CHANNELS as u32 {
                    audio_stream_data.deck_dsp = None;

                    return;
                }

                let mut deck_dsp = DeckDSP::new(audio_stream_data.format.rate());
                deck_dsp.reset();

                audio_stream_data.deck_dsp = Some(deck_dsp);

                audio_stream_data.master_output_buffers = Default::default();
                audio_stream_data.cue_output_buffers = Default::default();
            })
            .process(
                #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                move |stream, audio_stream_data| {
                    let Some(mut buffer) = stream.dequeue_buffer() else {
                        return;
                    };

                    let datas = buffer.datas_mut();
                    if datas.is_empty() {
                        return;
                    }

                    let Some(data) = datas.first_mut() else {
                        return;
                    };

                    let Some(deck_dsp) = &mut audio_stream_data.deck_dsp else {
                        return;
                    };

                    while let Ok((channel_index, track_data)) =
                        audio_stream_data.loaded_track_receiver.try_recv()
                    {
                        deck_dsp.assign_track_data(channel_index, track_data);
                    }

                    let delta_time = audio_stream_data.last_process_instant.elapsed();

                    audio_stream_data.last_process_instant = Instant::now();

                    audio_stream_data
                        .deck_state
                        .update(&mut audio_stream_data.deck_update_receiver, delta_time);

                    let _ = audio_stream_data
                        .deck_state_sender
                        .send_replace(audio_stream_data.deck_state.clone());

                    let number_channels = audio_stream_data.format.channels() as usize;
                    let stride = mem::size_of::<f32>() * number_channels;

                    let written_samples = if let Some(data_samples) = data.data() {
                        let available_samples = data_samples
                            .len()
                            .min(MAX_BUFFER_SIZE * mem::size_of::<f32>() * number_channels)
                            / stride;

                        deck_dsp.generate_samples(
                            &audio_stream_data.deck_state,
                            &mut audio_stream_data.master_output_buffers,
                            &mut audio_stream_data.cue_output_buffers,
                            available_samples,
                        );

                        for channel_index in 0..number_channels {
                            for sample_index in 0..available_samples {
                                let start =
                                    sample_index * stride + channel_index * mem::size_of::<f32>();
                                let end = start + mem::size_of::<f32>();

                                if let Some(sample_bytes) = data_samples.get_mut(start..end) {
                                    match channel_index {
                                        channel_number @ (0 | 1) => {
                                            // we ensure we have enough space
                                            // allocated for this
                                            // operation and we need speed here
                                            #[allow(clippy::indexing_slicing)]
                                            sample_bytes.copy_from_slice(
                                                &audio_stream_data.master_output_buffers
                                                    [channel_number][sample_index]
                                                    .to_le_bytes(),
                                            );
                                        }
                                        channel_number @ (2 | 3) => {
                                            // we ensure we have enough space
                                            // allocated for this
                                            // operation and we need speed here
                                            #[allow(clippy::indexing_slicing)]
                                            sample_bytes.copy_from_slice(
                                                &audio_stream_data.cue_output_buffers
                                                    [channel_number - 2][sample_index]
                                                    .to_le_bytes(),
                                            );
                                        }
                                        _ => {
                                            sample_bytes.copy_from_slice(&0.0f32.to_le_bytes());
                                        }
                                    }
                                }
                            }
                        }

                        available_samples
                    } else {
                        0
                    };

                    let chunk = data.chunk_mut();
                    *chunk.offset_mut() = 0;
                    *chunk.stride_mut() = stride as i32;
                    *chunk.size_mut() = (stride * written_samples) as u32;

                    drop(buffer);
                },
            )
            .register()
        {
            Ok(full_output_data) => full_output_data,
            Err(err) => {
                return Err(Box::new(err));
            }
        };

        let mut audio_info = spa::param::audio::AudioInfoRaw::new();
        audio_info.set_format(spa::param::audio::AudioFormat::F32LE);
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        audio_info.set_channels(2 * AUDIO_CHANNELS as u32);
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        audio_info.set_rate(PLAYBACK_SAMPLE_RATE as u32);

        let audio_parameters: Vec<u8> = (match PodSerializer::serialize(
            std::io::Cursor::new(Vec::new()),
            &Value::Object(Object {
                type_: SpaTypes::ObjectParamFormat.as_raw(),
                id: ParamType::EnumFormat.as_raw(),
                properties: audio_info.into(),
            }),
        ) {
            Ok(audio_parameters) => audio_parameters,
            Err(err) => {
                return Err(Box::new(err));
            }
        })
        .0
        .into_inner();

        match audio_stream.connect(
            Direction::Output,
            connected_node,
            StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS | StreamFlags::RT_PROCESS,
            &mut [Pod::from_bytes(&audio_parameters).unwrap()],
        ) {
            Ok(core) => core,
            Err(err) => {
                return Err(Box::new(err));
            }
        }

        let _ = audio_stream.set_active(true);

        self.streams.push(audio_stream);
        self.full_output_listeners.push(full_output_data);

        Ok(())
    }

    pub fn create_midi_input(
        &mut self,
    ) -> Result<tokio::sync::mpsc::UnboundedReceiver<MidiMessage>, Box<dyn std::error::Error>> {
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

        Ok(midi_receiver)
    }

    pub fn create_midi_output(
        &mut self,
    ) -> Result<tokio::sync::mpsc::UnboundedSender<MidiMessage>, Box<dyn std::error::Error>> {
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

        tokio::task::spawn(async move {
            while let Some(message) = &midi_receiver.recv().await {
                if let Err(e) = connection.send(message) {
                    eprintln!("[midi-sender] Send error: {e}");
                }
            }
        });

        Ok(midi_sender)
    }
}

impl Drop for AudioManager {
    fn drop(&mut self) {
        for listener in &mut self.full_output_listeners.drain(0..) {
            listener.unregister();
        }

        for stream in &self.streams {
            let _ = stream.set_active(false);
            let _ = stream.disconnect();
        }

        self.pipewire_thread.stop();

        // we perform all the necessary terminations to make this safe
        #[allow(clippy::undocumented_unsafe_blocks)]
        unsafe {
            pipewire::deinit();
        }
    }
}
