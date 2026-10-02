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
    clip: true

    Label {
        text: qsTr("Browser")

        font.pointSize: 12
        font.variableAxes: {
            "opsz": 10
        }
        font.weight: Font.Medium
    }

    FlexboxLayout {
        Layout.fillWidth: true
        Layout.fillHeight: false
        direction: FlexboxLayout.Column
        justifyContent: FlexboxLayout.JustifyCenter
        alignContent: FlexboxLayout.AlignStretch

        FlexboxLayout {
            Layout.fillWidth: true
            direction: FlexboxLayout.Row
            justifyContent: FlexboxLayout.JustifySpaceBetween
            alignContent: FlexboxLayout.AlignStretch

            Text {
                Layout.fillHeight: true
                Layout.preferredWidth: 400
                verticalAlignment: Text.AlignVCenter

                text: "Title"

                font.pointSize: 12
                font.variableAxes: {
                    "opsz": 10
                }
                font.weight: Font.Medium

                color: palette.text
            }

            Text {
                Layout.fillHeight: true
                Layout.preferredWidth: 200
                verticalAlignment: Text.AlignVCenter

                text: "Length"

                font.pointSize: 12
                font.variableAxes: {
                    "opsz": 10
                }
                font.weight: Font.Medium

                color: palette.text
            }

            Text {
                Layout.fillHeight: true
                Layout.preferredWidth: 200
                verticalAlignment: Text.AlignVCenter

                text: "Tempo"

                font.pointSize: 12
                font.variableAxes: {
                    "opsz": 10
                }
                font.weight: Font.Medium

                color: palette.text
            }

            FlexboxLayout {
                Layout.fillWidth: false
                Layout.preferredWidth: 350
                direction: FlexboxLayout.Row
                justifyContent: FlexboxLayout.JustifyEnd
                alignItems: FlexboxLayout.AlignCenter
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

            color: palette.mid
        }
    }

    ListView {
        id: listView
        required property EngineBridge engine

        Layout.fillWidth: true
        Layout.fillHeight: true
        clip: true

        engine: root.engine

        model: root.engine.browser_index
        delegate: FlexboxLayout {
            id: delegate_root
            required property int index
            required property BrowserEntryType entry_type
            required property int entry_number
            required property int node_id
            required property string title
            required property string artist
            required property int duration
            required property real bpm

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

                    text: delegate_root.artist + " - " + delegate_root.title

                    font.pointSize: 12
                    font.variableAxes: {
                        "opsz": 10
                    }
                    font.weight: Font.Medium

                    color: palette.text
                }

                Text {
                    Layout.fillHeight: true
                    Layout.preferredWidth: 200
                    verticalAlignment: Text.AlignVCenter

                    text: {
                        let time_seconds = delegate_root.duration / 1000000000;

                        let duration_minutes = Math.floor((time_seconds / 60) % 100);
                        let duration_seconds = Math.floor(time_seconds % 60);

                        return `${duration_minutes.toString()}:${duration_seconds.toString().padStart(2, "0")}`;
                    }

                    font.pointSize: 12
                    font.variableAxes: {
                        "opsz": 10
                    }
                    font.weight: Font.Medium

                    color: palette.text
                }

                Text {
                    Layout.fillHeight: true
                    Layout.preferredWidth: 200
                    verticalAlignment: Text.AlignVCenter

                    text: `${delegate_root.bpm.toFixed(1)} bpm`

                    font.pointSize: 12
                    font.variableAxes: {
                        "opsz": 10
                    }
                    font.weight: Font.Medium

                    color: palette.text
                }

                FlexboxLayout {
                    Layout.fillWidth: false
                    Layout.preferredWidth: 350
                    direction: FlexboxLayout.Row
                    justifyContent: FlexboxLayout.JustifyEnd
                    alignItems: FlexboxLayout.AlignCenter

                    gap: 4

                    Button {
                        text: "Load on 1"
                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        onClicked: () => {
                            listView.engine.load_track(0, listView.engine.active_device, delegate_root.node_id);
                        }
                        enabled: listView.engine.device_selected
                    }
                    Button {
                        text: "Load on 2"
                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        onClicked: () => {
                            listView.engine.load_track(1, listView.engine.active_device, delegate_root.node_id);
                        }
                        enabled: listView.engine.device_selected
                    }
                    Button {
                        text: "Load on 3"
                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        onClicked: () => {
                            listView.engine.load_track(2, listView.engine.active_device, delegate_root.node_id);
                        }
                        enabled: listView.engine.device_selected
                    }
                    Button {
                        text: "Load on 4"
                        font.pointSize: 12
                        font.variableAxes: {
                            "opsz": 10
                        }
                        font.weight: Font.Medium

                        onClicked: () => {
                            listView.engine.load_track(3, listView.engine.active_device, delegate_root.node_id);
                        }
                        enabled: listView.engine.device_selected
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
