#![cfg(unix)]

use crate::error::{NetworkError, VirtualGhostError};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tracing::info;

pub struct VsockConnection {
    stream: UnixStream,
}

impl VsockConnection {
    /// Connect to the guest via Firecracker's vsock Unix socket.
    ///
    /// Firecracker maps guest vsock ports to a host-side Unix socket.
    /// The host sends `CONNECT <port>\n` and receives `OK <id>\n` on success,
    /// after which the stream becomes a bidirectional byte pipe to the guest.
    pub async fn connect(uds_path: &Path, port: u32) -> Result<Self, VirtualGhostError> {
        info!(socket = %uds_path.display(), port, "Connecting to guest via vsock");

        let mut stream = UnixStream::connect(uds_path).await.map_err(|e| {
            NetworkError::VsockConnectionFailed(format!(
                "failed to connect to vsock socket {}: {e}",
                uds_path.display()
            ))
        })?;

        // Send the CONNECT handshake
        let connect_msg = format!("CONNECT {port}\n");
        stream.write_all(connect_msg.as_bytes()).await.map_err(|e| {
            NetworkError::VsockConnectionFailed(format!("handshake write failed: {e}"))
        })?;

        // Read the response (expect "OK <id>\n")
        let mut buf = [0u8; 64];
        let n = stream.read(&mut buf).await.map_err(|e| {
            NetworkError::VsockConnectionFailed(format!("handshake read failed: {e}"))
        })?;

        let response = String::from_utf8_lossy(&buf[..n]);
        if !response.starts_with("OK") {
            return Err(NetworkError::VsockConnectionFailed(format!(
                "vsock handshake rejected: {response}"
            ))
            .into());
        }

        info!(port, "Vsock connection established");
        Ok(Self { stream })
    }

    pub fn into_stream(self) -> UnixStream {
        self.stream
    }
}
