#!/bin/bash
set -euo pipefail

# Extract the Arch Linux stock kernel (bzImage) for Cloud Hypervisor.
# Cloud Hypervisor on x86_64 can boot bzImage directly.

OUTPUT_PATH="${1:?Usage: build-kernel.sh <output-path>}"

WORKDIR=$(mktemp -d /tmp/kernel-build.XXXXXX)
trap "rm -rf $WORKDIR" EXIT

echo "Downloading Arch Linux kernel package..."
# Download to system cache (has proper permissions), then extract from there
pacman -Sw --noconfirm linux

# Find the downloaded package in the system cache
KERNEL_PKG=$(find /var/cache/pacman/pkg -name 'linux-[0-9]*.pkg.tar.zst' -type f | sort -V | tail -1)
if [[ -z "$KERNEL_PKG" ]]; then
    echo "ERROR: kernel package not found in pacman cache" >&2
    exit 1
fi

echo "Extracting kernel from: $(basename "$KERNEL_PKG")"
mkdir -p "$WORKDIR/extract"
bsdtar -xf "$KERNEL_PKG" -C "$WORKDIR/extract" 'usr/lib/modules/*/vmlinuz'

# Find the extracted vmlinuz
VMLINUZ=$(find "$WORKDIR/extract/usr/lib/modules" -name vmlinuz -type f | head -1)
if [[ -z "$VMLINUZ" ]]; then
    echo "ERROR: Could not find vmlinuz in kernel package" >&2
    exit 1
fi

cp "$VMLINUZ" "$OUTPUT_PATH"
echo "Kernel extracted: $(ls -lh "$OUTPUT_PATH")"
