import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window

import engineering.fizzy.deck_application

Rectangle {
    id: root
    required property EngineBridge engine
    required property int player_number

    Layout.preferredHeight: inner_root.height

    color: palette.base

    FlexboxLayout {
        id: inner_root

        direction: FlexboxLayout.Row
        justifyContent: FlexboxLayout.JustifyCenter
        alignItems: FlexboxLayout.AlignCenter

        Label {
            Layout.preferredWidth: 24
            horizontalAlignment: Text.AlignHCenter
            rotation: 270

            font.pointSize: 12
            font.variableAxes: {
                "opsz": 4
            }
            font.weight: Font.Medium

            text: qsTr("Deck " + (root.player_number + 1))
        }

        FlexboxLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true

            direction: FlexboxLayout.Column
            justifyContent: FlexboxLayout.JustifyStart
            alignItems: FlexboxLayout.AlignCenter

            FlexboxLayout {
                direction: FlexboxLayout.Row
                justifyContent: FlexboxLayout.JustifyStart
                alignItems: FlexboxLayout.AlignCenter
                Layout.fillWidth: true
                Layout.fillHeight: true

                Label {
                    leftPadding: 4
                    rightPadding: 4
                    horizontalAlignment: Text.AlignHCenter

                    font.pointSize: 12
                    font.variableAxes: {
                        "opsz": 10
                    }
                    font.weight: Font.Medium

                    color: palette.text

                    text: {
                        if (root.engine.deck_state.mixer_channel(root.player_number).player.is_loading) {
                            return qsTr("Loading...");
                        } else if (root.engine.deck_state.mixer_channel(root.player_number).player.track_loaded) {
                            return root.engine.deck_state.mixer_channel(root.player_number).player.track_name;
                        } else {
                            return qsTr("Not Loaded");
                        }
                    }
                }
            }

            FlexboxLayout {
                direction: FlexboxLayout.Row
                justifyContent: FlexboxLayout.JustifySpaceBetween
                alignItems: FlexboxLayout.AlignCenter
                Layout.fillWidth: true
                Layout.fillHeight: false

                FlexboxLayout {
                    id: time_box

                    direction: FlexboxLayout.Column
                    justifyContent: FlexboxLayout.JustifyStart
                    alignContent: FlexboxLayout.AlignStretch

                    gap: 0

                    TapHandler {
                        onTapped: root.engine.deck_state.mixer_channel(root.player_number).player.setDisplay_remaining(!root.engine.deck_state.mixer_channel(root.player_number).player.display_remaining)
                    }

                    FlexboxLayout {
                        Layout.fillWidth: true

                        direction: FlexboxLayout.Row
                        justifyContent: FlexboxLayout.JustifyStart
                        alignItems: FlexboxLayout.AlignCenter

                        Label {
                            leftPadding: 4
                            rightPadding: 4
                            horizontalAlignment: Text.AlignHCenter

                            font.pointSize: 10
                            font.variableAxes: {
                                "opsz": 4
                            }
                            font.weight: Font.Medium

                            color: root.engine.deck_state.mixer_channel(root.player_number).player.display_remaining ? palette.text : palette.disabled.text

                            text: qsTr("REMAIN")
                        }
                        Label {

                            leftPadding: 4
                            rightPadding: 4
                            horizontalAlignment: Text.AlignHCenter

                            font.pointSize: 10
                            font.variableAxes: {
                                "opsz": 4
                            }
                            font.weight: Font.Medium

                            color: palette.disabled.text

                            text: "/"
                        }
                        Label {
                            leftPadding: 4
                            rightPadding: 4
                            horizontalAlignment: Text.AlignHCenter

                            font.pointSize: 10
                            font.variableAxes: {
                                "opsz": 4
                            }
                            font.weight: Font.Medium

                            color: root.engine.deck_state.mixer_channel(root.player_number).player.display_remaining ? palette.disabled.text : palette.text

                            text: qsTr("TIME")
                        }
                    }

                    FlexboxLayout {
                        Layout.fillWidth: true

                        direction: FlexboxLayout.Row
                        justifyContent: FlexboxLayout.JustifyCenter
                        alignItems: FlexboxLayout.AlignEnd

                        Label {
                            Layout.preferredWidth: 122
                            leftPadding: 4
                            topPadding: -8
                            bottomPadding: -8
                            horizontalAlignment: Text.AlignRight

                            font.pointSize: 32
                            font.variableAxes: {
                                "opsz": 30
                            }
                            font.weight: Font.Medium

                            color: palette.text

                            text: {
                                let time_seconds = root.engine.deck_state.mixer_channel(root.player_number).player.time / 1000000000;

                                if (root.engine.deck_state.mixer_channel(root.player_number).player.display_remaining) {
                                    time_seconds = (root.engine.deck_state.mixer_channel(root.player_number).player.track_length / 1000000000) - time_seconds;
                                }

                                let timecode_minutes = (time_seconds / 60) % 100;
                                let timecode_seconds = time_seconds % 60;

                                if (time_seconds >= 0) {
                                    timecode_minutes = Math.floor(timecode_minutes);
                                    timecode_seconds = Math.floor(timecode_seconds);
                                } else {
                                    timecode_minutes = Math.abs(Math.ceil(timecode_minutes));
                                    timecode_seconds = Math.abs(Math.ceil(timecode_seconds));
                                }

                                return `${timecode_minutes.toString().padStart(2, "0")}:${timecode_seconds.toString().padStart(2, "0")}`;
                            }
                        }

                        Label {
                            leftPadding: 0
                            rightPadding: 4
                            topPadding: -8
                            bottomPadding: -1
                            horizontalAlignment: Text.AlignLeft

                            font.pointSize: 18
                            font.variableAxes: {
                                "opsz": 10
                            }
                            font.weight: Font.Medium

                            color: palette.text

                            text: {
                                let time_milliseconds = root.engine.deck_state.mixer_channel(root.player_number).player.time / 1000000;

                                if (root.engine.deck_state.mixer_channel(root.player_number).player.display_remaining) {
                                    time_milliseconds = (root.engine.deck_state.mixer_channel(root.player_number).player.track_length / 1000000) - time_milliseconds;
                                }

                                let timecode_milliseconds = time_milliseconds % 1000;

                                if (time_milliseconds >= 0) {
                                    timecode_milliseconds = Math.floor(timecode_milliseconds);
                                } else {
                                    timecode_milliseconds = Math.abs(Math.ceil(timecode_milliseconds));
                                }

                                return `.${timecode_milliseconds.toString().padStart(3, "0")}`;
                            }
                        }
                    }
                }

                FlexboxLayout {
                    id: tempo_box

                    direction: FlexboxLayout.Column
                    justifyContent: FlexboxLayout.JustifyStart
                    alignContent: FlexboxLayout.AlignStretch

                    gap: 0

                    FlexboxLayout {
                        Layout.fillWidth: true

                        direction: FlexboxLayout.Row
                        justifyContent: FlexboxLayout.JustifySpaceBetween
                        alignItems: FlexboxLayout.AlignCenter

                        Label {
                            leftPadding: 4
                            rightPadding: 4
                            horizontalAlignment: Text.AlignHCenter

                            font.pointSize: 10
                            font.variableAxes: {
                                "opsz": 4
                            }
                            font.weight: Font.Medium

                            color: palette.text

                            text: qsTr("TEMPO")
                        }

                        Label {
                            leftPadding: 4
                            rightPadding: 4
                            horizontalAlignment: Text.AlignHCenter

                            font.pointSize: 10
                            font.variableAxes: {
                                "opsz": 4
                            }
                            font.weight: Font.Medium

                            color: palette.highlightedText

                            background: Rectangle {
                                color: palette.highlight

                                visible: true
                            }

                            text: {
                                switch (root.engine.deck_state.mixer_channel(root.player_number).player.tempo_range) {
                                case TempoRange.SixPercent:
                                    return "± 6%";
                                case TempoRange.TenPercent:
                                    return "± 10%";
                                case TempoRange.SixteenPercent:
                                    return "± 16%";
                                case TempoRange.OneHundredPercent:
                                    return "± 100%";
                                default:
                                    return "± ???%";
                                }
                            }
                        }
                    }

                    FlexboxLayout {
                        Layout.fillWidth: true

                        direction: FlexboxLayout.Row
                        justifyContent: FlexboxLayout.JustifyCenter
                        alignItems: FlexboxLayout.AlignEnd

                        Label {
                            Layout.preferredWidth: 34
                            leftPadding: 4
                            topPadding: -8
                            bottomPadding: -8
                            horizontalAlignment: Text.AlignRight

                            font.pointSize: 32
                            font.variableAxes: {
                                "opsz": 30
                            }
                            font.weight: Font.Medium

                            color: palette.text

                            text: {
                                if (root.engine.deck_state.mixer_channel(root.player_number).player.tempo_percent >= 1) {
                                    return "+";
                                } else {
                                    return "-";
                                }
                            }
                        }

                        Label {
                            Layout.preferredWidth: 84
                            leftPadding: 4
                            topPadding: -8
                            bottomPadding: -8
                            horizontalAlignment: Text.AlignRight

                            font.pointSize: 32
                            font.variableAxes: {
                                "opsz": 30
                            }
                            font.weight: Font.Medium

                            color: palette.text

                            text: Math.floor(Math.abs(root.engine.deck_state.mixer_channel(root.player_number).player.tempo_percent - 1) * 100).toString()
                        }

                        Label {
                            leftPadding: 0
                            rightPadding: 4
                            topPadding: -8
                            bottomPadding: -1
                            horizontalAlignment: Text.AlignLeft

                            font.pointSize: 18
                            font.variableAxes: {
                                "opsz": 10
                            }
                            font.weight: Font.Medium

                            color: palette.text

                            text: "." + Math.floor((Math.abs(root.engine.deck_state.mixer_channel(root.player_number).player.tempo_percent - 1) * 10000) % 100).toString().padStart(2, "0") + "%"
                        }
                    }
                }

                FlexboxLayout {
                    id: bpm_box

                    direction: FlexboxLayout.Column
                    justifyContent: FlexboxLayout.JustifyStart
                    alignContent: FlexboxLayout.AlignStretch

                    gap: 0

                    FlexboxLayout {
                        Layout.fillWidth: true

                        direction: FlexboxLayout.Row
                        justifyContent: FlexboxLayout.JustifySpaceBetween
                        alignItems: FlexboxLayout.AlignCenter

                        Label {
                            leftPadding: 4
                            rightPadding: 4
                            horizontalAlignment: Text.AlignHCenter

                            font.pointSize: 10
                            font.variableAxes: {
                                "opsz": 4
                            }
                            font.weight: Font.Medium

                            color: palette.text

                            text: qsTr("BPM")
                        }

                        Label {
                            leftPadding: 4
                            rightPadding: 4
                            horizontalAlignment: Text.AlignHCenter

                            font.pointSize: 10
                            font.variableAxes: {
                                "opsz": 4
                            }
                            font.weight: Font.Medium

                            color: palette.highlightedText
                            visible: root.engine.deck_state.master_channel_set && (root.engine.deck_state.master_channel == root.player_number)

                            background: Rectangle {
                                color: "#ff820e"

                                visible: true
                            }

                            text: qsTr("MASTER")
                        }
                    }

                    FlexboxLayout {
                        Layout.fillWidth: true

                        direction: FlexboxLayout.Row
                        justifyContent: FlexboxLayout.JustifyCenter
                        alignItems: FlexboxLayout.AlignEnd

                        Label {
                            Layout.preferredWidth: 112
                            leftPadding: 4
                            topPadding: -8
                            bottomPadding: -8
                            horizontalAlignment: Text.AlignRight

                            font.pointSize: 32
                            font.variableAxes: {
                                "opsz": 30
                            }
                            font.weight: Font.Medium

                            color: palette.text

                            text: Math.floor(root.engine.deck_state.mixer_channel(root.player_number).player.current_bpm).toString()
                        }

                        Label {
                            leftPadding: 0
                            rightPadding: 4
                            topPadding: -8
                            bottomPadding: -1
                            horizontalAlignment: Text.AlignLeft

                            font.pointSize: 18
                            font.variableAxes: {
                                "opsz": 10
                            }
                            font.weight: Font.Medium

                            color: palette.text

                            text: "." + Math.floor(((root.engine.deck_state.mixer_channel(root.player_number).player.current_bpm) * 10) % 10).toString()
                        }
                    }
                }

                FlexboxLayout {
                    id: key_box

                    direction: FlexboxLayout.Column
                    justifyContent: FlexboxLayout.JustifyStart
                    alignContent: FlexboxLayout.AlignStretch

                    gap: 0

                    FlexboxLayout {
                        Layout.fillWidth: true

                        direction: FlexboxLayout.Row
                        justifyContent: FlexboxLayout.JustifySpaceBetween
                        alignItems: FlexboxLayout.AlignCenter

                        Label {
                            leftPadding: 4
                            rightPadding: 4
                            horizontalAlignment: Text.AlignHCenter

                            font.pointSize: 10
                            font.variableAxes: {
                                "opsz": 4
                            }
                            font.weight: Font.Medium

                            color: palette.text

                            text: qsTr("KEY")
                        }

                        Label {
                            leftPadding: 4
                            rightPadding: 4
                            horizontalAlignment: Text.AlignHCenter

                            font.pointSize: 10
                            font.variableAxes: {
                                "opsz": 4
                            }
                            font.weight: Font.Medium

                            visible: root.engine.deck_state.mixer_channel(root.player_number).player.master_tempo

                            color: "#ff0000"

                            text: qsTr("MT")
                        }
                    }

                    FlexboxLayout {
                        Layout.fillWidth: true

                        direction: FlexboxLayout.Row
                        justifyContent: FlexboxLayout.JustifyCenter
                        alignItems: FlexboxLayout.AlignEnd

                        Label {
                            Layout.preferredWidth: 98
                            leftPadding: 4
                            topPadding: -8
                            bottomPadding: -8
                            horizontalAlignment: Text.AlignHCenter

                            font.pointSize: 32
                            font.variableAxes: {
                                "opsz": 30
                            }
                            font.weight: Font.Medium

                            color: palette.text

                            text: "--"
                        }
                    }
                }
            }

            Item {
                Layout.fillWidth: true
                Layout.preferredHeight: 16
            }

            ShaderEffect {
                Layout.fillWidth: true
                Layout.preferredHeight: 48

                vertexShader: "qrc:/shaders/preview_waveform.vert.qsb"
                fragmentShader: "qrc:/shaders/preview_waveform.frag.qsb"

                property real progress: root.engine.deck_state.mixer_channel(root.player_number).player.time / root.engine.deck_state.mixer_channel(root.player_number).player.track_length
                property variant waveform: root.engine.deck_state.mixer_channel(root.player_number).player.preview_waveform_texture_source
            }
        }
    }
}
