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
    direction: FlexboxLayout.Column
    justifyContent: FlexboxLayout.JustifyStart
    alignItems: FlexboxLayout.AlignCenter

    Label {
        text: qsTr("Source Select")

        font.pointSize: 12
        font.variableAxes: {
            "opsz": 10
        }
        font.weight: Font.Medium
    }

    Rectangle {
        Layout.fillWidth: true
        Layout.preferredHeight: 4

        color: palette.window
    }

    Rectangle {
        Layout.fillWidth: true
        Layout.preferredHeight: 1

        color: palette.mid
    }

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

            width: root.width
            direction: FlexboxLayout.Column
            justifyContent: FlexboxLayout.JustifyCenter
            alignContent: FlexboxLayout.AlignStretch

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 4

                color: palette.window
            }

            FlexboxLayout {
                Layout.fillWidth: true
                direction: FlexboxLayout.Row
                justifyContent: FlexboxLayout.JustifySpaceBetween
                alignContent: FlexboxLayout.AlignStretch

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

                FlexboxLayout {
                    direction: FlexboxLayout.Row
                    justifyContent: FlexboxLayout.JustifyEnd
                    alignItems: FlexboxLayout.AlignCenter

                    Layout.fillWidth: false

                    Button {
                        text: "Browse"
                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        onClicked: () => {
                            listView.engine.select_device(delegate_root.device_number);
                        }
                    }
                }
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 4

                color: palette.window
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 1

                visible: (delegate_root.index < (listView.count - 1))

                color: palette.mid
            }
        }
    }
}
