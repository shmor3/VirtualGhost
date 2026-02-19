#![cfg(unix)]

use anyhow::Result;
use russh::server::{self, Auth, Msg, Session};
use russh::{Channel, ChannelId};
use ssh_key::public::PublicKey;
use std::collections::HashMap;
use tracing::info;

pub async fn run(port: u32) -> Result<()> {
    info!(port, "SSH server ready (vsock listener not yet implemented)");

    // TODO: Replace with tokio-vsock VsockListener
    // let listener = VsockListener::bind(libc::VMADDR_CID_ANY, port)?;
    // loop {
    //     let (stream, addr) = listener.accept().await?;
    //     tokio::spawn(handle_connection(stream));
    // }

    // Placeholder: wait forever
    tokio::signal::ctrl_c().await?;
    info!("Shutting down");
    Ok(())
}

struct GhostlyServer {
    _authorized_keys: Vec<PublicKey>,
}

struct GhostlySession {
    channels: HashMap<ChannelId, ChannelState>,
}

struct ChannelState {
    _pty_requested: bool,
}

impl server::Server for GhostlyServer {
    type Handler = GhostlySession;

    fn new_client(&mut self, _peer_addr: Option<std::net::SocketAddr>) -> Self::Handler {
        GhostlySession {
            channels: HashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl server::Handler for GhostlySession {
    type Error = anyhow::Error;

    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        _session: &mut Session,
    ) -> Result<bool, Self::Error> {
        self.channels.insert(
            channel.id(),
            ChannelState {
                _pty_requested: false,
            },
        );
        Ok(true)
    }

    async fn auth_publickey(
        &mut self,
        _user: &str,
        _public_key: &PublicKey,
    ) -> Result<Auth, Self::Error> {
        // TODO: Check against authorized_keys list
        Ok(Auth::Accept)
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        // TODO: Forward data to PTY
        // For now, echo back
        session.data(channel, data.to_vec().into())?;
        Ok(())
    }
}
