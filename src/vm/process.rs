use crate::error::{VmError, VirtualGhostError};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::{Child, Command};
use tracing::{info, warn};

pub struct FirecrackerProcess {
    child: Child,
    socket_path: PathBuf,
}

impl FirecrackerProcess {
    pub async fn spawn(
        firecracker_bin: &Path,
        socket_path: &Path,
    ) -> Result<Self, VirtualGhostError> {
        // Clean up stale socket if it exists
        if socket_path.exists() {
            std::fs::remove_file(socket_path).ok();
        }

        info!(
            bin = %firecracker_bin.display(),
            socket = %socket_path.display(),
            "Spawning Firecracker process"
        );

        let child = Command::new(firecracker_bin)
            .arg("--api-sock")
            .arg(socket_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(VmError::SpawnFailed)?;

        // Wait briefly for the socket to appear
        for _ in 0..50 {
            if socket_path.exists() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        if !socket_path.exists() {
            return Err(VmError::BootTimeout.into());
        }

        info!("Firecracker process started, API socket ready");

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
        warn!("Killing Firecracker process");
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

impl Drop for FirecrackerProcess {
    fn drop(&mut self) {
        self.cleanup();
    }
}
