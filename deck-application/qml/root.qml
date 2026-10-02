import QtQuick 2.3

import "./main"
import "./mixer"

import engineering.fizzy.deck_application

MainWindow {
    id: main_window

    EngineBridge {
        id: engine

        source_index: SourceListModel {}
        browser_index: BrowserListModel {}
    }

    engine: engine

    Connections {
        target: main_window
        function onFrameSwapped() {
            engine.before_frame();
            main_window.update();
        }
    }

    palette {
        accent: "#224ba5"
        base: "#1fffffff"
        alternateBase: "#1f7f7f7f"
        dark: "#000000"
        mid: "#3effffff"
        midlight: "#5dffffff"
        light: "#ffffff"
        shadow: "#00000000"

        window: "#000000"
        windowText: "#ffffff"

        button: "#1fffffff"
        buttonText: "#000000"

        link: "#ffffff"
        linkVisited: "#ffffff"

        text: "#ffffff"
        brightText: "#ffffff"
        placeholderText: "#55ffffff"

        highlight: "#ffffff"
        highlightedText: "#000000"

        toolTipBase: "#000000"
        toolTipText: "#ffffff"

        disabled {
            accent: "#1f224ba5"
            base: "#1fffffff"
            alternateBase: "#1f7f7f7f"
            dark: "#1f000000"
            mid: "#1fffffff"
            midlight: "#3effffff"
            light: "#1fffffff"
            shadow: "#00000000"

            window: "#1f000000"
            windowText: "#55ffffff"

            button: "#1fffffff"
            buttonText: "#55000000"

            link: "#55ffffff"
            linkVisited: "#55ffffff"

            text: "#55ffffff"
            brightText: "#55ffffff"
            placeholderText: "#55ffffff"

            highlight: "#1fffffff"
            highlightedText: "#55000000"

            toolTipBase: "#1f000000"
            toolTipText: "#55ffffff"
        }
    }

    // MixerWindow {
    //     id: mixer_window
    //     engine: engine

    //     palette: main_window.palette
    // }
}
