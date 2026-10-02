import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window

import engineering.fizzy.deck_application

import "qrc:/test_images"
import "qrc:/shaders"

FlexboxLayout {
    id: root
    required property EngineBridge engine
    required property int player_number

    direction: FlexboxLayout.Row
    justifyContent: FlexboxLayout.JustifySpaceAround
    alignItems: FlexboxLayout.AlignCenter

    Item {
        id: waveform_container

        clip: true

        Layout.fillWidth: true
        Layout.fillHeight: true


        ShaderEffect {
            x: {
                let center_factor = (waveform_container.x + (waveform_container.width / 2));

                let waveform_width = (root.engine.deck_state.mixer_channel(root.player_number).player.waveform_length_seconds * root.engine.waveform_pixels_per_second) / Math.max(root.engine.deck_state.mixer_channel(root.player_number).player.tempo_percent, 0.1);

                let progress = (root.engine.deck_state.mixer_channel(root.player_number).player.time / 1000000000) / root.engine.deck_state.mixer_channel(root.player_number).player.waveform_length_seconds;

                return center_factor - (waveform_width * progress);
            }

            y: 0
            width: (root.engine.deck_state.mixer_channel(root.player_number).player.waveform_length_seconds * root.engine.waveform_pixels_per_second) / Math.max(root.engine.deck_state.mixer_channel(root.player_number).player.tempo_percent, 0.1)
            height: waveform_container.height

            vertexShader: "qrc:/shaders/waveform.vert.qsb"
            fragmentShader: "qrc:/shaders/waveform.frag.qsb"

            property real stride: root.engine.deck_state.mixer_channel(root.player_number).player.waveform_texture_stride
            property variant waveform: root.engine.deck_state.mixer_channel(root.player_number).player.waveform_texture_source
        }

        Rectangle {
            x: waveform_container.x + (waveform_container.width / 2)
            y: 0
            width: 1
            height: waveform_container.height

            color: palette.light
        }
    }
}
