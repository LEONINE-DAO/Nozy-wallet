# Print which terminals/services should be running and quick port checks.
# Usage: .\scripts\show-dev-stack.ps1

param(
    [string]$WslDistro = "Ubuntu",
    [string]$WslIp = ""
)

$ErrorActionPreference = "SilentlyContinue"

function Get-WslIPv4([string]$D) {
    if ($WslIp) { return $WslIp.Trim() }
    $raw = (wsl -d $D -- hostname -I 2>$null)
    if (-not $raw) { return $null }
    return ($raw -split "\s+")[0].Trim()
}

function Test-Port([string]$Host, [int]$Port) {
    try {
        return (Test-NetConnection -ComputerName $Host -Port $Port -WarningAction SilentlyContinue).TcpTestSucceeded
    } catch { return $false }
}

$ip = Get-WslIPv4 -D $WslDistro
if (-not $ip) { $ip = "?" }

Write-Host ""
Write-Host "=== Nozy dev stack (3 terminals) ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "TERMINAL 1 (WSL services - leave open)" -ForegroundColor Yellow
Write-Host "  zebrad:        powershell -File C:\Zebrad\scripts\start-zebrad-wsl.ps1 -Status"
Write-Host "  lightwalletd:  .\scripts\start-lightwalletd-wsl.ps1 -Status"
Write-Host ""
Write-Host "TERMINAL 2 (Nozy API - leave open for extension)" -ForegroundColor Yellow
Write-Host "  .\scripts\run-nozy-api.ps1 -HttpPort 3000"
Write-Host ""
Write-Host "TERMINAL 3 (commands only - wallet, smoke)" -ForegroundColor Yellow
Write-Host "  .\scripts\zeaking-lwd-smoke.ps1 -LiveSync"
Write-Host "  (do not start a second API here if Terminal 2 is running)"
Write-Host ""
Write-Host "Full guide: scripts\TERMINAL-PLAYBOOK.md" -ForegroundColor DarkGray
Write-Host ""
Write-Host "=== Port check (WSL IP: $ip) ===" -ForegroundColor Cyan

$zebra = Test-Port -Host $ip -Port 8232
$lwd = Test-Port -Host $ip -Port 9067
$api3000 = Test-Port -Host "127.0.0.1" -Port 3000
$api3001 = Test-Port -Host "127.0.0.1" -Port 3001

Write-Host ("  zebrad RPC      {0,-5} http://{1}:8232" -f $(if ($zebra) { "UP" } else { "DOWN" }), $ip) `
    -ForegroundColor $(if ($zebra) { "Green" } else { "Red" })
Write-Host ("  lightwalletd    {0,-5} http://{1}:9067" -f $(if ($lwd) { "UP" } else { "DOWN" }), $ip) `
    -ForegroundColor $(if ($lwd) { "Green" } else { "Red" })
Write-Host ("  nozy API :3000  {0,-5} http://127.0.0.1:3000" -f $(if ($api3000) { "UP" } else { "DOWN" })) `
    -ForegroundColor $(if ($api3000) { "Green" } else { "DarkYellow" })
if ($api3001) {
    Write-Host "  nozy API :3001  UP    (smoke may have used this port)" -ForegroundColor DarkYellow
}

Write-Host ""
if (-not $lwd) {
    Write-Host "Next: install + start lightwalletd (see TERMINAL-PLAYBOOK.md). -LiveSync will fail until 9067 is UP." -ForegroundColor Yellow
} elseif (-not $zebra) {
    Write-Host "Next: start zebrad in WSL." -ForegroundColor Yellow
} elseif ($lwd -and $zebra) {
    Write-Host "Ready for: .\scripts\zeaking-lwd-smoke.ps1 -LiveSync" -ForegroundColor Green
}
Write-Host ""
