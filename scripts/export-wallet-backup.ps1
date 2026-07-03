# Mirror gitignored wallet recovery files to Documents (outside the repo clone).
#
# Usage (from repo root):
#   .\scripts\export-wallet-backup.ps1
#
# Does NOT print mnemonics. Does NOT commit to git.

param(
    [string]$DestRoot = "",
    [switch]$Quiet
)

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path $PSScriptRoot -Parent
if (-not $DestRoot) {
    $DestRoot = Join-Path $env:USERPROFILE "Documents\NozyWallet-Recovery"
}

$Sources = @(
    (Join-Path $RepoRoot "WALLET_BACKUP.txt"),
    (Join-Path $RepoRoot "WALLET_BACKUP_SECURE.md")
)

$missing = @()
foreach ($src in $Sources) {
    if (-not (Test-Path -LiteralPath $src)) {
        $missing += $src
    }
}

if ($missing.Count -gt 0) {
    Write-Error "Missing canonical backup file(s):`n  $($missing -join "`n  ")`nCreate or restore them before exporting."
}

New-Item -ItemType Directory -Force -Path $DestRoot | Out-Null

$stamp = Get-Date -Format "yyyy-MM-dd_HHmmss"
$archiveDir = Join-Path $DestRoot "history\$stamp"
New-Item -ItemType Directory -Force -Path $archiveDir | Out-Null

$manifestLines = @(
    "exported_at=$stamp",
    "repo_root=$RepoRoot",
    "files="
)

foreach ($src in $Sources) {
    $name = Split-Path $src -Leaf
    Copy-Item -LiteralPath $src -Destination (Join-Path $DestRoot $name) -Force
    Copy-Item -LiteralPath $src -Destination (Join-Path $archiveDir $name) -Force
    $manifestLines += "  - $name"
}

$manifestLines += "history_snapshot=$archiveDir"
$manifestPath = Join-Path $DestRoot "LAST_EXPORT.txt"
$manifestLines | Set-Content -Path $manifestPath -Encoding UTF8

if (-not $Quiet) {
    Write-Host "Wallet recovery files mirrored to:"
    Write-Host "  $DestRoot"
    Write-Host "History snapshot:"
    Write-Host "  $archiveDir"
    Write-Host ""
    Write-Host "Keep a paper or encrypted off-device backup as well. Do not commit these files to git."
}
