# VirtualGhost

Launch [Ghostty](https://ghostty.org) in an isolated [Cloud Hypervisor](https://www.cloudhypervisor.org) VM with GPU passthrough. A single binary that boots a lightweight VM, passes through your GPU via VFIO, and runs Ghostty with native GPU acceleration.

## Platform Support

| Platform | Host binary | VM + GPU passthrough |
|----------|------------|---------------------|
| Linux    | Yes        | Yes (requires KVM + IOMMU) |
| macOS    | Yes (config only) | No |
| Windows  | Yes (config only) | No |

## Requirements

- Rust 1.75+
- Linux with KVM and IOMMU (Intel VT-d / AMD-Vi) for VM launch
- Cloud Hypervisor binary ([releases](https://github.com/cloud-hypervisor/cloud-hypervisor/releases))
- GPU with VFIO support (NVIDIA, AMD)

### Host Setup

```bash
# Enable IOMMU (add to kernel boot parameters)
# Intel: intel_iommu=on iommu=pt
# AMD:   amd_iommu=on iommu=pt

# Load VFIO modules
sudo modprobe vfio_pci vfio_iommu_type1
```

## Build

```bash
cargo build --release
```

## Usage

```bash
# Launch Ghostty in a VM with GPU passthrough
virtualghost run --kernel /path/to/vmlinux --rootfs /path/to/rootfs.ext4 --gpu 0000:01:00.0

# Configure VM resources
virtualghost run --vcpus 4 --memory 4096 --gpu 0000:01:00.0

# Show configuration
virtualghost config --show

# Clean cached assets
virtualghost clean
```

## Architecture

VirtualGhost is a headless launcher:

1. Prepares VFIO GPU passthrough (unbinds GPU from host driver, binds to vfio-pci)
2. Spawns Cloud Hypervisor with the VM configuration
3. Boots the VM with the GPU passed through
4. Inside the VM: Cage (Wayland kiosk compositor) runs Ghostty with native GPU rendering
5. Waits for the VM to exit when Ghostty closes

The guest agent (`ghostly-agent`) runs inside the VM as an SSH server for management.

## License

This project is licensed under the [MIT License](LICENSE).
