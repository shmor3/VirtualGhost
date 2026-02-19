mod cli;
mod config;
mod error;
mod network;
mod ssh;
mod terminal;
mod vm;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use cli::{Cli, Command};
use config::VirtualGhostConfig;
use terminal::App;
use vm::AssetManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let filter = if cli.verbose {
        EnvFilter::new("virtualghost=debug")
    } else {
        EnvFilter::new("virtualghost=info")
    };
    tracing_subscriber::fmt().with_env_filter(filter).init();

    match cli.effective_command() {
        Command::Run => cmd_run(&cli).await?,
        Command::Config { show } => cmd_config(*show).await?,
        Command::Clean => cmd_clean().await?,
    }

    Ok(())
}

async fn cmd_run(cli: &Cli) -> anyhow::Result<()> {
    let mut config = VirtualGhostConfig::load()?;

    // Apply CLI overrides
    config.vm.vcpus = cli.vcpus;
    config.vm.memory_mib = cli.memory;
    if let Some(ref kernel) = cli.kernel {
        config.vm.kernel_path = Some(kernel.clone());
    }
    if let Some(ref rootfs) = cli.rootfs {
        config.vm.rootfs_path = Some(rootfs.clone());
    }

    // Resolve asset paths
    let asset_manager = AssetManager::new();
    let kernel_path = config
        .vm
        .kernel_path
        .clone()
        .unwrap_or_else(|| asset_manager.kernel_path());
    let rootfs_path = config
        .vm
        .rootfs_path
        .clone()
        .unwrap_or_else(|| asset_manager.rootfs_path());

    // If no custom paths provided, try to extract embedded assets
    if config.vm.kernel_path.is_none() || config.vm.rootfs_path.is_none() {
        match asset_manager.ensure_assets() {
            Ok(()) => {}
            Err(e) => {
                tracing::warn!("Asset extraction skipped: {e}");
                tracing::info!("Starting in standalone terminal mode");
                let (cols, rows) = crossterm::terminal::size()?;
                let mut app = App::new(cols as usize, rows.saturating_sub(1) as usize);
                app.run_standalone().await?;
                return Ok(());
            }
        }
    }

    // TODO: Full VM boot sequence
    // 1. Spawn Firecracker process
    // 2. Configure VM (machine config, boot source, drives, vsock)
    // 3. Start instance
    // 4. Connect via vsock
    // 5. Establish SSH session
    // 6. Run terminal UI with SSH I/O

    tracing::info!(
        kernel = %kernel_path.display(),
        rootfs = %rootfs_path.display(),
        vcpus = config.vm.vcpus,
        memory = config.vm.memory_mib,
        "VM configuration ready"
    );

    // For now, run standalone terminal
    let (cols, rows) = crossterm::terminal::size()?;
    let mut app = App::new(cols as usize, rows.saturating_sub(1) as usize);
    app.set_status(format!(
        "VM: {}vcpu/{}MiB | Ctrl+Shift+Q to quit",
        config.vm.vcpus, config.vm.memory_mib
    ));
    app.run_standalone().await?;

    Ok(())
}

async fn cmd_config(show: bool) -> anyhow::Result<()> {
    if show {
        let config = VirtualGhostConfig::load()?;
        println!("{}", toml::to_string_pretty(&config)?);
    } else {
        println!("Config file: {}", VirtualGhostConfig::config_path().display());
        println!("Cache dir:   {}", VirtualGhostConfig::cache_dir().display());
    }
    Ok(())
}

async fn cmd_clean() -> anyhow::Result<()> {
    let asset_manager = AssetManager::new();
    asset_manager.clean_cache()?;
    println!("Cache cleaned.");
    Ok(())
}
