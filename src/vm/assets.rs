use crate::config::VirtualGhostConfig;
use crate::error::{VmError, VirtualGhostError};
use std::path::PathBuf;
use tracing::info;

#[cfg(has_embedded_kernel)]
static EMBEDDED_KERNEL: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/vmlinux.zst"));

#[cfg(has_embedded_rootfs)]
static EMBEDDED_ROOTFS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/rootfs.ext4.zst"));

#[cfg(has_embedded_qemu)]
static EMBEDDED_QEMU: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/qemu-bundle.tar.zst"));

pub struct AssetManager {
    cache_dir: PathBuf,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            cache_dir: VirtualGhostConfig::cache_dir(),
        }
    }

    pub fn kernel_path(&self) -> PathBuf {
        self.cache_dir.join("vmlinux")
    }

    pub fn rootfs_path(&self) -> PathBuf {
        self.cache_dir.join("rootfs.ext4")
    }

    pub fn qemu_dir(&self) -> PathBuf {
        self.cache_dir.join("qemu")
    }

    pub fn qemu_bin_path(&self) -> PathBuf {
        let dir = self.qemu_dir();
        if cfg!(target_os = "windows") {
            dir.join("qemu-system-x86_64.exe")
        } else {
            dir.join("qemu-system-x86_64")
        }
    }

    pub fn qemu_data_dir(&self) -> PathBuf {
        self.qemu_dir().join("share")
    }

    pub fn ensure_assets(&self) -> Result<(), VirtualGhostError> {
        std::fs::create_dir_all(&self.cache_dir).map_err(|e| {
            VmError::AssetExtraction(format!("failed to create cache dir: {e}"))
        })?;

        if !self.kernel_path().exists() {
            info!("Extracting kernel image to cache");
            self.extract_kernel()?;
        }

        if !self.rootfs_path().exists() {
            info!("Extracting rootfs image to cache");
            self.extract_rootfs()?;
        }

        if !self.qemu_bin_path().exists() {
            info!("Extracting embedded QEMU to cache");
            self.extract_qemu()?;
        }

        Ok(())
    }

    /// Stream-decompress an embedded zstd blob directly to a file.
    /// Avoids loading the full decompressed data into memory.
    fn stream_decompress_to_file(
        compressed: &[u8],
        dest: &std::path::Path,
        label: &str,
    ) -> Result<u64, VirtualGhostError> {
        use std::io::{self, Cursor};

        let reader = Cursor::new(compressed);
        let mut decoder = zstd::Decoder::new(reader).map_err(|e| {
            VmError::AssetExtraction(format!("failed to init {label} decompressor: {e}"))
        })?;

        let mut file = std::fs::File::create(dest).map_err(|e| {
            VmError::AssetExtraction(format!("failed to create {label} file: {e}"))
        })?;

        let bytes_written = io::copy(&mut decoder, &mut file).map_err(|e| {
            VmError::AssetExtraction(format!("failed to decompress {label}: {e}"))
        })?;

        Ok(bytes_written)
    }

    fn extract_kernel(&self) -> Result<(), VirtualGhostError> {
        #[cfg(has_embedded_kernel)]
        {
            let path = self.kernel_path();
            let size =
                Self::stream_decompress_to_file(EMBEDDED_KERNEL, &path, "kernel")?;
            info!(path = %path.display(), size, "Kernel extracted");
            return Ok(());
        }

        #[cfg(not(has_embedded_kernel))]
        Err(VmError::AssetExtraction(
            "no embedded kernel — provide --kernel path or place vmlinux in assets/ and rebuild"
                .to_string(),
        )
        .into())
    }

    fn extract_rootfs(&self) -> Result<(), VirtualGhostError> {
        #[cfg(has_embedded_rootfs)]
        {
            let path = self.rootfs_path();
            let size =
                Self::stream_decompress_to_file(EMBEDDED_ROOTFS, &path, "rootfs")?;
            info!(path = %path.display(), size, "Rootfs extracted");
            return Ok(());
        }

        #[cfg(not(has_embedded_rootfs))]
        Err(VmError::AssetExtraction(
            "no embedded rootfs — provide --rootfs path or place rootfs.ext4 in assets/ and rebuild"
                .to_string(),
        )
        .into())
    }

    fn extract_qemu(&self) -> Result<(), VirtualGhostError> {
        #[cfg(has_embedded_qemu)]
        {
            use std::io::Cursor;

            let qemu_dir = self.qemu_dir();
            std::fs::create_dir_all(&qemu_dir).map_err(|e| {
                VmError::AssetExtraction(format!("failed to create qemu dir: {e}"))
            })?;

            // Stream: zstd decompress → tar unpack (no intermediate buffer)
            let cursor = Cursor::new(EMBEDDED_QEMU);
            let decoder = zstd::Decoder::new(cursor).map_err(|e| {
                VmError::AssetExtraction(format!("failed to init QEMU decompressor: {e}"))
            })?;
            let mut archive = tar::Archive::new(decoder);
            archive.unpack(&qemu_dir).map_err(|e| {
                VmError::AssetExtraction(format!("failed to extract QEMU bundle: {e}"))
            })?;

            // Set executable permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let bin_path = self.qemu_bin_path();
                if bin_path.exists() {
                    let perms = std::fs::Permissions::from_mode(0o755);
                    std::fs::set_permissions(&bin_path, perms).map_err(|e| {
                        VmError::AssetExtraction(format!(
                            "failed to set qemu permissions: {e}"
                        ))
                    })?;
                }
            }

            // Strip macOS quarantine attribute
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("xattr")
                    .args(["-d", "com.apple.quarantine"])
                    .arg(&self.qemu_bin_path())
                    .status();
            }

            info!(
                path = %self.qemu_bin_path().display(),
                data_dir = %self.qemu_data_dir().display(),
                "QEMU extracted"
            );
            return Ok(());
        }

        #[cfg(not(has_embedded_qemu))]
        Err(VmError::AssetExtraction(
            "no embedded QEMU — set qemu_bin in config or place QEMU files in assets/qemu/ and rebuild"
                .to_string(),
        )
        .into())
    }

    pub fn clean_cache(&self) -> Result<(), VirtualGhostError> {
        if self.cache_dir.exists() {
            std::fs::remove_dir_all(&self.cache_dir).map_err(VirtualGhostError::Io)?;
            info!(path = %self.cache_dir.display(), "Cleaned asset cache");
        }
        Ok(())
    }
}
