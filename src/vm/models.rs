use serde::{Deserialize, Serialize};

/// Top-level VM configuration sent to PUT /api/v1/vm.create
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmCreateConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpus: Option<CpusConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<MemoryConfig>,
    pub payload: PayloadConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disks: Option<Vec<DiskConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net: Option<Vec<NetConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rng: Option<RngConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vsock: Option<VsockConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub devices: Option<Vec<DeviceConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial: Option<ConsoleConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub console: Option<ConsoleConfig>,
    #[serde(default)]
    pub iommu: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpusConfig {
    pub boot_vcpus: u32,
    pub max_vcpus: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Memory size in bytes (not MiB)
    pub size: u64,
    #[serde(default)]
    pub shared: bool,
    #[serde(default)]
    pub hugepages: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firmware: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kernel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmdline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initramfs: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskConfig {
    pub path: String,
    #[serde(default)]
    pub readonly: bool,
    #[serde(default)]
    pub direct: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tap: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mask: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RngConfig {
    pub src: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsockConfig {
    pub cid: u64,
    pub socket: String,
    #[serde(default)]
    pub iommu: bool,
}

/// VFIO device passthrough configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    /// sysfs path, e.g. "/sys/bus/pci/devices/0000:01:00.0/"
    pub path: String,
    #[serde(default)]
    pub iommu: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleConfig {
    pub mode: ConsoleMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub socket: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsoleMode {
    Off,
    Pty,
    Tty,
    File,
    Socket,
    Null,
}

/// Response from GET /api/v1/vm.info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmInfo {
    pub state: VmState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VmState {
    Created,
    Running,
    Shutdown,
    Paused,
}

/// Response from GET /api/v1/vmm.ping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmmPingResponse {
    pub build_version: String,
    pub version: String,
    pub pid: i64,
}
