use super::settings::*;

impl Default for VirtualGhostConfig {
    fn default() -> Self {
        Self {
            vm: VmConfig::default(),
            ssh: SshConfig::default(),
            terminal: TerminalConfig::default(),
        }
    }
}

impl Default for VmConfig {
    fn default() -> Self {
        Self {
            vcpus: 1,
            memory_mib: 128,
            kernel_path: None,
            rootfs_path: None,
            firecracker_bin: None,
        }
    }
}

impl Default for SshConfig {
    fn default() -> Self {
        Self {
            key_path: None,
            vsock_port: 52,
        }
    }
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            scrollback_lines: 10_000,
        }
    }
}
