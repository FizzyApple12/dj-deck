use base64::{Engine, prelude::BASE64_STANDARD};
use interprocess::local_socket::{
    GenericFilePath, ToFsName, tokio::Stream, traits::tokio::Stream as _,
};
use libdj::types::deck::{DeckState, DeckUpdate};
use thiserror::Error;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    task::JoinHandle,
};

use crate::types::ui::{ArchivedInternalUIEvent, InternalUIEvent, SOCKET_NAME, UIEvent, UIMessage};

#[derive(Debug)]
pub struct UI {
    ui_message_sender: tokio::sync::broadcast::Sender<UIMessage>,
    ui_event_receiver: tokio::sync::mpsc::Receiver<UIEvent>,

    ipc_task: JoinHandle<()>,
}

#[derive(Error, Debug)]
pub enum UISendError {
    #[error("Event Loop Error: {0}")]
    ChannelError(tokio::sync::broadcast::error::SendError<UIMessage>),
}

impl tokio_stream::Stream for UI {
    type Item = UIEvent;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        context: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.ui_event_receiver.poll_recv(context)
    }
}

#[derive(Error, Debug)]
pub enum StartUIError {
    #[error("Event Loop Error: {0}")]
    ChannelError(tokio::sync::oneshot::error::RecvError),

    #[error("Socket Name Error: {0}")]
    SocketNameError(std::io::Error),
}

impl UI {
    pub async fn start_ui(
        deck_update_sender: tokio::sync::mpsc::UnboundedSender<DeckUpdate>,
    ) -> Result<UI, StartUIError> {
        let (ui_message_sender, mut ui_message_receiver) = tokio::sync::broadcast::channel(16);
        let (ui_event_sender, ui_event_receiver) = tokio::sync::mpsc::channel(16);

        let (error_sender, error_receiver) = tokio::sync::oneshot::channel();

        let ipc_task = tokio::task::spawn(async move {
            let name = match SOCKET_NAME.to_fs_name::<GenericFilePath>() {
                Ok(name) => name,
                Err(err) => {
                    let _ = error_sender.send(Err(StartUIError::SocketNameError(err)));

                    return;
                }
            };

            let _ = error_sender.send(Ok(()));

            loop {
                let Ok(connection) = Stream::connect(name.clone()).await else {
                    continue;
                };

                let (receiver, mut sender) = connection.split();

                let mut receiver = BufReader::new(receiver);

                let mut received_message = Vec::new();

                loop {
                    tokio::select! {
                        result = receiver.read_until(b'\n', &mut received_message) => {
                            match result {
                                Ok(_) => {
                                    // remove the newline
                                    received_message.remove(received_message.len() - 1);

                                    if let Ok(serialized_message) = BASE64_STANDARD.decode(&received_message)
                                        && let Ok(archived) = rkyv::access::<ArchivedInternalUIEvent, rkyv::rancor::Error>(&serialized_message)
                                        && let Ok(deserialized) = rkyv::deserialize::<InternalUIEvent, rkyv::rancor::Error>(archived) {
                                        process_internal_ui_event(deserialized, &ui_event_sender, &deck_update_sender).await;
                                    }

                                    received_message.clear();
                                },
                                Err(_) => {
                                    break;
                                },
                             }
                        }
                        message = ui_message_receiver.recv() => {
                            if let Ok(message) = message &&
                                let Ok(serialized_message) = rkyv::to_bytes::<rkyv::rancor::Error>(&message) {
                                match sender.write_all((BASE64_STANDARD.encode(serialized_message) + "\n").as_bytes()).await {
                                    Ok(()) => {},
                                    Err(_) => {
                                        break;
                                    },
                                }
                            }
                        }
                    }
                }
            }
        });

        match error_receiver.await {
            Ok(Ok(())) => Ok(UI {
                ui_message_sender,
                ui_event_receiver,

                ipc_task,
            }),
            Ok(Err(err)) => Err(err),
            Err(err) => Err(StartUIError::ChannelError(err)),
        }
    }

