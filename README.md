# VirtualGhost

Isolated terminal sessions in Firecracker microVMs. A single binary that boots a lightweight VM, connects over vsock+SSH, and presents **Ghostly Term** — a terminal UI powered by ratatui.

## Platform Support

| Platform | Host binary | VM launch | Guest agent |
|----------|------------|-----------|-------------|
| Linux    | Yes        | Yes (requires KVM) | Yes (build target) |
| macOS    | Yes        | No        | No          |
| Windows  | Yes        | No        | No          |

The terminal UI and configuration work on all platforms. VM launch requires Linux with KVM and the [Firecracker binary](https://github.com/firecracker-microvm/firecracker/blob/main/docs/getting-started.md). On non-Linux platforms, the app runs in standalone terminal mode.

## Requirements

- Rust 1.75+

## Build

```bash
cargo build --release
```

## Usage

```bash
# Launch a VM with Ghostly Term
virtualghost run --kernel /path/to/vmlinux --rootfs /path/to/rootfs.ext4

# Configure VM resources
virtualghost run --vcpus 2 --memory 256

# Show configuration
virtualghost config --show

# Clean cached assets
virtualghost clean
```

## Architecture

The host binary manages the full lifecycle: spawn Firecracker, configure the VM via its REST API over a Unix socket, boot, establish a vsock tunnel, connect via SSH, and run the terminal UI.

The guest agent (`ghostly-agent`) runs inside the VM as an SSH server, allocating PTYs for shell sessions.

## License

This project is licensed under the [MIT License](LICENSE).
