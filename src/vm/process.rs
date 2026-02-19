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

        let child = Command::new(&config.qemu_bin)
            .args(&args)
            .stdin(std::process::Stdio::null())
            .spawn()
            .map_err(VmError::SpawnFailed)?;

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
