use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use nix::mount::{MntFlags, MsFlags, mount, umount2};
use nusb::{DeviceId, DeviceInfo};
use thiserror::Error;
use tokio::fs;

use crate::database::{Database, OpenDatabaseError};

pub(crate) const MOUNT_BASE: &str = "/tmp/usb";

const CLASS_MASS_STORAGE: u8 = 0x08;

const PROBE_RETRIES: u32 = 50;
const PROBE_DELAY: Duration = Duration::from_millis(200);

pub struct Device {
    pub number: u32,

    pub(crate) id: DeviceId,

    mount_point: PathBuf,

    pub name: String,

    pub database: Database,
}

#[derive(Error, Debug)]
pub enum OpenDeviceError {
    #[error("Invalid Device Type")]
    InvalidDeviceType,

    #[error("No Valid Block Device")]
    NoBlockDevice,

    #[error("Disk Mount Failed: {0}")]
    MountFailed(DiskMountError),

    #[error("Failed to open Device Database: {0}")]
    DatabaseOpenFailed(OpenDatabaseError),
}

impl Device {
    pub(crate) async fn open(info: DeviceInfo) -> Result<Device, OpenDeviceError> {
        if !has_mass_storage(&info) {
            return Err(OpenDeviceError::InvalidDeviceType);
        }

        let Some(disk) = get_device_fs_path(&info).await else {
            return Err(OpenDeviceError::NoBlockDevice);
        };

        let (mount_point, device_number, device_name) = match mount_disk(&disk).await {
            Ok(pair) => pair,
            Err(err) => {
                return Err(OpenDeviceError::MountFailed(err));
            }
        };

        let database = match Database::open(&mount_point) {
            Ok(database) => database,
            Err(err) => {
                if let Ok(()) = umount2(&mount_point, MntFlags::MNT_DETACH) {
                    let _ = fs::remove_dir(&mount_point).await;
                }

                return Err(OpenDeviceError::DatabaseOpenFailed(err));
            }
        };

        println!("name: {device_name}");

        Ok(Device {
            number: device_number,

            id: info.id(),

            mount_point,

            name: device_name,

            database,
        })
    }

    pub async fn eject(self: Device) {
        self.database.close();

        if let Ok(()) = umount2(&self.mount_point, MntFlags::MNT_DETACH) {
            let _ = fs::remove_dir(&self.mount_point).await;
        }
    }
}

fn has_mass_storage(info: &DeviceInfo) -> bool {
    if info.class() == CLASS_MASS_STORAGE {
        return true;
    }

    for interface in info.interfaces() {
        if interface.class() == CLASS_MASS_STORAGE {
            return true;
        }
    }

    false
}

async fn get_device_fs_path(info: &DeviceInfo) -> Option<String> {
    let bus = info.busnum();
    let ports = info.port_chain();

    let name = if ports.is_empty() {
        format!("usb{bus}")
    } else {
        let port_str = ports
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(".");
        format!("{bus}-{port_str}")
    };

    let usb_path = PathBuf::from("/sys/bus/usb/devices").join(name);

    for _ in 0..PROBE_RETRIES {
        tokio::time::sleep(PROBE_DELAY).await;

        if let Some(disk) = find_block_recursive(&usb_path).await {
            return Some(disk);
        }
    }

    None
}

async fn find_block_recursive(dir: &Path) -> Option<String> {
    let Ok(mut entries) = fs::read_dir(dir).await else {
        return None;
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().into_owned();

        if name == "block" {
            let Ok(mut block_entries) = fs::read_dir(entry.path()).await else {
                continue;
            };

            while let Ok(Some(block_entry)) = block_entries.next_entry().await {
                let block_name = block_entry.file_name().to_string_lossy().into_owned();

                if block_name.starts_with("sd") && block_name.len() == 3 {
                    return Some(block_name);
                }
            }
        }

        let Ok(file_type) = entry.file_type().await else {
            continue;
        };

        if file_type.is_symlink() {
            continue;
        }

        if file_type.is_dir()
            && let Some(found) = Box::pin(find_block_recursive(&entry.path())).await
        {
            return Some(found);
        }
    }

    None
}

#[derive(Error, Debug)]
pub enum DiskMountError {
    #[error("No Available Partitions: {0}")]
    NoPartitions(FindPartitionError),

    #[error("No Free Mount Points Available: {0}")]
    NoFreeMountPoints(FindMountPointError),

    #[error("Could not Create Mount Point: {0}")]
    MountPointCreationFailed(tokio::io::Error),

    #[error("Failed to Find a Matching Filesystem Type")]
    NoMatchingFilesystem,
}

