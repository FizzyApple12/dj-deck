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
    direction: FlexboxLayout.Row
    justifyContent: FlexboxLayout.JustifySpaceBetween
    alignContent: FlexboxLayout.AlignStretch
    alignItems: FlexboxLayout.AlignCenter

    PlayerWaveform {
        player_number: root.player_number

        Layout.fillWidth: true
    }

    FlexboxLayout {
        direction: FlexboxLayout.Column
        justifyContent: FlexboxLayout.JustifySpaceAround
        alignItems: FlexboxLayout.AlignCenter

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 1

            color: palette.mid
        }

        PlayerDetails {
            Layout.preferredWidth: 640

            player_number: root.player_number
        }
    }
}
