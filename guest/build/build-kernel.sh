#!/bin/bash
set -euo pipefail

# Extract the Arch Linux stock kernel (bzImage) for Cloud Hypervisor.
# Cloud Hypervisor on x86_64 can boot bzImage directly.

OUTPUT_PATH="${1:?Usage: build-kernel.sh <output-path>}"

WORKDIR=$(mktemp -d)
trap "rm -rf $WORKDIR" EXIT

echo "Downloading Arch Linux kernel package..."
pacman -Sw --noconfirm --cachedir "$WORKDIR" linux

# Extract the bzImage from the package
KERNEL_PKG=$(ls "$WORKDIR"/linux-*.pkg.tar.zst | head -1)
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
KERNEL_SIZE=$(stat -c%s "$OUTPUT_PATH" 2>/dev/null || stat -f%z "$OUTPUT_PATH")
echo "Kernel extracted: $(ls -lh "$OUTPUT_PATH") ($KERNEL_SIZE bytes)"
