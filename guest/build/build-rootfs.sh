#!/bin/bash
set -euo pipefail

OUTPUT_PATH="${1:?Usage: build-rootfs.sh <output-path> <agent-binary>}"
AGENT_BIN="${2:?Usage: build-rootfs.sh <output-path> <agent-binary>}"
SCRIPT_DIR="/opt/builder"

ROOTFS_DIR=$(mktemp -d)
MOUNT_DIR=$(mktemp -d)
trap 'umount -lf "$MOUNT_DIR" 2>/dev/null || true; rm -rf "$ROOTFS_DIR" "$MOUNT_DIR"' EXIT

echo "=== Building Arch Linux rootfs ==="

# -------------------------------------------------------
# Step 1: Install base packages with pacstrap
# -------------------------------------------------------
echo "--- pacstrap: installing base packages ---"
pacstrap -c -G -M "$ROOTFS_DIR" \
    base \
    systemd \
    linux-firmware \
    mesa \
    vulkan-icd-loader \
    vulkan-radeon \
    vulkan-intel \
    cage \
    ghostty \
    dbus

# -------------------------------------------------------
# Step 2: Install ghostly-agent binary
# -------------------------------------------------------
echo "--- Installing ghostly-agent ---"
install -Dm755 "$AGENT_BIN" "$ROOTFS_DIR/usr/local/bin/ghostly-agent"

# -------------------------------------------------------
# Step 3: Install systemd service files
# -------------------------------------------------------
echo "--- Installing systemd units ---"
install -Dm644 "$SCRIPT_DIR/rootfs/ghostty-session.service" \
    "$ROOTFS_DIR/etc/systemd/system/ghostty-session.service"
install -Dm644 "$SCRIPT_DIR/rootfs/ghostly-agent.service" \
    "$ROOTFS_DIR/etc/systemd/system/ghostly-agent.service"

# Enable services
mkdir -p "$ROOTFS_DIR/etc/systemd/system/multi-user.target.wants"
ln -sf /etc/systemd/system/ghostty-session.service \
    "$ROOTFS_DIR/etc/systemd/system/multi-user.target.wants/ghostty-session.service"
ln -sf /etc/systemd/system/ghostly-agent.service \
    "$ROOTFS_DIR/etc/systemd/system/multi-user.target.wants/ghostly-agent.service"

# -------------------------------------------------------
# Step 4: System configuration
# -------------------------------------------------------
echo "--- Configuring system ---"

# Hostname
echo "virtualghost" > "$ROOTFS_DIR/etc/hostname"

# Locale
echo "en_US.UTF-8 UTF-8" > "$ROOTFS_DIR/etc/locale.gen"
echo "LANG=en_US.UTF-8" > "$ROOTFS_DIR/etc/locale.conf"

# Root password: disable (agent handles access)
sed -i 's|^root:.*|root::0:0:root:/root:/bin/bash|' "$ROOTFS_DIR/etc/passwd"

# fstab: root on /dev/vda (raw ext4, no partition table)
cat > "$ROOTFS_DIR/etc/fstab" <<'FSTAB'
# VirtualGhost rootfs
/dev/vda    /    ext4    defaults,noatime    0 1
FSTAB

# Serial console getty (debug access via serial)
mkdir -p "$ROOTFS_DIR/etc/systemd/system/serial-getty@ttyS0.service.d"
cat > "$ROOTFS_DIR/etc/systemd/system/serial-getty@ttyS0.service.d/autologin.conf" <<'GETTY'
[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin root --noclear %I 115200 linux
GETTY

# Enable serial console
ln -sf /usr/lib/systemd/system/serial-getty@.service \
    "$ROOTFS_DIR/etc/systemd/system/getty.target.wants/serial-getty@ttyS0.service"

# Create ghostty user for the kiosk session
chroot "$ROOTFS_DIR" useradd -m -s /bin/bash -G video,input ghostty 2>/dev/null || true

# -------------------------------------------------------
# Step 5: Size optimizations
# -------------------------------------------------------
echo "--- Optimizing rootfs size ---"

# Remove package cache
rm -rf "$ROOTFS_DIR/var/cache/pacman/pkg"/*

# Remove docs, man pages, locale data
rm -rf "$ROOTFS_DIR/usr/share/doc"
rm -rf "$ROOTFS_DIR/usr/share/man"
rm -rf "$ROOTFS_DIR/usr/share/info"
rm -rf "$ROOTFS_DIR/usr/share/locale"

# Remove non-GPU firmware (WiFi, Bluetooth, etc.)
rm -rf "$ROOTFS_DIR/usr/lib/firmware/intel"
rm -rf "$ROOTFS_DIR/usr/lib/firmware/mediatek"
rm -rf "$ROOTFS_DIR/usr/lib/firmware/realtek"
rm -rf "$ROOTFS_DIR/usr/lib/firmware/iwlwifi"*
rm -rf "$ROOTFS_DIR/usr/lib/firmware/ath"*
rm -rf "$ROOTFS_DIR/usr/lib/firmware/brcm"
rm -rf "$ROOTFS_DIR/usr/lib/firmware/ti-connectivity"
rm -rf "$ROOTFS_DIR/usr/lib/firmware/qcom"
rm -rf "$ROOTFS_DIR/usr/lib/firmware/cypress"
rm -rf "$ROOTFS_DIR/usr/lib/firmware/mrvl"

# Remove ldconfig cache
rm -f "$ROOTFS_DIR/etc/ld.so.cache"

echo "Rootfs tree size: $(du -sh "$ROOTFS_DIR" | cut -f1)"

# -------------------------------------------------------
# Step 6: Create ext4 image
# -------------------------------------------------------
echo "--- Creating ext4 image ---"

# Calculate required size (usage + 20% headroom, minimum 512 MB)
USAGE_KB=$(du -sk "$ROOTFS_DIR" | cut -f1)
REQUIRED_KB=$(( USAGE_KB * 120 / 100 ))
if [[ $REQUIRED_KB -lt 524288 ]]; then
    REQUIRED_KB=524288
fi

# Round up to nearest 4 MB boundary
REQUIRED_KB=$(( (REQUIRED_KB + 4095) / 4096 * 4096 ))
echo "Image size: $(( REQUIRED_KB / 1024 )) MB"

dd if=/dev/zero of="$OUTPUT_PATH" bs=1024 count="$REQUIRED_KB" status=progress
mkfs.ext4 -F -L virtualghost -b 4096 "$OUTPUT_PATH"
mount -o loop "$OUTPUT_PATH" "$MOUNT_DIR"

# Copy rootfs content into the image
cp -a "$ROOTFS_DIR"/* "$MOUNT_DIR"/

# Finalize
sync
umount "$MOUNT_DIR"

# Shrink to minimum size
e2fsck -fy "$OUTPUT_PATH" || true
resize2fs -M "$OUTPUT_PATH"

echo "Final image: $(ls -lh "$OUTPUT_PATH")"
