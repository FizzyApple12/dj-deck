use timecode::{Duration, Timecode};

use crate::types::deck::{
    BeatSyncMode, CrossFaderSide, DeckState, MasterFXChannel, PlayState, TempoRange,
};

pub fn mixer_master_cue_press(deck_state: &mut DeckState) {
    deck_state.master_cue = !deck_state.master_cue;
}

pub fn mixer_master_master_fx_target_press(deck_state: &mut DeckState) {
    deck_state.master_fx.channel = MasterFXChannel::Master;
}

pub fn mixer_master_fx_enable_press(deck_state: &mut DeckState) {
    deck_state.master_fx.enabled = !deck_state.master_fx.enabled;
}

pub fn mixer_channel_cue_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.cue = !channel.cue;
    }
}

pub fn mixer_channel_master_fx_target_press(deck_state: &mut DeckState, channel: usize) {
    if deck_state.mixer_channels.get(channel).is_some() {
        deck_state.master_fx.channel = MasterFXChannel::Channel(channel);
    }
}

pub fn mixer_channel_crossfader_assign_a_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.cross_fader_side = CrossFaderSide::A;
    }
}

pub fn mixer_channel_crossfader_assign_none_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.cross_fader_side = CrossFaderSide::None;
    }
}

pub fn mixer_channel_crossfader_assign_b_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.cross_fader_side = CrossFaderSide::B;
    }
}

pub fn player_quantize_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.player.quanitze = !channel.player.quanitze;
    }
}

pub fn player_slip_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.player.slip = !channel.player.slip;
    }
}

pub fn player_play_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        let play_direction = &mut channel.player.play_state;

        match play_direction {
            PlayState::Cue | PlayState::Stop => *play_direction = PlayState::Play,
            PlayState::Play => *play_direction = PlayState::Stop,
        }
    }
}

pub fn player_cue_press(deck_state: &mut DeckState, channel: usize, alt: bool) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        if alt {
            channel.player.time = Timecode::zero();
            channel.player.cue_time = None;
        } else {
            match channel.player.play_state {
                PlayState::Stop => {
                    if let Some(cue_time) = channel.player.cue_time
                        && channel.player.time == cue_time
                    {
                        channel.player.play_state = PlayState::Cue;
                    } else {
                        let cue_time = if channel.player.quanitze {
                            channel
                                .player
                                .get_closest_beat()
                                .map_or(channel.player.time, |beat| beat.time)
                        } else {
                            channel.player.time
                        };

                        channel.player.play_state = PlayState::Stop;
                        channel.player.cue_time = Some(cue_time);
                        channel.player.time = cue_time;
                    }
                }
                PlayState::Play => {
                    let cue_time = channel
                        .player
                        .cue_time
                        .unwrap_or(if channel.player.quanitze {
                            if let Some(ref track_analysis) = channel.player.current_track_analysis
                                && let Some(first_beat) = track_analysis.beat_grid.first()
                            {
                                first_beat.time
                            } else {
                                Timecode::zero()
                            }
                        } else {
                            Timecode::zero()
                        });

                    channel.player.play_state = PlayState::Stop;
                    channel.player.cue_time = Some(cue_time);
                    channel.player.time = cue_time;
                }
                PlayState::Cue => {}
            }
        }
    }
}

pub fn player_cue_release(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        let play_direction = &mut channel.player.play_state;
        let cue_time = &mut channel.player.cue_time;

        if *play_direction == PlayState::Cue {
            *play_direction = PlayState::Stop;

            if let Some(position) = cue_time {
                channel.player.time = *position;
            }
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
pub fn player_beat_jump_release(
    deck_state: &mut DeckState,
    channel: usize,
    direction: bool,
    long: bool,
) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel)
        && let Some(bpm) = channel.player.get_current_source_bpm()
    {
        let distance = (60_000_000_000.0 / bpm).floor() as i64 * if long { 16 } else { 4 };

        if direction {
            channel.player.time += Duration::from_nanoseconds(distance);
        } else {
            channel.player.time -= Duration::from_nanoseconds(distance);
        }
    }
}

pub fn player_beat_sync_press(deck_state: &mut DeckState, mode: BeatSyncMode, channel: usize) {
    let Some(beat_sync) = deck_state
        .mixer_channels
        .get_mut(channel)
        .map(|channel| channel.player.beat_sync)
    else {
        return;
    };

    let new_beat_sync = match beat_sync {
        BeatSyncMode::Off => {
            if deck_state.master_channel.is_none() {
                deck_state.master_channel = Some(channel);
            }

            mode
        }
        BeatSyncMode::BPMSync | BeatSyncMode::BeatSync => {
            if let Some(master_channel) = deck_state.master_channel
                && master_channel == channel
            {
                deck_state.find_new_master(&[channel]);
            }

            BeatSyncMode::Off
        }
    };

    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.player.beat_sync = new_beat_sync;
    }
}

pub fn player_key_sync_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.player.key_sync = !channel.player.key_sync;
    }
}

pub fn player_master_press(deck_state: &mut DeckState, channel: usize) {
    if deck_state.mixer_channels.get(channel).is_some() {
        deck_state.master_channel = Some(channel);
    }
}

pub fn player_tempo_reset_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.player.tempo_reset = !channel.player.tempo_reset;
    }
}

pub fn player_tempo_range_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.player.tempo_range = match channel.player.tempo_range {
            TempoRange::SixPercent => TempoRange::TenPercent,
            TempoRange::TenPercent => TempoRange::SixteenPercent,
            TempoRange::SixteenPercent => TempoRange::OneHundredPercent,
            TempoRange::OneHundredPercent => TempoRange::SixPercent,
        };
    }
}

pub fn player_master_tempo_press(deck_state: &mut DeckState, channel: usize) {
    if let Some(channel) = deck_state.mixer_channels.get_mut(channel) {
        channel.player.master_tempo = !channel.player.master_tempo;
    }
}
