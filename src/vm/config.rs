use std::path::PathBuf;

/// Hardware accelerator for QEMU.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
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
        // WHPX on Windows is unreliable — use TCG (software emulation) for now
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
#[allow(dead_code)]
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
    pub qemu_data_dir: Option<PathBuf>,
    /// TCP port for QMP on Windows (dynamically allocated).
    pub qmp_tcp_port: Option<u16>,
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
        let display = DisplayMode::detect();

        Self {
            qemu_bin,
            vcpus,
            memory_mib,
            kernel_path: kernel_path.to_string(),
            rootfs_path: rootfs_path.to_string(),
            cmdline: "console=ttyS0 root=/dev/vda rw quiet".to_string(),
            display,
            accel,
            gpu_passthrough: Vec::new(),
            vsock_cid: None,
            ssh_port_forward: None,
            qmp_socket: PathBuf::new(),
            qemu_data_dir: None,
            qmp_tcp_port: None,
        }
    }

    /// Build QEMU command-line arguments.
    pub fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        // Data directory (BIOS, VGA BIOS, keymaps, etc.)
        if let Some(ref data_dir) = self.qemu_data_dir {
            args.extend(["-L".into(), data_dir.display().to_string()]);
        }

        // Accelerator
        args.extend(["-accel".into(), self.accel.as_arg().into()]);

        // CPU — only KVM and HVF safely support -cpu host
        let cpu = match self.accel {
            Accelerator::Kvm | Accelerator::Hvf => "host",
            _ => "max",
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
            if self.accel == Accelerator::Kvm || self.accel == Accelerator::Hvf {
                // virgl 3D: host GL context passed to guest via virtio-gpu
                args.extend(["-device".into(), "virtio-vga-gl".into()]);
                args.extend([
                    "-display".into(),
                    format!("{},gl=on", self.display.as_arg()),
                ]);
            } else {
                // TCG/software: no virgl support, basic VGA framebuffer.
                // Guest uses wlroots pixman renderer (software rendering).
                args.extend(["-device".into(), "virtio-vga".into()]);
                args.extend(["-display".into(), self.display.as_arg().into()]);
            }
            args.extend(["-device".into(), "virtio-keyboard-pci".into()]);
            args.extend(["-device".into(), "virtio-mouse-pci".into()]);
        }

        // Serial console
        args.extend(["-serial".into(), "stdio".into()]);

        // QMP socket for graceful shutdown
        if cfg!(unix) {
            let qmp_path = self.qmp_socket.display().to_string();
            args.extend([
                "-qmp".into(),
                format!("unix:{qmp_path},server,nowait"),
            ]);
        } else if let Some(port) = self.qmp_tcp_port {
            // Windows: use TCP with a dynamically allocated port
            args.extend([
                "-qmp".into(),
                format!("tcp:127.0.0.1:{port},server,nowait"),
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
