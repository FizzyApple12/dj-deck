use crate::{KNOB_DEADBAND, types::deck::DeckState};

pub fn mixer_master_gain_set(deck_state: &mut DeckState, position: f32) {
    deck_state.master_gain = position;
}

pub fn mixer_master_fx_depth_set(deck_state: &mut DeckState, position: f32) {
    deck_state.master_fx.depth = position;
}

pub fn mixer_channel_gain_set(deck_state: &mut DeckState, channel: usize, position: f32) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.gain = position;
    }
}

pub fn mixer_channel_eq_low_set(deck_state: &mut DeckState, channel: usize, position: f32) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.eq.0 = if (-KNOB_DEADBAND..=KNOB_DEADBAND).contains(&position) {
            0.0
        } else {
            position
        }
    }
}

pub fn mixer_channel_eq_mid_set(deck_state: &mut DeckState, channel: usize, position: f32) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.eq.1 = if (-KNOB_DEADBAND..=KNOB_DEADBAND).contains(&position) {
            0.0
        } else {
            position
        }
    }
}

pub fn mixer_channel_eq_high_set(deck_state: &mut DeckState, channel: usize, position: f32) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.eq.2 = if (-KNOB_DEADBAND..=KNOB_DEADBAND).contains(&position) {
            0.0
        } else {
            position
        }
    }
}

pub fn mixer_channel_fx_set(deck_state: &mut DeckState, channel: usize, position: f32) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.fx = if (-KNOB_DEADBAND..=KNOB_DEADBAND).contains(&position) {
            0.0
        } else {
            position
        }
    }
}
