#!/usr/bin/env bash
set -euo pipefail

OUTPUT_DIR="assets/qemu"

# Find QEMU binary
QEMU_BIN=$(command -v qemu-system-x86_64 2>/dev/null || true)
if [ -z "$QEMU_BIN" ]; then
    echo "Error: qemu-system-x86_64 not found in PATH"
    echo "Install QEMU first: brew install qemu (macOS) or apt install qemu-system-x86 (Linux)"
    exit 1
fi

# Find QEMU share directory
QEMU_SHARE=""
for dir in /usr/share/qemu /usr/local/share/qemu /opt/homebrew/share/qemu; do
    if [ -d "$dir" ]; then
        QEMU_SHARE="$dir"
        break
    fi
done
if [ -z "$QEMU_SHARE" ]; then
    echo "Error: QEMU share directory not found"
    exit 1
fi

# Clean and create output directory
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR/share/keymaps" "$OUTPUT_DIR/share/firmware"

# Copy QEMU binary
cp "$QEMU_BIN" "$OUTPUT_DIR/"
echo "Copied qemu-system-x86_64"

# Copy shared libraries
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    ldd "$QEMU_BIN" | grep "=> /" | awk '{print $3}' | while read -r lib; do
        cp "$lib" "$OUTPUT_DIR/"
    done
    echo "Copied shared libraries (Linux)"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    otool -L "$QEMU_BIN" | tail -n +2 | awk '{print $1}' | grep -v /usr/lib | grep -v /System | while read -r lib; do
        [ -f "$lib" ] && cp "$lib" "$OUTPUT_DIR/"
    done
    echo "Copied shared libraries (macOS)"
fi

# Copy essential share files
SHARE_FILES=(
    bios.bin bios-256k.bin bios-microvm.bin
    kvmvapic.bin linuxboot.bin linuxboot_dma.bin
    multiboot.bin multiboot_dma.bin pvh.bin
    efi-virtio.rom efi-e1000.rom efi-e1000e.rom
    edk2-x86_64-code.fd edk2-x86_64-secure-code.fd
    edk2-i386-code.fd edk2-i386-secure-code.fd edk2-i386-vars.fd
    edk2-licenses.txt
    vgabios.bin vgabios-ati.bin vgabios-bochs-display.bin
    vgabios-cirrus.bin vgabios-qxl.bin vgabios-ramfb.bin
    vgabios-stdvga.bin vgabios-virtio.bin vgabios-vmware.bin
    pxe-virtio.rom pxe-e1000.rom
)
count=0
for f in "${SHARE_FILES[@]}"; do
    [ -f "$QEMU_SHARE/$f" ] && cp "$QEMU_SHARE/$f" "$OUTPUT_DIR/share/" && ((count++))
done
echo "Copied $count share files"

# Copy keymaps
cp "$QEMU_SHARE/keymaps/"* "$OUTPUT_DIR/share/keymaps/" 2>/dev/null || true
echo "Copied keymaps"

# Copy firmware descriptors (x86_64 only)
cp "$QEMU_SHARE/firmware/"*x86_64* "$OUTPUT_DIR/share/firmware/" 2>/dev/null || true
cp "$QEMU_SHARE/firmware/"*i386* "$OUTPUT_DIR/share/firmware/" 2>/dev/null || true
echo "Copied firmware descriptors"

chmod +x "$OUTPUT_DIR/qemu-system-x86_64"

echo ""
echo "QEMU bundle packaged: $(du -sh "$OUTPUT_DIR" | cut -f1) in $OUTPUT_DIR"
