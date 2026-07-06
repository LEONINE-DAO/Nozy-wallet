# WSL helper for ironwood-valar lightwalletd (testnet). Avoids PowerShell/bash quoting issues.
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File scripts\ironwood-lwd-wsl.ps1 -Install -Start
#   powershell -ExecutionPolicy Bypass -File scripts\ironwood-lwd-wsl.ps1 -Start
#   powershell -ExecutionPolicy Bypass -File scripts\ironwood-lwd-wsl.ps1 -Status

param(
    [string]$Distro = "Ubuntu",
    [switch]$Testnet = $true,
    [switch]$Install,
    [switch]$Start,
    [switch]$Status,
    [int]$GrpcPort = 9068,
    [string]$ZebraRpcHost = "",
    [int]$ZebraRpcPort = 18232
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$WslRepoRoot = (wsl -d $Distro -- wslpath -a "$RepoRoot").Trim()
$HostScript = "$WslRepoRoot/scripts/ironwood-lwd-wsl.sh"

if ($Status) {
    Write-Host "=== Ironwood LWD (WSL) ===" -ForegroundColor Cyan
    wsl -d $Distro -- bash -lc "pgrep -a lightwalletd || echo '(not running)'; ss -ltn | grep ':$GrpcPort' || echo '(not listening on $GrpcPort)'"
    $ip = (wsl -d $Distro -- hostname -I).Trim().Split()[0]
    if ($ip) {
        Write-Host "From Windows: LIGHTWALLETD_GRPC=http://${ip}:${GrpcPort}" -ForegroundColor Green
    }
    exit 0
}

$bashArgs = @()
if ($Testnet) { $bashArgs += '--testnet' }
if ($Install) { $bashArgs += '--install' }
if ($Start) { $bashArgs += '--start' }
if ($GrpcPort -ne 9068) { $bashArgs += '--grpc-port'; $bashArgs += "$GrpcPort" }
if ($ZebraRpcHost) { $bashArgs += '--zebra-host'; $bashArgs += $ZebraRpcHost }
if ($ZebraRpcPort -ne 18232) { $bashArgs += '--zebra-port'; $bashArgs += "$ZebraRpcPort" }

if ($bashArgs.Count -eq 0) {
    Write-Host "Pass -Install and/or -Start (or -Status)." -ForegroundColor Yellow
    exit 1
}

# Copy script to WSL (strip CRLF) and run
$wslScript = "/tmp/ironwood-lwd-wsl.sh"
wsl -d $Distro -- bash -lc "tr -d '\r' < '$HostScript' > /tmp/ironwood-lwd-wsl.sh && chmod +x /tmp/ironwood-lwd-wsl.sh"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
wsl -d $Distro -- bash $wslScript @bashArgs
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

if ($Start) {
    & $PSScriptRoot\ironwood-lwd-wsl.ps1 -Status
}
