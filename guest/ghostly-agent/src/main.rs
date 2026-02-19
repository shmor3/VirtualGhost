use anyhow::Result;
use tracing::info;

#[cfg(unix)]
mod server;

const VSOCK_PORT: u32 = 52;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("ghostly_agent=debug")
        .init();

    info!("Ghostly Agent starting on vsock port {VSOCK_PORT}");

    #[cfg(unix)]
    server::run(VSOCK_PORT).await?;

    #[cfg(not(unix))]
    {
        tracing::error!("Ghostly Agent only runs on Linux (inside a Firecracker VM)");
        std::process::exit(1);
    }
}
