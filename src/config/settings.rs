use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualGhostConfig {
    pub vm: VmSettings,
    pub ssh: SshSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmSettings {
    pub vcpus: u32,
    pub memory_mib: u32,
    pub kernel_path: Option<PathBuf>,
    pub rootfs_path: Option<PathBuf>,
    pub qemu_bin: Option<PathBuf>,
    pub gpu_pci_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshSettings {
    pub key_path: Option<PathBuf>,
    pub vsock_port: u32,
}

impl VirtualGhostConfig {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = Self::config_path();
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Self = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn config_path() -> PathBuf {
        directories::ProjectDirs::from("com", "virtualghost", "VirtualGhost")
            .map(|dirs| dirs.config_dir().join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("config.toml"))
    }

    pub fn cache_dir() -> PathBuf {
        directories::ProjectDirs::from("com", "virtualghost", "VirtualGhost")
            .map(|dirs| dirs.cache_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".cache"))
    }
}

impl Default for VirtualGhostConfig {
    fn default() -> Self {
        Self {
            vm: VmSettings {
                vcpus: 2,
                memory_mib: 2048,
                kernel_path: None,
                rootfs_path: None,
                qemu_bin: None,
                gpu_pci_address: None,
            },
            ssh: SshSettings {
                key_path: None,
                vsock_port: 52,
            },
        }
    }
}
