# Start Nozy Sync Engine (Windows)
# Prefer this over lightwalletd for compact sync (#274).
#
# Usage:
#   .\scripts\start-nozy-sync-engine.ps1
#   .\scripts\start-nozy-sync-engine.ps1 -Bind 127.0.0.1:9067 -RpcUrl http://127.0.0.1:8232
#   .\scripts\start-nozy-sync-engine.ps1 -Release

param(
    [string]$RpcUrl = $(if ($env:ZEBRA_RPC_URL) { $env:ZEBRA_RPC_URL } else { "http://127.0.0.1:8232" }),
    [string]$Bind = $(if ($env:NOZY_SYNC_ENGINE_BIND) { $env:NOZY_SYNC_ENGINE_BIND } else { "127.0.0.1:9067" }),
    [string]$DbPath = $(if ($env:NOZY_SYNC_ENGINE_DB) { $env:NOZY_SYNC_ENGINE_DB } else { "nozy_sync_engine_compact.sqlite" }),
    [switch]$Release
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

$env:ZEBRA_RPC_URL = $RpcUrl
$env:LIGHTWALLETD_GRPC = "http://$($Bind -replace '^http://','')"
if ($Bind -notmatch '^https?://') {
    $env:LIGHTWALLETD_GRPC = "http://$Bind"
} else {
    $env:LIGHTWALLETD_GRPC = $Bind
}

Write-Host "Starting Zeaking (Nozy Sync Engine) — logo prints in this terminal."
Write-Host "  DB: $DbPath"

$cargoArgs = @("run", "-p", "nozy-sync-engine")
if ($Release) { $cargoArgs += "--release" }
$cargoArgs += @(
    "--",
    "--rpc-url", $RpcUrl,
    "--bind", $Bind,
    "--db-path", $DbPath
)

& cargo @cargoArgs
