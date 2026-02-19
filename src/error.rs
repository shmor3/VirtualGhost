use thiserror::Error;

#[derive(Debug, Error)]
pub enum VirtualGhostError {
    #[error("VM error: {0}")]
    Vm(#[from] VmError),

    #[error("SSH error: {0}")]
    Ssh(#[from] SshError),

    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Terminal error: {0}")]
    Terminal(#[from] TerminalError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum VmError {
    #[error("failed to spawn firecracker process: {0}")]
    SpawnFailed(std::io::Error),

    #[error("firecracker API error: {status} {body}")]
    ApiError { status: u16, body: String },

    #[error("VM boot timed out")]
    BootTimeout,

    #[error("asset extraction failed: {0}")]
    AssetExtraction(String),

    #[error("firecracker process exited unexpectedly: code {0:?}")]
    ProcessExited(Option<i32>),
}

#[derive(Debug, Error)]
pub enum SshError {
    #[error("SSH connection failed: {0}")]
    ConnectionFailed(String),

    #[error("SSH authentication failed")]
    AuthFailed,

    #[error("SSH channel error: {0}")]
    ChannelError(String),

    #[error("key generation failed: {0}")]
    KeyGeneration(String),
}

#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("vsock connection failed: {0}")]
    VsockConnectionFailed(String),

    #[error("tunnel error: {0}")]
    TunnelError(String),
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("invalid configuration: {0}")]
    Invalid(String),

    #[error("config file error: {0}")]
    FileError(String),
}

#[derive(Debug, Error)]
pub enum TerminalError {
    #[error("terminal initialization failed: {0}")]
    InitFailed(String),

    #[error("render error: {0}")]
    RenderError(String),
}

pub type Result<T> = std::result::Result<T, VirtualGhostError>;
