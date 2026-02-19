# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

VirtualGhost is a single-binary launcher that runs Ghostty (GPU-accelerated terminal emulator) inside an isolated QEMU VM. It supports VFIO GPU passthrough on Linux and virgl GPU emulation on all platforms (Windows, Linux, macOS). The binary embeds the kernel, rootfs, and QEMU — no external dependencies needed.

## Build Commands

```bash
cargo check                                                    # Type-check host binary
cargo build                                                    # Build host binary (debug)
cargo build --release                                          # Build host binary (release, optimized for size)
cargo check --manifest-path guest/ghostly-agent/Cargo.toml     # Type-check guest agent
```

Cross-compile guest agent for the VM:
```bash
cargo build --manifest-path guest/ghostly-agent/Cargo.toml --target x86_64-unknown-linux-musl --release
```

Build VM assets (kernel + rootfs) via Docker:
```bash
make docker-build    # Build the Docker builder image
make assets          # Build kernel + rootfs into assets/
make package-qemu    # Package host QEMU into assets/qemu/ for embedding
```

## Architecture

```
Host Binary (virtualghost) — launches QEMU VM
├── cli.rs           — clap CLI (run/config/clean, --gpu/--vcpus/--memory flags)
├── config/          — TOML config, settings structs
├── vm/
│   ├── config.rs    — QemuConfig: builds QEMU command-line args (accel, display, virtio-gpu, QMP)
│   ├── process.rs   — QemuProcess: spawn/manage QEMU process with platform-specific env setup
│   ├── assets.rs    — AssetManager: extract embedded kernel/rootfs/QEMU from zstd to cache
│   └── models.rs    — Shared VM types
├── vfio/            — (Linux only) VFIO GPU passthrough: sysfs discovery, driver unbind/rebind
├── network/         — (Unix only) Vsock host-side connection and byte-stream tunnel
├── ssh/             — russh SSH client, session/PTY management, ephemeral key gen
└── error.rs         — Unified error types with thiserror

Guest Binary (ghostly-agent)
└── server.rs        — Vsock listener + russh SSH server + PTY manager (linux-only)

Guest VM (Arch Linux rootfs)
├── seatd            — Wayland seat manager (DRM device access)
├── cage             — Wayland kiosk compositor (runs Ghostty as sole app)
├── ghostty          — GPU-accelerated terminal emulator
├── mesa (virgl)     — OpenGL via virtio-gpu-gl for emulated GPU rendering
└── ghostly-agent    — SSH server for host-guest management
```

**Flow:** Spawn QEMU (with virtio-gpu-gl or VFIO GPU) → boot kernel → systemd starts seatd → Cage compositor → Ghostty renders via virgl/GPU → user closes Ghostty → Cage exits → VM poweroff → cleanup

## Key Design Decisions

- **QEMU over Cloud Hypervisor**: CH lacks cross-platform support. QEMU runs on Windows, Linux, and macOS with virgl GPU emulation everywhere.
- **Embedded assets**: Kernel, rootfs, and QEMU binary are zstd-compressed and embedded via `include_bytes!()`. Extracted to a platform-specific cache dir on first run.
- **virgl GPU emulation**: Uses `virtio-gpu-gl-pci` with `-display sdl,gl=on` for OpenGL passthrough to the guest via mesa's virgl driver. No physical GPU required.
- **VFIO passthrough**: Still supported on Linux for native GPU performance (NVIDIA, AMD). Handles IOMMU group discovery and driver unbind/rebind.
- **seatd**: Lightweight seat manager required by Cage/wlroots to access DRM devices. Runs as a systemd service before ghostty-session.
- **Platform-specific QMP**: Unix uses domain sockets; Windows dynamically allocates a free TCP port.
- **No TUI**: Ghostty renders in the QEMU SDL window (emulated) or on the physical GPU (passthrough). The host binary is headless.

## Platform-Specific Code

- `vfio/` — `#[cfg(unix)]` — VFIO passthrough requires Linux KVM + IOMMU
- `network/vsock` — `#[cfg(unix)]` — Vsock requires Linux KVM
- QMP socket — `cfg!(unix)` for domain sockets, TCP on Windows
- Process env — `#[cfg(target_os = "...")]` for PATH/LD_LIBRARY_PATH/DYLD_LIBRARY_PATH
- Accelerator — KVM (Linux), HVF (macOS), TCG software emulation (Windows/fallback)
- Display — GTK (Linux), Cocoa (macOS), SDL (Windows/TCG)

## Conventions

- Error types in `src/error.rs` using thiserror; each module has its own error enum
- All async code uses tokio; SSH uses russh 0.48 with `ssh-key` crate types
- Release profile: LTO + strip + codegen-units=1 + opt-level="z"
- Assets cached in platform-specific dirs via the `directories` crate