    pub fn send(self: &UI, message: UIMessage) -> Result<(), UISendError> {
        match self.ui_message_sender.send(message) {
            Ok(_) => Ok(()),
            Err(err) => Err(UISendError::ChannelError(err)),
        }
    }
}

impl Drop for UI {
    fn drop(&mut self) {
        self.ipc_task.abort();
    }
}

async fn process_internal_ui_event(
    event: InternalUIEvent,
    ui_event_sender: &tokio::sync::mpsc::Sender<UIEvent>,
    deck_update_sender: &tokio::sync::mpsc::UnboundedSender<DeckUpdate>,
) {
    match event {
        InternalUIEvent::LoadTrack { device, id, player } => {
            let _ = ui_event_sender
                .send(UIEvent::LoadTrack { device, id, player })
                .await;
        }
        InternalUIEvent::Eject(player) => {
            let _ = ui_event_sender.send(UIEvent::Eject(player)).await;
        }
        InternalUIEvent::GetLibrary(device) => {
            let _ = ui_event_sender.send(UIEvent::GetLibrary(device)).await;
        }
        InternalUIEvent::EjectDevice(device) => {
            let _ = ui_event_sender.send(UIEvent::EjectDevice(device)).await;
        }
        InternalUIEvent::TouchCue { cue_time, player } => {
            let _ = deck_update_sender.send(Box::new(move |deck_state: &mut DeckState| {
                if let Some(mixer_channel) = deck_state.mixer_channels.get_mut(player) {
                    mixer_channel.player.touch_cue_time = cue_time;
                }
            }));
        }
        InternalUIEvent::BeatJump { beats, player } => {
            let _ = deck_update_sender.send(Box::new(move |deck_state: &mut DeckState| {
                if let Some(mixer_channel) = deck_state.mixer_channels.get_mut(player)
                    && let Some(bpm) = mixer_channel.player.get_current_bpm()
                {
                    mixer_channel.player.time += beats * (1. / bpm) * 60.;
                }
            }));
        }
        InternalUIEvent::SetBeatLoop { beats, player } => {
            let _ = deck_update_sender.send(Box::new(move |deck_state: &mut DeckState| {
                if let Some(mixer_channel) = deck_state.mixer_channels.get_mut(player)
                    && let Some(bpm) = mixer_channel.player.get_current_bpm()
                {
                    mixer_channel.player.beat_loop_start = Some(mixer_channel.player.time);
                    mixer_channel.player.beat_loop_end =
                        Some(mixer_channel.player.time + (beats * (1. / bpm) * 60.));
                }
            }));
        }
        InternalUIEvent::DoubleBeatLoop(player) => {
            let _ = deck_update_sender.send(Box::new(move |deck_state: &mut DeckState| {
                if let Some(mixer_channel) = deck_state.mixer_channels.get_mut(player)
                    && let Some(beat_loop_start) = mixer_channel.player.beat_loop_start
                    && let Some(beat_loop_end) = mixer_channel.player.beat_loop_end
                {
                    mixer_channel.player.beat_loop_start = Some(mixer_channel.player.time);
                    mixer_channel.player.beat_loop_end =
                        Some(beat_loop_end + (beat_loop_end - beat_loop_start));
                }
            }));
        }
        InternalUIEvent::HalveBeatLoop(player) => {
            let _ = deck_update_sender.send(Box::new(move |deck_state: &mut DeckState| {
                if let Some(mixer_channel) = deck_state.mixer_channels.get_mut(player)
                    && let Some(beat_loop_start) = mixer_channel.player.beat_loop_start
                    && let Some(beat_loop_end) = mixer_channel.player.beat_loop_end
                {
                    mixer_channel.player.beat_loop_start = Some(mixer_channel.player.time);
                    mixer_channel.player.beat_loop_end =
                        Some(beat_loop_end - ((beat_loop_end - beat_loop_start) / 2.));
                }
            }));
        }
        InternalUIEvent::SetKeyShift { player, semitones } => {
            let _ = deck_update_sender.send(Box::new(move |deck_state: &mut DeckState| {
                if let Some(mixer_channel) = deck_state.mixer_channels.get_mut(player) {
                    mixer_channel.player.keyshift = semitones;
                }
            }));
        }
    }
}
