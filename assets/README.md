# Assets

This directory holds the VM kernel, rootfs, and QEMU files. These are compressed and embedded in the binary at build time by `build.rs`.

## Required Files

- `vmlinux` — Linux kernel (extracted from Arch Linux package)
- `rootfs.ext4` — Arch Linux ext4 image with Ghostty, Cage, seatd, Mesa, and guest agent
- `qemu/` — QEMU binary and support files (platform-specific)

All files are gitignored (large binaries).

## Building Assets

From the project root:

```bash
# Build kernel + rootfs via Docker (requires --privileged)
make docker-build
make assets

# Package QEMU from a local install into assets/qemu/
make package-qemu
```

The Docker build creates an Arch Linux container, cross-compiles the guest agent (musl static binary), extracts the kernel, and creates the rootfs via `pacstrap` with: base, systemd, mesa, cage, ghostty, seatd, and dbus.

## Embedding in the Binary

```bash
cargo build --release
```

`build.rs` detects the assets, compresses them with zstd (level 19), and embeds them via `include_bytes!()`. At runtime, the binary extracts them to a platform-specific cache directory on first launch.

- Kernel: ~16 MB raw, ~15 MB compressed
- Rootfs: ~1.4 GB raw, ~420 MB compressed
- QEMU: ~140 MB tar, ~42 MB compressed
