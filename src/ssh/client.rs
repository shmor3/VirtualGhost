use crate::error::{SshError, VirtualGhostError};
use russh::*;
use ssh_key::public::PublicKey;
use ssh_key::private::PrivateKey;
use std::sync::Arc;
use tracing::info;

struct ClientHandler;

#[async_trait::async_trait]
impl client::Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        // Accept all host keys — the guest agent uses ephemeral keys too
        Ok(true)
    }
}

pub struct SshClient {
    handle: client::Handle<ClientHandler>,
}

impl SshClient {
    /// Connect to the guest SSH server over an already-established stream.
    /// The stream is typically a vsock Unix socket connection.
    pub async fn connect<S>(stream: S, user: &str, key: &PrivateKey) -> Result<Self, VirtualGhostError>
    where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
    {
        let config = Arc::new(client::Config::default());
        let handler = ClientHandler;

        let mut session = client::connect_stream(config, stream, handler)
            .await
            .map_err(|e| SshError::ConnectionFailed(e.to_string()))?;

        let auth_result = session
            .authenticate_publickey(user, Arc::new(key.clone()))
            .await
            .map_err(|e| SshError::ConnectionFailed(e.to_string()))?;

        if !auth_result {
            return Err(SshError::AuthFailed.into());
        }

        info!(user, "SSH authentication successful");

        Ok(Self { handle: session })
    }

    pub fn handle(&self) -> &client::Handle<ClientHandler> {
        &self.handle
    }
}
