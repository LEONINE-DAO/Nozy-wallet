# Nozy Lite — measure CLI (and optional desktop) size / cold start.
# Usage: .\scripts\nozy-lite-bench.ps1 [-DesktopExe path\to\NozyWallet.exe]
param(
    [string]$DesktopExe = ""
)

$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent $PSScriptRoot)

Write-Host "Building release nozy..."
cargo build -p nozy --bin nozy --release | Out-Host

$targetRoot = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path (Get-Location) "target" }
$cli = Join-Path $targetRoot "release\nozy.exe"
if (-not (Test-Path $cli)) {
    throw "Missing CLI binary: $cli"
}

$cliBytes = (Get-Item $cli).Length
$sw = [System.Diagnostics.Stopwatch]::StartNew()
& $cli --version | Out-Null
$sw.Stop()
$coldMs = $sw.ElapsedMilliseconds
$sw.Restart()
& $cli --version | Out-Null
$sw.Stop()
$warmMs = $sw.ElapsedMilliseconds

$healthMs = $null
$healthExit = $null
try {
    $sw.Restart()
    & $cli health --json 2>$null | Out-Null
    $healthExit = $LASTEXITCODE
    $sw.Stop()
    $healthMs = $sw.ElapsedMilliseconds
} catch {
    $healthMs = "error"
}

$deskLine = "| Desktop (Tauri) | (not provided) | — | — |"
if ($DesktopExe -and (Test-Path $DesktopExe)) {
    $dBytes = (Get-Item $DesktopExe).Length
    $deskLine = "| Desktop (Tauri) | ``$DesktopExe`` | $dBytes | $([math]::Round($dBytes/1MB, 1)) |"
}

$date = Get-Date -Format "yyyy-MM-dd"
Write-Host ""
Write-Host "### Paste into docs/reference/NOZY_LITE_BENCHES.md"
Write-Host @"
| Field | Value |
|-------|--------|
| Date | $date |
| Host | $([System.Environment]::OSVersion.VersionString) |
| Build | cargo build -p nozy --bin nozy --release |
| CLI path | $cli |

| Surface | Artifact | Size (bytes) | Size (MiB) |
|---------|----------|--------------|------------|
| **Nozy Lite (CLI)** | ``nozy.exe`` release | $cliBytes | $([math]::Round($cliBytes/1MB, 1)) |
$deskLine

| Metric | ms |
|--------|-----|
| ``nozy --version`` cold sample | $coldMs |
| ``nozy --version`` warm sample | $warmMs |
| ``nozy health --json`` wall | $healthMs (exit $healthExit) |

Idle/sync RSS: not measured by this script — attach Task Manager / Get-Process WorkingSet64 while ``nozy tui`` or ``nozy sync`` runs.
"@
