pub mod external_app;
pub mod ui_elements;

use std::{sync::Arc, thread};

use masonry::{core::DefaultProperties, theme::default_property_set};
use thiserror::Error;
use tokio_stream::Stream;
use winit::error::EventLoopError;
use xilem::{
    Blob, EventLoop, WidgetView, WindowOptions, Xilem,
    view::{Axis, ChildAlignment, CrossAxisAlignment, MainAxisAlignment, ZStackExt, flex},
    winit::platform::wayland::EventLoopBuilderExtWayland,
};

use crate::{
    gui::{
        external_app::ExternalApp,
        ui_elements::{small_players::players, waveforms},
        waveforms::waveforms,
    },
    types::ui::{UIEvent, UIMessage},
};

pub struct UIState {
    ui_message_sender: tokio::sync::broadcast::Sender<UIMessage>,
    ui_event_sender: tokio::sync::mpsc::Sender<UIEvent>,
}

#[derive(Debug)]
pub struct UI {
    ui_message_sender: tokio::sync::broadcast::Sender<UIMessage>,
    ui_event_receiver: tokio::sync::mpsc::Receiver<UIEvent>,
}

#[derive(Error, Debug)]
pub enum UISendError {
    #[error("Event Loop Error: {0}")]
    ChannelError(tokio::sync::broadcast::error::SendError<UIMessage>),
}

impl Stream for UI {
    type Item = UIEvent;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        context: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.ui_event_receiver.poll_recv(context)
    }
}

fn app_logic(data: &mut UIState) -> impl WidgetView<UIState> + use<> {
    flex(Axis::Vertical, (waveforms(data), players(data)))
        .main_axis_alignment(MainAxisAlignment::SpaceBetween)
        .cross_axis_alignment(CrossAxisAlignment::Fill)
}

#[derive(Error, Debug)]
pub enum StartUIError {
    #[error("Event Loop Error: {0}")]
    ChannelError(tokio::sync::oneshot::error::RecvError),

    #[error("Event Loop Error: {0}")]
    EventLoopError(EventLoopError),
}

const HELVETICA: &[u8] = include_bytes!("../../assets/helvetica.ttf");

impl UI {
    pub async fn start_ui() -> Result<UI, StartUIError> {
        let (ui_message_sender, _) = tokio::sync::broadcast::channel(16);
        let (ui_event_sender, ui_event_receiver) = tokio::sync::mpsc::channel(16);

        let app_state = UIState {
            ui_message_sender: ui_message_sender.clone(),
            ui_event_sender,
        };

        let (error_sender, error_receiver) = tokio::sync::oneshot::channel();

        let _ = thread::spawn(move || {
            let xilem = Xilem::new_simple(app_state, app_logic, WindowOptions::new("djui"))
                .with_font(Blob::new(Arc::new(HELVETICA)));

            let event_loop = match EventLoop::with_user_event().with_any_thread(true).build() {
                Ok(event_loop) => event_loop,
                Err(err) => {
                    let _ = error_sender.send(Err(StartUIError::EventLoopError(err)));
                    return;
                }
            };

            let proxy = event_loop.create_proxy();
            let (driver, windows) = xilem
                .into_driver_and_windows(move |event| proxy.send_event(event).map_err(|err| err.0));

            let masonry_state = masonry_winit::app::MasonryState::new(
                event_loop.create_proxy(),
                windows,
                default_property_set(),
            );

            let mut app = ExternalApp {
                masonry_state,
                app_driver: Box::new(driver),
            };

            let _ = error_sender.send(Ok(()));

            let _ = event_loop.run_app(&mut app);
        });

        match error_receiver.await {
            Ok(Ok(())) => Ok(UI {
                ui_message_sender,
                ui_event_receiver,
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
