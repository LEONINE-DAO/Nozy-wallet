# Ironwood lightwalletd GetBlockRange smoke (testnet default :9068).
#
# Prerequisites:
#   - Ironwood-capable zebrad on Windows :18232 (or pass -ZebraRpcHost / -ZebraRpcPort)
#   - WSL + Ubuntu with golang (for -InstallLwd)
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File scripts\ironwood-lwd-smoke.ps1 -InstallLwd
#   powershell -ExecutionPolicy Bypass -File scripts\ironwood-lwd-smoke.ps1
#   powershell -ExecutionPolicy Bypass -File scripts\ironwood-lwd-smoke.ps1 -LightwalletdUrl http://127.0.0.1:9068

param(
    [string]$LightwalletdUrl = "",
    [switch]$UseWslLwd = $true,
    [string]$WslDistro = "Ubuntu",
    [int]$LwdPort = 9068,
    [int]$StartHeight = 4136100,
    [int]$EndHeight = 0,
    [switch]$InstallLwd,
    [switch]$StartLwd,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$env:CARGO_TARGET_DIR = Join-Path $RepoRoot "target"
$SmokeBin = Join-Path $env:CARGO_TARGET_DIR "release\ironwood_lwd_smoke.exe"

function Get-WslIPv4 {
    param([string]$D)
    $raw = (wsl -d $D -- hostname -I 2>$null)
    if (-not $raw) { return $null }
    return (($raw -split "\s+")[0]).Trim()
}

function Write-Step($msg) {
    Write-Host ""
    Write-Host "-> $msg" -ForegroundColor Yellow
}

function Write-Ok($msg) {
    Write-Host "   OK: $msg" -ForegroundColor Green
}

function Write-Warn($msg) {
    Write-Host "   WARN: $msg" -ForegroundColor DarkYellow
}

function Write-Fail($msg) {
    Write-Host "   FAIL: $msg" -ForegroundColor Red
    exit 1
}

Push-Location $RepoRoot
try {
    Write-Host "== Ironwood LWD smoke ==" -ForegroundColor Cyan

    if ($InstallLwd -or $StartLwd) {
        Write-Step "start/install ironwood-valar lightwalletd (WSL testnet)"
        $lwdArgs = @(
            "-ExecutionPolicy", "Bypass",
            "-File", (Join-Path $RepoRoot "scripts\start-lightwalletd-wsl.ps1"),
            "-Testnet",
            "-Distro", $WslDistro,
            "-GrpcPort", "$LwdPort"
        )
        if ($InstallLwd) { $lwdArgs += "-Install" }
        & powershell @lwdArgs
        if ($LASTEXITCODE -ne 0) { Write-Fail "start-lightwalletd-wsl.ps1 exited $LASTEXITCODE" }
    }

    if (-not $LightwalletdUrl) {
        if ($UseWslLwd) {
            $wslIp = Get-WslIPv4 -D $WslDistro
            if ($wslIp) {
                $LightwalletdUrl = "http://${wslIp}:${LwdPort}"
            } else {
                $LightwalletdUrl = "http://127.0.0.1:${LwdPort}"
            }
        } else {
            $LightwalletdUrl = "http://127.0.0.1:${LwdPort}"
        }
    }

    Write-Host "   LIGHTWALLETD_GRPC = $LightwalletdUrl" -ForegroundColor DarkGray
    Write-Host "   range = $StartHeight..$EndHeight" -ForegroundColor DarkGray

    $lwdHost = ([uri]$LightwalletdUrl).Host
    $lwdUp = $false
    try {
        $tcp = Test-NetConnection -ComputerName $lwdHost -Port $LwdPort -WarningAction SilentlyContinue
        $lwdUp = $tcp.TcpTestSucceeded
    } catch { $lwdUp = $false }

    if (-not $lwdUp) {
        Write-Warn "lightwalletd not listening on ${lwdHost}:${LwdPort}"
        Write-Host "   Try: powershell -File scripts\start-lightwalletd-wsl.ps1 -Testnet -Install" -ForegroundColor DarkGray
        Write-Host "        powershell -File scripts\start-lightwalletd-wsl.ps1 -Testnet" -ForegroundColor DarkGray
        Write-Fail "lightwalletd unreachable"
    }
    Write-Ok "lightwalletd reachable"

    if (-not $SkipBuild) {
        Write-Step "build ironwood_lwd_smoke (release)"
        & cargo build --release --bin ironwood_lwd_smoke 2>&1 | Out-Host
        if ($LASTEXITCODE -ne 0) { Write-Fail "cargo build --bin ironwood_lwd_smoke" }
    }
    if (-not (Test-Path $SmokeBin)) { Write-Fail "missing $SmokeBin" }
    Write-Ok "binary ready"

    Write-Step "GetBlockRange smoke"
    $smokeArgs = @($LightwalletdUrl, "$StartHeight")
    if ($EndHeight -gt 0) { $smokeArgs += "$EndHeight" }
    & $SmokeBin @smokeArgs
    $code = $LASTEXITCODE
    if ($code -eq 0) {
        Write-Ok "Ironwood LWD smoke passed"
    } elseif ($code -eq 2) {
        Write-Warn "smoke finished with warnings (exit 2) - likely vanilla lightwalletd or pre-activation range"
        exit 2
    } else {
        Write-Fail "ironwood_lwd_smoke exit $code"
    }
} finally {
    Pop-Location
}
