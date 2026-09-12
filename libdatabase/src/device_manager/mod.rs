use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Seek},
    path::PathBuf,
    sync::Arc,
};

use kobject_uevent::{ActionType, UEvent};
use log::{debug, info, warn};
use netlink_sys::{
    AsyncSocket, AsyncSocketExt, SocketAddr, TokioSocket, protocols::NETLINK_KOBJECT_UEVENT,
};
use thiserror::Error;
use tokio::{
    io::{Interest, unix::AsyncFd},
    sync::Mutex,
    task::JoinHandle,
};

use crate::device_manager::device::Device;

pub mod device;

pub struct DeviceManager {
    pub devices: Arc<Mutex<HashMap<usize, Device>>>,

    local_watcher_task_handle: Option<JoinHandle<()>>,
    prodj_link_watcher_task_handle: Option<JoinHandle<()>>,

    event_sender: tokio::sync::broadcast::Sender<DeviceManagerEvent>,
    event_receiver: tokio::sync::broadcast::Receiver<DeviceManagerEvent>,
}

pub struct DeviceManagerHandle {
    pub devices: Arc<Mutex<HashMap<usize, Device>>>,

    event_receiver: tokio::sync::broadcast::Receiver<DeviceManagerEvent>,
}

#[derive(Error, Debug)]
pub enum StartLocalDeviceWatchError {
    #[error("Failed to connect to NetLink: {0}")]
    NetLinkConnectFailed(std::io::Error),

    #[error("Failed to bind the NetLink listener: {0}")]
    NetLinkBindFailed(std::io::Error),

    #[error("Failed to open mounts file: {0}")]
    MountsFileOpenFailed(std::io::Error),

    #[error("Failed to listen to the mounts file: {0}")]
    MountsListenFailed(std::io::Error),
}

#[derive(Error, Debug)]
pub enum StartProDJLinkWatchError {}

#[derive(Debug, Clone, Copy)]
pub enum DeviceManagerEvent {
    DeviceConnected(usize),
    DeviceDisconnected(usize),
}

impl Default for DeviceManager {
    fn default() -> Self {
        let (event_sender, event_receiver) = tokio::sync::broadcast::channel(16);

        Self {
            devices: Arc::new(Mutex::new(HashMap::new())),

            local_watcher_task_handle: None,
            prodj_link_watcher_task_handle: None,

            event_sender,
            event_receiver,
        }
    }
}

impl DeviceManager {
    pub fn subscribe(&self) -> DeviceManagerHandle {
        DeviceManagerHandle {
            devices: self.devices.clone(),

            event_receiver: self.event_sender.subscribe(),
        }
    }

