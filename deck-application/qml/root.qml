import QtQuick 2.3

import "./main"
import "./mixer"

MainWindow {
    id: main_window

    palette {
        accent: "#ff820e"
        base: "#33ffffff"
        alternateBase: "#33ffffff"
        dark: "#000000"
        mid: "#555555"
        midlight: "#aaaaaa"
        light: "#ffffff"
        shadow: "#00000000"

        window: "#000000"
        windowText: "#ffffff"

        button: "#33ffffff"
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
            accent: "#33ff820e"
            base: "#33ffffff"
            alternateBase: "#33ffffff"
            dark: "#33000000"
            mid: "#33555555"
            midlight: "#33aaaaaa"
            light: "#33ffffff"
            shadow: "#00000000"

            window: "#33000000"
            windowText: "#55ffffff"

            button: "#33ffffff"
            buttonText: "#55000000"

            link: "#55ffffff"
            linkVisited: "#55ffffff"

            text: "#55ffffff"
            brightText: "#55ffffff"
            placeholderText: "#55ffffff"

            highlight: "#33ffffff"
            highlightedText: "#55000000"

            toolTipBase: "#33000000"
            toolTipText: "#55ffffff"
        }
    }

    MixerWindow {
        id: mixer_window

        palette: main_window.palette
    }
}
