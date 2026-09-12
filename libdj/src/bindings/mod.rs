use crate::types::deck::BeatSyncMode;
#[allow(clippy::wildcard_imports)]
use crate::{
    bindings::{buttons::*, jog::*, knobs::*, sliders::*},
    types::{
        bindings::{DeckControlEvent, UIControlEvent},
        deck::DeckState,
    },
};

pub mod buttons;
pub mod jog;
pub mod knobs;
pub mod sliders;

// suffix terminology
// press -> use when button was pressed for a short amount of time
// long_press -> use when button was held for a long amount of time
// release -> use when button was released
// set -> used when some analog value was updated

impl DeckControlEvent {
    // there are a lot of button bindings
    pub fn use_binding(
        self,
        deck_state: &mut DeckState,
        playback_event_sender: &mut tokio::sync::mpsc::UnboundedSender<UIControlEvent>,
    ) {
        match self {
            // browser
            DeckControlEvent::USBEjectPress { slot } => {
                let _ = playback_event_sender.send(UIControlEvent::USBEjectPress { slot });
            }
            DeckControlEvent::USBEjectRelease { slot } => {
                let _ = playback_event_sender.send(UIControlEvent::USBEjectRelease { slot });
            }

            DeckControlEvent::BrowserEncoderAdjust { delta } => {
                let _ = playback_event_sender.send(UIControlEvent::BrowserEncoderAdjust { delta });
            }
            DeckControlEvent::BrowserEncoderPress => {
                let _ = playback_event_sender.send(UIControlEvent::BrowserEncoderPress);
            }

            DeckControlEvent::BrowserBackPress => {
                let _ = playback_event_sender.send(UIControlEvent::BrowserBackPress);
            }
            DeckControlEvent::BrowserSourcePress => {
                let _ = playback_event_sender.send(UIControlEvent::BrowserSourcePress);
            }
            DeckControlEvent::BrowserBrowsePress => {
                let _ = playback_event_sender.send(UIControlEvent::BrowserBrowsePress);
            }
            DeckControlEvent::BrowserPlaylistPress => {
                let _ = playback_event_sender.send(UIControlEvent::BrowserPlaylistPress);
            }
            DeckControlEvent::BrowserSearchPress => {
                let _ = playback_event_sender.send(UIControlEvent::BrowserSearchPress);
            }

            // mixer
            DeckControlEvent::MixerMasterGainSet { position } => {
                mixer_master_gain_set(deck_state, position);
            }
            DeckControlEvent::MixerMasterCuePress => mixer_master_cue_press(deck_state),
            DeckControlEvent::MixerMasterMasterFXTargetPress => {
                mixer_master_master_fx_target_press(deck_state);
            }

            DeckControlEvent::MixerMasterFXSelect { delta } => {
                log::warn!("Unbound Action: Mixer Master FX Select by {delta}");
            }
            DeckControlEvent::MixerMasterFXSelectTouchPress => {
                let _ = playback_event_sender.send(UIControlEvent::MixerMasterFXSelectTouchPress);
            }
            DeckControlEvent::MixerMasterFXSelectTouchRelease => {
                let _ = playback_event_sender.send(UIControlEvent::MixerMasterFXSelectTouchRelease);
            }
            DeckControlEvent::MixerMasterFXParameterAdjust { delta } => {
                log::warn!("Unbound Action: Mixer Master FX Parameter Adjust by {delta}");
            }
            DeckControlEvent::MixerMasterFXParameterSelectPress => {
                log::warn!("Unbound Action: Mixer Master FX Parameter Select Press");
            }
            DeckControlEvent::MixerMasterFXBPMAdjust { delta } => {
                log::warn!("Unbound Action: Mixer Master FX BPM Adjust by {delta}");
            }
            DeckControlEvent::MixerMasterFXBPMSelectPress => {
                log::warn!("Unbound Action: Mixer Master FX BPM Select Press");
            }
            DeckControlEvent::MixerMasterFXDepthSet { position } => {
                mixer_master_fx_depth_set(deck_state, position);
            }
            DeckControlEvent::MixerMasterFXEnablePress => mixer_master_fx_enable_press(deck_state),

            DeckControlEvent::MixerChannelFXPress { number } => {
                log::warn!("Unbound Action: Mixer Channel FX {number} Press");
            }
            DeckControlEvent::MixerChannelFXProximityPress => {
                let _ = playback_event_sender.send(UIControlEvent::MixerChannelFXProximityPress);
            }
            DeckControlEvent::MixerChannelFXProximityRelease => {
                let _ = playback_event_sender.send(UIControlEvent::MixerChannelFXProximityRelease);
            }

            DeckControlEvent::MixerCrossfaderSet { position } => {
                mixer_crossfader_set(deck_state, position);
            }

            DeckControlEvent::MixerHeadphonesMixSet { position } => {
                log::warn!("Unbound Action: Mixer Headphones Mix Set to {position}");
            }
            DeckControlEvent::MixerHeadphonesGainSet { position } => {
                log::warn!("Unbound Action: Mixer Headphones Gain Set to {position}");
            }

            // mixer channels
            DeckControlEvent::MixerChannelGainSet { channel, position } => {
                mixer_channel_gain_set(deck_state, channel, position);
            }

            DeckControlEvent::MixerChannelEqHighSet { channel, position } => {
                mixer_channel_eq_high_set(deck_state, channel, position);
            }
            DeckControlEvent::MixerChannelEqMidSet { channel, position } => {
                mixer_channel_eq_mid_set(deck_state, channel, position);
            }
            DeckControlEvent::MixerChannelEqLowSet { channel, position } => {
                mixer_channel_eq_low_set(deck_state, channel, position);
            }

            DeckControlEvent::MixerChannelFXSet { channel, position } => {
                mixer_channel_fx_set(deck_state, channel, position);
            }

            DeckControlEvent::MixerChannelCuePress { channel } => {
                mixer_channel_cue_press(deck_state, channel);
            }
            DeckControlEvent::MixerChannelMasterFXTargetPress { channel } => {
                mixer_channel_master_fx_target_press(deck_state, channel);
            }

            DeckControlEvent::MixerChannelFaderSet { channel, position } => {
                mixer_channel_fader_set(deck_state, channel, position);
            }

            DeckControlEvent::MixerChannelCrossfaderAssignAPress { channel } => {
                mixer_channel_crossfader_assign_a_press(deck_state, channel);
            }
            DeckControlEvent::MixerChannelCrossfaderAssignNonePress { channel } => {
                mixer_channel_crossfader_assign_none_press(deck_state, channel);
            }
            DeckControlEvent::MixerChannelCrossfaderAssignBPress { channel } => {
                mixer_channel_crossfader_assign_b_press(deck_state, channel);
            }

            // player
            DeckControlEvent::PlayerJogVelocitySet { channel, velocity } => {
                player_jog_velocity_set(deck_state, channel, false, velocity);
            }
            DeckControlEvent::PlayerJogSearchVelocitySet { channel, velocity } => {
                player_jog_velocity_set(deck_state, channel, true, velocity);
            }
            DeckControlEvent::PlayerJogTouchPress { channel } => {
                player_jog_press(deck_state, channel);
            }
            DeckControlEvent::PlayerJogTouchRelease { channel } => {
                player_jog_release(deck_state, channel);
            }

            DeckControlEvent::PlayerQuantizePress { channel } => {
                player_quantize_press(deck_state, channel);
            }
            DeckControlEvent::PlayerSlipPress { channel } => player_slip_press(deck_state, channel),

            DeckControlEvent::PlayerPlayPress { channel } => player_play_press(deck_state, channel),
            DeckControlEvent::PlayerReversePress { channel } => {
                log::warn!("Unbound Action: Player Reverse Press on channel {channel}");
            }
            DeckControlEvent::PlayerReverseRelease { channel } => {
                log::warn!("Unbound Action: Player Reverse Release on channel {channel}");
            }
            DeckControlEvent::PlayerCuePress { channel } => {
                player_cue_press(deck_state, channel, false);
            }
            DeckControlEvent::PlayerAltCuePress { channel } => {
                player_cue_press(deck_state, channel, true);
            }
            DeckControlEvent::PlayerCueRelease { channel }
            | DeckControlEvent::PlayerAltCueRelease { channel } => {
                player_cue_release(deck_state, channel);
            }

            DeckControlEvent::PlayerBeatJumpBackwardRelease { channel } => {
                player_beat_jump_release(deck_state, channel, false, false);
            }
            DeckControlEvent::PlayerLongBeatJumpBackwardRelease { channel } => {
                player_beat_jump_release(deck_state, channel, false, true);
            }
            DeckControlEvent::PlayerBeatJumpForwardRelease { channel } => {
                player_beat_jump_release(deck_state, channel, true, false);
            }
            DeckControlEvent::PlayerLongBeatJumpForwardRelease { channel } => {
                player_beat_jump_release(deck_state, channel, true, true);
            }

            DeckControlEvent::PlayerBeatSyncPress { channel } => {
                player_beat_sync_press(deck_state, BeatSyncMode::BeatSync, channel);
            }
            DeckControlEvent::PlayerBPMSyncPress { channel } => {
                player_beat_sync_press(deck_state, BeatSyncMode::BPMSync, channel);
            }
            DeckControlEvent::PlayerKeySyncPress { channel } => {
                player_key_sync_press(deck_state, channel);
            }
            DeckControlEvent::PlayerMasterPress { channel } => {
                player_master_press(deck_state, channel);
            }

            DeckControlEvent::PlayerTempoResetPress { channel } => {
                player_tempo_reset_press(deck_state, channel);
            }
            DeckControlEvent::PlayerTempoRangePress { channel } => {
                player_tempo_range_press(deck_state, channel);
            }
            DeckControlEvent::PlayerMasterTempoPress { channel } => {
                player_master_tempo_press(deck_state, channel);
            }
            DeckControlEvent::PlayerTempoSet { channel, position } => {
                player_tempo_set(deck_state, channel, position);
            }

            DeckControlEvent::PlayerBeatLoopInPress { channel } => {
                log::warn!("Unbound Action: Player Beat Loop In Press on channel {channel}");
            }
            DeckControlEvent::PlayerBeatLoopInAdjustPress { channel } => {
                log::warn!("Unbound Action: Player Beat Loop In Adjust Press on channel {channel}");
            }
            DeckControlEvent::PlayerBeatLoopOutPress { channel } => {
                log::warn!("Unbound Action: Player Beat Loop Out Press on channel {channel}");
            }
            DeckControlEvent::PlayerBeatLoopOutAdjustPress { channel } => {
                log::warn!(
                    "Unbound Action: Player Beat Loop Out Adjust Press on channel {channel}"
                );
            }
            DeckControlEvent::PlayerReLoopPress { channel } => {
                log::warn!("Unbound Action: Player ReLoop Press on channel {channel}");
            }
            DeckControlEvent::PlayerInstantLoopPress { channel } => {
                log::warn!("Unbound Action: Player Instant Loop Press on channel {channel}");
            }

            DeckControlEvent::PlayerPadPress { channel, number } => {
                log::warn!("Unbound Action: Player Pad {number} on channel {channel}");
            }
        }
    }
}