    pub fn start_local_watch(
        &mut self,
        local_automounts: &'static [(&str, usize)],
    ) -> Result<(), StartLocalDeviceWatchError> {
        info!("Connecting to netlink");

        let mut socket = TokioSocket::new(NETLINK_KOBJECT_UEVENT)
            .map_err(StartLocalDeviceWatchError::NetLinkConnectFailed)?;

        info!("Binding to netlink socket");

        socket
            .socket_mut()
            .bind(&SocketAddr::new(0, 1))
            .map_err(StartLocalDeviceWatchError::NetLinkBindFailed)?;

        info!("Opening mounts file");

        let mounts_file =
            File::open("/proc/mounts").map_err(StartLocalDeviceWatchError::MountsFileOpenFailed)?;

        info!("Listening to mounts with priority interest");

        let mut mounts_file = AsyncFd::with_interest(mounts_file, Interest::PRIORITY)
            .map_err(StartLocalDeviceWatchError::MountsListenFailed)?;

        let cloned_device_list = self.devices.clone();
        let cloned_event_sender = self.event_sender.clone();

        info!("Starting device event processor");

        let task_handle = tokio::task::spawn(async move {
            let device_list = cloned_device_list;
            let event_sender = cloned_event_sender;

            // loop {
            info!("Performing initial index");

            let mut locked_device_list = device_list.lock().await;

            for (path, id) in local_automounts {
                let Some(has_mount) = check_for_mount(&mut mounts_file, path) else {
                    continue;
                };

                match (has_mount, locked_device_list.get_mut(id)) {
                    (true, None) => {
                        let new_device = match Device::open(*id, PathBuf::from(path)).await {
                            Ok(new_device) => new_device,
                            Err(err) => {
                                warn!("Failed to open Device {id}: {err}");

                                continue;
                            }
                        };

                        locked_device_list.insert(*id, new_device);

                        let _ = event_sender.send(DeviceManagerEvent::DeviceConnected(*id));
                    }
                    (false, Some(_)) => {
                        // we can't eject a device that's already been removed, we just need
                        // to yeet it
                        drop(locked_device_list.remove(id));

                        let _ = event_sender.send(DeviceManagerEvent::DeviceDisconnected(*id));
                    }
                    (false, None) | (true, Some(_)) => {}
                }
            }

            drop(locked_device_list);

            info!("Waiting for devices");

            loop {
                tokio::select! {
                    message = socket.recv_from_full() => {
                        if let Ok((buffer, address)) = message {
                            if address.port_number() != 0 {
                                continue;
                            }

                            let Ok(uevent) = UEvent::from_netlink_packet(&buffer) else {
                                continue;
                            };

                            let Some(dev_type) = uevent.env.get("DEVTYPE") else {
                                continue;
                            };

                            if uevent.subsystem != "block" || dev_type != "partition" {
                                continue;
                            }

                            if let Some(dev_name) = uevent.env.get("DEVNAME")
                                && uevent.action == ActionType::Remove
                            {
                                let mut locked_device_list = device_list.lock().await;

                                warn!("Device yank detected for {dev_name}");

                                let mut removed_devices: HashMap<usize, Device> = locked_device_list
                                    .extract_if(|_, device| device.dev_name == *dev_name)
                                    .collect();

                                for (key, device) in removed_devices.drain() {
                                    // we can't eject a device that's already been removed, we just need
                                    // to yeet it
                                    drop(device);

                                    let _ = event_sender.send(DeviceManagerEvent::DeviceDisconnected(key));
                                }

                                drop(locked_device_list);
                            }
                        }
                    }

                    guard = mounts_file.ready(Interest::PRIORITY) => {
                        if let Ok(mut guard) = guard {
                            guard.clear_ready();
                        }

                        let mut locked_device_list = device_list.lock().await;

                        for (path, id) in local_automounts {
                            let Some(has_mount) = check_for_mount(&mut mounts_file, path) else {
                                continue;
                            };

                            match (has_mount, locked_device_list.get_mut(id)) {
                                (true, None) => {
                                    let new_device = match Device::open(*id, PathBuf::from(path)).await {
                                        Ok(new_device) => new_device,
                                        Err(err) => {
                                            warn!("Failed to open Device {id}: {err}");

                                            continue;
                                        }
                                    };

                                    locked_device_list.insert(*id, new_device);

                                    let _ = event_sender.send(DeviceManagerEvent::DeviceConnected(*id));
                                }
                                (false, Some(_)) => {
                                    // we can't eject a device that's already been removed, we just need
                                    // to yeet it
                                    drop(locked_device_list.remove(id));

                                    let _ = event_sender.send(DeviceManagerEvent::DeviceDisconnected(*id));
                                }
                                (false, None) | (true, Some(_)) => {}
                            }
                        }

                        drop(locked_device_list);
                    }
                };
            }
        });

        self.local_watcher_task_handle = Some(task_handle);

        Ok(())
    }

    pub fn start_pro_dj_link_watch(
        &mut self,
        device_number_start: usize,
    ) -> Result<(), StartProDJLinkWatchError> {
        todo!("Start Pro DJ Link at Device Number {device_number_start}")
    }

    // for some reason clippy doesn't recognise that this is fixed
    #[allow(clippy::await_holding_lock)]
    pub async fn eject(&mut self, id: usize) {
        debug!("Removing device from maps");

        let mut locked_device_list = self.devices.lock().await;

        if let Some(device) = locked_device_list.remove(&id) {
            drop(locked_device_list);

            info!("Ejecting device {id}");

            device.eject().await;

            info!("Device {id} closed");

            let _ = self
                .event_sender
                .send(DeviceManagerEvent::DeviceDisconnected(id));
        }
    }

    #[allow(clippy::await_holding_lock)] // we need to hold the lock until the drain is complete
    pub async fn stop(mut self: DeviceManager) {
        info!("Stopping device manager");

        if let Some(task_handle) = self.local_watcher_task_handle.take() {
            task_handle.abort();
        }

        if let Some(task_handle) = self.prodj_link_watcher_task_handle.take() {
            task_handle.abort();
        }

        let mut locked_device_list = self.devices.lock().await;

        for (id, device) in locked_device_list.drain() {
            info!("Ejecting device {id}");

            device.eject().await;
        }

        drop(locked_device_list);

        info!("Closed");
    }

    pub async fn next(&mut self) -> Option<DeviceManagerEvent> {
        self.event_receiver.recv().await.ok()
    }
}

impl Drop for DeviceManager {
    fn drop(&mut self) {
        if let Some(task_handle) = self.local_watcher_task_handle.take() {
            task_handle.abort();
        }

        if let Some(task_handle) = self.prodj_link_watcher_task_handle.take() {
            task_handle.abort();
        }
    }
}

impl DeviceManagerHandle {
    pub async fn next(&mut self) -> Option<DeviceManagerEvent> {
        self.event_receiver.recv().await.ok()
    }
}

fn check_for_mount(mount_file: &mut AsyncFd<File>, mount: &str) -> Option<bool> {
    let file = mount_file.get_mut();
    file.rewind().ok()?;

    let mut file_buffer: String = String::new();

    let _ = file.read_to_string(&mut file_buffer).ok()?;

    Some(file_buffer.lines().any(|line| {
        let mut fields = line.split_whitespace();

        fields.next();
        fields
            .next()
            .is_some_and(|mount_point| mount_point == mount)
    }))
}
