use std::path::{Path, PathBuf};

use log::{debug, info};
use thiserror::Error;
use tokio::fs;

use crate::database::{Database, OpenDatabaseError};

pub struct Device {
    pub number: usize,

    pub(crate) dev_name: String,

    pub name: String,

    pub database: Database,
}

#[derive(Error, Debug)]
pub enum OpenDeviceError {
    #[error("No Valid Block Device")]
    NoBlockDevice,

    #[error("Failed to open Device Database: {0}")]
    DatabaseOpenFailed(OpenDatabaseError),
}

impl Device {
    pub(crate) async fn open(
        number: usize,
        mount_point: PathBuf,
    ) -> Result<Device, OpenDeviceError> {
        info!(target: "libdatabase::device_manager::device", "Opening device {number} at {}", mount_point.display());

        let block_device = get_block_device(&mount_point)
            .await
            .ok_or(OpenDeviceError::NoBlockDevice)?;

        let name = get_partition_name(&block_device)
            .await
            .unwrap_or(format!("USB{number}"));

        debug!(target: "libdatabase::device_manager::device", "Device was mounted, name: {name}");

        let database = match Database::open(&mount_point) {
            Ok(database) => database,
            Err(err) => {
                return Err(OpenDeviceError::DatabaseOpenFailed(err));
            }
        };

        debug!(target: "libdatabase::device_manager::device", "Database opened");

        Ok(Device {
            number,

            dev_name: block_device,

            name,

            database,
        })
    }

    // i'll probably need this later
    #[allow(clippy::unused_async)]
    pub async fn eject(self: Device) {
        info!(target: "libdatabase::device_manager::device", "Ejecting device {}", self.number);

        self.database.close();

        debug!(target: "libdatabase::device_manager::device", "Database closed");

        debug!(target: "libdatabase::device_manager::device", "Device unmounted");
    }
}

async fn get_block_device(path: &Path) -> Option<String> {
    let Ok(mounts) = fs::read_to_string("/proc/mounts").await else {
        return None;
    };

    let target = path.to_string_lossy();

    let mount_line = mounts.lines().find(|line| {
        let mut fields = line.split_whitespace();

        fields.next();
        fields
            .next()
            .is_some_and(|mount_point| mount_point == target.as_ref())
    })?;

    Some(
        mount_line
            .split_whitespace()
            .next()?
            .split('/')
            .next_back()?
            .to_string(),
    )
}

async fn get_partition_name(block_device: &str) -> Option<String> {
    let Ok(udev_device) =
        fs::read_to_string(Path::new(&format!("/sys/class/block/{block_device}/dev"))).await
    else {
        return None;
    };

    let udev_device = udev_device.replace('\n', "");

    debug!(target: "libdatabase::device_manager::device", "has udev device: {udev_device}");

    let Ok(udev_device_info) =
        fs::read_to_string(Path::new(&format!("/run/udev/data/b{udev_device}"))).await
    else {
        return None;
    };

    debug!(target: "libdatabase::device_manager::device", "has udev info");

    for line in udev_device_info.lines() {
        if (line.contains("ID_FS_LABEL") || line.contains("ID_FS_LABEL_ENC"))
            && let Some(name) = line.split('=').next_back()
        {
            return Some(name.to_string());
        }
    }

    debug!(target: "libdatabase::device_manager::device", "no fs label");

    None
}
