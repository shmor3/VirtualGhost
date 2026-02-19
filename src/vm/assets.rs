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

    fn extract_qemu(&self) -> Result<(), VirtualGhostError> {
        #[cfg(has_embedded_qemu)]
        {
            use std::io::Cursor;

            let qemu_dir = self.qemu_dir();
            std::fs::create_dir_all(&qemu_dir).map_err(|e| {
                VmError::AssetExtraction(format!("failed to create qemu dir: {e}"))
            })?;

            // Decompress zstd
            let tar_data = zstd::decode_all(EMBEDDED_QEMU).map_err(|e| {
                VmError::AssetExtraction(format!("failed to decompress QEMU bundle: {e}"))
            })?;

            info!(
                compressed_size = EMBEDDED_QEMU.len(),
                decompressed_size = tar_data.len(),
                "QEMU bundle decompressed"
            );

            // Untar
            let cursor = Cursor::new(tar_data);
            let mut archive = tar::Archive::new(cursor);
            archive.unpack(&qemu_dir).map_err(|e| {
                VmError::AssetExtraction(format!("failed to extract QEMU tar: {e}"))
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
