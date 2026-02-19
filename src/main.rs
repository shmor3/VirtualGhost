mod cli;
mod config;
mod error;
mod network;
mod ssh;
mod vfio;
mod vm;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use cli::{Cli, Command};
use config::VirtualGhostConfig;
use vm::AssetManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

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
    if let Some(ref gpu) = cli.gpu {
        config.vm.gpu_pci_address = Some(gpu.clone());
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

    // Try to extract embedded assets if no custom paths provided
    if config.vm.kernel_path.is_none() || config.vm.rootfs_path.is_none() {
        if let Err(e) = asset_manager.ensure_assets() {
            anyhow::bail!(
                "No kernel/rootfs available: {e}\n\
                 Provide --kernel and --rootfs paths, or place assets in cache."
            );
        }
    }

    // GPU passthrough setup (Linux only)
    #[cfg(unix)]
    let gpu_devices = if let Some(ref pci_addr) = config.vm.gpu_pci_address {
        tracing::info!(pci_addr, "Preparing GPU for VFIO passthrough");
        let gpu = vfio::discover_gpu(pci_addr)?;
        vfio::prepare_passthrough(&gpu)?;
        gpu.to_device_configs()
    } else {
        Vec::new()
    };

    #[cfg(not(unix))]
    if config.vm.gpu_pci_address.is_some() {
        anyhow::bail!("GPU passthrough requires Linux with KVM and IOMMU support");
    }

    tracing::info!(
        kernel = %kernel_path.display(),
        rootfs = %rootfs_path.display(),
        vcpus = config.vm.vcpus,
        memory_mib = config.vm.memory_mib,
        gpu = ?config.vm.gpu_pci_address,
        "Starting VirtualGhost"
    );

    // Boot the VM (Linux only)
    #[cfg(unix)]
    {
        use std::path::PathBuf;

        let ch_bin = config
            .vm
            .cloud_hypervisor_bin
            .clone()
            .unwrap_or_else(|| PathBuf::from("cloud-hypervisor"));

        let socket_path = std::env::temp_dir().join(format!(
            "virtualghost-{}.sock",
            uuid::Uuid::new_v4()
        ));
        let vsock_path = std::env::temp_dir().join(format!(
            "virtualghost-vsock-{}.sock",
            uuid::Uuid::new_v4()
        ));

        // Spawn Cloud Hypervisor
        let mut ch_process =
            vm::CloudHypervisorProcess::spawn(&ch_bin, &socket_path).await?;
        let client = vm::CloudHypervisorClient::new(&socket_path);

        // Build VM configuration
        let mut builder = vm::VmConfigBuilder::new(
            config.vm.vcpus,
            config.vm.memory_mib,
            &kernel_path.to_string_lossy(),
            &rootfs_path.to_string_lossy(),
        )
        .vsock(&vsock_path.to_string_lossy(), 3)
        .serial_console();

        // Add GPU devices if configured
        for device in &gpu_devices {
            builder = builder.add_vfio_device(&device.path);
        }

        // Create and boot VM
        builder.apply(&client).await?;
        tracing::info!("VM booted — Ghostty should appear on the GPU display");

        // Wait for the VM process to exit (user closes Ghostty)
        let status = ch_process.wait().await?;
        tracing::info!(?status, "Cloud Hypervisor exited");
    }

    #[cfg(not(unix))]
    anyhow::bail!(
        "VM launch requires Linux with KVM support. \
         VirtualGhost runs on macOS/Windows for configuration only."
    );

    #[cfg(unix)]
    Ok(())
}

async fn cmd_config(show: bool) -> anyhow::Result<()> {
    if show {
        let config = VirtualGhostConfig::load()?;
        println!("{}", toml::to_string_pretty(&config)?);
    } else {
        println!(
            "Config file: {}",
            VirtualGhostConfig::config_path().display()
        );
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
