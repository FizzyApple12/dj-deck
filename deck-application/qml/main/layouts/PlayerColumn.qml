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
    direction: FlexboxLayout.Column
    justifyContent: FlexboxLayout.JustifySpaceBetween
    alignContent: FlexboxLayout.AlignStretch
    alignItems: FlexboxLayout.AlignCenter

    PlayerSync {
        engine: root.engine

        Layout.fillWidth: true
    }

    PlayerWaveform {
        engine: root.engine

        player_number: root.player_number

        Layout.fillHeight: true
    }

    PlayerDetails {
        engine: root.engine

        player_number: root.player_number

        Layout.fillWidth: true
    }
}
