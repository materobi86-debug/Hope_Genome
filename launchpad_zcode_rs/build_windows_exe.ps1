# PowerShell script to build standalone Windows 11 Executables & System Tray app for Zcode Launchpad Mini MK3

Write-Host "============================================================" -ForegroundColor Green
Write-Host " Building Zcode Launchpad Mini MK3 Windows 11 Executables " -ForegroundColor Green
Write-Host "============================================================" -ForegroundColor Green

# 1. Build release binaries
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "Cargo build failed!" -ForegroundColor Red
    exit 1
}

# 2. Create distribution directory
$distDir = "dist/ZcodeLaunchpad-Windows11"
New-Item -ItemType Directory -Force -Path $distDir | Out-Null

Copy-Item "target/release/zcode-launchpad-panel.exe" "$distDir/" -ErrorAction SilentlyContinue
Copy-Item "target/release/zcode-launchpad-daemon.exe" "$distDir/" -ErrorAction SilentlyContinue
Copy-Item "target/release/zcode-launchpad-notify.exe" "$distDir/" -ErrorAction SilentlyContinue
Copy-Item "target/release/zcode-launchpad-tray.exe" "$distDir/" -ErrorAction SilentlyContinue
Copy-Item "README.md" "$distDir/README.txt" -ErrorAction SilentlyContinue

Write-Host "`nSuccessfully created standalone Windows 11 release package in $distDir!" -ForegroundColor Cyan
Write-Host "Executables generated:" -ForegroundColor Yellow
Write-Host "  1. zcode-launchpad-panel.exe   (Control Panel & Virtual MIDI GUI)" -ForegroundColor Yellow
Write-Host "  2. zcode-launchpad-tray.exe    (System Tray app with icon)" -ForegroundColor Yellow
Write-Host "  3. zcode-launchpad-daemon.exe  (Background daemon)" -ForegroundColor Yellow
Write-Host "  4. zcode-launchpad-notify.exe  (CLI hook notifier)" -ForegroundColor Yellow
