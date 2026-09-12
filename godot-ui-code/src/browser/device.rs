use godot::{
    classes::{HBoxContainer, IHBoxContainer, Label},
    prelude::*,
};

use crate::{ipc::IPC, menu::Menu};

#[derive(GodotClass)]
#[class(base=HBoxContainer)]
pub struct DeviceListEntry {
    base: Base<HBoxContainer>,

    pub device_number: usize,

    pub ipc: Option<Gd<IPC>>,

    pub menu: Option<Gd<Menu>>,

    #[export]
    device_name_label: OnEditor<Gd<Label>>,
}

impl DeviceListEntry {}

#[godot_api]
impl IHBoxContainer for DeviceListEntry {
    fn init(base: Base<HBoxContainer>) -> Self {
        Self {
            base,

            device_number: 0,

            ipc: None,

            menu: None,

            device_name_label: OnEditor::default(),
        }
    }

    fn process(&mut self, _delta: f64) {
        let Some(ipc) = &self.ipc else {
            return;
        };

        let ipc = ipc.bind();

        #[allow(clippy::cast_sign_loss)]
        let Some((device_name, _)) = ipc.devices.get(&self.device_number) else {
            return;
        };

        self.device_name_label.set_text(device_name);
    }
}

#[godot_api]
impl DeviceListEntry {
    #[func]
    fn eject(&mut self) {
        if let Some(ipc) = &mut self.ipc {
            // ipc.bind_mut()
            //     .send_event(InternalUIEvent::EjectDevice(self.
            // device_number));
        }
    }

    #[func]
    fn browse(&mut self) {
        if let Some(menu) = &mut self.menu {
            menu.bind_mut().select_device(self.device_number);
        }
    }
}

impl Drop for DeviceListEntry {
    fn drop(&mut self) {}
}
