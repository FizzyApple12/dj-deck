use crate::types::deck::{BeatSyncMode, DeckState};

pub fn player_tempo_set(deck_state: &mut DeckState, channel: usize, position: f32) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.player.tempo_slider_position = position;

        let tempo_differential =
            (channel.player.tempo_percent - channel.player.get_actual_slider_tempo()).abs();

        if channel.player.tempo_slider_is_accurate {
            if tempo_differential > 0.1 {
                channel.player.beat_sync = BeatSyncMode::Off;
            }
        } else if tempo_differential < 0.01 {
            channel.player.tempo_slider_is_accurate = true;
        }
    }
}

pub fn mixer_channel_fader_set(deck_state: &mut DeckState, channel: usize, position: f32) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.fade = position;
    }
}

pub fn mixer_crossfader_set(deck_state: &mut DeckState, position: f32) {
    deck_state.crossfade = position;
}
