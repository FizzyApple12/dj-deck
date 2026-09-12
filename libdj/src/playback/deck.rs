use log::info;
use timecode::Timecode;

use crate::{
    MIXER_CHANNELS,
    playback::{channel::ChannelUpdateResults, player::PlayerUpdateResults},
    types::{
        analysis::Beat,
        bindings::{DeckControlEvent, UIControlEvent},
        deck::{ChannelState, DeckState},
        playback::DeckUpdate,
    },
};

pub struct DeckUpdateResults {
    pub channels: [ChannelUpdateResults; MIXER_CHANNELS],
}

impl DeckState {
    pub fn find_new_master(&mut self, exclude_channels: &[usize]) {
        for channel_number in 0..MIXER_CHANNELS {
            if !exclude_channels.contains(&channel_number)
                && let Some(potential_channel) = self.mixer_channels.get(channel_number)
                && potential_channel.is_valid_master()
            {
                self.master_channel = Some(channel_number);

                return;
            }
        }

        self.master_channel = None;
    }

    pub fn update(
        &mut self,
        control_event_receiver: &mut tokio::sync::mpsc::UnboundedReceiver<DeckControlEvent>,
        control_event_sender: &mut tokio::sync::mpsc::UnboundedSender<UIControlEvent>,
        update_receiver: &mut tokio::sync::mpsc::UnboundedReceiver<DeckUpdate>,
        start_time: Timecode,
        end_time: Timecode,
    ) -> DeckUpdateResults {
        // we need to do this before updating the deck state to make inactivity falloff
        // work
        for channel in &mut self.mixer_channels {
            channel.update_jog(start_time, end_time);
        }

        while let Ok(control_change) = control_event_receiver.try_recv() {
            info!(target: "libdj::playback::deck", "Control change: {control_change:?}");

            control_change.use_binding(self, control_event_sender);
        }

        while let Ok(deck_state_update_function) = update_receiver.try_recv() {
            deck_state_update_function(self);
        }

        if let Some(master_channel_number) = self.master_channel
            && let Some(master_channel) = self.mixer_channels.get(master_channel_number)
            && !master_channel.is_valid_master()
        {
            self.find_new_master(&[master_channel_number]);
        }

        let mut channel_updates = [None, None, None, None];
        let mut mut_channels: [Option<&mut ChannelState>; MIXER_CHANNELS] =
            self.mixer_channels.each_mut().map(Some);

        if let Some(master_channel_number) = self.master_channel
            && let Some(master_channel) = mut_channels.get_mut(master_channel_number)
            && let Some(master_channel) = master_channel.take()
        {
            if let Some(channel_update) = channel_updates.get_mut(master_channel_number) {
                *channel_update =
                    Some(master_channel.update_playback(true, start_time, end_time, None, None));
            }

            let master_track_bpm: Option<f32> = master_channel.player.get_current_bpm();

            let master_beat_sync_data: Option<(&[Beat], Timecode, f32)> =
                if let Some(ref track_analysis) = master_channel.player.current_track_analysis {
                    Some((
                        &track_analysis.beat_grid,
                        master_channel.player.time,
                        master_channel.player.tempo_percent,
                    ))
                } else {
                    None
                };

            for (channel_state, channel_update) in
                mut_channels.iter_mut().zip(channel_updates.iter_mut())
            {
                if let Some(channel) = channel_state {
                    *channel_update = Some(channel.update_playback(
                        false,
                        start_time,
                        end_time,
                        master_track_bpm,
                        master_beat_sync_data,
                    ));
                }
            }
        } else {
            for (channel_state, channel_update) in
                mut_channels.iter_mut().zip(channel_updates.iter_mut())
            {
                if let Some(channel) = channel_state {
                    *channel_update =
                        Some(channel.update_playback(false, start_time, end_time, None, None));
                }
            }
        }

        DeckUpdateResults {
            channels: channel_updates.map(|update| {
                update.unwrap_or(ChannelUpdateResults {
                    player: PlayerUpdateResults {
                        playback_frame_start_time: Timecode::zero(),
                        playback_frame_end_time: Timecode::zero(),
                        playback_wrap_times: None,
                        touch_cue_playback_times: None,
                    },
                })
            }),
        }
    }
}
