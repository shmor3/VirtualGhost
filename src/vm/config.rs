#![cfg(unix)]

use crate::config::VmConfig;
use crate::error::VirtualGhostError;

use super::api::FirecrackerClient;
use super::models::*;

pub struct VmConfigBuilder {
    machine_config: MachineConfig,
    kernel_path: String,
    rootfs_path: String,
    vsock_uds_path: Option<String>,
    boot_args: String,
}

impl VmConfigBuilder {
    pub fn from_config(config: &VmConfig, kernel_path: &str, rootfs_path: &str) -> Self {
        Self {
            machine_config: MachineConfig {
                vcpu_count: config.vcpus,
                mem_size_mib: config.memory_mib,
            },
            kernel_path: kernel_path.to_string(),
            rootfs_path: rootfs_path.to_string(),
            vsock_uds_path: None,
            boot_args: "console=ttyS0 reboot=k panic=1 pci=off".to_string(),
        }
    }

    pub fn vsock_uds_path(mut self, path: &str) -> Self {
        self.vsock_uds_path = Some(path.to_string());
        self
    }

    pub fn boot_args(mut self, args: &str) -> Self {
        self.boot_args = args.to_string();
        self
    }

    pub async fn apply(&self, client: &FirecrackerClient) -> Result<(), VirtualGhostError> {
        client.set_machine_config(&self.machine_config).await?;

        client
            .set_boot_source(&BootSource {
                kernel_image_path: self.kernel_path.clone(),
                boot_args: self.boot_args.clone(),
            })
            .await?;

        client
            .set_drive(&Drive {
                drive_id: "rootfs".to_string(),
                path_on_host: self.rootfs_path.clone(),
                is_root_device: true,
                is_read_only: false,
            })
            .await?;

        if let Some(ref uds_path) = self.vsock_uds_path {
            client
                .set_vsock(&VsockConfig {
                    guest_cid: 3,
                    uds_path: uds_path.clone(),
                })
                .await?;
        }

        Ok(())
    }
}
