import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window

Window {
    x: 2048
    y: 0
    width: 280
    height: 456
    visible: true
    flags: Qt.FramelessWindowHint
    color: palette.window
    title: "2"

    FlexboxLayout {
        anchors.fill: parent
        direction: FlexboxLayout.Column
        justifyContent: FlexboxLayout.JustifySpaceAround
        alignItems: FlexboxLayout.AlignCenter

        Label {
            Layout.fillWidth: true
            lineHeight: 0.75
            leftPadding: 4
            rightPadding: 4
            wrapMode: Text.WordWrap
            horizontalAlignment: Text.AlignHCenter

            font.pointSize: 32
            font.variableAxes: {
                "opsz": 30
            }
            font.weight: Font.Medium

            color: "#ff0000"

            text: qsTr("ED02: NO DJ ENGINE CONNECTION")
        }
    }

    FlexboxLayout {
        anchors.fill: parent
        direction: FlexboxLayout.Column
        justifyContent: FlexboxLayout.JustifySpaceAround
        alignItems: FlexboxLayout.AlignCenter

        visible: false

        Label {
            Layout.fillWidth: true
            lineHeight: 0.75
            leftPadding: 4
            rightPadding: 4
            wrapMode: Text.WordWrap
            horizontalAlignment: Text.AlignHCenter

            font.pointSize: 38
            font.variableAxes: {
                "opsz": 30
            }
            font.weight: Font.Medium

            color: palette.text

            text: qsTr("LOW CUT ECHO")
        }
    }

    Rectangle {
        color: palette.highlight

        x: bpmBox.x
        y: bpmBox.y + 16
        width: bpmBox.width
        height: bpmBox.height - 8

        visible: false
    }

    FlexboxLayout {
        anchors.fill: parent
        anchors.topMargin: 16
        anchors.bottomMargin: 16
        direction: FlexboxLayout.Column
        justifyContent: FlexboxLayout.JustifySpaceBetween
        alignItems: FlexboxLayout.AlignCenter

        visible: false

        FlexboxLayout {
            id: bpmBox

            direction: FlexboxLayout.Column
            justifyContent: FlexboxLayout.JustifyStart
            alignItems: FlexboxLayout.AlignStretch

            FlexboxLayout {
                Layout.preferredWidth: 180
                direction: FlexboxLayout.Row
                justifyContent: FlexboxLayout.JustifySpaceBetween
                alignItems: FlexboxLayout.AlignCenter

                Label {
                    Layout.preferredWidth: 76
                    leftPadding: 4
                    rightPadding: 4
                    horizontalAlignment: Text.AlignHCenter

                    font.pointSize: 16
                    font.variableAxes: {
                        "opsz": 4
                    }
                    font.weight: Font.DemiBold

                    color: palette.text

                    text: qsTr("BPM")
                }

                Label {
                    Layout.preferredWidth: 76
                    leftPadding: 4
                    rightPadding: 4
                    horizontalAlignment: Text.AlignHCenter

                    font.pointSize: 16
                    font.variableAxes: {
                        "opsz": 4
                    }
                    font.weight: Font.DemiBold

                    color: palette.highlightedText

                    background: Rectangle {
                        color: palette.accent

                        visible: true
                    }

                    text: qsTr("AUTO")
                }
            }

            FlexboxLayout {
                Layout.preferredHeight: 68
                Layout.preferredWidth: 180
                direction: FlexboxLayout.Row
                justifyContent: FlexboxLayout.JustifyEnd
                alignItems: FlexboxLayout.AlignEnd

                Label {
                    leftPadding: 4
                    rightPadding: 4
                    topInset: 8
                    bottomInset: 8
                    horizontalAlignment: Text.AlignHCenter

                    font.pointSize: 38
                    font.variableAxes: {
                        "opsz": 30
                    }
                    font.weight: Font.Medium

                    color: palette.text

                    text: qsTr("9999")
                }

                Label {
                    Layout.preferredHeight: 54
                    leftPadding: 4
                    rightPadding: 4
                    // topInset: 8
                    bottomInset: 8
                    horizontalAlignment: Text.AlignHCenter

                    font.pointSize: 24
                    font.variableAxes: {
                        "opsz": 10
                    }
                    font.weight: Font.Medium

                    color: palette.text

                    text: qsTr(".9")
                }
            }
        }

        FlexboxLayout {
            direction: FlexboxLayout.Column
            justifyContent: FlexboxLayout.JustifyCenter
            alignItems: FlexboxLayout.AlignCenter

            FlexboxLayout {
                Layout.preferredHeight: 56
                direction: FlexboxLayout.Row
                justifyContent: FlexboxLayout.JustifySpaceAround
                alignItems: FlexboxLayout.AlignCenter
                gap: 4

                // visible: false

                Label {
                    leftPadding: 8
                    rightPadding: 8

                    font.pointSize: 24
                    font.variableAxes: {
                        "opsz": 10
                    }
                    font.weight: Font.Medium

                    color: palette.text

                    text: qsTr("1/8")
                }

                Label {
                    leftPadding: 12
                    rightPadding: 12

                    font.pointSize: 28
                    font.variableAxes: {
                        "opsz": 20
                    }
                    font.weight: Font.Medium

                    color: palette.highlightedText
                    background: Rectangle {
                        color: palette.highlight
                    }

                    text: qsTr("1/4")
                }

                Label {
                    leftPadding: 8
                    rightPadding: 8

                    font.pointSize: 24
                    font.variableAxes: {
                        "opsz": 10
                    }
                    font.weight: Font.Medium

                    color: palette.text

                    text: qsTr("1/2")
                }
            }

            FlexboxLayout {
                Layout.preferredHeight: 56
                direction: FlexboxLayout.Row
                justifyContent: FlexboxLayout.JustifySpaceAround
                alignItems: FlexboxLayout.AlignCenter
                gap: 4

                // visible: false

                Label {
                    leftPadding: 8
                    rightPadding: 8

                    font.pointSize: 24
                    font.variableAxes: {
                        "opsz": 10
                    }
                    font.weight: Font.Medium

                    color: palette.disabled.text

                    text: qsTr("100ms")
                }
            }
        }
    }
}
