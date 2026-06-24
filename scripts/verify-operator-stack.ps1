# Verify a public Zcash operator stack for NozyWallet + Zec.rocks rewards eligibility.
#
# Checks: DNS, TCP ports, Zebra JSON-RPC, lightwalletd gRPC (donation UA), optional Nozy CLI.
#
# Usage:
#   Set-Location C:\Users\User\NozyWallet
#   powershell -ExecutionPolicy Bypass -File .\scripts\verify-operator-stack.ps1
#   powershell -ExecutionPolicy Bypass -File .\scripts\verify-operator-stack.ps1 -OperatorHost zec.leoninedao.org
#   powershell -ExecutionPolicy Bypass -File .\scripts\verify-operator-stack.ps1 -SkipBuild

param(
    [Alias("HostName")]
    [string]$OperatorHost = "zec.leoninedao.org",
    [int]$ZebraPort = 8232,
    [int]$LightwalletdPort = 9067,
    [int]$ZainoGrpcPort = 8137,
    [int]$HttpsPort = 443,
    [switch]$UseHttpsForZebra,
    [string]$ZebraScheme = "",
    [switch]$SkipBuild,
    [switch]$RunNozyStatus
)

$ErrorActionPreference = "Continue"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$NozyBin = Join-Path $RepoRoot "target\release\nozy.exe"
$VerifyLwdBin = Join-Path $RepoRoot "target\release\verify_lwd.exe"

$script:FailCount = 0
$script:WarnCount = 0
$script:PassCount = 0

function Write-Title($msg) {
    Write-Host ""
    Write-Host "=== $msg ===" -ForegroundColor Cyan
}

function Write-Pass($msg) {
    $script:PassCount++
    Write-Host "  PASS: $msg" -ForegroundColor Green
}

function Write-Warn($msg) {
    $script:WarnCount++
    Write-Host "  WARN: $msg" -ForegroundColor DarkYellow
}

function Write-Fail($msg) {
    $script:FailCount++
    Write-Host "  FAIL: $msg" -ForegroundColor Red
}

function Test-TcpOpen {
    param([string]$TargetHost, [int]$Port, [int]$TimeoutMs = 8000)
    try {
        $client = New-Object System.Net.Sockets.TcpClient
        $iar = $client.BeginConnect($TargetHost, $Port, $null, $null)
        $ok = $iar.AsyncWaitHandle.WaitOne($TimeoutMs, $false)
        if (-not $ok) { return $false }
        $client.EndConnect($iar)
        $client.Close()
        return $true
    } catch {
        return $false
    }
}

function Invoke-ZebraRpc {
    param([string]$Url, [string]$Method, [object]$Params = @())
    $body = @{
        jsonrpc = "2.0"
        method  = $Method
        params  = $Params
        id      = 1
    } | ConvertTo-Json -Compress
    return Invoke-RestMethod -Uri $Url -Method Post -ContentType "application/json" -Body $body -TimeoutSec 30
}

function Get-LwdGrpcUrl {
    param([int]$Port)
    return "http://${OperatorHost}:${Port}"
}

function Test-LightwalletdEndpoint {
    param([int]$Port, [string]$Label)
    $url = Get-LwdGrpcUrl -Port $Port
    Write-Host ""
    Write-Host "  [$Label] $url" -ForegroundColor Yellow
    if (-not (Test-TcpOpen -TargetHost $OperatorHost -Port $Port)) {
        Write-Fail "$Label port $Port not reachable"
        return
    }
    Write-Pass "$Label TCP $Port open"
    if (-not (Test-Path $VerifyLwdBin)) {
        Write-Warn "verify_lwd.exe missing - run without -SkipBuild"
        return
    }
    $out = & $VerifyLwdBin $url 2>&1
    $code = $LASTEXITCODE
    $out | ForEach-Object { Write-Host "    $_" -ForegroundColor DarkGray }
    if ($code -eq 0) {
        Write-Pass "$Label GetLightdInfo + donation UA"
    } elseif ($code -eq 2) {
        Write-Fail "$Label reachable but donation_address missing (Zec.rocks disqualifier)"
    } else {
        Write-Fail "$Label gRPC check failed (exit $code)"
    }
}

Write-Title "NozyWallet operator stack verification"
Write-Host "Host: $OperatorHost"
Write-Host "Forum program: https://forum.zcashcommunity.com/t/zcash-operators-earn-zec-for-your-uptime/52090"
Write-Host "Repo: $RepoRoot"

# --- DNS ---
Write-Title "1. DNS"
try {
    $dns = Resolve-DnsName -Name $OperatorHost -ErrorAction Stop | Where-Object { $_.IPAddress }
    $ips = ($dns | ForEach-Object { $_.IPAddress }) -join ", "
    Write-Pass "Resolved $OperatorHost -> $ips"
} catch {
    Write-Fail "DNS failed: $_"
}

