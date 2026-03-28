use masonry::parley::FontFamily;
use xilem::{
    WidgetView,
    view::{Axis, flex, label},
};

use crate::gui::UIState;

pub fn player(data: &mut UIState) -> impl WidgetView<UIState> + use<> {
    flex(
        Axis::Horizontal,
        label("BPM 124.3 TIME 02:35.000")
            .text_size(48.0)
            .font(FontFamily::parse("Helvetica").unwrap()),
    )
}
