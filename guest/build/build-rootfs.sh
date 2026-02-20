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
    linux \
    systemd \
    mesa \
    cage \
    ghostty \
    dbus \
    seatd \
    openssh

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
ln -sf /usr/lib/systemd/system/seatd.service \
    "$ROOTFS_DIR/etc/systemd/system/multi-user.target.wants/seatd.service"

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

# Mask vconsole-setup (we stripped /usr/share/kbd for size; not needed in a kiosk VM)
ln -sf /dev/null "$ROOTFS_DIR/etc/systemd/system/systemd-vconsole-setup.service"

# Create ghostty user for the kiosk session
chroot "$ROOTFS_DIR" useradd -m -s /bin/bash -G video,input ghostty 2>/dev/null || true

# -------------------------------------------------------
# Step 5: Aggressive size optimizations
# -------------------------------------------------------
echo "--- Optimizing rootfs size ---"
echo "Before optimization: $(du -sh "$ROOTFS_DIR" | cut -f1)"

# --- Package cache ---
rm -rf "$ROOTFS_DIR/var/cache/pacman/pkg"/*
rm -rf "$ROOTFS_DIR/var/lib/pacman/sync"

# --- Documentation and locale ---
rm -rf "$ROOTFS_DIR/usr/share/doc"
rm -rf "$ROOTFS_DIR/usr/share/man"
rm -rf "$ROOTFS_DIR/usr/share/info"
rm -rf "$ROOTFS_DIR/usr/share/locale"
rm -rf "$ROOTFS_DIR/usr/share/i18n"
rm -rf "$ROOTFS_DIR/usr/share/licenses"
rm -rf "$ROOTFS_DIR/usr/share/zsh"
rm -rf "$ROOTFS_DIR/usr/share/bash-completion"

# --- Firmware (VM uses virtio, no physical hardware) ---
rm -rf "$ROOTFS_DIR/usr/lib/firmware"

# --- Kernel build/source dirs (only needed for compiling modules, not loading them) ---
rm -rf "$ROOTFS_DIR/usr/lib/modules"/*/build
rm -rf "$ROOTFS_DIR/usr/lib/modules"/*/extramodules

# --- LLVM ---
# NOTE: Cannot remove libLLVM — Mesa's libEGL links against it at load time.
# Removing it breaks EGL platform support (EGL_EXT_platform_base), which
# prevents wlroots/Cage from initializing. ~150 MB cost but required.

# --- Vulkan (virgl only supports OpenGL, not Vulkan) ---
# Keep libvulkan.so — Cage/wlroots links against it at load time even when using GLES2 renderer
rm -rf "$ROOTFS_DIR/usr/share/vulkan"
rm -rf "$ROOTFS_DIR/usr/lib/libVk"*
rm -rf "$ROOTFS_DIR/usr/lib/libSPIRV"*
rm -rf "$ROOTFS_DIR/usr/lib/libspirv-cross"*

# --- Unused mesa DRI drivers (keep only virtio_gpu, swrast, kms_swrast, libdril, zink) ---
# libdril_dri.so is the real binary; all others are symlinks to it.
# Delete unused symlinks and any extra regular files.
find "$ROOTFS_DIR/usr/lib/dri/" \( -type f -o -type l \) \
    ! -name 'virtio_gpu_dri.so' \
    ! -name 'virtio_gpu_drv_video.so' \
    ! -name 'swrast_dri.so' \
    ! -name 'kms_swrast_dri.so' \
    ! -name 'libdril_dri.so' \
    ! -name 'zink_dri.so' \
    -delete 2>/dev/null || true

# --- Unused kernel modules (keep virtio, drm, gpu + their dependencies) ---
# drm_kms_helper depends on: fb_sys_fops, sysimgblt, sysfillrect, syscopyarea, i2c
if [ -d "$ROOTFS_DIR/usr/lib/modules" ]; then
    find "$ROOTFS_DIR/usr/lib/modules" -type f -name '*.ko*' \
        ! -path '*/virtio*' \
        ! -path '*/drm*' \
        ! -path '*/gpu*' \
        ! -path '*/i2c*' \
        ! -path '*/video*' \
        ! -name 'fb.ko*' \
        ! -name 'fb_sys_fops.ko*' \
        ! -name 'sysimgblt.ko*' \
        ! -name 'sysfillrect.ko*' \
        ! -name 'syscopyarea.ko*' \
        -delete 2>/dev/null || true
    # Remove empty directories left after module deletion
    find "$ROOTFS_DIR/usr/lib/modules" -type d -empty -delete 2>/dev/null || true
    # Rebuild module dependencies so modprobe works
    KVER=$(ls "$ROOTFS_DIR/usr/lib/modules/" | head -1)
    if [ -n "$KVER" ]; then
        chroot "$ROOTFS_DIR" depmod -a "$KVER" 2>/dev/null || true
    fi
    # Verify critical modules survived cleanup
    if ! find "$ROOTFS_DIR/usr/lib/modules" -name 'virtio-gpu*' | grep -q .; then
        echo "ERROR: virtio-gpu module missing after cleanup!" >&2
        exit 1
    fi
