use std::{
    io::Write,
    sync::{Arc, Mutex},
};

use base64::{Engine, prelude::BASE64_STANDARD};
use godot::{
    classes::{FileAccess, INode, Node, file_access::ModeFlags},
    prelude::*,
};
use interprocess::local_socket::{
    GenericFilePath, Listener, ListenerNonblockingMode, ListenerOptions, Stream, prelude::*,
};
use libui::types::ui::{SOCKET_NAME, UIEvent};
use thiserror::Error;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct IPC {
    base: Base<Node>,

    listener: Listener,

    streams: Vec<Stream>,

    event_queue: Vec<UIEvent>,
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
            event_queue: Vec::new(),
        }
    }

    fn process(&mut self, _delta: f64) {
        while let Ok(stream) = self.listener.accept() {
            self.streams.push(stream);
        }

        for stream in &self.streams {
            // stream.read
        }

        // process events

        while let Some(event) = self.event_queue.pop() {
            if let Ok(serialized_event) = rkyv::to_bytes::<rkyv::rancor::Error>(&event) {
                let mut dead_streams: Vec<usize> = self
                    .streams
                    .iter()
                    .enumerate()
                    .map(|(i, mut stream)| {
                        match stream.write_all(
                            (BASE64_STANDARD.encode(serialized_event.clone()) + "\n").as_bytes(),
                        ) {
                            Ok(()) => (i, true),
                            Err(_) => (i, false),
                        }
                    })
                    .filter_map(|(i, success)| if success { None } else { Some(i) })
                    .collect();

                dead_streams.sort_by(|a, b| b.cmp(a));

                for index in dead_streams {
                    let _ = self.streams.try_remove(index);
                }
            }
        }
    }
}

impl Drop for IPC {
    fn drop(&mut self) {}
}
