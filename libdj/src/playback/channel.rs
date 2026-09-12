use timecode::Timecode;

use crate::{
    playback::player::PlayerUpdateResults,
    types::{analysis::Beat, deck::ChannelState},
};

pub struct ChannelUpdateResults {
    pub player: PlayerUpdateResults,
}

impl ChannelState {
    pub fn is_valid_master(&self) -> bool {
        self.player.is_valid_master()
    }

    pub fn update_jog(&mut self, start_time: Timecode, end_time: Timecode) {
        self.player.update_jog(start_time, end_time);
    }

    pub fn update_playback(
        &mut self,
        is_master: bool,
        start_time: Timecode,
        end_time: Timecode,
        master_track_bpm: Option<f32>,
        master_beat_sync_data: Option<(&[Beat], Timecode, f32)>,
    ) -> ChannelUpdateResults {
        ChannelUpdateResults {
            player: self.player.update_playback(
                is_master,
                start_time,
                end_time,
                master_track_bpm,
                master_beat_sync_data,
            ),
        }
    }
}
