import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window

import engineering.fizzy.deck_application

FlexboxLayout {
    id: root
    required property EngineBridge engine

    Layout.fillWidth: true
    Layout.fillHeight: true
    direction: FlexboxLayout.Row
    justifyContent: FlexboxLayout.JustifyStart
    alignItems: FlexboxLayout.AlignCenter

    gap: 8

    ListView {
        id: listView
        required property EngineBridge engine

        Layout.fillWidth: true
        Layout.fillHeight: true
        clip: true

        engine: root.engine

        model: root.engine.source_index
        delegate: FlexboxLayout {
            id: delegate_root
            required property int index
            required property int device_number
            required property string label

            width: listView.width
            direction: FlexboxLayout.Column
            justifyContent: FlexboxLayout.JustifyCenter
            alignContent: FlexboxLayout.AlignStretch

            Item {
                Layout.fillWidth: true
                Layout.preferredHeight: 8
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 48

                color: palette.base

                TapHandler {
                    onTapped: listView.engine.select_device(delegate_root.device_number)
                }

                FlexboxLayout {
                    anchors.fill: parent
                    direction: FlexboxLayout.Row
                    justifyContent: FlexboxLayout.JustifyStart
                    alignContent: FlexboxLayout.AlignStretch

                    gap: 8

                    Item {
                        Layout.preferredWidth: 48
                        Layout.preferredHeight: 48

                        Rectangle {
                            width: 48
                            height: 48

                            color: palette.accent

                            Image {
                                anchors.centerIn: parent
                                width: 24
                                height: 24
                                sourceSize.width: 24
                                sourceSize.height: 24

                                source: "qrc:/icons/usb.svg"
                            }
                        }

                        Rectangle {
                            width: 14
                            height: 14

                            color: palette.light

                            Label {
                                anchors.centerIn: parent

                                text: "" + delegate_root.device_number

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

                    Text {
                        Layout.fillHeight: true
                        Layout.preferredWidth: 400
                        verticalAlignment: Text.AlignVCenter

                        text: delegate_root.label

                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        color: palette.text
                    }

                    Item {
                        Layout.fillWidth: true
                    }

                    Item {
                        Layout.preferredWidth: 48
                        Layout.preferredHeight: 48

                        Image {
                            anchors.centerIn: parent
                            width: 24
                            height: 24
                            sourceSize.width: 24
                            sourceSize.height: 24

                            source: {
                                if (root.engine.device_selected && root.engine.active_device == delegate_root.device_number) {
                                    return "qrc:/icons/check.svg";
                                } else {
                                    return "qrc:/icons/chevron-right.svg";
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Rectangle {
        Layout.fillWidth: true
        Layout.fillHeight: true

        color: palette.base

        FlexboxLayout {
            anchors.centerIn: parent
            direction: FlexboxLayout.Column
            justifyContent: FlexboxLayout.JustifyCenter
            alignContent: FlexboxLayout.AlignCenter
            alignItems: FlexboxLayout.AlignCenter

            gap: 8

            Image {
                Layout.preferredWidth: 200
                Layout.preferredHeight: 200
                sourceSize.width: 200
                sourceSize.height: 200

                source: "qrc:/icons/fizzyengineering.svg"
            }

            Text {
                Layout.preferredWidth: 400
                verticalAlignment: Text.AlignVCenter
                horizontalAlignment: Text.AlignHCenter

                text: "DJ Software"

                font.pointSize: 24
                font.variableAxes: {
                    "opsz": 30
                }
                font.weight: Font.Medium

                color: palette.text
            }

            Text {
                verticalAlignment: Text.AlignVCenter
                horizontalAlignment: Text.AlignLeft

                text: "Development Testing Build"

                font.pointSize: 12
                font.variableAxes: {
                    "opsz": 10
                }
                font.weight: Font.Medium

                color: palette.text
            }
        }
    }
}
