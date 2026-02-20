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

`build.rs` detects the assets, compresses them with zstd, and embeds them via `include_bytes!()`. At runtime, the binary stream-decompresses them to a platform-specific cache directory on first launch.

- Kernel: ~16 MB raw → ~16 MB compressed (zstd level 22)
- Rootfs: ~570 MB raw → ~111 MB compressed (zstd level 19, streaming)
- QEMU: ~142 MB tar → ~40 MB compressed (zstd level 22)

The rootfs is aggressively stripped during build: LLVM, ICU, firmware, Vulkan/SPIRV, unused mesa DRI drivers, unused kernel modules, build tools, debug/profiling libraries, GStreamer, docs, man pages, and static libraries are all removed since the VM only needs virtio drivers and virgl uses host-side shader compilation.