async fn mount_disk(disk: &str) -> Result<(PathBuf, u32, String), DiskMountError> {
    let (partition_name, device_path) = match find_first_partition(disk).await {
        Ok(pair) => pair,
        Err(err) => {
            return Err(DiskMountError::NoPartitions(err));
        }
    };

    let (mount_point, device_number) = match find_free_mount_point().await {
        Ok(pair) => pair,
        Err(err) => {
            return Err(DiskMountError::NoFreeMountPoints(err));
        }
    };

    if let Err(err) = fs::create_dir_all(&mount_point).await {
        return Err(DiskMountError::MountPointCreationFailed(err));
    }

    let filesystems = [
        "vfat", "exfat", "ntfs", "ext4", "ext3", "ext2", "btrfs", "xfs",
    ];

    for filesystem in filesystems {
        if let Ok(()) = mount(
            Some(device_path.as_path()),
            mount_point.as_path(),
            Some(filesystem),
            MsFlags::MS_RELATIME,
            None::<&str>,
        ) {
            return Ok((mount_point, device_number, partition_name));
        }
    }

    let _ = fs::remove_dir(&mount_point).await;

    Err(DiskMountError::NoMatchingFilesystem)
}

#[derive(Error, Debug)]
pub enum FindPartitionError {
    #[error("No Free Mount Points Available: {0}")]
    BlockReadError(tokio::io::Error),

    #[error("No Partitions Found on Device")]
    NoAvailablePartitions,
}

async fn find_first_partition(disk: &str) -> Result<(String, PathBuf), FindPartitionError> {
    let block = Path::new("/sys/class/block");

    let mut entries = match fs::read_dir(block).await {
        Ok(entries) => entries,
        Err(err) => {
            return Err(FindPartitionError::BlockReadError(err));
        }
    };

    let mut partitions = Vec::new();

    while let Ok(Some(entry)) = entries.next_entry().await {
        let entry_name = entry.file_name().to_string_lossy().into_owned();

        if entry_name.starts_with(disk)
            && entry_name.len() > disk.len()
            && entry_name[disk.len()..].chars().all(|c| c.is_ascii_digit())
        {
            partitions.push(entry_name);
        }
    }

    if partitions.is_empty() {
        let partition_name = get_partition_name(disk).await.unwrap_or(disk.to_string());

        return Ok((partition_name, PathBuf::from(format!("/dev/{disk}"))));
    }

    partitions.sort();

    let Some(partition) = partitions.first() else {
        return Err(FindPartitionError::NoAvailablePartitions);
    };

    let partition_name = get_partition_name(&format!("{disk}/{partition}"))
        .await
        .unwrap_or(partition.clone());

    Ok((partition_name, PathBuf::from(format!("/dev/{partition}"))))
}

async fn get_partition_name(block_device: &str) -> Option<String> {
    let Ok(udev_device) =
        fs::read_to_string(Path::new(&format!("/sys/class/block/{block_device}/dev"))).await
    else {
        return None;
    };

    let udev_device = udev_device.replace('\n', "");

    let Ok(udev_device_info) =
        fs::read_to_string(Path::new(&format!("/run/udev/data/b{udev_device}"))).await
    else {
        return None;
    };

    for line in udev_device_info.lines() {
        if (line.contains("ID_FS_LABEL") || line.contains("ID_FS_LABEL_ENC"))
            && let Some(name) = line.split('=').next_back()
        {
            return Some(name.to_string());
        }
    }

    None
}

#[derive(Error, Debug)]
pub enum FindMountPointError {
    #[error("Parent Directory Creation Failed: {0}")]
    ParentDirectoryCreationFailed(tokio::io::Error),

    #[error("Mount Points Exhausted")]
    MountPointsExhausted,
}

async fn find_free_mount_point() -> Result<(PathBuf, u32), FindMountPointError> {
    let base = PathBuf::from(MOUNT_BASE);

    if !base.exists()
        && let Err(err) = fs::create_dir(MOUNT_BASE).await
    {
        return Err(FindMountPointError::ParentDirectoryCreationFailed(err));
    }

    for disk_number in 0u32.. {
        let candidate = PathBuf::from(format!("{MOUNT_BASE}{disk_number}"));

        if !candidate.exists() || !is_mounted(&candidate).await {
            return Ok((candidate, disk_number));
        }
    }

    Err(FindMountPointError::MountPointsExhausted)
}

async fn is_mounted(path: &Path) -> bool {
    let Ok(mounts) = fs::read_to_string("/proc/mounts").await else {
        return false;
    };

    let target = path.to_string_lossy();

    mounts.lines().any(|line| {
        let mut fields = line.split_whitespace();

        fields.next();
        fields
            .next()
            .is_some_and(|mount_point| mount_point == target.as_ref())
    })
}
