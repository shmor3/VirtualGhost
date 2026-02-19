#![cfg(unix)]

use crate::error::{VmError, VirtualGhostError};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use walkdir::WalkDir;

use super::GpuDevice;

/// Discover a GPU device and its IOMMU group from a PCI address.
pub fn discover_gpu(pci_address: &str) -> Result<GpuDevice, VirtualGhostError> {
    let sysfs_path = format!("/sys/bus/pci/devices/{pci_address}/");
    let sysfs = Path::new(&sysfs_path);

    if !sysfs.exists() {
        return Err(VmError::GpuNotFound(format!(
            "PCI device {pci_address} not found at {sysfs_path}"
        ))
        .into());
    }

    // Find IOMMU group
    let iommu_link = sysfs.join("iommu_group");
    if !iommu_link.exists() {
        return Err(VmError::VfioError(format!(
            "No IOMMU group for {pci_address} — is IOMMU enabled in BIOS?"
        ))
        .into());
    }

    let iommu_path = fs::read_link(&iommu_link).map_err(|e| {
        VmError::VfioError(format!("Failed to read IOMMU group link: {e}"))
    })?;

    let iommu_group: u32 = iommu_path
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| VmError::VfioError("Failed to parse IOMMU group number".to_string()))?;

    // Find sibling devices in the same IOMMU group
    let group_devices_path = format!("/sys/kernel/iommu_groups/{iommu_group}/devices");
    let mut siblings = Vec::new();

    if let Ok(entries) = fs::read_dir(&group_devices_path) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name != pci_address {
                    siblings.push(name.to_string());
                }
            }
        }
    }

    if !siblings.is_empty() {
        info!(
            pci_address,
            iommu_group,
            siblings = ?siblings,
            "GPU has IOMMU group siblings that will also be passed through"
        );
    }

    Ok(GpuDevice {
        pci_address: pci_address.to_string(),
        sysfs_path,
        iommu_group,
        siblings,
    })
}

/// Unbind a PCI device from its current driver.
pub fn unbind_driver(pci_address: &str) -> Result<(), VirtualGhostError> {
    let driver_path = format!("/sys/bus/pci/devices/{pci_address}/driver");
    let driver = Path::new(&driver_path);

    if !driver.exists() {
        info!(pci_address, "Device has no driver bound, skipping unbind");
        return Ok(());
    }

    let unbind_path = format!("{driver_path}/unbind");
    info!(pci_address, "Unbinding current driver");
    fs::write(&unbind_path, pci_address).map_err(|e| {
        VmError::VfioError(format!("Failed to unbind driver for {pci_address}: {e}"))
    })?;

    Ok(())
}

/// Bind a PCI device to the vfio-pci driver.
pub fn bind_vfio(pci_address: &str) -> Result<(), VirtualGhostError> {
    // Set driver_override to vfio-pci
    let override_path = format!("/sys/bus/pci/devices/{pci_address}/driver_override");
    info!(pci_address, "Setting driver_override to vfio-pci");
    fs::write(&override_path, "vfio-pci").map_err(|e| {
        VmError::VfioError(format!("Failed to set driver_override for {pci_address}: {e}"))
    })?;

    // Trigger driver probe
    let probe_path = "/sys/bus/pci/drivers_probe";
    fs::write(probe_path, pci_address).map_err(|e| {
        VmError::VfioError(format!("Failed to probe vfio-pci for {pci_address}: {e}"))
    })?;

    info!(pci_address, "Bound to vfio-pci");
    Ok(())
}

/// Validate that the host supports VFIO passthrough.
pub fn validate_host() -> Result<(), VirtualGhostError> {
    // Check IOMMU is enabled
    let iommu_groups = Path::new("/sys/kernel/iommu_groups");
    if !iommu_groups.exists() {
        return Err(VmError::VfioError(
            "IOMMU not available — enable Intel VT-d or AMD-Vi in BIOS and add \
             intel_iommu=on (or amd_iommu=on) to kernel boot parameters"
                .to_string(),
        )
        .into());
    }

    // Check vfio module is loaded
    let vfio_path = Path::new("/dev/vfio/vfio");
    if !vfio_path.exists() {
        return Err(VmError::VfioError(
            "VFIO not available — run: modprobe vfio_pci vfio_iommu_type1".to_string(),
        )
        .into());
    }

    Ok(())
}

/// Prepare a GPU and all its IOMMU group siblings for passthrough.
pub fn prepare_passthrough(gpu: &GpuDevice) -> Result<(), VirtualGhostError> {
    validate_host()?;

    // Unbind and bind the main GPU device
    unbind_driver(&gpu.pci_address)?;
    bind_vfio(&gpu.pci_address)?;

    // Do the same for all siblings in the IOMMU group
    for sibling in &gpu.siblings {
        unbind_driver(sibling)?;
        bind_vfio(sibling)?;
    }

    info!(
        pci_address = gpu.pci_address,
        iommu_group = gpu.iommu_group,
        "GPU passthrough prepared"
    );

    Ok(())
}
