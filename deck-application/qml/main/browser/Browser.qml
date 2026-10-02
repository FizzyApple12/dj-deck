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
    clip: true

    gap: 8

    FlexboxLayout {
        Layout.fillWidth: false
        Layout.preferredWidth: 48
        Layout.fillHeight: true
        direction: FlexboxLayout.Column
        justifyContent: FlexboxLayout.JustifyStart
        alignItems: FlexboxLayout.AlignCenter
        clip: true

        Item {
            Layout.preferredHeight: 8
        }

        Rectangle {
            Layout.preferredWidth: 48
            Layout.preferredHeight: 48

            color: palette.accent

            Image {
                anchors.centerIn: parent
                width: 24
                height: 24
                sourceSize.width: 24
                sourceSize.height: 24

                source: "qrc:/icons/track.svg"
            }
        }

        Rectangle {
            Layout.preferredWidth: 48
            Layout.preferredHeight: 48

            color: palette.base

            Image {
                anchors.centerIn: parent
                width: 24
                height: 24
                sourceSize.width: 24
                sourceSize.height: 24

                source: "qrc:/icons/artist.svg"
            }
        }

        Rectangle {
            Layout.preferredWidth: 48
            Layout.preferredHeight: 48

            color: palette.base

            Image {
                anchors.centerIn: parent
                width: 24
                height: 24
                sourceSize.width: 24
                sourceSize.height: 24

                source: "qrc:/icons/album.svg"
            }
        }

        Rectangle {
            Layout.preferredWidth: 48
            Layout.preferredHeight: 48

            color: palette.base

            Image {
                anchors.centerIn: parent
                width: 24
                height: 24
                sourceSize.width: 24
                sourceSize.height: 24

                source: "qrc:/icons/key.svg"
            }
        }

        Rectangle {
            Layout.preferredWidth: 48
            Layout.preferredHeight: 48

            color: palette.base

            Image {
                anchors.centerIn: parent
                width: 24
                height: 24
                sourceSize.width: 24
                sourceSize.height: 24

                source: "qrc:/icons/playlist.svg"
            }
        }

        Rectangle {
            Layout.preferredWidth: 48
            Layout.preferredHeight: 48

            color: palette.base

            Image {
                anchors.centerIn: parent
                width: 24
                height: 24
                sourceSize.width: 24
                sourceSize.height: 24

                source: "qrc:/icons/history.svg"
            }
        }

        Rectangle {
            Layout.preferredWidth: 48
            Layout.preferredHeight: 48

            color: palette.base

            Image {
                anchors.centerIn: parent
                width: 24
                height: 24
                sourceSize.width: 24
                sourceSize.height: 24

                source: "qrc:/icons/folder.svg"
            }
        }

        Rectangle {
            Layout.preferredWidth: 48
            Layout.fillHeight: true

            color: palette.base
        }
    }

    FlexboxLayout {
        Layout.fillWidth: true
        Layout.fillHeight: true
        direction: FlexboxLayout.Column
        justifyContent: FlexboxLayout.JustifyStart
        alignItems: FlexboxLayout.AlignCenter
        clip: true

        gap: 8

        Item {
            Layout.preferredHeight: 0
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 48

            color: palette.base

            FlexboxLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                direction: FlexboxLayout.Column
                justifyContent: FlexboxLayout.JustifyCenter
                alignContent: FlexboxLayout.AlignStretch

                FlexboxLayout {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 48
                    direction: FlexboxLayout.Row
                    justifyContent: FlexboxLayout.JustifySpaceBetween
                    alignContent: FlexboxLayout.AlignStretch

                    Item {
                        Layout.preferredWidth: 16
                    }

                    Text {
                        Layout.fillHeight: true
                        Layout.preferredWidth: 1000
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

                    Item {
                        Layout.preferredWidth: 16
                    }
                }
            }
        }

        ListView {
            id: listView
            required property EngineBridge engine

            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            highlight: Rectangle {
                color: palette.accent
            }
            highlightMoveDuration: 0
            focus: true

            engine: root.engine

            model: root.engine.browser_index
            delegate: Rectangle {
                id: delegate_root
                required property int index
                required property BrowserEntryType entry_type
                required property int entry_number
                required property int node_id
                required property string title
                required property string artist
                required property var duration
                required property real bpm

                width: listView.width
                height: 24

                color: (index % 2 == 0) ? palette.base : palette.alternateBase

                TapHandler {
                    onTapped: listView.currentIndex = index
                }

                FlexboxLayout {
                    anchors.fill: parent
                    direction: FlexboxLayout.Row
                    justifyContent: FlexboxLayout.JustifyStart
                    alignContent: FlexboxLayout.AlignStretch

                    Item {
                        Layout.preferredWidth: 16
                    }

                    Text {
                        Layout.fillHeight: true
                        Layout.preferredWidth: 1000
                        verticalAlignment: Text.AlignVCenter

                        text: {
                        	if (!delegate_root.artist) {
                       			return delegate_root.title;
                         	} else {
                          		return `${delegate_root.artist} - ${delegate_root.title}`;
                          	}
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

                    Item {
                        Layout.fillWidth: true
                    }

                    FlexboxLayout {
                        Layout.fillWidth: false
                        Layout.fillHeight: true
                        Layout.preferredWidth: 350
                        direction: FlexboxLayout.Row
                        justifyContent: FlexboxLayout.JustifyEnd
                        alignItems: FlexboxLayout.AlignCenter

                        visible: listView.currentIndex == index

                        gap: 1

                        Button {
                        	Layout.fillHeight: true

                            text: "  Load on 1  "
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
                        	Layout.fillHeight: true

                            text: "  Load on 2  "
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
                        	Layout.fillHeight: true

                            text: "  Load on 3  "
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
                        	Layout.fillHeight: true

                            text: "  Load on 4  "
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
            }
        }
    }
}
