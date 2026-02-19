# Assets

This directory holds the VM kernel and rootfs images. These are compressed and embedded in the binary at build time by `build.rs`.

## Required Files

- `vmlinux` — Linux kernel (bzImage from Arch Linux)
- `rootfs.ext4` — Arch Linux ext4 image with Ghostty, Cage, Mesa, and guest agent

Both files are gitignored (large binaries).

## Building Assets

From the project root:

```bash
make assets
```

This builds a Docker container with Arch Linux, cross-compiles the guest agent, extracts the kernel, and creates the rootfs via `pacstrap`. Requires Docker with `--privileged` support.

## Then Build the Binary

```bash
cargo build --release
```

`build.rs` detects the assets, compresses them with zstd, and embeds them via `include_bytes!()`.
