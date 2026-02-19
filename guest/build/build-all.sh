#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUTPUT_DIR="/output"

echo "=== VirtualGhost Guest Asset Builder ==="
echo "Output directory: $OUTPUT_DIR"

# Step 1: Build the guest agent (musl static binary)
echo ""
echo "--- Building ghostly-agent ---"
cd /build
cargo build \
    --manifest-path guest/ghostly-agent/Cargo.toml \
    --target x86_64-unknown-linux-musl \
    --release
AGENT_BIN="/build/guest/ghostly-agent/target/x86_64-unknown-linux-musl/release/ghostly-agent"
echo "Agent binary: $(ls -lh "$AGENT_BIN")"

# Step 2: Extract kernel
echo ""
echo "--- Extracting kernel ---"
"$SCRIPT_DIR/build-kernel.sh" "$OUTPUT_DIR/vmlinux"

# Step 3: Build rootfs
echo ""
echo "--- Building rootfs ---"
"$SCRIPT_DIR/build-rootfs.sh" "$OUTPUT_DIR/rootfs.ext4" "$AGENT_BIN"

echo ""
echo "=== Build complete ==="
ls -lh "$OUTPUT_DIR/vmlinux" "$OUTPUT_DIR/rootfs.ext4"
