import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window

import engineering.fizzy.deck_application

import "../player"

FlexboxLayout {
    id: root
    required property EngineBridge engine
    required property int player_number

    Layout.fillWidth: true
    Layout.fillHeight: true
    direction: FlexboxLayout.Row
    justifyContent: FlexboxLayout.JustifySpaceBetween
    alignContent: FlexboxLayout.AlignStretch
    alignItems: FlexboxLayout.AlignCenter

    PlayerWaveform {
        engine: root.engine
        player_number: root.player_number

        Layout.fillWidth: true
        Layout.fillHeight: true
    }

    FlexboxLayout {
        Layout.fillWidth: false
        Layout.fillHeight: true
        direction: FlexboxLayout.Column
        justifyContent: FlexboxLayout.JustifySpaceAround
        alignItems: FlexboxLayout.AlignCenter

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 1

            color: palette.mid
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true

            color: palette.window
        }

        PlayerDetails {
            engine: root.engine
            Layout.preferredWidth: 640

            player_number: root.player_number
        }
    }
}
