use masonry::parley::FontFamily;
use xilem::{
    WidgetView,
    style::Style,
    view::{Axis, flex, label},
};

use crate::gui::{UIState, theme::NEUTRAL_900};

pub fn player(data: &mut UIState, player_number: usize) -> impl WidgetView<UIState> + use<> {
    flex(
        Axis::Horizontal,
        flex(
            Axis::Vertical,
            (
                flex(
                    Axis::Horizontal,
                    label("124.3")
                        .text_size(48.0)
                        .font(FontFamily::parse("Helvetica").unwrap()),
                ),
                flex(Axis::Horizontal, bpm_view(124.3)),
            ),
        ),
    )
    .background_color(NEUTRAL_900)
}

pub fn bpm_view(bpm: f32) -> impl WidgetView<UIState> + use<> {
    flex(
        Axis::Horizontal,
        label("124.3")
            .text_size(48.0)
            .font(FontFamily::parse("Helvetica").unwrap()),
    )
}
