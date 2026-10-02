use crate::types::deck::DeckState;

pub fn player_jog_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.player.jog_hold = true;
    }
}

pub fn player_jog_release(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.player.jog_hold = false;

        if !(-f32::EPSILON..=f32::EPSILON).contains(&channel.player.jog_velocity) {
            channel.player.jog_wait = true;
        }
    }
}

pub fn player_jog_velocity_set(
    deck_state: &mut DeckState,
    channel: usize,
    search: bool,
    velocity: f32,
) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.player.jog_velocity = velocity * if search { 50.0 } else { 1.0 };
    }
}
