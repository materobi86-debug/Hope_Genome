# Zcode Launchpad One-Click Installer & Automatic Setup Script
Param (
    [string]$InstallDir = "$env:LOCALAPPDATA\ZcodeLaunchpad"
)

Write-Host "==========================================================" -ForegroundColor Green
Write-Host "  ZCODE / HOPE CODE LAUNCHPAD MINI MK3 ONE-CLICK INSTALLER " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Green

Write-Host "`n[1/5] Telepítési könyvtár létrehozása: $InstallDir ..." -ForegroundColor Yellow
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

Write-Host "[2/5] Végrehajtható binárisok másolása..." -ForegroundColor Yellow
$Binaries = @(
    "zcode-launchpad-panel.exe",
    "zcode-launchpad-daemon.exe",
    "zcode-launchpad-notify.exe",
    "zcode-launchpad-tray.exe"
)

foreach ($bin in $Binaries) {
    if (Test-Path $bin) {
        Copy-Item -Path $bin -Destination $InstallDir -Force
        Write-Host "  -> $bin másolva." -ForegroundColor Green
    }
}

Write-Host "[3/5] Windows Tűzfal beállítása Tailscale és Hálózati eléréshez (Port 8080)..." -ForegroundColor Yellow
try {
    netsh advfirewall firewall add rule name="HOPE CODE PWA Control Panel (Port 8080)" dir=in action=allow protocol=TCP localport=8080 | Out-Null
    Write-Host "  -> Tűzfal szabály sikeresen hozzáadva (Port 8080 nyitva Tailscale-hez)." -ForegroundColor Green
} catch {
    Write-Host "  -> Tűzfal beállítás átugorva." -ForegroundColor Gray
}

Write-Host "[4/5] Windows Automatikus Indítás (Autostart) Beállítása..." -ForegroundColor Yellow
$RegPath = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
$TrayExe = "$InstallDir\zcode-launchpad-tray.exe"
if (Test-Path $TrayExe) {
    Set-ItemProperty -Path $RegPath -Name "ZcodeLaunchpadTray" -Value "`"$TrayExe`""
    Write-Host "  -> Windows Autostart Rendszerindítás sikeresen regisztrálva!" -ForegroundColor Green
}

Write-Host "[5/5] Alkalmazás Indítása..." -ForegroundColor Yellow
if (Test-Path $TrayExe) {
    Start-Process -FilePath $TrayExe -WindowStyle Hidden
    Write-Host "  -> Zcode Launchpad Tálca & Control Panel elindítva!" -ForegroundColor Green
}

Write-Host "`n==========================================================" -ForegroundColor Green
Write-Host "  A TELEPÍTÉS SIKERESEN BEFEJEZŐDÖTT!" -ForegroundColor Cyan
Write-Host "  PWA Elérés iPhone / Mobilról (Tailscale / Wi-Fi): http://<TAILSCALE-IP>:8080" -ForegroundColor Yellow
Write-Host "==========================================================" -ForegroundColor Green
