import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window

import "./player"
import "./layouts"

ApplicationWindow {
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

        visible: true

        PlayerColumn {
            player_number: 2
        }

        Rectangle {
            Layout.fillHeight: true
            Layout.preferredWidth: 1

            color: palette.mid
        }

        PlayerColumn {
            player_number: 0
        }

        Rectangle {
            Layout.fillHeight: true
            Layout.preferredWidth: 1

            color: palette.mid
        }

        PlayerColumn {
            player_number: 1
        }

        Rectangle {
            Layout.fillHeight: true
            Layout.preferredWidth: 1

            color: palette.mid
        }

        PlayerColumn {
            player_number: 3
        }
    }

    FlexboxLayout {
        anchors.fill: parent
        direction: FlexboxLayout.Column
        justifyContent: FlexboxLayout.JustifySpaceBetween
        alignItems: FlexboxLayout.AlignCenter

        visible: false

        FlexboxLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true

            direction: FlexboxLayout.Row
            justifyContent: FlexboxLayout.JustifySpaceBetween
            alignContent: FlexboxLayout.AlignStretch
            alignItems: FlexboxLayout.AlignCenter

            PlayerSync {
                Layout.fillWidth: true
            }

            Rectangle {
                Layout.preferredWidth: 640
            }
        }

        PlayerRow {
            player_number: 2
        }

        PlayerRow {
            player_number: 0
        }

        PlayerRow {
            player_number: 1
        }

        PlayerRow {
            player_number: 3
        }
    }
}
