pub mod waveform;

use xilem::{
    WidgetView,
    view::{Axis, CrossAxisAlignment, MainAxisAlignment, flex, label},
};

use crate::gui::UIState;

pub fn waveforms(data: &mut UIState) -> impl WidgetView<UIState> + use<> {
    flex(
        Axis::Vertical,
        (label("test"), label("test"), label("test"), label("test")),
    )
    .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
    .cross_axis_alignment(CrossAxisAlignment::Fill)
}
