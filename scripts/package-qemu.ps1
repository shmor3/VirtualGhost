# Package QEMU from system install into assets/qemu/ for embedding
param(
    [string]$QemuDir = "C:\Program Files\qemu",
    [string]$OutputDir = "assets\qemu"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path "$QemuDir\qemu-system-x86_64.exe")) {
    Write-Error "QEMU not found at $QemuDir"
    exit 1
}

# Clean and create output directory
if (Test-Path $OutputDir) { Remove-Item -Recurse -Force $OutputDir }
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
New-Item -ItemType Directory -Path "$OutputDir\share" -Force | Out-Null
New-Item -ItemType Directory -Path "$OutputDir\share\keymaps" -Force | Out-Null
New-Item -ItemType Directory -Path "$OutputDir\share\firmware" -Force | Out-Null

# Copy QEMU binary (only x86_64)
Copy-Item "$QemuDir\qemu-system-x86_64.exe" "$OutputDir\"
Write-Host "Copied qemu-system-x86_64.exe"

# Copy ALL DLLs (QEMU needs various ones at runtime)
Copy-Item "$QemuDir\*.dll" "$OutputDir\"
$dllCount = (Get-ChildItem "$OutputDir\*.dll").Count
Write-Host "Copied $dllCount DLLs"

# Copy essential share/ files for x86_64 direct kernel boot
$shareFiles = @(
    "bios.bin", "bios-256k.bin", "bios-microvm.bin",
    "kvmvapic.bin", "linuxboot.bin", "linuxboot_dma.bin",
    "multiboot.bin", "multiboot_dma.bin", "pvh.bin",
    "efi-virtio.rom", "efi-e1000.rom", "efi-e1000e.rom",
    "edk2-x86_64-code.fd", "edk2-x86_64-secure-code.fd",
    "edk2-i386-code.fd", "edk2-i386-secure-code.fd", "edk2-i386-vars.fd",
    "edk2-licenses.txt",
    "vgabios.bin", "vgabios-ati.bin", "vgabios-bochs-display.bin",
    "vgabios-cirrus.bin", "vgabios-qxl.bin", "vgabios-ramfb.bin",
    "vgabios-stdvga.bin", "vgabios-virtio.bin", "vgabios-vmware.bin",
    "pxe-virtio.rom", "pxe-e1000.rom"
)
$shareCount = 0
foreach ($f in $shareFiles) {
    $src = "$QemuDir\share\$f"
    if (Test-Path $src) {
        Copy-Item $src "$OutputDir\share\"
        $shareCount++
    }
}
Write-Host "Copied $shareCount share files"

# Copy keymaps
Copy-Item "$QemuDir\share\keymaps\*" "$OutputDir\share\keymaps\"
Write-Host "Copied keymaps"

# Copy firmware descriptors (x86_64 only)
Get-ChildItem "$QemuDir\share\firmware\*x86_64*" -ErrorAction SilentlyContinue | Copy-Item -Destination "$OutputDir\share\firmware\"
Get-ChildItem "$QemuDir\share\firmware\*i386*" -ErrorAction SilentlyContinue | Copy-Item -Destination "$OutputDir\share\firmware\"
Write-Host "Copied firmware descriptors"

# Copy VERSION file if present
if (Test-Path "$QemuDir\VERSION") {
    Copy-Item "$QemuDir\VERSION" "$OutputDir\"
}

# Report size
$size = (Get-ChildItem -Recurse $OutputDir | Measure-Object -Property Length -Sum).Sum
Write-Host "`nQEMU bundle packaged: $([math]::Round($size / 1MB, 1)) MB in $OutputDir"
