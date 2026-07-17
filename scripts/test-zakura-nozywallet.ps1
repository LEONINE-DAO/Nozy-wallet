# Zakurad <-> NozyWallet connectivity smoke test (console output only)
$ErrorActionPreference = "Continue"
$RepoRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)

Write-Host ""
Write-Host "Zakurad <-> NozyWallet connectivity test"
Write-Host ""

$configPath = Join-Path $env:APPDATA "nozy\nozy\config\config.json"
$configZebraUrl = $null

if (Test-Path $configPath) {
    try {
        $raw = [System.IO.File]::ReadAllText($configPath)
        if ($raw.StartsWith([char]0xFEFF)) { $raw = $raw.Substring(1) }
        $cfg = $raw | ConvertFrom-Json
        $configZebraUrl = $cfg.zebra_url
        Write-Host "Config: $configPath"
        Write-Host "  zebra_url: $configZebraUrl"
    } catch {
        Write-Host "Config: failed to parse $configPath"
    }
} else {
    Write-Host "Config: not found at $configPath"
}

function Invoke-NodeRpc {
    param([string]$Url, [string]$Method, [object[]]$Params = @())
    $body = @{ jsonrpc = "2.0"; method = $Method; params = $Params; id = 1 } | ConvertTo-Json -Compress
    return Invoke-RestMethod -Uri $Url -Method POST -ContentType "application/json" -Body $body -TimeoutSec 15
}

function Test-RpcTip {
    param([string]$Url, [string]$Label)
    try {
        $r = Invoke-NodeRpc -Url $Url -Method "getblockcount"
        return @{ ok = $true; tip = [int]$r.result; url = $Url; label = $Label }
    } catch {
        return @{ ok = $false; url = $Url; label = $Label; error = $_.Exception.Message }
    }
}

function Test-Subversion {
    param([string]$Url)
    try {
        $r = Invoke-NodeRpc -Url $Url -Method "getnetworkinfo"
        $sub = $r.result.subversion
        $kind = if ($sub -match "zakura") { "Zakura" } elseif ($sub -match "zebra") { "Zebra" } else { "unknown" }
        return @{ ok = $true; subversion = $sub; kind = $kind }
    } catch {
        return @{ ok = $false; error = $_.Exception.Message }
    }
}

function Test-Treestate {
    param([string]$Url, [int]$Height)
    try {
        $r = Invoke-NodeRpc -Url $Url -Method "z_gettreestate" -Params @([string]$Height)
        $orchard = $r.result.orchard
        if (-not $orchard) {
            return @{ ok = $false; error = "z_gettreestate missing orchard section" }
        }
        return @{ ok = $true }
    } catch {
        return @{ ok = $false; error = $_.Exception.Message }
    }
}

Write-Host ""
$targets = @(
    @{ label = "config_zebra_url"; url = $configZebraUrl },
    @{ label = "localhost_8232"; url = "http://127.0.0.1:8232" }
)

$wslIp = (wsl hostname -I 2>$null)
if ($wslIp) {
    $wslIp = ($wslIp -split "\s+")[0].Trim()
    if ($wslIp) {
        $targets += @{ label = "wsl_dynamic"; url = "http://${wslIp}:8232" }
    }
}

$seen = @{}
$workingUrl = $null
foreach ($t in ($targets | Where-Object { $_.url })) {
    if ($seen.ContainsKey($t.url)) { continue }
    $seen[$t.url] = $true
    $res = Test-RpcTip -Url $t.url -Label $t.label
    if ($res.ok) {
        Write-Host "[OK]   $($t.label) $($t.url) tip=$($res.tip)"
        if (-not $workingUrl) { $workingUrl = $t.url }
        $sv = Test-Subversion -Url $t.url
        if ($sv.ok) {
            Write-Host "       node=$($sv.kind) subversion=$($sv.subversion)"
            $ts = Test-Treestate -Url $t.url -Height $res.tip
            if ($ts.ok) {
                Write-Host "       z_gettreestate(orchard) at tip: OK"
            } else {
                Write-Host "       z_gettreestate at tip: FAIL - $($ts.error)"
            }
        }
    } else {
        Write-Host "[FAIL] $($t.label) $($t.url) - $($res.error)"
    }
}

# lightwalletd gRPC port (TCP only)
$lwdUp = $false
try {
    $tcp = New-Object System.Net.Sockets.TcpClient
    $tcp.Connect("127.0.0.1", 9067)
    $lwdUp = $true
    $tcp.Close()
} catch {}
Write-Host ""
Write-Host ("lightwalletd :9067  {0}" -f $(if ($lwdUp) { "UP" } else { "DOWN" }))

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

$zakurad = Get-Process zakurad -ErrorAction SilentlyContinue
$zebrad = Get-Process zebrad -ErrorAction SilentlyContinue
Write-Host ""
Write-Host "Windows zakurad process: $(if ($zakurad) { "running (PID $($zakurad.Id))" } else { "not running" })"
Write-Host "Windows zebrad process:  $(if ($zebrad) { "running (PID $($zebrad.Id))" } else { "not running" })"
Write-Host ""
Write-Host "Guide: docs/reference/ZAKURA_NOZYWALLET_CONNECTIVITY.md"
Write-Host ""
