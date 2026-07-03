# Generate Tauri desktop icons from the canonical NozyWallet logo.
# Source: landing/src/assets/logo.png (yellow zebra + "Nozy wallet")
param(
    [string]$LogoPath = (Join-Path $PSScriptRoot "..\landing\src\assets\logo.png"),
    [string]$IconsDir = (Join-Path $PSScriptRoot "..\desktop-client\src-tauri\icons"),
    [string]$PublicLogo = (Join-Path $PSScriptRoot "..\desktop-client\public\logo.png"),
    [string]$AssetsLogo = (Join-Path $PSScriptRoot "..\assets\logo.png"),
    [string]$DesktopClient = (Join-Path $PSScriptRoot "..\desktop-client")
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $LogoPath)) {
    Write-Error "Logo not found: $LogoPath"
}

New-Item -ItemType Directory -Force -Path (Split-Path $PublicLogo) | Out-Null
New-Item -ItemType Directory -Force -Path (Split-Path $AssetsLogo) | Out-Null

Copy-Item $LogoPath $PublicLogo -Force
Copy-Item $LogoPath $AssetsLogo -Force

$logoResolved = (Resolve-Path $LogoPath).Path
Push-Location $DesktopClient
try {
    # Tauri CLI writes Windows-compatible DIB-based icon.ico (required by RC.EXE / winres).
    npm exec tauri -- icon $logoResolved
    if ($LASTEXITCODE -ne 0) {
        throw "tauri icon failed with exit code $LASTEXITCODE"
    }
} finally {
    Pop-Location
}

if (-not (Test-Path (Join-Path $IconsDir "icon.ico"))) {
    Write-Error "icon.ico was not created in $IconsDir"
}

Write-Host "Desktop icons generated in: $IconsDir"
Write-Host "Public logo: $PublicLogo"
Write-Host "Assets logo: $AssetsLogo"
