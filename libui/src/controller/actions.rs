use libdj::types::{
    deck::{DeckState, PlayState},
    timecode::{Duration, Timecode},
};

#[allow(clippy::cast_lossless)]
fn jog_2_rpm(jog: u8) -> f32 {
    if jog >= 0x41 {
        // +
        (6.0 * jog as f32) - 390.0
    } else if jog <= 0x3F {
        // -
        (6.0 * jog as f32) - 378.0
    } else {
        0.0
    }
}

#[allow(clippy::indexing_slicing)]
pub fn jog_touch(deck_state: &mut DeckState, channel: usize) {
    deck_state.mixer_channels[channel].player.jog_hold = true;
}
#[allow(clippy::indexing_slicing)]
pub fn jog_release(deck_state: &mut DeckState, channel: usize) {
    deck_state.mixer_channels[channel].player.jog_hold = false;

    if !(-f32::EPSILON..=f32::EPSILON)
        .contains(&deck_state.mixer_channels[channel].player.jog_velocity)
    {
        deck_state.mixer_channels[channel].player.jog_wait = true;
    }
}
#[allow(clippy::indexing_slicing)]
pub fn jog_distance(deck_state: &mut DeckState, channel: usize, search: bool, velocity: u8) {
    deck_state.mixer_channels[channel].player.jog_velocity =
        jog_2_rpm(velocity) * if search { 50.0 } else { 1.0 };
}

#[allow(clippy::indexing_slicing)]
pub fn play_press(deck_state: &mut DeckState, channel: usize) {
    let play_direction = &mut deck_state.mixer_channels[channel].player.play_state;

    match play_direction {
        PlayState::Cue | PlayState::Stop => *play_direction = PlayState::Play,
        PlayState::Play => *play_direction = PlayState::Stop,
    }
}

#[allow(clippy::indexing_slicing)]
pub fn cue_press(deck_state: &mut DeckState, channel: usize, shift: bool) {
    if shift {
        deck_state.mixer_channels[channel].player.time = Timecode::zero();
        deck_state.mixer_channels[channel].player.cue_time = None;
    } else {
        match deck_state.mixer_channels[channel].player.play_state {
            PlayState::Stop => {
                if let Some(cue_time) = deck_state.mixer_channels[channel].player.cue_time
                    && deck_state.mixer_channels[channel].player.time == cue_time
                {
                    deck_state.mixer_channels[channel].player.play_state = PlayState::Cue;
                } else {
                    let cue_time = if deck_state.mixer_channels[channel].player.quanitze {
                        deck_state.mixer_channels[channel]
                            .player
                            .get_closest_beat()
                            .map_or(deck_state.mixer_channels[channel].player.time, |beat| {
                                beat.time
                            })
                    } else {
                        deck_state.mixer_channels[channel].player.time
                    };

                    deck_state.mixer_channels[channel].player.play_state = PlayState::Stop;
                    deck_state.mixer_channels[channel].player.cue_time = Some(cue_time);
                    deck_state.mixer_channels[channel].player.time = cue_time;
                }
            }
            PlayState::Play => {
                let cue_time = deck_state.mixer_channels[channel]
                    .player
                    .cue_time
                    .unwrap_or(if deck_state.mixer_channels[channel].player.quanitze {
                        if let Some(ref track_analysis) = deck_state.mixer_channels[channel]
                            .player
                            .current_track_analysis
                            && let Some(first_beat) = track_analysis.beat_grid.first()
                        {
                            first_beat.time
                        } else {
                            Timecode::zero()
                        }
                    } else {
                        Timecode::zero()
                    });

                deck_state.mixer_channels[channel].player.play_state = PlayState::Stop;
                deck_state.mixer_channels[channel].player.cue_time = Some(cue_time);
                deck_state.mixer_channels[channel].player.time = cue_time;
            }
            PlayState::Cue => {}
        }
    }
}
#[allow(clippy::indexing_slicing)]
pub fn cue_release(deck_state: &mut DeckState, channel: usize) {
    let play_direction = &mut deck_state.mixer_channels[channel].player.play_state;
    let cue_time = &mut deck_state.mixer_channels[channel].player.cue_time;

    if let PlayState::Cue = play_direction {
        *play_direction = PlayState::Stop;

        if let Some(position) = cue_time {
            deck_state.mixer_channels[channel].player.time = *position;
        }
    }
}

#[allow(clippy::indexing_slicing, clippy::cast_possible_truncation)]
pub fn beat_jump_release(deck_state: &mut DeckState, channel: usize, direction: bool, shift: bool) {
    if let Some(bpm) = deck_state.mixer_channels[channel]
        .player
        .get_current_source_bpm()
    {
        let distance = (60_000_000_000.0 / bpm).floor() as i64 * if shift { 16 } else { 4 };

        if direction {
            deck_state.mixer_channels[channel].player.time += Duration::from_nanoseconds(distance);
        } else {
            deck_state.mixer_channels[channel].player.time -= Duration::from_nanoseconds(distance);
        }
    }
}
