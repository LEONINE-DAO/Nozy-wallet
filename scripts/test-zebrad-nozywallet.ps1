# Zebrad <-> NozyWallet connectivity smoke test (console output only)
$ErrorActionPreference = "Continue"
$RepoRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)

Write-Host ""
Write-Host "Zebrad <-> NozyWallet connectivity test"
Write-Host ""

$configPath = Join-Path $env:APPDATA "nozy\nozy\config\config.json"
$configZebraUrl = $null
$configLastScan = $null

if (Test-Path $configPath) {
    try {
        $raw = [System.IO.File]::ReadAllText($configPath)
        if ($raw.StartsWith([char]0xFEFF)) { $raw = $raw.Substring(1) }
        $cfg = $raw | ConvertFrom-Json
        $configZebraUrl = $cfg.zebra_url
        $configLastScan = $cfg.last_scan_height
        Write-Host "Config: $configPath"
        Write-Host "  zebra_url: $configZebraUrl"
        Write-Host "  last_scan_height: $configLastScan"
    } catch {
        Write-Host "Config: failed to parse $configPath"
    }
} else {
    Write-Host "Config: not found at $configPath"
}

function Test-Rpc {
    param([string]$Url, [string]$Label)
    try {
        $body = '{"jsonrpc":"2.0","method":"getblockcount","params":[],"id":1}'
        $r = Invoke-RestMethod -Uri $Url -Method POST -ContentType "application/json" -Body $body -TimeoutSec 12
        return @{ ok = $true; tip = [int]$r.result; url = $Url; label = $Label }
    } catch {
        return @{ ok = $false; url = $Url; label = $Label; error = $_.Exception.Message }
    }
}

Write-Host ""
$targets = @(
    @{ label = "config_zebra_url"; url = $configZebraUrl },
    @{ label = "localhost_8232"; url = "http://127.0.0.1:8232" },
    @{ label = "localhost_18232"; url = "http://127.0.0.1:18232" }
)

$wslIp = (wsl hostname -I 2>$null)
if ($wslIp) {
    $wslIp = ($wslIp -split "\s+")[0].Trim()
    if ($wslIp) {
        $targets += @{ label = "wsl_dynamic"; url = "http://${wslIp}:8232" }
    }
}

$seen = @{}
foreach ($t in ($targets | Where-Object { $_.url })) {
    if ($seen.ContainsKey($t.url)) { continue }
    $seen[$t.url] = $true
    $res = Test-Rpc -Url $t.url -Label $t.label
    if ($res.ok) {
        Write-Host "[OK]   $($t.label) $($t.url) tip=$($res.tip)"
    } else {
        Write-Host "[FAIL] $($t.label) $($t.url) - $($res.error)"
    }
}

$nozy = Join-Path $RepoRoot "target\release\nozy.exe"
if (-not (Test-Path $nozy)) {
    $nozy = Join-Path $RepoRoot "nozy.exe"
}

Write-Host ""
if (Test-Path $nozy) {
    Write-Host "nozy test-zebra:"
    & $nozy test-zebra
} else {
    Write-Host "nozy.exe not found - build with: cargo build --release --bin nozy"
}

$zebrad = Get-Process zebrad -ErrorAction SilentlyContinue
Write-Host ""
Write-Host "Windows zebrad process: $(if ($zebrad) { "running (PID $($zebrad.Id))" } else { "not running" })"
Write-Host ""
