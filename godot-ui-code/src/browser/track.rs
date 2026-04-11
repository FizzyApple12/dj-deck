use godot::{
    classes::{HBoxContainer, IHBoxContainer, Label},
    prelude::*,
};
use libdj::types::library::TrackID;
use libui::types::ui::InternalUIEvent;

use crate::{ipc::IPC, menu::Menu};

#[derive(GodotClass)]
#[class(base=HBoxContainer)]
pub struct TrackListEntry {
    base: Base<HBoxContainer>,

    pub device_number: u32,
    pub track_id: TrackID,
    pub track_number: u32,

    pub ipc: Option<Gd<IPC>>,

    pub menu: Option<Gd<Menu>>,

    #[export]
    track_number_label: OnEditor<Gd<Label>>,

    #[export]
    track_name_label: OnEditor<Gd<Label>>,

    #[export]
    track_bpm_label: OnEditor<Gd<Label>>,
    #[export]
    track_key_label: OnEditor<Gd<Label>>,
}

impl TrackListEntry {}

#[godot_api]
impl IHBoxContainer for TrackListEntry {
    fn init(base: Base<HBoxContainer>) -> Self {
        Self {
            base,

            device_number: 0,
            track_id: 0,
            track_number: 0,

            ipc: None,

            menu: None,

            track_number_label: OnEditor::default(),

            track_name_label: OnEditor::default(),

            track_bpm_label: OnEditor::default(),
            track_key_label: OnEditor::default(),
        }
    }

    fn process(&mut self, _delta: f64) {
        let Some(ipc) = &self.ipc else {
            return;
        };

        let ipc = ipc.bind();

        #[allow(clippy::cast_sign_loss)]
        let Some((_, Some(library))) = ipc.devices.get(&self.device_number) else {
            return;
        };

        let Some(track_data) = library.tracks.get(&self.track_id) else {
            return;
        };

        self.track_number_label
            .set_text(&format!("{}", self.track_number));

        self.track_name_label.set_text(&track_data.title);

        self.track_bpm_label
            .set_text(&format!("{:.1}", track_data.bpm));

        if let Some(key) = library.keys.get(&track_data.key_id) {
            self.track_key_label.set_text(&key.name);
        }
    }
}

#[godot_api]
impl TrackListEntry {
    #[func]
    #[allow(clippy::cast_sign_loss)]
    fn load(&mut self, player: i32) {
        if let Some(ipc) = &mut self.ipc {
            ipc.bind_mut().send_event(InternalUIEvent::LoadTrack {
                device: self.device_number,
                id: self.track_id,
                player: player as usize,
            });
        }

        if let Some(menu) = &mut self.menu {
            menu.bind_mut().go_to_players();
        }
    }
}

impl Drop for TrackListEntry {
    fn drop(&mut self) {}
}