fi

# --- Dev-only files (headers, pkgconfig, static libs, GObject introspection) ---
rm -rf "$ROOTFS_DIR/usr/include"
rm -rf "$ROOTFS_DIR/usr/lib/pkgconfig"
rm -rf "$ROOTFS_DIR/usr/lib/"*.a
rm -rf "$ROOTFS_DIR/usr/share/gir-1.0"
rm -rf "$ROOTFS_DIR/usr/lib/girepository-1.0"

# --- Debug/profiling libraries (not needed at runtime) ---
rm -f "$ROOTFS_DIR/usr/lib/libasan"*.so*
rm -f "$ROOTFS_DIR/usr/lib/liblsan"*.so*
rm -f "$ROOTFS_DIR/usr/lib/libtsan"*.so*
rm -f "$ROOTFS_DIR/usr/lib/libubsan"*.so*
rm -f "$ROOTFS_DIR/usr/lib/libgfortran"*.so*
rm -f "$ROOTFS_DIR/usr/lib/libgprofng"*.so*
rm -f "$ROOTFS_DIR/usr/lib/libobjc"*.so*

# --- Build tools (not needed at runtime) ---
rm -f "$ROOTFS_DIR/usr/bin/xgettext"
rm -f "$ROOTFS_DIR/usr/bin/msgfmt"
rm -f "$ROOTFS_DIR/usr/bin/msgmerge"
rm -f "$ROOTFS_DIR/usr/bin/msginit"
rm -f "$ROOTFS_DIR/usr/bin/msgcat"
rm -f "$ROOTFS_DIR/usr/bin/msgconv"
rm -f "$ROOTFS_DIR/usr/bin/msgfilter"
rm -f "$ROOTFS_DIR/usr/bin/msggrep"
rm -f "$ROOTFS_DIR/usr/bin/msgunfmt"
rm -f "$ROOTFS_DIR/usr/bin/msguniq"
rm -f "$ROOTFS_DIR/usr/bin/msgattrib"
rm -f "$ROOTFS_DIR/usr/bin/msgcmp"
rm -f "$ROOTFS_DIR/usr/bin/msgcomm"
rm -f "$ROOTFS_DIR/usr/bin/msgexec"
rm -f "$ROOTFS_DIR/usr/bin/recode-sr-latin"
rm -f "$ROOTFS_DIR/usr/bin/gtk4-encode-symbolic-svg"
rm -f "$ROOTFS_DIR/usr/bin/gtk4-builder-tool"
rm -f "$ROOTFS_DIR/usr/bin/rsvg-convert"
rm -f "$ROOTFS_DIR/usr/bin/glycin-thumbnailer"
rm -f "$ROOTFS_DIR/usr/bin/sqlite3"
rm -f "$ROOTFS_DIR/usr/bin/ld" "$ROOTFS_DIR/usr/bin/ld.bfd" "$ROOTFS_DIR/usr/bin/ld.gold"
rm -f "$ROOTFS_DIR/usr/bin/as" "$ROOTFS_DIR/usr/bin/ar" "$ROOTFS_DIR/usr/bin/ranlib"
rm -f "$ROOTFS_DIR/usr/bin/nm" "$ROOTFS_DIR/usr/bin/objcopy" "$ROOTFS_DIR/usr/bin/objdump"
rm -f "$ROOTFS_DIR/usr/bin/strip" "$ROOTFS_DIR/usr/bin/readelf" "$ROOTFS_DIR/usr/bin/strings"
rm -f "$ROOTFS_DIR/usr/bin/size" "$ROOTFS_DIR/usr/bin/addr2line" "$ROOTFS_DIR/usr/bin/c++filt"
rm -f "$ROOTFS_DIR/usr/bin/elfedit" "$ROOTFS_DIR/usr/bin/gprof"
rm -rf "$ROOTFS_DIR/usr/lib/ldscripts"

