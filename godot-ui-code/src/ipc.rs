use std::{
    collections::{BTreeMap, VecDeque},
    io::{BufRead, BufReader, Write},
};

use base64::{Engine, prelude::BASE64_STANDARD};
use godot::{
    classes::{INode, Node},
    prelude::*,
};
use interprocess::local_socket::{
    GenericFilePath, Listener, ListenerNonblockingMode, ListenerOptions, RecvHalf, SendHalf,
    prelude::*,
};
use libdj::types::{deck::DeckState, library::Library};
use libui::types::ui::{ArchivedUIMessage, InternalUIEvent, SOCKET_NAME, UIMessage};

#[derive(GodotClass)]
#[class(base=Node)]
pub struct IPC {
    base: Base<Node>,

    listener: Listener,

    streams: Vec<(SendHalf, BufReader<RecvHalf>, Vec<u8>)>,

    event_queue: VecDeque<InternalUIEvent>,

    pub devices: BTreeMap<u32, (String, Option<Library>)>,

    pub deck_state: DeckState,

    pub devices_changed: bool,
}

impl IPC {}

#[godot_api]
impl INode for IPC {
    fn init(base: Base<Node>) -> Self {
        let name = SOCKET_NAME
            .to_fs_name::<GenericFilePath>()
            .expect("socket name valid");

        let listener = ListenerOptions::new()
            .name(name)
            .nonblocking(ListenerNonblockingMode::Both)
            .create_sync()
            .expect("socket created");

        Self {
            base,

            listener,

            streams: Vec::new(),

            event_queue: VecDeque::new(),

            devices: BTreeMap::new(),

            deck_state: DeckState::default(),

            devices_changed: false,
        }
    }

    fn process(&mut self, _delta: f64) {
        self.devices_changed = false;

        while let Ok(connection) = self.listener.accept() {
            let (receiver, sender) = connection.split();

            let receiver = BufReader::new(receiver);

            self.streams.push((sender, receiver, Vec::new()));
        }

        for (_, reader, read_buffer) in &mut self.streams {
            let Ok(_) = reader.read_until(b'\n', read_buffer) else {
                continue;
            };
            read_buffer.remove(read_buffer.len() - 1);

            if let Ok(serialized_message) = BASE64_STANDARD.decode(&read_buffer)
                && let Ok(archived) =
                    rkyv::access::<ArchivedUIMessage, rkyv::rancor::Error>(&serialized_message)
                && let Ok(deserialized) =
                    rkyv::deserialize::<UIMessage, rkyv::rancor::Error>(archived)
            {
                match deserialized {
                    UIMessage::DeviceConnected(device, name) => {
                        let _ = self.devices.insert(device, (name, None));

                        godot_print!("device connected");

                        self.event_queue
                            .push_back(InternalUIEvent::GetLibrary(device));

                        self.devices_changed = true;
                    }
                    UIMessage::DeviceDisconnected(device) => {
                        let _ = self.devices.remove(&device);

                        self.devices_changed = true;

                        godot_print!("device disconnected");
                    }
                    UIMessage::UpdateDeckState(deck_state) => {
                        self.deck_state = deck_state;
                    }
                    UIMessage::DeviceLibrary { device, library } => {
                        if let Some(device) = self.devices.get_mut(&device) {
                            device.1 = Some(library);
                        }

                        self.devices_changed = true;

                        godot_print!("device library");
                    }
                }
            }

            read_buffer.clear();
        }

        while let Some(event) = self.event_queue.pop_front() {
            if let Ok(serialized_event) = rkyv::to_bytes::<rkyv::rancor::Error>(&event) {
                for (stream, _, _) in &mut self.streams {
                    let _ = stream.write_all(
                        (BASE64_STANDARD.encode(serialized_event.clone()) + "\n").as_bytes(),
                    );
                }
            }
        }
    }
}

impl IPC {
    pub fn send_event(&mut self, ui_event: InternalUIEvent) {
        self.event_queue.push_back(ui_event);
    }
}

impl Drop for IPC {
    fn drop(&mut self) {}
}
