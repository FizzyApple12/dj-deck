use godot::{
    classes::{CanvasItem, INode, Node, VBoxContainer},
    obj::WithBaseField,
    prelude::*,
};
use libdj::types::library::PlaylistTreeNodeID;

use crate::{
    browser::{device::DeviceListEntry, track::TrackListEntry},
    ipc::IPC,
};

#[derive(Debug, Clone, Copy)]
pub enum MenuState {
    Players,
    BrowseDevices,
    BrowsePlaylist,
    BrowseTracks,
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct Menu {
    base: Base<Node>,

    #[export]
    ipc: OnEditor<Gd<IPC>>,

    #[export]
    players: OnEditor<Gd<CanvasItem>>,

    #[export]
    browser: OnEditor<Gd<CanvasItem>>,

    #[export]
    browser_devices: OnEditor<Gd<CanvasItem>>,
    browser_devices_list: Vec<Gd<DeviceListEntry>>,

    #[export]
    browser_playlist: OnEditor<Gd<CanvasItem>>,
    browser_playlist_list: Vec<Gd<VBoxContainer>>,

    #[export]
    browser_tracks: OnEditor<Gd<CanvasItem>>,
    browser_tracks_list: Vec<Gd<TrackListEntry>>,

    #[export]
    browser_device_list_entry: OnEditor<Gd<PackedScene>>,
    #[export]
    browser_playlist_entry: OnEditor<Gd<PackedScene>>,
    #[export]
    browser_track_entry: OnEditor<Gd<PackedScene>>,

    menu_state: MenuState,

    current_device: Option<u32>,
    current_playlist: Option<PlaylistTreeNodeID>,
}

impl Menu {}

#[godot_api]
impl INode for Menu {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,

            ipc: OnEditor::default(),

            players: OnEditor::default(),

            browser: OnEditor::default(),

            browser_devices: OnEditor::default(),
            browser_devices_list: Vec::new(),

            browser_playlist: OnEditor::default(),
            browser_playlist_list: Vec::new(),

            browser_tracks: OnEditor::default(),
            browser_tracks_list: Vec::new(),

            browser_device_list_entry: OnEditor::default(),
            browser_playlist_entry: OnEditor::default(),
            browser_track_entry: OnEditor::default(),

            menu_state: MenuState::Players,

            current_device: None,
            current_playlist: None,
        }
    }

    fn process(&mut self, _delta: f64) {
        let ipc = self.ipc.bind();

        if let Some(current_device_id) = self.current_device
            && !ipc.devices.contains_key(&current_device_id)
        {
            self.current_device = None;

            drop(ipc);

            self.refresh_device_list();
        } else {
            drop(ipc);
        }

        match self.menu_state {
            MenuState::Players => {
                self.players.set_visible(true);

                self.browser.set_visible(false);
                self.browser_devices.set_visible(false);
                self.browser_playlist.set_visible(false);
                self.browser_tracks.set_visible(false);
            }
            MenuState::BrowseDevices => {
                if self.ipc.bind().devices_changed {
                    self.refresh_device_list();
                }

                self.players.set_visible(false);

                self.browser.set_visible(true);
                self.browser_devices.set_visible(true);
                self.browser_playlist.set_visible(false);
                self.browser_tracks.set_visible(false);
            }
            MenuState::BrowsePlaylist => {
                self.players.set_visible(false);

                self.browser.set_visible(true);
                self.browser_devices.set_visible(false);
                self.browser_playlist.set_visible(true);
                self.browser_tracks.set_visible(false);
            }
            MenuState::BrowseTracks => {
                self.players.set_visible(false);

                self.browser.set_visible(true);
                self.browser_devices.set_visible(false);
                self.browser_playlist.set_visible(false);
                self.browser_tracks.set_visible(true);
            }
        }
    }
}

impl Menu {
    fn refresh_device_list(&mut self) {
        for device in self.browser_devices_list.drain(..) {
            device.free();
        }

        let ipc = self.ipc.bind();

        for device in ipc.devices.keys() {
            let mut new_device_entry = self
                .browser_device_list_entry
                .instantiate_as::<DeviceListEntry>();

            new_device_entry.bind_mut().ipc = Some(self.ipc.clone());
            new_device_entry.bind_mut().menu = Some(self.to_gd());
            new_device_entry.bind_mut().device_number = *device;

            self.browser_devices.add_child(&new_device_entry);

            self.browser_devices_list.push(new_device_entry);
        }
    }

    pub fn select_device(&mut self, device: u32) {
        self.current_device = Some(device);

        self.refresh_tracks_list();
        self.menu_state = MenuState::BrowseTracks;
    }

    pub fn go_to_players(&mut self) {
        self.menu_state = MenuState::Players;
    }

    #[allow(clippy::cast_possible_truncation)]
    fn refresh_playlist_list(&mut self) {}

    #[allow(clippy::cast_possible_truncation)]
    fn refresh_tracks_list(&mut self) {
        let ipc = self.ipc.bind();

        if let Some(device) = self.current_device
            && let Some((_, Some(device_library))) = ipc.devices.get(&device)
        {
            for device in self.browser_tracks_list.drain(..) {
                device.free();
            }

            for (track_number, track) in device_library.tracks.keys().enumerate() {
                let mut new_track_entry =
                    self.browser_track_entry.instantiate_as::<TrackListEntry>();

                new_track_entry.bind_mut().ipc = Some(self.ipc.clone());
                new_track_entry.bind_mut().menu = Some(self.to_gd());
                new_track_entry.bind_mut().device_number = device;
                new_track_entry.bind_mut().track_id = *track;
                new_track_entry.bind_mut().track_number = track_number as u32;

                self.browser_tracks.add_child(&new_track_entry);

                self.browser_tracks_list.push(new_track_entry);
            }
        } else {
            self.menu_state = MenuState::BrowseDevices;
        }
    }
}

#[godot_api]
impl Menu {
    #[func]
    fn source_pressed(&mut self) {
        if let MenuState::BrowseDevices = self.menu_state {
            self.menu_state = MenuState::Players;

            return;
        }

        self.refresh_device_list();
        self.menu_state = MenuState::BrowseDevices;
    }

    #[func]
    fn browse_pressed(&mut self) {
        if let MenuState::BrowseTracks = self.menu_state {
            self.menu_state = MenuState::Players;

            return;
        }

        if self.current_device.is_none() {
            self.refresh_device_list();
            self.menu_state = MenuState::BrowseDevices;

            return;
        }

        self.refresh_tracks_list();
        self.menu_state = MenuState::BrowseTracks;
    }

    #[func]
    fn playlist_pressed(&mut self) {
        if let MenuState::BrowsePlaylist = self.menu_state {
            self.menu_state = MenuState::Players;

            return;
        }

        if self.current_device.is_none() {
            self.refresh_device_list();
            self.menu_state = MenuState::BrowseDevices;

            return;
        }

        self.refresh_playlist_list();
        self.menu_state = MenuState::BrowsePlaylist;
    }
}

impl Drop for Menu {
    fn drop(&mut self) {}
}
