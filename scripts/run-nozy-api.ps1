# Start Nozy api-server (zeaking HTTP companion) on Windows while Zebrad + lightwalletd run in WSL.
#
# Prerequisites (inside WSL):
#   - Zebrad RPC reachable from Windows: listen on 0.0.0.0:8232 (see run-nozy.ps1 / zebrad.toml).
#   - lightwalletd running; gRPC must listen on 0.0.0.0:9067 (not only 127.0.0.1) so this host can connect.
#
# HTTP port: default 0 = pick first free port in 3000-3100 (avoids Windows error 10048).
# Use -HttpPort 3000 to require that port (script errors early if busy). Extension: set companion baseUrl to match.
# Zebra RPC in the extension: http://<WSL-IP>:8232 (see browser-extension/COMPANION.md).
#
# Build dir: default ...\NozyWallet-cargo-api-server\pid-<PID> avoids lock waits between two windows.

param(
    [string]$Distro = "Ubuntu",
    [string]$NozyRoot = "",
    [int]$LwdGrpcPort = 9067,
    [string]$LwdGrpcUrl = "",
    # 0 = auto (first free 3000-3100). Set e.g. 3000 to pin a port.
    [int]$HttpPort = 0,
    [switch]$DebugBuild,
    # Use workspace default target/ (may block if rust-analyzer or another cargo holds the lock).
    [switch]$SharedTarget,
    # Reuse one LocalAppData target dir across runs (faster incremental); two terminals can block each other.
    [switch]$PinnedApiTarget
)

$ErrorActionPreference = "Stop"

if (-not $NozyRoot) {
    # This file lives in <repo>/scripts/ — repo root is one level up.
    $NozyRoot = Split-Path $PSScriptRoot -Parent
}

function Test-TcpPortAvailable {
    param([int]$Port)
    $listener = $null
    try {
        $listener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Any, $Port)
        $listener.Start()
        return $true
    } catch {
        return $false
    } finally {
        if ($null -ne $listener) {
            try { $listener.Stop() } catch { }
        }
    }
}

if ($HttpPort -eq 0) {
    $picked = 0
    for ($p = 3000; $p -le 3100; $p++) {
        if (Test-TcpPortAvailable -Port $p) {
            $picked = $p
            break
        }
    }
    if ($picked -eq 0) {
        throw "No free TCP port in 3000-3100. Free a port or stop another nozywallet-api instance."
    }
    $HttpPort = $picked
    if ($HttpPort -ne 3000) {
        Write-Host "Port 3000 busy; using $HttpPort (NOZY_HTTP_PORT). Point the extension companion at http://127.0.0.1:$HttpPort" -ForegroundColor Yellow
    }
} elseif (-not (Test-TcpPortAvailable -Port $HttpPort)) {
    throw "Port $HttpPort is already in use. Use -HttpPort 0 to auto-pick, or stop the process holding that port."
}

function Get-WslIPv4 {
    param([string]$D)
    $raw = (wsl -d $D -- hostname -I 2>$null)
    if (-not $raw) { throw "Could not get WSL IP. Is distro '$D' installed? Try: wsl -l -v" }
    $ip = ($raw -split "\s+")[0].Trim()
    if (-not $ip) { throw "Empty WSL IP from hostname -I" }
    return $ip
}

$grpc = if ($LwdGrpcUrl) { $LwdGrpcUrl } else {
    $wslIp = Get-WslIPv4 -D $Distro
    "http://${wslIp}:${LwdGrpcPort}"
}

$apiDir = Join-Path $NozyRoot "api-server"
if (-not (Test-Path $apiDir)) {
    throw "api-server not found at: $apiDir. Use -NozyRoot with your Nozy-wallet clone path on Windows."
}

$env:LIGHTWALLETD_GRPC = $grpc
$env:NOZY_HTTP_PORT = "$HttpPort"

# Separate target dir avoids workspace target/ lock (IDE rust-analyzer, other cargo).
# Per-PID subdir avoids two PowerShell windows both waiting on the SAME isolated dir while compiling.
$isoTarget = $null
if (-not $SharedTarget) {
    $base = Join-Path $env:LOCALAPPDATA "NozyWallet-cargo-api-server"
    if ($PinnedApiTarget) {
        $isoTarget = $base
    } else {
        $isoTarget = Join-Path $base "pid-$PID"
    }
    New-Item -ItemType Directory -Path $isoTarget -Force | Out-Null
    $env:CARGO_TARGET_DIR = $isoTarget
}

Write-Host "== Nozy api-server (zeaking) ==" -ForegroundColor Cyan
Write-Host "NozyRoot = $NozyRoot"
Write-Host "LIGHTWALLETD_GRPC = $env:LIGHTWALLETD_GRPC"
Write-Host "NOZY_HTTP_PORT = $env:NOZY_HTTP_PORT (api-server reads this env)"
if ($isoTarget) {
    $hint = if ($PinnedApiTarget) { "-PinnedApiTarget (shared under LocalAppData)" } else { "per-shell PID (use -PinnedApiTarget to reuse one cache)" }
    Write-Host "Isolated build dir: $isoTarget ($hint)" -ForegroundColor DarkGray
}
Write-Host "Listening: http://127.0.0.1:$HttpPort"
Write-Host "Test: curl.exe http://127.0.0.1:$HttpPort/api/lwd/info"
Write-Host ""

# Run from workspace root so `-p nozywallet-api` resolves reliably; `--target-dir` forces the isolated lock path.
Push-Location $NozyRoot
try {
    $cargoArgs = @("run", "-p", "nozywallet-api")
    if (-not $DebugBuild) {
        $cargoArgs += "--release"
    }
    if ($isoTarget) {
        $cargoArgs += @("--target-dir", $isoTarget)
    }
    & cargo @cargoArgs
} finally {
    Pop-Location
}
