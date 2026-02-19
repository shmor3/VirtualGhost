#![cfg(unix)]

use crate::error::{VmError, VirtualGhostError};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::{Child, Command};
use tracing::{info, warn};

pub struct CloudHypervisorProcess {
    child: Child,
    socket_path: PathBuf,
}

impl CloudHypervisorProcess {
    pub async fn spawn(
        ch_bin: &Path,
        socket_path: &Path,
    ) -> Result<Self, VirtualGhostError> {
        if socket_path.exists() {
            std::fs::remove_file(socket_path).ok();
        }

        info!(
            bin = %ch_bin.display(),
            socket = %socket_path.display(),
            "Spawning Cloud Hypervisor process"
        );

        let child = Command::new(ch_bin)
            .arg("--api-socket")
            .arg(format!("path={}", socket_path.display()))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(VmError::SpawnFailed)?;

        // Wait for the API socket to appear
        for _ in 0..50 {
            if socket_path.exists() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        if !socket_path.exists() {
            return Err(VmError::BootTimeout.into());
        }

        info!("Cloud Hypervisor process started, API socket ready");

        Ok(Self {
            child,
            socket_path: socket_path.to_path_buf(),
        })
    }

    pub fn id(&self) -> Option<u32> {
        self.child.id()
    }

    pub async fn wait(&mut self) -> Result<std::process::ExitStatus, VirtualGhostError> {
        self.child.wait().await.map_err(VirtualGhostError::Io)
    }

    pub async fn kill(&mut self) -> Result<(), VirtualGhostError> {
        warn!("Killing Cloud Hypervisor process");
        self.child.kill().await.map_err(VirtualGhostError::Io)?;
        self.cleanup();
        Ok(())
    }

    fn cleanup(&self) {
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path).ok();
        }
    }
}

impl Drop for CloudHypervisorProcess {
    fn drop(&mut self) {
        self.cleanup();
    }
}
