# Set one Zebra RPC URL across Nozy components.
#
# Applies to:
# - CLI (`nozy` binary config)
# - desktop-client / api-server (they read `nozy` config via load_config())
# - current PowerShell session (`$env:ZEBRA_RPC_URL`)
#
# Extension note:
# Chrome extension settings are browser storage, so set the same URL once in
# the extension UI (Settings -> RPC endpoint).
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File .\scripts\set-wallet-rpc.ps1
#   powershell -ExecutionPolicy Bypass -File .\scripts\set-wallet-rpc.ps1 -RpcUrl "http://172.20.199.206:8232"

param(
    [string]$RpcUrl = "http://172.20.199.206:8232",
    [string]$NozyRoot = ""
)

$ErrorActionPreference = "Stop"

if (-not $NozyRoot) {
    $NozyRoot = Split-Path $PSScriptRoot -Parent
}

Push-Location $NozyRoot
try {
    $env:ZEBRA_RPC_URL = $RpcUrl
    Write-Host "Session ZEBRA_RPC_URL = $env:ZEBRA_RPC_URL" -ForegroundColor Cyan

    & cargo run -- config --set-zebra-url $RpcUrl
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to set Nozy config zebra_url"
    }

    & cargo run -- config --show-backend
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to read backend config"
    }

    Write-Host ""
    Write-Host "Extension (manual one-time):" -ForegroundColor Yellow
    Write-Host "  Set RPC endpoint to: $RpcUrl"
    Write-Host "  If using companion mode, keep baseUrl at your api-server URL (for example http://127.0.0.1:3000)."
}
finally {
    Pop-Location
}

