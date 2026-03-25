use libdatabase::device_manager::{DeviceManager, DeviceManagerEvent, device::OpenDeviceError};
use tokio::signal;
use tokio_stream::StreamExt;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let mut device_manager = match DeviceManager::start() {
        Ok(device_manager) => device_manager,
        Err(err) => {
            println!("failed to start device manager: {err:#?}");

            return;
        }
    };

    loop {
        tokio::select! {
            _ctrl_c =
                signal::ctrl_c() => {
                    break;
                }
            device_event = device_manager.next() => {
                match device_event {
                    Some(DeviceManagerEvent::DeviceConnected(Ok(id))) => {
                        println!("Device {id} Connected Successfully");
                        if let Ok(locked_device_database) = device_manager.devices.lock()
                            && let Some(device) = locked_device_database.get(&id) {
                            println!("Device Database: {:#?}", device.database);
                        }
                    }
                    Some(DeviceManagerEvent::DeviceConnected(Err(err))) => {
                        if let OpenDeviceError::InvalidDeviceType = err {
                            continue;
                        }

                        println!("Device Connect Failed: {err}");
                    }
                    Some(DeviceManagerEvent::DeviceDisconnected(id)) => {
                        println!("Device {id} Disconnected");
                    }
                    None => {
                        println!("Device Manager Hung Up");
                        break;
                    }
                }
            }
        }
    }

    device_manager.stop().await;
}
