#![allow(dead_code)]

use crate::error::VirtualGhostError;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tracing::{debug, info};

/// Connects to the guest agent via TCP (QEMU user-mode port forwarding).
/// Used on macOS/Windows where vsock is not available.
pub struct GuestTunnel;

impl GuestTunnel {
    /// Connect to the guest SSH server via TCP port forwarding.
    pub async fn connect_tcp(port: u16) -> Result<TcpStream, VirtualGhostError> {
        let addr = format!("127.0.0.1:{port}");
        info!(%addr, "Connecting to guest via TCP");

        let stream = TcpStream::connect(&addr).await.map_err(|e| {
            crate::error::NetworkError::VsockConnectionFailed(format!(
                "failed to connect to guest at {addr}: {e}"
            ))
        })?;

        info!(%addr, "TCP connection to guest established");
        Ok(stream)
    }

    /// Copy data bidirectionally between two async streams until one side closes.
    pub async fn bridge<A, B>(a: A, b: B) -> Result<(), VirtualGhostError>
    where
        A: AsyncRead + AsyncWrite + Unpin,
        B: AsyncRead + AsyncWrite + Unpin,
    {
        let (mut a_read, mut a_write) = tokio::io::split(a);
        let (mut b_read, mut b_write) = tokio::io::split(b);

        let a_to_b = tokio::io::copy(&mut a_read, &mut b_write);
        let b_to_a = tokio::io::copy(&mut b_read, &mut a_write);

        tokio::select! {
            result = a_to_b => {
                debug!(bytes = ?result, "tunnel a->b closed");
            }
            result = b_to_a => {
                debug!(bytes = ?result, "tunnel b->a closed");
            }
        }

        Ok(())
    }
}
