import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window

import engineering.fizzy.deck_application

Rectangle {
    id: root
    required property EngineBridge engine
    required property string page_title

    Layout.fillWidth: true
    Layout.preferredHeight: 48

    color: palette.base

    FlexboxLayout {
        anchors.fill: parent
        direction: FlexboxLayout.Row
        justifyContent: FlexboxLayout.JustifyStart
        alignContent: FlexboxLayout.AlignStretch
        alignItems: FlexboxLayout.AlignCenter

        gap: 8

        Item {
            Layout.preferredWidth: 48
            Layout.preferredHeight: 48

            TapHandler {
                onTapped: root.engine.setSource_open(true)
            }

            Rectangle {
                width: 48
                height: 48

                color: root.engine.device_selected ? palette.accent : palette.base

                Image {
                    anchors.centerIn: parent
                    width: 24
                    height: 24
                    sourceSize.width: 24
                    sourceSize.height: 24

                    source: {
                        if (root.engine.device_selected) {
                            return "qrc:/icons/usb.svg";
                        } else {
                            return "qrc:/icons/empty.svg";
                        }
                    }
                }
            }

            Rectangle {
                width: 14
                height: 14

                visible: root.engine.device_selected
                color: palette.light

                Label {
                    anchors.centerIn: parent

                    text: "" + root.engine.active_device

                    font.pointSize: 8
                    font.variableAxes: {
                        "opsz": 10
                    }
                    font.weight: Font.Medium

                    color: palette.dark
                }
            }
        }

        Item {
            Layout.preferredWidth: 0
        }

        Label {
            text: qsTr(root.page_title)

            font.pointSize: 12
            font.variableAxes: {
                "opsz": 10
            }
            font.weight: Font.Medium
        }

        Item {
            Layout.fillWidth: true
        }

        Rectangle {
            Layout.preferredWidth: 48
            Layout.preferredHeight: 48

            color: "#00000000"

            Image {
                anchors.centerIn: parent
                width: 24
                height: 24
                sourceSize.width: 24
                sourceSize.height: 24

                source: "qrc:/icons/nothing.svg"
            }
        }

        Rectangle {
            Layout.preferredWidth: 48
            Layout.preferredHeight: 48

            color: "#00000000"

            Image {
                anchors.centerIn: parent
                width: 24
                height: 24
                sourceSize.width: 24
                sourceSize.height: 24

                source: "qrc:/icons/network.svg"
            }
        }

        Rectangle {
            Layout.preferredWidth: 48
            Layout.preferredHeight: 48

            color: "#00000000"

            Image {
                anchors.centerIn: parent
                width: 24
                height: 24
                sourceSize.width: 24
                sourceSize.height: 24

                source: "qrc:/icons/settings.svg"
            }
        }
    }
}
