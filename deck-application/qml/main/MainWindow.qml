import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window

import engineering.fizzy.deck_application

import "./player"
import "./layouts"
import "./browser"

ApplicationWindow {
    id: root
    required property EngineBridge engine

    x: 0
    y: 0
    width: 2560
    height: 700
    visible: true
    flags: Qt.FramelessWindowHint
    color: "#000000"
    title: "1"

    FlexboxLayout {
        anchors.fill: parent
        direction: FlexboxLayout.Row
        justifyContent: FlexboxLayout.JustifySpaceBetween
        alignItems: FlexboxLayout.AlignCenter

        visible: false

        PlayerColumn {
            engine: root.engine
            player_number: 2
        }

        Rectangle {
            Layout.fillHeight: true
            Layout.preferredWidth: 1

            color: palette.mid
        }

        PlayerColumn {
            engine: root.engine
            player_number: 0
        }

        Rectangle {
            Layout.fillHeight: true
            Layout.preferredWidth: 1

            color: palette.mid
        }

        PlayerColumn {
            engine: root.engine
            player_number: 1
        }

        Rectangle {
            Layout.fillHeight: true
            Layout.preferredWidth: 1

            color: palette.mid
        }

        PlayerColumn {
            engine: root.engine
            player_number: 3
        }
    }

    FlexboxLayout {
        anchors.fill: parent
        direction: FlexboxLayout.Column
        justifyContent: FlexboxLayout.JustifySpaceBetween
        alignItems: FlexboxLayout.AlignCenter

        visible: true

        FlexboxLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true

            direction: FlexboxLayout.Row
            justifyContent: FlexboxLayout.JustifySpaceBetween
            alignContent: FlexboxLayout.AlignStretch
            alignItems: FlexboxLayout.AlignCenter

            PlayerSync {
                engine: root.engine
                Layout.fillWidth: true
            }

            Rectangle {
                Layout.preferredWidth: 640
            }
        }

        PlayerRow {
            engine: root.engine
            player_number: 2
        }

        PlayerRow {
            engine: root.engine
            player_number: 0
        }

        PlayerRow {
            engine: root.engine
            player_number: 1
        }

        PlayerRow {
            engine: root.engine
            player_number: 3
        }
    }

    Rectangle {
        x: 0
        y: 0
        width: 2560
        height: 700

        visible: root.engine.browser_page != BrowserPage.Closed

        color: "#000000"
    }

    FlexboxLayout {
        anchors.fill: parent
        direction: FlexboxLayout.Row
        justifyContent: FlexboxLayout.JustifySpaceBetween
        alignItems: FlexboxLayout.AlignCenter

        visible: root.engine.browser_page != BrowserPage.Closed

        Browser {
            engine: root.engine
        }
    }

    Rectangle {
        x: 0
        y: 0
        width: 2560
        height: 700

        visible: root.engine.source_open

        color: "#000000"
    }

    FlexboxLayout {
        anchors.fill: parent
        direction: FlexboxLayout.Row
        justifyContent: FlexboxLayout.JustifySpaceBetween
        alignItems: FlexboxLayout.AlignCenter

        visible: root.engine.source_open

        SourceSelect {
            engine: root.engine
        }
    }
}
