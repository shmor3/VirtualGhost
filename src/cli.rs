use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "virtualghost",
    about = "Launch Ghostty in an isolated Cloud Hypervisor VM with GPU passthrough"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Number of vCPUs for the VM
    #[arg(long, default_value_t = 2, global = true)]
    pub vcpus: u32,

    /// Memory in MiB for the VM
    #[arg(long, default_value_t = 2048, global = true)]
    pub memory: u32,

    /// Path to custom kernel image
    #[arg(long, global = true)]
    pub kernel: Option<PathBuf>,

    /// Path to custom rootfs image
    #[arg(long, global = true)]
    pub rootfs: Option<PathBuf>,

    /// PCI address of GPU for VFIO passthrough (e.g., 0000:01:00.0)
    #[arg(long, global = true)]
    pub gpu: Option<String>,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Launch a VM with Ghostty (default)
    Run,

    /// Show or edit configuration
    Config {
        /// Show the current configuration
        #[arg(long)]
        show: bool,
    },

    /// Clean cached assets
    Clean,
}

impl Cli {
    pub fn effective_command(&self) -> &Command {
        static DEFAULT: Command = Command::Run;
        self.command.as_ref().unwrap_or(&DEFAULT)
    }
}
