use crate::error::{SshError, VirtualGhostError};
use russh::{client, Channel, ChannelMsg};
use tracing::info;

pub struct SshSession {
    channel: Channel<client::Msg>,
}

impl SshSession {
    /// Open a new PTY session on the SSH connection.
    pub async fn open<H: client::Handler>(
        handle: &client::Handle<H>,
        cols: u32,
        rows: u32,
    ) -> Result<Self, VirtualGhostError> {
        let channel = handle
            .channel_open_session()
            .await
            .map_err(|e| SshError::ChannelError(e.to_string()))?;

        channel
            .request_pty(false, "xterm-256color", cols, rows, 0, 0, &[])
            .await
            .map_err(|e| SshError::ChannelError(format!("PTY request failed: {e}")))?;

        channel
            .request_shell(false)
            .await
            .map_err(|e| SshError::ChannelError(format!("shell request failed: {e}")))?;

        info!(cols, rows, "SSH PTY session opened");

        Ok(Self { channel })
    }

    /// Send data (keystrokes) to the remote shell.
    pub async fn write(&self, data: &[u8]) -> Result<(), VirtualGhostError> {
        self.channel
            .data(data)
            .await
            .map_err(|e| SshError::ChannelError(e.to_string()))?;
        Ok(())
    }

    /// Notify the remote side of a terminal resize.
    pub async fn resize(&self, cols: u32, rows: u32) -> Result<(), VirtualGhostError> {
        self.channel
            .window_change(cols, rows, 0, 0)
            .await
            .map_err(|e| SshError::ChannelError(format!("window change failed: {e}")))?;
        Ok(())
    }

    /// Wait for the next message from the channel.
    pub async fn read(&mut self) -> Option<ChannelMsg> {
        self.channel.wait().await
    }
}
