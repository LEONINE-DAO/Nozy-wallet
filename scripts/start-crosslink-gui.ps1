# Launch Crosslink GUI v15 only — leave it running (no WSL, no auto-kill).
# Feature Net cache name (v15): crosslink_featurenet_20260929_staking3d
# It must stay the network folder inside cache_dir from zebrad-crosslink-gui.toml.
# Starting this exe with no -c creates a new cache and a new wallet.
$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path $PSScriptRoot -Parent
$GuiExe = Join-Path $RepoRoot "tools\crosslink\zebrad_v15_win32.exe"
$GuiConfig = Join-Path $env:LOCALAPPDATA "zebra\zebrad-crosslink-gui.toml"
$FeatureNetCacheName = "crosslink_featurenet_20260929_staking3d"
$RpcUrl = "http://127.0.0.1:18232"

if (-not (Test-Path $GuiExe)) {
    Write-Host "GUI not found. Run setup-crosslink-gui-windows.ps1 once to download it." -ForegroundColor Red
    exit 1
}

$cacheHit = Get-ChildItem (Join-Path $env:LOCALAPPDATA "zebra") -Recurse -Directory -Filter $FeatureNetCacheName -ErrorAction SilentlyContinue |
    Where-Object { Test-Path (Join-Path $_.FullName "secret.seed") } |
    Select-Object -First 1
if (-not $cacheHit) {
    Write-Host "Missing Feature Net cache '$FeatureNetCacheName' (with secret.seed) under %LOCALAPPDATA%\zebra." -ForegroundColor Red
    Write-Host "Do not start v15 until that folder is in place. A fresh cache rerolls the wallet." -ForegroundColor Red
    exit 1
}
Write-Host "Cache: $($cacheHit.FullName)" -ForegroundColor DarkGray

$running = Get-Process -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $GuiExe }
if ($running) {
    Write-Host "Crosslink GUI already running (PID $($running.Id))." -ForegroundColor Yellow
} else {
    Write-Host "Starting Crosslink GUI - leave it open for sync, faucet, and staking." -ForegroundColor Cyan
    if (-not (Test-Path $GuiConfig)) {
        Write-Host "Missing $GuiConfig - create it first (see repo docs) or run setup-crosslink-gui-windows.ps1" -ForegroundColor Red
        exit 1
    }
    Start-Process -FilePath $GuiExe -ArgumentList @("-c", $GuiConfig)
}

Write-Host ""
Write-Host "In the GUI:"
Write-Host "  - Wait for sync (blocks one-at-a-time at tip = ready)"
Write-Host "  - Bottom-right: Receive cTAZ (faucet)"
Write-Host "  - Right panel: Copy Identity (must NOT be all zeros)"
Write-Host "  - On Staking Day: Stake, Paste Identity, 1 cTAZ"
Write-Host ""
Write-Host "When RPC is up:"
Write-Host "  .\target\release\nozy.exe config --set-crosslink-url $RpcUrl"
Write-Host "  .\target\release\nozy.exe crosslink status"
Write-Host ""
Write-Host "Do NOT start WSL background node until GUI staking works." -ForegroundColor Green
