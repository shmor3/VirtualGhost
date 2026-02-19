use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=assets/");
    println!("cargo::rustc-check-cfg=cfg(has_embedded_kernel)");
    println!("cargo::rustc-check-cfg=cfg(has_embedded_rootfs)");

    let out_dir = std::env::var("OUT_DIR").unwrap();

    compress_asset("assets/vmlinux", "vmlinux.zst", "has_embedded_kernel", &out_dir);
    compress_asset(
        "assets/rootfs.ext4",
        "rootfs.ext4.zst",
        "has_embedded_rootfs",
        &out_dir,
    );
}

fn compress_asset(src: &str, dst_name: &str, cfg_flag: &str, out_dir: &str) {
    let src_path = Path::new(src);
    if !src_path.exists() {
        return;
    }

    let data = fs::read(src_path).unwrap_or_else(|e| panic!("failed to read {src}: {e}"));
    let compressed = zstd::encode_all(data.as_slice(), 19)
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
