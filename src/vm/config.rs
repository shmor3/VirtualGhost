#![cfg(unix)]

use crate::error::VirtualGhostError;

use super::api::CloudHypervisorClient;
use super::models::*;

pub struct VmConfigBuilder {
    cpus: CpusConfig,
    memory: MemoryConfig,
    payload: PayloadConfig,
    disks: Vec<DiskConfig>,
    vsock: Option<VsockConfig>,
    devices: Vec<DeviceConfig>,
    serial: Option<ConsoleConfig>,
    console: Option<ConsoleConfig>,
}

impl VmConfigBuilder {
    pub fn new(vcpus: u32, memory_mib: u32, kernel_path: &str, rootfs_path: &str) -> Self {
        Self {
            cpus: CpusConfig {
                boot_vcpus: vcpus,
                max_vcpus: vcpus,
            },
            memory: MemoryConfig {
                size: (memory_mib as u64) * 1024 * 1024,
                shared: false,
                hugepages: false,
            },
            payload: PayloadConfig {
                kernel: Some(kernel_path.to_string()),
                cmdline: Some("console=ttyS0 root=/dev/vda rw".to_string()),
                firmware: None,
                initramfs: None,
            },
            disks: vec![DiskConfig {
                path: rootfs_path.to_string(),
                readonly: false,
                direct: false,
                id: Some("rootfs".to_string()),
            }],
            vsock: None,
            devices: Vec::new(),
            serial: None,
            console: None,
        }
    }

    pub fn vsock(mut self, socket_path: &str, cid: u64) -> Self {
        self.vsock = Some(VsockConfig {
            cid,
            socket: socket_path.to_string(),
            iommu: false,
        });
        self
    }

    pub fn add_vfio_device(mut self, sysfs_path: &str) -> Self {
        self.devices.push(DeviceConfig {
            path: sysfs_path.to_string(),
            iommu: false,
            id: None,
        });
        self
    }

    pub fn serial_console(mut self) -> Self {
        self.serial = Some(ConsoleConfig {
            mode: ConsoleMode::Tty,
            file: None,
            socket: None,
        });
        self.console = Some(ConsoleConfig {
            mode: ConsoleMode::Off,
            file: None,
            socket: None,
        });
        self
    }

    pub fn cmdline(mut self, args: &str) -> Self {
        self.payload.cmdline = Some(args.to_string());
        self
    }

    pub fn build(self) -> VmCreateConfig {
        VmCreateConfig {
            cpus: Some(self.cpus),
            memory: Some(self.memory),
            payload: self.payload,
            disks: Some(self.disks),
            net: None,
            rng: Some(RngConfig {
                src: "/dev/urandom".to_string(),
            }),
            vsock: self.vsock,
            devices: if self.devices.is_empty() {
                None
            } else {
                Some(self.devices)
            },
            serial: self.serial,
            console: self.console,
            iommu: false,
        }
    }

    /// Send configuration to Cloud Hypervisor and boot.
    pub async fn apply(self, client: &CloudHypervisorClient) -> Result<(), VirtualGhostError> {
        let vm_config = self.build();
        client.vm_create(&vm_config).await?;
        client.vm_boot().await?;
        Ok(())
    }
}
