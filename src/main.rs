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

    // Extract embedded assets (kernel, rootfs, QEMU) if not already cached
    if let Err(e) = asset_manager.ensure_assets() {
        let need_kernel = config.vm.kernel_path.is_none() && !asset_manager.kernel_path().exists();
        let need_rootfs = config.vm.rootfs_path.is_none() && !asset_manager.rootfs_path().exists();
        let need_qemu = config.vm.qemu_bin.is_none() && !asset_manager.qemu_bin_path().exists();

        if need_kernel || need_rootfs {
            anyhow::bail!(
                "No kernel/rootfs available: {e}\n\
                 Provide --kernel and --rootfs paths, or place assets in cache."
            );
        }
        if need_qemu {
            anyhow::bail!(
                "No QEMU binary available: {e}\n\
                 Set qemu_bin in config, or place QEMU files in assets/qemu/ and rebuild."
            );
        }
        tracing::warn!("Non-critical asset extraction issue: {e}");
    }

    // GPU passthrough setup (Linux only)
    #[allow(unused_mut)]
    let mut gpu_pci_addresses: Vec<String> = Vec::new();
    #[cfg(unix)]
    if let Some(ref pci_addr) = config.vm.gpu_pci_address {
        tracing::info!(pci_addr, "Preparing GPU for VFIO passthrough");
        let gpu = vfio::discover_gpu(pci_addr)?;
        vfio::prepare_passthrough(&gpu)?;
        for device_config in gpu.to_device_configs() {
            gpu_pci_addresses.push(device_config.path);
        }
    }

    #[cfg(not(unix))]
    if config.vm.gpu_pci_address.is_some() {
        anyhow::bail!("GPU passthrough requires Linux with KVM and IOMMU support");
    }

    let accel = vm::Accelerator::detect();
    tracing::info!(
        kernel = %kernel_path.display(),
        rootfs = %rootfs_path.display(),
        vcpus = config.vm.vcpus,
        memory_mib = config.vm.memory_mib,
        accel = ?accel,
        gpu = ?config.vm.gpu_pci_address,
        "Starting VirtualGhost"
    );

    // Build QEMU configuration
    let qemu_bin = config
        .vm
        .qemu_bin
        .clone()
        .unwrap_or_else(|| asset_manager.qemu_bin_path());

    let qmp_socket = std::env::temp_dir().join(format!(
        "virtualghost-qmp-{}.sock",
        uuid::Uuid::new_v4()
    ));

    let mut qemu_config = vm::QemuConfig::new(
        qemu_bin,
        config.vm.vcpus,
        config.vm.memory_mib,
        &kernel_path.to_string_lossy(),
        &rootfs_path.to_string_lossy(),
    );
    // If using embedded QEMU, point it to the extracted share/ directory
    if config.vm.qemu_bin.is_none() {
        qemu_config.qemu_data_dir = Some(asset_manager.qemu_data_dir());
    }
    qemu_config.qmp_socket = qmp_socket;

    // On Windows, find a free TCP port for QMP
    #[cfg(not(unix))]
    {
        let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        drop(listener);
        qemu_config.qmp_tcp_port = Some(port);
    }
    qemu_config.gpu_passthrough = gpu_pci_addresses;

    // Use vsock on Linux (direct host-guest channel), TCP port forwarding elsewhere
    if cfg!(target_os = "linux") {
        qemu_config.vsock_cid = Some(3);
    } else {
        qemu_config.ssh_port_forward = Some(2222);
    }

    // GPU passthrough: no virtual display, Cage uses the physical GPU
    if !qemu_config.gpu_passthrough.is_empty() {
        qemu_config.display = vm::DisplayMode::None;
    }

    // Spawn QEMU
    let mut qemu_process = vm::QemuProcess::spawn(&qemu_config).await?;
    tracing::info!("QEMU running — Ghostty should appear shortly");

    // Wait for the VM process to exit (user closes Ghostty)
    let status = qemu_process.wait().await?;
    tracing::info!(?status, "QEMU exited");

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
