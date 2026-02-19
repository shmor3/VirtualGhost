use crate::config::VirtualGhostConfig;
use crate::error::{VmError, VirtualGhostError};
use std::path::PathBuf;
use tracing::info;

#[cfg(has_embedded_kernel)]
static EMBEDDED_KERNEL: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/vmlinux.zst"));

#[cfg(has_embedded_rootfs)]
static EMBEDDED_ROOTFS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/rootfs.ext4.zst"));

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

        Ok(())
    }

    fn extract_kernel(&self) -> Result<(), VirtualGhostError> {
        #[cfg(has_embedded_kernel)]
        {
            let decompressed = zstd::decode_all(EMBEDDED_KERNEL).map_err(|e| {
                VmError::AssetExtraction(format!("failed to decompress kernel: {e}"))
            })?;
            std::fs::write(self.kernel_path(), &decompressed).map_err(|e| {
                VmError::AssetExtraction(format!("failed to write kernel: {e}"))
            })?;
            info!(
                path = %self.kernel_path().display(),
                size = decompressed.len(),
                "Kernel extracted"
            );
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
            let decompressed = zstd::decode_all(EMBEDDED_ROOTFS).map_err(|e| {
                VmError::AssetExtraction(format!("failed to decompress rootfs: {e}"))
            })?;
            std::fs::write(self.rootfs_path(), &decompressed).map_err(|e| {
                VmError::AssetExtraction(format!("failed to write rootfs: {e}"))
            })?;
            info!(
                path = %self.rootfs_path().display(),
                size = decompressed.len(),
                "Rootfs extracted"
            );
            return Ok(());
        }

        #[cfg(not(has_embedded_rootfs))]
        Err(VmError::AssetExtraction(
            "no embedded rootfs — provide --rootfs path or place rootfs.ext4 in assets/ and rebuild"
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
