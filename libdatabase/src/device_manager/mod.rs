use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use nix::mount::{MntFlags, umount2};
use nusb::{DeviceId, hotplug::HotplugEvent};
use thiserror::Error;
use tokio::{fs, task::JoinHandle};
use tokio_stream::{Stream, StreamExt};

use crate::device_manager::device::{Device, MOUNT_BASE, OpenDeviceError};

pub mod device;

pub struct DeviceManager {
    pub devices: Arc<Mutex<HashMap<u32, Device>>>,

    id_to_number_map: Arc<Mutex<HashMap<DeviceId, u32>>>,

    task_handle: JoinHandle<()>,

    event_sender: tokio::sync::mpsc::Sender<DeviceManagerEvent>,
    event_receiver: tokio::sync::mpsc::Receiver<DeviceManagerEvent>,
}

#[derive(Error, Debug)]
pub enum StartDeviceManagerError {
    #[error("Failed to start watching for new devices: {0}")]
    HotplugWatchFailed(nusb::Error),
}

#[derive(Debug)]
pub enum DeviceManagerEvent {
    DeviceConnected(Result<u32, OpenDeviceError>),
    DeviceDisconnected(u32),
}

impl DeviceManager {
    #[allow(clippy::too_many_lines)]
    pub fn start() -> Result<DeviceManager, StartDeviceManagerError> {
        tokio::task::spawn_blocking(async || {
            unmount_stale_mounts().await;
        });

        let device_list = Arc::new(Mutex::new(HashMap::new()));
        let id_to_number_map = Arc::new(Mutex::new(HashMap::new()));

        let (event_sender, event_receiver) = tokio::sync::mpsc::channel(16);

        let mut watch = match nusb::watch_devices() {
            Ok(watch) => watch,
            Err(err) => {
                return Err(StartDeviceManagerError::HotplugWatchFailed(err));
            }
        };

        let cloned_device_list = device_list.clone();
        let cloned_event_sender = event_sender.clone();
        let cloned_id_to_number_map = id_to_number_map.clone();

        let task_handle = tokio::task::spawn(async move {
            let device_list = cloned_device_list;
            let event_sender = cloned_event_sender;
            let id_to_number_map = cloned_id_to_number_map;

            if let Ok(devices) = nusb::list_devices().await {
                for device in devices {
                    match Device::open(device).await {
                        Ok(open_device) => {
                            let device_number = open_device.number;

                            if let Ok(mut locked_id_to_number_map) = id_to_number_map.lock()
                                && let Ok(mut locked_device_list) = device_list.lock()
                            {
                                locked_id_to_number_map.insert(open_device.id, device_number);
                                locked_device_list.insert(open_device.number, open_device);

                                drop(locked_device_list);
                                drop(locked_id_to_number_map);
                            }

                            let _ = event_sender
                                .send(DeviceManagerEvent::DeviceConnected(Ok(device_number)))
                                .await;
                        }
                        Err(err) => {
                            let _ = event_sender
                                .send(DeviceManagerEvent::DeviceConnected(Err(err)))
                                .await;
                        }
                    }
                }
            }

            while let Some(event) = watch.next().await {
                match event {
                    HotplugEvent::Connected(device) => {
                        let cloned_id_to_number_map = id_to_number_map.clone();
                        let cloned_device_list = device_list.clone();
                        let cloned_event_sender = event_sender.clone();

                        tokio::task::spawn(async move {
                            let id_to_number_map = cloned_id_to_number_map;
                            let device_list = cloned_device_list;
                            let event_sender = cloned_event_sender;

                            match Device::open(device).await {
                                Ok(open_device) => {
                                    let device_number = open_device.number;

                                    if let Ok(mut locked_id_to_number_map) = id_to_number_map.lock()
                                        && let Ok(mut locked_device_list) = device_list.lock()
                                    {
                                        locked_id_to_number_map
                                            .insert(open_device.id, device_number);
                                        locked_device_list.insert(open_device.number, open_device);

                                        drop(locked_device_list);
                                        drop(locked_id_to_number_map);
                                    }

                                    let _ = event_sender
                                        .send(DeviceManagerEvent::DeviceConnected(Ok(
                                            device_number,
                                        )))
                                        .await;
                                }
                                Err(err) => {
                                    let _ = event_sender
                                        .send(DeviceManagerEvent::DeviceConnected(Err(err)))
                                        .await;
                                }
                            }
                        });
                    }
                    HotplugEvent::Disconnected(id) => {
                        let mut device_number = Option::None;

                        if let Ok(mut locked_id_to_number_map) = id_to_number_map.lock()
                            && let Ok(mut locked_device_list) = device_list.lock()
                        {
                            if let Some(found_device_number) = locked_id_to_number_map.remove(&id) {
                                device_number = Some(found_device_number);

                                locked_device_list.remove(&found_device_number);
                            }

                            drop(locked_device_list);
                        }

                        unmount_stale_mounts().await;

                        if let Some(device_number) = device_number {
                            let _ = event_sender
                                .send(DeviceManagerEvent::DeviceDisconnected(device_number))
                                .await;
                        }
                    }
                }
            }
        });

        Ok(DeviceManager {
            devices: device_list,

            id_to_number_map,

            task_handle,

            event_sender,
            event_receiver,
        })
    }

    // for some reason clippy doesn't recognise that this is fixed
    #[allow(clippy::await_holding_lock)]
    pub async fn eject(&mut self, id: u32) {
        if let Ok(mut locked_device_database) = self.devices.lock()
            && let Ok(mut locked_id_to_number_map) = self.id_to_number_map.lock()
            && let Some(device) = locked_device_database.remove(&id)
        {
            locked_id_to_number_map.retain(|_, value| *value != id);

            drop(locked_device_database);
            drop(locked_id_to_number_map);

            device.eject().await;

            unmount_stale_mounts().await;

            let _ = self
                .event_sender
                .send(DeviceManagerEvent::DeviceDisconnected(id))
                .await;
        }
    }

    #[allow(clippy::await_holding_lock)] // we need to hold the lock until the drain is complete
    pub async fn stop(self: DeviceManager) {
        self.task_handle.abort();

        if let Ok(mut locked_device_list) = self.devices.lock() {
            for (_, device) in locked_device_list.drain() {
                device.eject().await;
            }

            drop(locked_device_list);
        }
    }
}

impl Stream for DeviceManager {
    type Item = DeviceManagerEvent;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        context: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.event_receiver.poll_recv(context)
    }
}

async fn unmount_stale_mounts() {
    let Ok(mounts) = fs::read_to_string("/proc/mounts").await else {
        return;
    };

    for line in mounts.lines() {
        let mut fields = line.split_whitespace();

        let (Some(device), Some(mount_point)) = (fields.next(), fields.next()) else {
            continue;
        };

        if !mount_point.starts_with(MOUNT_BASE) {
            continue;
        }

        if Path::new(device).exists() {
            continue;
        }

        let mount_point = PathBuf::from(mount_point);

        if let Ok(()) = umount2(&mount_point, MntFlags::MNT_DETACH) {
            let _ = fs::remove_dir(&mount_point).await;
        }
    }
}
