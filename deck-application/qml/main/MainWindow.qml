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
    color: palette.window
    title: "1"

    FlexboxLayout {
        anchors.fill: parent
        direction: FlexboxLayout.Column
        justifyContent: FlexboxLayout.JustifySpaceBetween
        alignItems: FlexboxLayout.AlignCenter

        visible: false

        TopBar {
            engine: root.engine
            page_title: ""
        }

        FlexboxLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            direction: FlexboxLayout.Row
            justifyContent: FlexboxLayout.JustifySpaceBetween
            alignItems: FlexboxLayout.AlignCenter

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
    }

    FlexboxLayout {
        anchors.fill: parent
        direction: FlexboxLayout.Column
        justifyContent: FlexboxLayout.JustifySpaceBetween
        alignItems: FlexboxLayout.AlignEnd

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

            FlexboxLayout {
                Layout.fillWidth: false
                Layout.fillHeight: true
                Layout.preferredWidth: 640
                direction: FlexboxLayout.Column
                justifyContent: FlexboxLayout.JustifyStart
                alignContent: FlexboxLayout.AlignStart
                alignItems: FlexboxLayout.AlignStart

                TopBar {
                    engine: root.engine
                    page_title: ""
                }
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

        color: palette.window
    }

    FlexboxLayout {
        anchors.fill: parent
        direction: FlexboxLayout.Column
        justifyContent: FlexboxLayout.JustifyStart
        alignItems: FlexboxLayout.AlignStart

        visible: root.engine.browser_page != BrowserPage.Closed

        TopBar {
            engine: root.engine
            page_title: "Browser"
        }

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

        color: palette.window
    }

    FlexboxLayout {
        anchors.fill: parent
        direction: FlexboxLayout.Column
        justifyContent: FlexboxLayout.JustifyStart
        alignItems: FlexboxLayout.AlignStart

        visible: root.engine.source_open

        TopBar {
            engine: root.engine
            page_title: "Source"
        }

        SourceSelect {
            engine: root.engine
        }
    }
}
