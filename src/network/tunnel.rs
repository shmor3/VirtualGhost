#![cfg(unix)]

use crate::error::{NetworkError, VirtualGhostError};
use tokio::io::{AsyncRead, AsyncWrite, ReadHalf, WriteHalf};
use tokio::net::UnixStream;
use tracing::debug;

/// Bidirectional byte-stream tunnel over a vsock connection.
/// Bridges the SSH client to the guest agent through the Firecracker vsock socket.
pub struct VsockTunnel {
    reader: ReadHalf<UnixStream>,
    writer: WriteHalf<UnixStream>,
}

impl VsockTunnel {
    pub fn new(stream: UnixStream) -> Self {
        let (reader, writer) = tokio::io::split(stream);
        Self { reader, writer }
    }

    pub fn split(self) -> (ReadHalf<UnixStream>, WriteHalf<UnixStream>) {
        (self.reader, self.writer)
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
