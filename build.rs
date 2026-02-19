use std::fs;
use std::io::{self, BufReader, Write};
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=assets/");
    println!("cargo::rustc-check-cfg=cfg(has_embedded_kernel)");
    println!("cargo::rustc-check-cfg=cfg(has_embedded_rootfs)");
    println!("cargo::rustc-check-cfg=cfg(has_embedded_qemu)");

    let out_dir = std::env::var("OUT_DIR").unwrap();

    // Kernel is small (~16 MB) — use max compression
    compress_asset("assets/vmlinux", "vmlinux.zst", "has_embedded_kernel", 22, &out_dir);
    // Rootfs is large (~1 GB+) — stream-compress at level 19 to keep build times sane
    stream_compress_asset(
        "assets/rootfs.ext4",
        "rootfs.ext4.zst",
        "has_embedded_rootfs",
        19,
        &out_dir,
    );
    compress_qemu_bundle(&out_dir);
}

/// Read-all + compress for small assets (kernel). Loads entire file into memory.
fn compress_asset(src: &str, dst_name: &str, cfg_flag: &str, level: i32, out_dir: &str) {
    let src_path = Path::new(src);
    if !src_path.exists() {
        return;
    }

    let data = fs::read(src_path).unwrap_or_else(|e| panic!("failed to read {src}: {e}"));
    let compressed = zstd::encode_all(data.as_slice(), level)
        .unwrap_or_else(|e| panic!("failed to compress {src}: {e}"));

    let dst_path = Path::new(out_dir).join(dst_name);
    let mut file =
        fs::File::create(&dst_path).unwrap_or_else(|e| panic!("failed to create {dst_name}: {e}"));
    file.write_all(&compressed)
        .unwrap_or_else(|e| panic!("failed to write {dst_name}: {e}"));

    println!("cargo:rustc-cfg={cfg_flag}");
    println!(
        "cargo:warning=Embedded {src} ({} bytes → {} bytes compressed)",
        data.len(),
        compressed.len()
    );
}

/// Stream-compress for large assets (rootfs). Avoids loading the full file into memory.
fn stream_compress_asset(src: &str, dst_name: &str, cfg_flag: &str, level: i32, out_dir: &str) {
    let src_path = Path::new(src);
    if !src_path.exists() {
        return;
    }

    let src_len = fs::metadata(src_path)
        .unwrap_or_else(|e| panic!("failed to stat {src}: {e}"))
        .len();

    let mut reader = BufReader::with_capacity(
        1024 * 1024, // 1 MB read buffer
        fs::File::open(src_path).unwrap_or_else(|e| panic!("failed to open {src}: {e}")),
    );

    let dst_path = Path::new(out_dir).join(dst_name);
    let writer =
        fs::File::create(&dst_path).unwrap_or_else(|e| panic!("failed to create {dst_name}: {e}"));

    let mut encoder = zstd::Encoder::new(writer, level)
        .unwrap_or_else(|e| panic!("failed to init zstd encoder for {src}: {e}"));

    io::copy(&mut reader, &mut encoder)
        .unwrap_or_else(|e| panic!("failed to stream-compress {src}: {e}"));
    let writer = encoder
        .finish()
        .unwrap_or_else(|e| panic!("failed to finalize compression for {src}: {e}"));
    let compressed_len = writer
        .metadata()
        .map(|m| m.len())
        .unwrap_or(0);

    println!("cargo:rustc-cfg={cfg_flag}");
    println!(
        "cargo:warning=Embedded {src} ({src_len} bytes → {compressed_len} bytes compressed)",
    );
}

fn compress_qemu_bundle(out_dir: &str) {
    let qemu_dir = Path::new("assets/qemu");
    if !qemu_dir.exists() || !qemu_dir.is_dir() {
        return;
    }

    // Create tar archive in memory
    let mut tar_data = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_data);
        builder
            .append_dir_all(".", qemu_dir)
            .unwrap_or_else(|e| panic!("failed to create qemu tar: {e}"));
        builder
            .finish()
            .unwrap_or_else(|e| panic!("failed to finalize qemu tar: {e}"));
    }

    // Compress with zstd (ultra mode for best ratio)
    let compressed = zstd::encode_all(tar_data.as_slice(), 22)
        .unwrap_or_else(|e| panic!("failed to compress qemu bundle: {e}"));

    let dst_path = Path::new(out_dir).join("qemu-bundle.tar.zst");
    let mut file = fs::File::create(&dst_path)
        .unwrap_or_else(|e| panic!("failed to create qemu-bundle.tar.zst: {e}"));
    file.write_all(&compressed)
        .unwrap_or_else(|e| panic!("failed to write qemu-bundle.tar.zst: {e}"));

    println!("cargo:rustc-cfg=has_embedded_qemu");
    println!(
        "cargo:warning=Embedded QEMU bundle ({} bytes tar → {} bytes compressed)",
        tar_data.len(),
        compressed.len()
    );
}
