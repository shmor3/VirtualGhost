#![allow(dead_code)]

#[cfg(unix)]
mod setup;

#[cfg(unix)]
pub use setup::*;

use crate::vm::DeviceConfig;

/// Represents a GPU device for VFIO passthrough.
#[derive(Debug, Clone)]
pub struct GpuDevice {
    pub pci_address: String,
    pub sysfs_path: String,
    pub iommu_group: u32,
    /// Other devices in the same IOMMU group (must also be passed through)
    pub siblings: Vec<String>,
}

impl GpuDevice {
    /// Convert this GPU device (and its siblings) into Cloud Hypervisor DeviceConfig entries.
    pub fn to_device_configs(&self) -> Vec<DeviceConfig> {
        let mut configs = vec![DeviceConfig {
            path: self.sysfs_path.clone(),
            iommu: false,
            id: Some("gpu0".to_string()),
        }];

        for (i, sibling) in self.siblings.iter().enumerate() {
            configs.push(DeviceConfig {
                path: format!("/sys/bus/pci/devices/{sibling}/"),
                iommu: false,
                id: Some(format!("gpu0_sibling{i}")),
            });
        }

        configs
    }
}
