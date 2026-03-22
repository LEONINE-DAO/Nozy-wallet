param(
    [string]$Distro = "Ubuntu",
    [switch]$KeepZebra
)

$ErrorActionPreference = "Stop"

function Invoke-WslBash {
    param([string]$Command)
    wsl -d $Distro -- bash -lc $Command
}

Write-Host "Stopping Nozy desktop processes..." -ForegroundColor Yellow
Invoke-WslBash "pkill -f 'node .*vite|cargo tauri dev|nozy-wallet-desktop' >/dev/null 2>&1 || true"

if (-not $KeepZebra) {
    Write-Host "Stopping Zebra..." -ForegroundColor Yellow
    Invoke-WslBash "pkill -f '^zebrad start' >/dev/null 2>&1 || true"
} else {
    Write-Host "Keeping Zebra running (-KeepZebra)." -ForegroundColor Cyan
}

Start-Sleep -Seconds 1
$zebraState = (Invoke-WslBash "pgrep -af '^zebrad start' >/dev/null && echo RUNNING || echo STOPPED").Trim()
$desktopState = (Invoke-WslBash "pgrep -af 'node .*vite|cargo tauri dev|nozy-wallet-desktop' >/dev/null && echo RUNNING || echo STOPPED").Trim()

Write-Host "Desktop processes: $desktopState"
Write-Host "Zebra: $zebraState"
