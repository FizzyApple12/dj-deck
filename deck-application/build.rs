use cxx_qt_build::{CxxQtBuilder, MocArguments, QmlModule};
use qt_build_utils::QtBuild;

fn main() {
    println!("cargo:rerun-if-changed=include/qfont_features.h");
    println!("cargo:rerun-if-changed=include/gpu_texture_source.h");

    let mut qt_build = QtBuild::new(vec![
        "Core".into(),
        "Gui".into(),
        "Qml".into(),
        "Quick".into(),
    ])
    .expect("Could not find Qt installation");

    let moc = qt_build
        .moc()
        .compile("include/gpu_texture_source.h", MocArguments::default());

    let mut cpp_build = cc::Build::new();
    cpp_build
        .cpp(true)
        .std("c++17")
        .include("include")
        .file(&moc.cpp);

    qt_build.cargo_link_libraries(&mut cpp_build);

    cpp_build.compile("gpu_texture_source");

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
                "qml/main/layouts/TopBar.qml",
                "qml/main/layouts/PlayerRow.qml",
                "qml/main/layouts/PlayerColumn.qml",
                "qml/mixer/MixerWindow.qml",
            ]),
        )
        .qrc("qml/fonts/fonts.qrc")
        .qrc("qml/shaders/shaders.qrc")
        .qrc("qml/icons/icons.qrc")
        .qrc("qml/test_images/test_images.qrc")
        .include_dir("include/")
        .files(vec![
            "src/components/mod.rs",
            "src/components/engine_bridge.rs",
        ])
        .qt_module("Qml")
        .qt_module("Network")
        .qt_module("QuickLayouts")
        .qt_module("Svg")
        .cc_builder(|cc| {
            // cc.define("QT_QML_DEBUG", None);

            let _ = cc;
        })
        .build();
    }
}
