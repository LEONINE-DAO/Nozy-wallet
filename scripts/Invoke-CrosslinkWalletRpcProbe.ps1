# Probe Crosslink node for get_wallet_sync_status (PR #51).
param(
    [string]$RpcUrl = "http://127.0.0.1:18232"
)

$body = @{
    jsonrpc = "2.0"
    id      = "nozy-wallet-probe"
    method  = "get_wallet_sync_status"
    params  = @()
} | ConvertTo-Json -Depth 4

Write-Host "Probing $RpcUrl for get_wallet_sync_status ..." -ForegroundColor Cyan

try {
    $resp = Invoke-RestMethod -Uri $RpcUrl -Method Post -Body $body -ContentType "application/json" -TimeoutSec 15
}
catch {
    Write-Host "RPC request failed: $_" -ForegroundColor Red
    Write-Host "Is the Crosslink node running with RPC enabled?" -ForegroundColor Yellow
    exit 1
}

if ($resp.error) {
    $msg = $resp.error.message
    Write-Host "RPC error: $msg" -ForegroundColor Red
    if ($msg -match "Method not found") {
        Write-Host ""
        Write-Host "Your node does not expose get_wallet_sync_status yet." -ForegroundColor Yellow
        Write-Host "Upgrade guide: docs/CROSSLINK_WALLET_RPC_UPGRADE.md" -ForegroundColor Yellow
        Write-Host "Upstream: https://github.com/ShieldedLabs/crosslink_monolith/pull/51" -ForegroundColor Yellow
    }
    exit 2
}

$result = $resp.result
if (-not $result) {
    Write-Host "Unexpected empty result." -ForegroundColor Red
    exit 3
}

function Zat-ToCtaz([uint64]$zat) {
    return [math]::Round($zat / 100000000.0, 8)
}

$shielded = if ($null -ne $result.user_shielded_spendable_zats) {
    [uint64]$result.user_shielded_spendable_zats
} else { [uint64]0 }
$unshielded = if ($null -ne $result.user_unshielded_zats) {
    [uint64]$result.user_unshielded_zats
} else { [uint64]0 }
$available = $shielded + $unshielded
$sync = $result.sync_height
$tip = $result.tip_height

Write-Host ''
Write-Host 'OK - wallet RPC available' -ForegroundColor Green
Write-Host ('  Available to stake: {0} cTAZ' -f (Zat-ToCtaz $available))
Write-Host ('  Shielded spendable: {0} cTAZ' -f (Zat-ToCtaz $shielded))
if ($unshielded -gt 0) {
    Write-Host ('  Unshielded:         {0} cTAZ' -f (Zat-ToCtaz $unshielded))
}
Write-Host ('  Wallet scan:        {0} / {1}' -f $sync, $tip)
Write-Host ''
Write-Host 'Nozy: nozy crosslink wallet | desktop Crosslink tab | GET /api/crosslink/wallet-status'
