import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window

FlexboxLayout {
    id: root
    required property int player_number

    Layout.fillHeight: false

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
        Layout.fillHeight: false

        direction: FlexboxLayout.Column
        justifyContent: FlexboxLayout.JustifyStart
        alignItems: FlexboxLayout.AlignCenter

        FlexboxLayout {
            direction: FlexboxLayout.Row
            justifyContent: FlexboxLayout.JustifyStart
            alignItems: FlexboxLayout.AlignCenter
            Layout.fillWidth: true
            Layout.fillHeight: false

            Image {
                Layout.preferredWidth: 42
                Layout.preferredHeight: 42

                source: "qrc:/test_images/test-album-art.png"
            }

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

                text: qsTr("Above The Cloud (Original Mix)")
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
                alignItems: FlexboxLayout.AlignStretch

                gap: 0

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

                        color: palette.text

                        text: qsTr("TIME")
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

                        color: palette.text

                        text: qsTr("/")
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

                        color: palette.text

                        text: qsTr("REMAIN")
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

                        text: qsTr("00:00")
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

                        text: qsTr(".000")
                    }
                }
            }

            FlexboxLayout {
                id: tempo_box

                direction: FlexboxLayout.Column
                justifyContent: FlexboxLayout.JustifyStart
                alignItems: FlexboxLayout.AlignStretch

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

                        text: qsTr("± 16")
                    }
                }

                FlexboxLayout {
                    Layout.fillWidth: true

                    direction: FlexboxLayout.Row
                    justifyContent: FlexboxLayout.JustifyCenter
                    alignItems: FlexboxLayout.AlignEnd

                    Label {
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

                        text: qsTr("+")
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

                        text: qsTr("0")
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

                        text: qsTr(".00%")
                    }
                }
            }

            FlexboxLayout {
                id: bpm_box

                direction: FlexboxLayout.Column
                justifyContent: FlexboxLayout.JustifyStart
                alignItems: FlexboxLayout.AlignStretch

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

                        background: Rectangle {
                            color: palette.accent

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

                        text: qsTr("0")
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

                        text: qsTr(".0")
                    }
                }
            }

            FlexboxLayout {
                id: key_box

                direction: FlexboxLayout.Column
                justifyContent: FlexboxLayout.JustifyStart
                alignItems: FlexboxLayout.AlignStretch

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

                        text: qsTr("4A")
                    }
                }
            }
        }

        Image {
            id: preview_waveform_image

            visible: false

            sourceSize {
                width: 16384
                height: 2
            }

            source: "qrc:/test_images/test-preview-waveform.png"
        }

        ShaderEffect {
            Layout.fillWidth: true
            Layout.preferredHeight: 48

            vertexShader: "qrc:/shaders/preview_waveform.vert.qsb"
            fragmentShader: "qrc:/shaders/preview_waveform.frag.qsb"

            property real progress: 0.35
            property variant waveform: preview_waveform_image
        }
    }
}