# --- Build helpers ---
if (-not $SkipBuild) {
    Write-Title "2. Build verify_lwd (release)"
    Push-Location $RepoRoot
    try {
        & cargo build --release --bin verify_lwd 2>&1 | Out-Host
        if ($LASTEXITCODE -ne 0) {
            Write-Fail "cargo build --bin verify_lwd failed"
        } else {
            Write-Pass "verify_lwd built"
        }
    } finally {
        Pop-Location
    }
} else {
    Write-Title "2. Build (skipped)"
}

# --- TCP ports ---
Write-Title "3. TCP ports"
$portChecks = @(
    @{ Port = $ZebraPort; Name = "Zebra RPC" },
    @{ Port = $LightwalletdPort; Name = "lightwalletd gRPC" },
    @{ Port = $ZainoGrpcPort; Name = "Zaino gRPC" },
    @{ Port = $HttpsPort; Name = "HTTPS (optional front)" }
)
foreach ($p in $portChecks) {
    if (Test-TcpOpen -TargetHost $OperatorHost -Port $p.Port) {
        Write-Pass "$($p.Name) :$($p.Port)"
    } else {
        if ($p.Port -eq $HttpsPort) {
            Write-Warn "$($p.Name) :$($p.Port) closed (optional)"
        } else {
            Write-Fail "$($p.Name) :$($p.Port) not reachable"
        }
    }
}

# --- Zebra RPC ---
Write-Title "4. Zebra JSON-RPC"
if (-not $ZebraScheme) {
    if ($UseHttpsForZebra -or ($ZebraPort -eq 443)) {
        $ZebraScheme = "https"
    } else {
        $ZebraScheme = "http"
    }
}
$zebraUrl = "${ZebraScheme}://${OperatorHost}:${ZebraPort}"
Write-Host "  URL: $zebraUrl"
try {
    $block = Invoke-ZebraRpc -Url $zebraUrl -Method "getblockcount"
    if ($null -ne $block.result) {
        Write-Pass "getblockcount = $($block.result)"
    } else {
        Write-Fail "getblockcount returned no result"
    }
    try {
        $net = Invoke-ZebraRpc -Url $zebraUrl -Method "getnetworkinfo"
        if ($net.result.subversion) {
            Write-Pass "subversion: $($net.result.subversion)"
        }
    } catch {
        Write-Warn "getnetworkinfo unavailable"
    }
} catch {
    Write-Fail "Zebra RPC: $($_.Exception.Message)"
    if ($ZebraScheme -eq "http" -and -not $UseHttpsForZebra) {
        Write-Host "  Tip: try -UseHttpsForZebra or -ZebraPort 443 if RPC is behind TLS" -ForegroundColor DarkGray
    }
}

# --- lightwalletd / Zaino gRPC ---
Write-Title "5. Light wallet server (gRPC + donation UA)"
Test-LightwalletdEndpoint -Port $LightwalletdPort -Label "lightwalletd"
if ($ZainoGrpcPort -ne $LightwalletdPort) {
    Test-LightwalletdEndpoint -Port $ZainoGrpcPort -Label "Zaino"
}

# --- Optional Nozy status ---
if ($RunNozyStatus) {
    Write-Title "6. Nozy CLI status (wallet + sync endpoints)"
    if (-not (Test-Path $NozyBin)) {
        Write-Warn "nozy.exe not found - run: cargo build --release"
    } else {
        $env:ZEBRA_RPC_URL = $zebraUrl
        $lwdUrl = Get-LwdGrpcUrl -Port $LightwalletdPort
        $env:LIGHTWALLETD_GRPC = $lwdUrl
        Write-Host "  ZEBRA_RPC_URL=$env:ZEBRA_RPC_URL"
        Write-Host "  LIGHTWALLETD_GRPC=$env:LIGHTWALLETD_GRPC"
        & $NozyBin status 2>&1 | ForEach-Object { Write-Host "  $_" }
    }
} else {
    Write-Title "6. Nozy CLI (skipped)"
    Write-Host "  Re-run with -RunNozyStatus after wallet is configured locally." -ForegroundColor DarkGray
}

# --- Forum checklist ---
Write-Title "7. Zec.rocks operator checklist"
Write-Host "  [ ] Public hostname:port listed on forum thread (Hosh)"
Write-Host "  [ ] donation UA broadcasting on lightwalletd (--donation-address)"
Write-Host "  [ ] Latest Zebra + lightwalletd/Zaino (per program rules)"
Write-Host "  [ ] Stable uptime over 30 days (Hosh metrics)"
Write-Host "  [ ] Dogfood: nozy sync --to-tip against your endpoints"

Write-Title "Summary"
Write-Host "  PASS: $script:PassCount  WARN: $script:WarnCount  FAIL: $script:FailCount"
if ($script:FailCount -eq 0) {
    Write-Host "  Stack looks reachable for Nozy + operator rewards prep." -ForegroundColor Green
    exit 0
} else {
    Write-Host "  Fix FAIL items before announcing on the forum." -ForegroundColor Red
    exit 1
}
