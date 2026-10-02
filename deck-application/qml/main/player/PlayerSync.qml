import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window

import engineering.fizzy.deck_application

FlexboxLayout {
    id: root
    required property EngineBridge engine

    Layout.preferredHeight: 64
    Layout.fillHeight: false
    direction: FlexboxLayout.Row
    justifyContent: FlexboxLayout.JustifySpaceAround
    alignItems: FlexboxLayout.AlignCenter

    Item {
        id: waveform_container

        clip: true

        Layout.fillWidth: true
        Layout.fillHeight: true

        Rectangle {
            x: waveform_container.x + (waveform_container.width / 2)
            y: 0
            width: 1
            height: waveform_container.height

            color: palette.light
        }
    }
}
