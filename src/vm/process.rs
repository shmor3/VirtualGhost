#![allow(dead_code)]

use crate::error::{VmError, VirtualGhostError};
use std::process::ExitStatus;
use tokio::process::{Child, Command};
use tracing::info;

use super::config::QemuConfig;

pub struct QemuProcess {
    child: Child,
}

impl QemuProcess {
    /// Spawn QEMU with the given configuration.
    pub async fn spawn(config: &QemuConfig) -> Result<Self, VirtualGhostError> {
        let args = config.to_args();

        info!(
            bin = %config.qemu_bin.display(),
            args = ?args,
            "Spawning QEMU"
        );

        let mut cmd = Command::new(&config.qemu_bin);
        cmd.args(&args).stdin(std::process::Stdio::null());

        // Ensure extracted QEMU can find its DLLs/shared libraries
        if let Some(qemu_dir) = config.qemu_bin.parent() {
            #[cfg(target_os = "windows")]
            {
                let path_var = std::env::var("PATH").unwrap_or_default();
                let new_path = format!("{};{}", qemu_dir.display(), path_var);
                cmd.env("PATH", new_path);
            }

            #[cfg(target_os = "linux")]
            {
                let ld_path = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();
                let new_path = if ld_path.is_empty() {
                    qemu_dir.display().to_string()
                } else {
                    format!("{}:{}", qemu_dir.display(), ld_path)
                };
                cmd.env("LD_LIBRARY_PATH", new_path);
            }

            #[cfg(target_os = "macos")]
            {
                let dyld_path = std::env::var("DYLD_LIBRARY_PATH").unwrap_or_default();
                let new_path = if dyld_path.is_empty() {
                    qemu_dir.display().to_string()
                } else {
                    format!("{}:{}", qemu_dir.display(), dyld_path)
                };
                cmd.env("DYLD_LIBRARY_PATH", new_path);
            }
        }

        let child = cmd.spawn().map_err(VmError::SpawnFailed)?;

        info!(pid = child.id(), "QEMU process started");

        Ok(Self { child })
    }

    /// Wait for the QEMU process to exit.
    pub async fn wait(&mut self) -> Result<ExitStatus, VirtualGhostError> {
        let status = self
            .child
            .wait()
            .await
            .map_err(VmError::SpawnFailed)?;
        Ok(status)
    }

    /// Kill the QEMU process.
    pub async fn kill(&mut self) -> Result<(), VirtualGhostError> {
        self.child
            .kill()
            .await
            .map_err(VmError::SpawnFailed)?;
        Ok(())
    }
}

impl Drop for QemuProcess {
    fn drop(&mut self) {
        // Best-effort kill on drop
        let _ = self.child.start_kill();
    }
}
