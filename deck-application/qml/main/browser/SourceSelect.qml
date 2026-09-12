import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Window

FlexboxLayout {
    anchors.fill: parent
    direction: FlexboxLayout.Row
    justifyContent: FlexboxLayout.JustifySpaceAround
    alignItems: FlexboxLayout.AlignCenter

    Label {
        text: qsTr("Source Select")
    }
}
