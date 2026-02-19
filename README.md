# VirtualGhost

Launch [Ghostty](https://ghostty.org) in an isolated [QEMU](https://www.qemu.org) VM. A single binary that embeds the kernel, rootfs, and QEMU — boots a lightweight Arch Linux VM and runs Ghostty with GPU-accelerated rendering via virgl, or native GPU passthrough via VFIO on Linux.

## Platform Support

| Platform | Host binary | GPU rendering | Accelerator |
|----------|------------|---------------|-------------|
| Linux    | Yes        | virgl (emulated) or VFIO (passthrough) | KVM |
| macOS    | Yes        | virgl (emulated) | HVF |
| Windows  | Yes        | virgl (emulated) | TCG (software) |

## Requirements

- Rust 1.75+
- Docker (to build VM assets)
- No external QEMU install needed — the binary is embedded

### Optional: GPU Passthrough (Linux only)

```bash
# Enable IOMMU (add to kernel boot parameters)
# Intel: intel_iommu=on iommu=pt
# AMD:   amd_iommu=on iommu=pt

# Load VFIO modules
sudo modprobe vfio_pci vfio_iommu_type1
```

## Build

```bash
# Build VM assets (kernel + rootfs) via Docker
make docker-build
make assets

# Package QEMU for embedding
make package-qemu

# Build the release binary (embeds all assets)
cargo build --release
```

The resulting binary at `target/release/virtualghost` is fully self-contained.

## Usage

```bash
# Launch Ghostty in a VM (default: 2 vCPUs, 2 GB RAM)
virtualghost

# Configure VM resources
virtualghost run --vcpus 4 --memory 4096

# GPU passthrough (Linux with IOMMU)
virtualghost run --gpu 0000:01:00.0

# Custom kernel/rootfs
virtualghost run --kernel /path/to/vmlinux --rootfs /path/to/rootfs.ext4

# Show configuration
virtualghost config --show

# Clean cached assets
virtualghost clean
```

## How It Works

1. Extracts embedded kernel, rootfs, and QEMU to a local cache (first run only)
2. Spawns QEMU with virtio-gpu-gl (virgl 3D) or VFIO GPU passthrough
3. Boots Arch Linux with systemd, seatd (seat manager), and Cage (Wayland kiosk compositor)
4. Cage launches Ghostty as its sole application with GPU-accelerated rendering
5. When Ghostty exits, the VM powers off automatically

### Guest VM Stack

```
Ghostty (terminal emulator)
  └── Cage (Wayland kiosk compositor)
      └── seatd (DRM seat manager)
          └── mesa virgl (OpenGL via virtio-gpu)
              └── Linux kernel (virtio drivers)
```

The guest agent (`ghostly-agent`) runs inside the VM as an SSH server for host-guest management via vsock (Linux) or TCP port forwarding (macOS/Windows).

## License

This project is licensed under the [MIT License](LICENSE).
