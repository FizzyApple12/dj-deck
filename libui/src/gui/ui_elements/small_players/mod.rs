pub mod player;

use xilem::{
    WidgetView,
    view::{Axis, CrossAxisAlignment, MainAxisAlignment, flex},
};

use crate::gui::{UIState, ui_elements::small_players::player::player};

pub fn players(data: &mut UIState) -> impl WidgetView<UIState> + use<> {
    flex(
        Axis::Horizontal,
        (
            flex(Axis::Vertical, (player(data), player(data))),
            flex(Axis::Vertical, (player(data), player(data))),
        ),
    )
    .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
    .cross_axis_alignment(CrossAxisAlignment::End)
}
