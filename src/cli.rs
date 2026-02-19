use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "virtualghost", about = "Isolated terminal sessions in Firecracker microVMs")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Number of vCPUs for the VM
    #[arg(long, default_value_t = 1, global = true)]
    pub vcpus: u32,

    /// Memory in MiB for the VM
    #[arg(long, default_value_t = 128, global = true)]
    pub memory: u32,

    /// Path to custom kernel image
    #[arg(long, global = true)]
    pub kernel: Option<PathBuf>,

    /// Path to custom rootfs image
    #[arg(long, global = true)]
    pub rootfs: Option<PathBuf>,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Launch a VM and open Ghostly Term (default)
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
