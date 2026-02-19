use crate::config::VirtualGhostConfig;
use crate::error::{VmError, VirtualGhostError};
use std::path::PathBuf;
use tracing::info;

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
        // TODO: Decompress embedded kernel from include_bytes!()
        // For now, check if a kernel exists in the assets directory
        Err(VmError::AssetExtraction(
            "embedded kernel not yet available — provide --kernel path".to_string(),
        )
        .into())
    }

    fn extract_rootfs(&self) -> Result<(), VirtualGhostError> {
        // TODO: Decompress embedded rootfs from include_bytes!()
        Err(VmError::AssetExtraction(
            "embedded rootfs not yet available — provide --rootfs path".to_string(),
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
