use base64::{Engine, prelude::BASE64_STANDARD};
use interprocess::local_socket::{
    GenericFilePath, ToFsName, tokio::Stream, traits::tokio::Stream as _,
};
use thiserror::Error;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    task::JoinHandle,
};

use crate::types::ui::{ArchivedUIEvent, SOCKET_NAME, UIEvent, UIMessage};

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
    pub async fn start_ui() -> Result<UI, StartUIError> {
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
                                        && let Ok(archived) = rkyv::access::<ArchivedUIEvent, rkyv::rancor::Error>(&serialized_message)
                                        && let Ok(deserialized) = rkyv::deserialize::<UIEvent, rkyv::rancor::Error>(archived) {
                                        let _ = ui_event_sender.send(deserialized).await;
                                    }
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
