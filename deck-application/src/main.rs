pub mod components;
pub mod logger;
pub mod statuses;

use std::pin::Pin;

use cxx_qt_lib::{QFont, QGuiApplication, QQmlApplicationEngine, QString, QUrl};
use libdatabase::device_manager::DeviceManager;
use libdj::{engine::DJEngine, types::deck::DeckState};
use libio::controller::Controller;
use log::{debug, info, warn};

use crate::{
    components::{engine_bridge::EngineBridgeRust, ffi::set_qfont_feature},
    logger::setup_logger,
};

const LOCAL_AUTOMOUNTS: [(&str, usize); 2] = [("/djusb/usb0", 0), ("/djusb/usb1", 1)];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger()?;

    info!("Starting tokio runtime...");

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()?;

    debug!("Entering tokio to start managers");

    let enter_handle = runtime.enter();

    debug!("Starting IO Manager...");

    let (deck_control_event_sender, deck_control_event_receiver) =
        tokio::sync::mpsc::unbounded_channel();

    // todo: this needs to be replaced with a proper io manager
    let mut controller = Controller::default();

    'controller_init: {
        if let Err(err) = controller.open_midi_input() {
            warn!("Failed to find MIDI input for testing, MIDI has been disabled: {err}");
            break 'controller_init;
        }
        if let Err(err) = controller.open_midi_output() {
            warn!("Failed to find MIDI output for testing, MIDI has been disabled: {err}");
            break 'controller_init;
        }

        if let Err(err) = controller.start(deck_control_event_sender) {
            warn!("Failed to start MIDI processing, MIDI has been disabled: {err}");
            break 'controller_init;
        }
    }

    debug!("Starting Device Manager...");

    let mut device_manager = DeviceManager::default();

    device_manager.start_local_watch(&LOCAL_AUTOMOUNTS)?;
    // device_manager.start_prodj_link_watch(0x10)?;

    debug!("Starting DJ Engine...");

    let (deck_state_sender, deck_state_receiver) =
        tokio::sync::watch::channel(DeckState::default());
    let (deck_update_sender, deck_update_receiver) = tokio::sync::mpsc::unbounded_channel();
    let (loaded_track_sender, loaded_track_receiver) = tokio::sync::mpsc::unbounded_channel();

    let (ui_control_event_sender, ui_control_event_receiver) =
        tokio::sync::mpsc::unbounded_channel();

    let dj_engine = DJEngine::start(
        deck_control_event_receiver,
        ui_control_event_sender,
        deck_update_receiver,
        deck_state_sender,
        loaded_track_receiver,
    );

    drop(enter_handle);

    let handle = runtime.handle().clone();

    EngineBridgeRust::register(
        handle,
        dj_engine,
        deck_state_receiver,
        deck_update_sender,
        ui_control_event_receiver,
        loaded_track_sender,
        device_manager,
    );

    let mut app = QGuiApplication::new();

    if let Some(app) = app.as_mut() {
        let mut font = QFont::default();

        font.set_family(&QString::from("Helvetica"));
        // font.clear_features();

        // i hate this but cxx_qt is out of date and i can't be bothered to open a pr
        // right now
        set_qfont_feature(Pin::new(&mut font), &QString::from("kern"), 1);
        set_qfont_feature(Pin::new(&mut font), &QString::from("liga"), 1);
        set_qfont_feature(Pin::new(&mut font), &QString::from("tnum"), 1);

        app.set_application_font(&font);
    }

    let mut engine = QQmlApplicationEngine::new();

    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from(
            "qrc:/qt/qml/engineering/fizzy/deck_application/qml/root.qml",
        ));
    }

    if let Some(app) = app.as_mut() {
        app.exec();
    }

    runtime.shutdown_background();

    Ok(())
}
