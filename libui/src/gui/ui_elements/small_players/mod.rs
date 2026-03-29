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
            flex(Axis::Vertical, (player(data, 1), player(data, 2))),
            flex(Axis::Vertical, (player(data, 3), player(data, 4))),
        ),
    )
    .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
    .cross_axis_alignment(CrossAxisAlignment::End)
}
