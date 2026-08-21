# Build Script for One-Click Installer Executable
Set-Location -Path $PSScriptRoot

Write-Host "Building release binaries..." -ForegroundColor Cyan
cargo build --release

$ReleaseDir = "target\release"
$InstallDir = "installer_bundle"

if (Test-Path $InstallDir) {
    Remove-Item -Recurse -Force $InstallDir
}
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

Copy-Item "$ReleaseDir\zcode-launchpad-panel.exe" "$InstallDir\" -ErrorAction SilentlyContinue
Copy-Item "$ReleaseDir\zcode-launchpad-daemon.exe" "$InstallDir\" -ErrorAction SilentlyContinue
Copy-Item "$ReleaseDir\zcode-launchpad-notify.exe" "$InstallDir\" -ErrorAction SilentlyContinue
Copy-Item "$ReleaseDir\zcode-launchpad-tray.exe" "$InstallDir\" -ErrorAction SilentlyContinue
Copy-Item "install_zcode_launchpad.ps1" "$InstallDir\"

Write-Host "Installer bundle prepared in $InstallDir!" -ForegroundColor Green
