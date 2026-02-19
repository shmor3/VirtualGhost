use std::path::PathBuf;

/// Hardware accelerator for QEMU.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Accelerator {
    Kvm,
    Hvf,
    Whpx,
    Tcg,
}

impl Accelerator {
    /// Detect the best available accelerator for the current platform.
    pub fn detect() -> Self {
        if cfg!(target_os = "linux") {
            if std::path::Path::new("/dev/kvm").exists() {
                return Self::Kvm;
            }
        }
        if cfg!(target_os = "macos") {
            return Self::Hvf;
        }
        if cfg!(target_os = "windows") {
            return Self::Whpx;
        }
        Self::Tcg
    }

    fn as_arg(&self) -> &str {
        match self {
            Self::Kvm => "kvm",
            Self::Hvf => "hvf",
            Self::Whpx => "whpx",
            Self::Tcg => "tcg",
        }
    }
}

/// Display backend for QEMU.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisplayMode {
    Sdl,
    Cocoa,
    Gtk,
    None,
}

impl DisplayMode {
    /// Detect the best display for the current platform.
    pub fn detect() -> Self {
        if cfg!(target_os = "macos") {
            Self::Cocoa
        } else if cfg!(target_os = "windows") {
            Self::Sdl
        } else {
            Self::Gtk
        }
    }

    fn as_arg(&self) -> &str {
        match self {
            Self::Sdl => "sdl",
            Self::Cocoa => "cocoa",
            Self::Gtk => "gtk",
            Self::None => "none",
        }
    }
}

/// QEMU VM configuration — builds command-line arguments.
pub struct QemuConfig {
    pub qemu_bin: PathBuf,
    pub vcpus: u32,
    pub memory_mib: u32,
    pub kernel_path: String,
    pub rootfs_path: String,
    pub cmdline: String,
    pub display: DisplayMode,
    pub accel: Accelerator,
    pub gpu_passthrough: Vec<String>,
    pub vsock_cid: Option<u64>,
    pub ssh_port_forward: Option<u16>,
    pub qmp_socket: PathBuf,
}

impl QemuConfig {
    pub fn new(
        qemu_bin: PathBuf,
        vcpus: u32,
        memory_mib: u32,
        kernel_path: &str,
        rootfs_path: &str,
    ) -> Self {
        let accel = Accelerator::detect();
        let display = if accel == Accelerator::Tcg {
            DisplayMode::Sdl
        } else {
            DisplayMode::detect()
        };

        Self {
            qemu_bin,
            vcpus,
            memory_mib,
            kernel_path: kernel_path.to_string(),
            rootfs_path: rootfs_path.to_string(),
            cmdline: "console=ttyS0 root=/dev/vda rw".to_string(),
            display,
            accel,
            gpu_passthrough: Vec::new(),
            vsock_cid: None,
            ssh_port_forward: None,
            qmp_socket: PathBuf::new(),
        }
    }

    /// Build QEMU command-line arguments.
    pub fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        // Accelerator
        args.extend(["-accel".into(), self.accel.as_arg().into()]);

        // CPU
        let cpu = if self.accel == Accelerator::Tcg {
            "max"
        } else {
            "host"
        };
        args.extend(["-cpu".into(), cpu.into()]);
        args.extend(["-smp".into(), self.vcpus.to_string()]);
        args.extend(["-m".into(), self.memory_mib.to_string()]);

        // Kernel + cmdline
        args.extend(["-kernel".into(), self.kernel_path.clone()]);
        args.extend(["-append".into(), self.cmdline.clone()]);

        // Rootfs disk
        args.extend([
            "-drive".into(),
            format!(
                "file={},format=raw,if=virtio",
                self.rootfs_path
            ),
        ]);

        // Display + input
        if !self.gpu_passthrough.is_empty() {
            // GPU passthrough: no virtual display needed, Cage uses the physical GPU
            args.extend(["-display".into(), "none".into()]);
            args.extend(["-vga".into(), "none".into()]);
        } else {
            args.extend(["-device".into(), "virtio-gpu-pci".into()]);
            args.extend(["-display".into(), self.display.as_arg().into()]);
            args.extend(["-device".into(), "virtio-keyboard-pci".into()]);
            args.extend(["-device".into(), "virtio-mouse-pci".into()]);
        }

        // Serial console
        args.extend(["-serial".into(), "stdio".into()]);

        // QMP socket for graceful shutdown
        let qmp_path = self.qmp_socket.display().to_string();
        if cfg!(unix) {
            args.extend([
                "-qmp".into(),
                format!("unix:{qmp_path},server,nowait"),
            ]);
        } else {
            // Windows: use TCP for QMP
            args.extend([
                "-qmp".into(),
                "tcp:127.0.0.1:4444,server,nowait".into(),
            ]);
        }

        // Vsock (Linux only — vhost-vsock-pci requires KVM)
        if let Some(cid) = self.vsock_cid {
            args.extend([
                "-device".into(),
                format!("vhost-vsock-pci,guest-cid={cid}"),
            ]);
        }

        // Network + SSH port forwarding (macOS/Windows)
        if let Some(port) = self.ssh_port_forward {
            args.extend([
                "-netdev".into(),
                format!("user,id=net0,hostfwd=tcp::{port}-:22"),
            ]);
            args.extend(["-device".into(), "virtio-net-pci,netdev=net0".into()]);
        }

        // VFIO GPU passthrough (Linux only)
        for pci_addr in &self.gpu_passthrough {
            args.extend(["-device".into(), format!("vfio-pci,host={pci_addr}")]);
        }

        // Misc
        args.push("-nodefaults".into());

        args
    }
}
