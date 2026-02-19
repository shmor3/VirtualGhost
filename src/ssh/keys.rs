use crate::error::{SshError, VirtualGhostError};
use ssh_key::private::PrivateKey;
use tracing::info;

pub struct KeyManager;

impl KeyManager {
    /// Generate an ephemeral Ed25519 key pair for a single session.
    pub fn generate_ephemeral() -> Result<PrivateKey, VirtualGhostError> {
        info!("Generating ephemeral Ed25519 key pair");
        let key = PrivateKey::random(&mut rand::thread_rng(), ssh_key::Algorithm::Ed25519)
            .map_err(|e| SshError::KeyGeneration(e.to_string()))?;
        Ok(key)
    }
}
