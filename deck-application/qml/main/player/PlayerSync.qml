import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window

FlexboxLayout {
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

            color: "#ff0000"
        }
    }
}
