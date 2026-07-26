#Requires -Version 5.1
<#
.SYNOPSIS
  Run Nozy Nym smolmix D2 evidence steps (issue #147 / track B).

.PARAMETER ZebraUrl
  JSON-RPC base URL. Defaults to http://127.0.0.1:8232 for dry-reachability demo.

.PARAMETER IpRelocate
  Run Cloudflare clearnet vs mixnet IP compare (needs network + NYM credentials).

.PARAMETER RpcProbe
  Run getblockcount over mixnet (requires exit-reachable ZebraUrl).

.PARAMETER SkipBuild
  Do not cargo build --release the spike.
#>
param(
    [string]$ZebraUrl = "http://127.0.0.1:8232",
    [switch]$IpRelocate,
    [switch]$RpcProbe,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$SpikeDir = Join-Path $RepoRoot "tools\nym-smolmix-broadcast-spike"
$EvidenceDir = Join-Path $RepoRoot "docs\reference\evidence"
$Stamp = Get-Date -Format "yyyyMMdd-HHmmss"

if (-not (Test-Path $SpikeDir)) {
    throw "Spike directory missing: $SpikeDir"
}

New-Item -ItemType Directory -Force -Path $EvidenceDir | Out-Null

Push-Location $SpikeDir
try {
    if (-not $SkipBuild) {
        Write-Host "==> cargo build --release (nym-smolmix-broadcast-spike)"
        cargo build --release
        if ($LASTEXITCODE -ne 0) { throw "cargo build failed ($LASTEXITCODE)" }
    }

    $Bin = Join-Path $SpikeDir "target\release\nym-smolmix-broadcast-spike.exe"
    if (-not (Test-Path $Bin)) {
        $Bin = Join-Path $SpikeDir "target\release\nym-smolmix-broadcast-spike"
    }
    if (-not (Test-Path $Bin)) {
        throw "Helper binary not found under target/release"
    }

    Write-Host "==> dry-reachability zebra=$ZebraUrl"
    $DryOut = Join-Path $EvidenceDir "nym-d2-dry-$Stamp.json"
    & $Bin --dry-reachability --zebra $ZebraUrl --evidence-json $DryOut
    if ($LASTEXITCODE -ne 0) { throw "dry-reachability failed ($LASTEXITCODE)" }

    # Always also record LAN refusal evidence for the case breakdown.
    $LanOut = Join-Path $EvidenceDir "nym-d2-lan-refuse-$Stamp.json"
    Write-Host "==> dry-reachability LAN refuse demo"
    & $Bin --dry-reachability --zebra "http://172.20.199.206:18232" --evidence-json $LanOut

    if ($IpRelocate) {
        Write-Host "==> ip-relocate (D2a)"
        $IpOut = Join-Path $EvidenceDir "nym-d2a-$Stamp.json"
        & $Bin --ip-relocate --evidence-json $IpOut
        if ($LASTEXITCODE -ne 0) { throw "ip-relocate failed ($LASTEXITCODE)" }
    }

    if ($RpcProbe) {
        Write-Host "==> rpc-probe (D2b) zebra=$ZebraUrl"
        $RpcOut = Join-Path $EvidenceDir "nym-d2b-$Stamp.json"
        & $Bin --rpc-probe --zebra $ZebraUrl --evidence-json $RpcOut
        if ($LASTEXITCODE -ne 0) { throw "rpc-probe failed ($LASTEXITCODE)" }
    }

    Write-Host ""
    Write-Host "Done. Evidence files under: $EvidenceDir"
    Write-Host "Case breakdown: docs/reference/NYM_MIXNET_BROADCAST_CASE_BREAKDOWN.md"
    Write-Host "Wallet readiness: nozy privacy-network nym-mixnet"
}
finally {
    Pop-Location
}