# --- Multimedia frameworks (not needed for terminal emulator) ---
# Keep libgst*.so — GTK4 has GStreamer in its ELF NEEDED entries; removing breaks Ghostty load
# Keep libglycin*.so — librsvg/GTK4 links against it at load time
rm -rf "$ROOTFS_DIR/usr/lib/gstreamer-1.0"
rm -rf "$ROOTFS_DIR/usr/lib/glycin-loaders"

# --- Large data files not needed in VM ---
rm -rf "$ROOTFS_DIR/usr/share/hwdata"
rm -rf "$ROOTFS_DIR/usr/share/file"
rm -rf "$ROOTFS_DIR/usr/share/libwacom"
rm -rf "$ROOTFS_DIR/usr/share/kbd"
rm -rf "$ROOTFS_DIR/usr/share/mime"
rm -rf "$ROOTFS_DIR/usr/share/model"
rm -rf "$ROOTFS_DIR/usr/share/ghostty/terminfo"
rm -rf "$ROOTFS_DIR/usr/share/terminfo"/[!x]*
rm -rf "$ROOTFS_DIR/usr/share/appstream"

# --- ICU (keep — Ghostty needs libicuuc via tinysparql/appstream) ---
# Only remove libicuio (I/O stream utils, not needed at runtime)
rm -f "$ROOTFS_DIR/usr/lib/libicuio"*.so* 2>/dev/null || true
rm -f "$ROOTFS_DIR/usr/lib/libicutest"*.so* 2>/dev/null || true
rm -f "$ROOTFS_DIR/usr/lib/libicutu"*.so* 2>/dev/null || true

# --- gconv modules (glibc charset converters — keep only UTF-8 related) ---
if [ -d "$ROOTFS_DIR/usr/lib/gconv" ]; then
    find "$ROOTFS_DIR/usr/lib/gconv/" -type f -name '*.so' \
        ! -name 'UTF*' ! -name 'UNICODE*' ! -name 'ISO8859*' \
        -delete 2>/dev/null || true
fi

# --- Trim icon themes (keep only essential Adwaita icons) ---
rm -rf "$ROOTFS_DIR/usr/share/icons/hicolor"
find "$ROOTFS_DIR/usr/share/icons/" -name '*.svg' -delete 2>/dev/null || true

# --- Timezone data (keep only UTC and basic POSIX) ---
if [ -d "$ROOTFS_DIR/usr/share/zoneinfo" ]; then
    find "$ROOTFS_DIR/usr/share/zoneinfo" -type f \
        ! -name 'UTC' ! -name 'UCT' ! -name 'GMT' \
        ! -path '*/posix/*' ! -path '*/Etc/*' \
        -delete 2>/dev/null || true
    rm -rf "$ROOTFS_DIR/usr/share/zoneinfo-posix"
fi

# --- Trim udev (remove hardware database, keep essential rules) ---
rm -rf "$ROOTFS_DIR/usr/lib/udev/hwdb.d"
rm -f "$ROOTFS_DIR/usr/lib/udev/hwdb.bin"

# --- Remove ldconfig and systemd caches ---
rm -f "$ROOTFS_DIR/etc/ld.so.cache"
rm -rf "$ROOTFS_DIR/usr/lib/systemd/catalog"

# --- Strip ELF binaries ---
find "$ROOTFS_DIR/usr/bin" "$ROOTFS_DIR/usr/lib" -type f \
    \( -name '*.so*' -o -executable \) \
    -exec strip --strip-unneeded {} \; 2>/dev/null || true

echo "After optimization: $(du -sh "$ROOTFS_DIR" | cut -f1)"

# -------------------------------------------------------
# Step 6: Create ext4 image
# -------------------------------------------------------
echo "--- Creating ext4 image ---"

# Calculate required size (usage + 40% headroom for ext4 metadata/journal/inodes)
# resize2fs -M at the end will shrink back to minimum
USAGE_KB=$(du -sk "$ROOTFS_DIR" | cut -f1)
REQUIRED_KB=$(( USAGE_KB * 140 / 100 ))

# Round up to nearest 4 MB boundary
REQUIRED_KB=$(( (REQUIRED_KB + 4095) / 4096 * 4096 ))
echo "Image size: $(( REQUIRED_KB / 1024 )) MB"

dd if=/dev/zero of="$OUTPUT_PATH" bs=1024 count="$REQUIRED_KB" status=progress
# Use 1024-byte inodes, disable reserved blocks (not needed for VM rootfs)
mkfs.ext4 -F -L virtualghost -b 4096 -I 256 -m 0 "$OUTPUT_PATH"
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
