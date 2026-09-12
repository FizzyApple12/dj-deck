import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window

import "qrc:/test_images"
import "qrc:/shaders"

FlexboxLayout {
    required property int player_number

    direction: FlexboxLayout.Row
    justifyContent: FlexboxLayout.JustifySpaceAround
    alignItems: FlexboxLayout.AlignCenter

    Item {
        id: waveform_container

        clip: true

        Layout.fillWidth: true
        Layout.fillHeight: true

        Image {
            id: waveform_image

            visible: false

            sourceSize {
                width: 16384
                height: 2
            }

            source: "qrc:/test_images/test-waveform.png"
        }

        ShaderEffect {
            x: (waveform_container.x + (waveform_container.width / 2)) - 15000
            y: 0
            width: 50000
            height: waveform_container.height

            vertexShader: "qrc:/shaders/waveform.vert.qsb"
            fragmentShader: "qrc:/shaders/waveform.frag.qsb"

            property real stride: 1.8656616
            property variant waveform: waveform_image
        }

        Rectangle {
            x: waveform_container.x + (waveform_container.width / 2)
            y: 0
            width: 1
            height: waveform_container.height

            color: "#ff0000"
        }
    }
}
