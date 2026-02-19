# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

VirtualGhost is a single-binary launcher that runs Ghostty (GPU-accelerated terminal emulator) inside an isolated Cloud Hypervisor VM with VFIO GPU passthrough. It is a headless launcher — no host-side TUI.

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

## Architecture

```
Host Binary (virtualghost) — headless launcher
├── cli.rs           — clap CLI (run/config/clean, --gpu flag)
├── config/          — TOML config, settings structs
├── vm/
│   ├── api.rs       — Cloud Hypervisor REST client (vm.create + vm.boot over Unix socket)
│   ├── models.rs    — CH API types (VmCreateConfig, CpusConfig, PayloadConfig, DeviceConfig, etc.)
│   ├── config.rs    — VmConfigBuilder (single VmCreateConfig → vm.create → vm.boot)
│   ├── process.rs   — Spawn/manage cloud-hypervisor process
│   └── assets.rs    — Kernel/rootfs extraction
├── vfio/
│   ├── mod.rs       — GpuDevice struct, to_device_configs()
│   └── setup.rs     — discover_gpu(), unbind_driver(), bind_vfio(), validate_host()
├── network/         — Vsock host-side connection and byte-stream tunnel (unix-only)
├── ssh/             — russh SSH client, session/PTY management, ephemeral key gen
└── error.rs         — Unified error types with thiserror

Guest Binary (ghostly-agent)
└── server.rs        — Vsock listener + russh SSH server + PTY manager (linux-only)
```

**Flow:** Launch cloud-hypervisor → vm.create (with VFIO GPU devices) → vm.boot → Ghostty renders on GPU via Cage compositor inside VM → wait for exit → cleanup

## Key Design Decisions

- **Cloud Hypervisor over Firecracker**: Firecracker has no PCI/VFIO support. CH supports GPU passthrough.
- **Single API call**: CH uses `PUT /api/v1/vm.create` (full config blob) + `PUT /api/v1/vm.boot`, unlike Firecracker's 4-5 sequential PUT calls.
- **VFIO module**: Handles sysfs GPU discovery, IOMMU group enumeration, driver unbind/rebind to vfio-pci.
- **Vsock unchanged**: CH uses the same `CONNECT <port>\n` / `OK <id>\n` handshake as Firecracker.
- **No TUI**: Ghostty renders natively via GPU passthrough. The host binary is headless.
- **Unix-gated code**: vm/api, vm/process, vm/config, network/, vfio/setup are `#[cfg(unix)]`

## Conventions

- Error types in `src/error.rs` using thiserror; each module has its own error enum
- CH API types in `src/vm/models.rs` mirror the OpenAPI spec (serde Serialize/Deserialize)
- All async code uses tokio; SSH uses russh 0.48 with `ssh-key` crate types
- Release profile: LTO + strip + codegen-units=1 + opt-level="z"
