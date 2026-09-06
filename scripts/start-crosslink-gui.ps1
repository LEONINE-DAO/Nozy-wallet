# Launch Crosslink GUI v13 only — leave it running (no WSL, no auto-kill).
$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path $PSScriptRoot -Parent
$GuiExe = Join-Path $RepoRoot "tools\crosslink\zebrad_win32_gui_v13.exe"
$GuiConfig = Join-Path $env:LOCALAPPDATA "zebra\zebrad-crosslink-gui.toml"
$RpcUrl = "http://127.0.0.1:18232"

if (-not (Test-Path $GuiExe)) {
    Write-Host "GUI not found. Run setup-crosslink-gui-windows.ps1 once to download it." -ForegroundColor Red
    exit 1
}

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
