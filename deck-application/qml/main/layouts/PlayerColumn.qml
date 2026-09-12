import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window

import "../player"

FlexboxLayout {
    id: root
    required property int player_number

    Layout.fillWidth: true
    Layout.fillHeight: true
    direction: FlexboxLayout.Column
    justifyContent: FlexboxLayout.JustifySpaceBetween
    alignContent: FlexboxLayout.AlignStretch
    alignItems: FlexboxLayout.AlignCenter

    PlayerSync {
        Layout.fillWidth: true
    }

    PlayerWaveform {
        player_number: root.player_number

        Layout.fillHeight: true
    }

    Rectangle {
        Layout.fillWidth: true
        Layout.preferredHeight: 1

        color: palette.mid
    }

    PlayerDetails {
        player_number: root.player_number

        Layout.fillWidth: true
    }
}
