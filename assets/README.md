# Assets

This directory holds the Firecracker kernel and rootfs images.

## Required Files

- `vmlinux` — Firecracker-compatible uncompressed Linux kernel
- `rootfs.ext4` — ext4 filesystem image containing the Ghostly Agent

## Obtaining a Kernel

Download a pre-built Firecracker kernel:

```bash
curl -fsSL -o vmlinux https://s3.amazonaws.com/spec.ccfc.min/firecracker-ci/v1.12/x86_64/vmlinux-6.1
```

## Building the Rootfs

See `guest/ghostly-agent/` for the guest-side agent. The rootfs must contain:

1. The `ghostly-agent` binary (compiled for x86_64-unknown-linux-musl)
2. An init system that starts the agent on boot
3. Basic shell utilities (busybox or similar)

Use `guest/rootfs-builder/build-rootfs.sh` (when available) to automate this.
