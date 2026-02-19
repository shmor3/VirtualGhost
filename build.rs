fn main() {
    // TODO: Compress kernel and rootfs assets with zstd at build time
    // For now, this is a placeholder. When assets/ contains vmlinux and rootfs.ext4,
    // this script will compress them and make them available via include_bytes!().
    println!("cargo:rerun-if-changed=assets/");
}
