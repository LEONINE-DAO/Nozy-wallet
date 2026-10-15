# Reuse the WSL Crosslink wallet (secret.seed) in the Windows monolith GUI (v15).
# Run from repo root:  powershell -NoProfile -ExecutionPolicy Bypass -File scripts/setup-crosslink-gui-windows.ps1
$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path $PSScriptRoot -Parent

$ToolDir = Join-Path $RepoRoot "tools\crosslink"
$GuiExe = Join-Path $ToolDir "zebrad_v15_win32.exe"
$SeedStaging = Join-Path $ToolDir "secret.seed.from-wsl"
$FeatureNetCacheName = "crosslink_featurenet_20260929_staking3d"
$WslSeed = "/home/lowo/zebra-crosslink/cache/zebra_crosslink_workshop_season_one_v3_ehtedht_cache_delete_me/secret.seed"
$GuiUrl = "https://github.com/ShieldedLabs/crosslink_monolith/releases/download/v15/zebrad_v15_win32.exe"

Write-Host "== Crosslink GUI setup (reuse WSL wallet) ==" -ForegroundColor Cyan

# 1) Stop WSL headless node so ports/wallet do not fight the GUI node.
Write-Host "`n[1/5] Stopping WSL Crosslink service (avoid dual-node)..."
& "$RepoRoot\scripts\stop-crosslink-zebra.ps1"

# 2) Export secret.seed from WSL.
Write-Host "`n[2/5] Copying secret.seed from WSL..."
New-Item -ItemType Directory -Force -Path $ToolDir | Out-Null
$WslSeedWin = ($SeedStaging -replace '\\', '/').Replace('C:/', '/mnt/c/').Replace('c:/', '/mnt/c/')
wsl.exe -d Ubuntu -- cp $WslSeed $WslSeedWin
$seedLen = (Get-Item $SeedStaging).Length
if ($seedLen -ne 32) {
    throw "Expected 32-byte secret.seed, got $seedLen bytes at $SeedStaging"
}
Write-Host "  Staged: $SeedStaging ($seedLen bytes)"

# 3) Download GUI if missing.
Write-Host "`n[3/5] Ensuring Windows GUI v15..."
if (-not (Test-Path $GuiExe)) {
    Write-Host "  Downloading $GuiUrl ..."
    Invoke-WebRequest -Uri $GuiUrl -OutFile $GuiExe
}
Write-Host "  GUI: $GuiExe"

# 4) First launch creates %LOCALAPPDATA%\zebra\<cache-folder>\ — run briefly if no cache yet.
$ZebraCacheRoot = Join-Path $env:LOCALAPPDATA "zebra"
Write-Host "`n[4/5] Cache root: $ZebraCacheRoot"
$existing = @()
if (Test-Path $ZebraCacheRoot) {
    $existing = Get-ChildItem $ZebraCacheRoot -Directory -ErrorAction SilentlyContinue
}

if ($existing.Count -eq 0) {
    Write-Host "  No GUI cache yet. Start the GUI manually and wait ~15s for cache folder:"
    Write-Host "    $GuiExe"
    Write-Host "  Then re-run this script to install secret.seed (or copy it yourself)."
    exit 1
}

# v15 Feature Net cache. A differently named sibling is a fresh wallet — do not install the seed there.
$named = Get-ChildItem $ZebraCacheRoot -Recurse -Directory -Filter $FeatureNetCacheName -ErrorAction SilentlyContinue |
    Select-Object -First 1
if (-not $named) {
    Write-Host "  No '$FeatureNetCacheName' cache yet." -ForegroundColor Yellow
    Write-Host "  If v15 created a similarly named folder, stop the GUI, back it up, and rename that folder to:"
    Write-Host "    $FeatureNetCacheName"
    Write-Host "  Then re-run this script. Do not copy a new cache over the old one."
    exit 1
}
$targetDir = $named.FullName
$targetSeed = Join-Path $targetDir "secret.seed"
if (Test-Path $targetSeed) {
    Copy-Item $targetSeed (Join-Path $ToolDir "secret.seed.backup-before-overwrite") -Force
    Write-Host "  Backed up existing GUI secret.seed"
}
Copy-Item $SeedStaging $targetSeed -Force
Write-Host "  Installed WSL seed -> $targetSeed"

# 5) Launch GUI for real.
Write-Host "`n[5/5] Starting Crosslink GUI with your WSL wallet..."
Write-Host @"

Next in the GUI:
  1. Wait for sync (batched blocks at tip = still syncing)
  2. Bottom-right: Receive cTAZ (faucet) OR let CPU mining run
  3. Right panel: Copy Identity  (your finalizer hex)
  4. On Staking Day: Stake -> Paste Identity -> pick 1 cTAZ (or 0.01 per UI)

Nozy still points at Crosslink RPC. After GUI is up, set:
  nozy config --set-crosslink-url http://127.0.0.1:18232
(this GUI config listens on 18232; a launch without -c mints a new wallet)

"@ -ForegroundColor Green

$GuiConfig = Join-Path $env:LOCALAPPDATA "zebra\zebrad-crosslink-gui.toml"
if (-not (Test-Path $GuiConfig)) {
    throw "Missing $GuiConfig — refusing to start v15 without it (that creates a new cache)."
}
Start-Process -FilePath $GuiExe -ArgumentList @("-c", $GuiConfig)
Write-Host "Done. Cache: $targetDir"
