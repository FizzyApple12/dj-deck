use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    // Safety: This will, at worst, panic during compile time, and rightfully should
    // if something is wrong
    unsafe {
        CxxQtBuilder::new_qml_module(
            QmlModule::new("engineering.fizzy.deck_application").qml_files(vec![
                "qml/root.qml",
                "qml/main/MainWindow.qml",
                "qml/main/browser/SourceSelect.qml",
                "qml/main/browser/Browser.qml",
                "qml/main/player/PlayerDetails.qml",
                "qml/main/player/PlayerWaveform.qml",
                "qml/main/player/PlayerSync.qml",
                "qml/main/layouts/PlayerRow.qml",
                "qml/main/layouts/PlayerColumn.qml",
                "qml/mixer/MixerWindow.qml",
            ]),
        )
        .qrc("qml/fonts/fonts.qrc")
        .qrc("qml/shaders/shaders.qrc")
        .qrc("qml/test_images/test_images.qrc")
        .include_dir("include/")
        .files(vec!["src/components/mod.rs", "src/components/greeter.rs"])
        .qt_module("Qml")
        .qt_module("Network")
        .qt_module("QuickLayouts")
        .cc_builder(|cc| {
            cc.define("QT_QML_DEBUG", None);
        })
        .build();
    }
}
