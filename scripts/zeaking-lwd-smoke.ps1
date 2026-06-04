# Smoke test for Zeaking Phase 2 sync-to-tip (API + unit tests).
# Requires: built nozywallet-api; lightwalletd gRPC (default WSL IP :9067 when -UseWslLwd).
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File scripts\zeaking-lwd-smoke.ps1
#   powershell -ExecutionPolicy Bypass -File scripts\zeaking-lwd-smoke.ps1 -LiveSync
#   powershell -ExecutionPolicy Bypass -File scripts\zeaking-lwd-smoke.ps1 -LightwalletdUrl http://127.0.0.1:9067

param(
    [switch]$LiveSync,
    # 0 = pick first free port in 3000-3010 (use when run-nozy-api already holds 3000).
    [int]$ApiPort = 0,
    [string]$LightwalletdUrl = "",
    [switch]$UseWslLwd = $true,
    [string]$WslDistro = "Ubuntu",
    [int]$LwdPort = 9067
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$ApiBin = Join-Path $RepoRoot "target\release\nozywallet-api.exe"
$SmokeDb = Join-Path $env:TEMP "nozy-lwd-smoke-$(Get-Date -Format 'yyyyMMddHHmmss').sqlite"

function Get-WslIPv4 {
    param([string]$D)
    $raw = (wsl -d $D -- hostname -I 2>$null)
    if (-not $raw) { return $null }
    $ip = ($raw -split "\s+")[0].Trim()
    if ($ip) { return $ip }
    return $null
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

function Resolve-ApiPort {
    param([int]$Preferred)
    if ($Preferred -gt 0) {
        if (Test-TcpPortAvailable -Port $Preferred) { return $Preferred }
        throw "Port $Preferred is in use. Stop run-nozy-api / nozywallet-api or pass -ApiPort 0 to auto-pick."
    }
    for ($p = 3000; $p -le 3010; $p++) {
        if (Test-TcpPortAvailable -Port $p) { return $p }
    }
    throw "No free TCP port in 3000-3010."
}

function Test-ApiAlreadyRunning {
    param([int]$Port)
    try {
        Invoke-RestMethod -Uri "http://127.0.0.1:$Port/api/lwd/chain-tip" -Method Get -TimeoutSec 3 | Out-Null
        return $true
    } catch {
        return $false
    }
}

Write-Host "== Zeaking LWD smoke ==" -ForegroundColor Cyan
Push-Location $RepoRoot
$apiProc = $null
$reuseApi = $false
try {
    Write-Step "zeaking unit tests (lightwalletd feature)"
    $env:CARGO_TERM_COLOR = "never"
    & cargo test -p zeaking --features lightwalletd -q 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) { Write-Fail "zeaking tests" }
    Write-Ok "zeaking lwd tests passed"

    if (-not (Test-Path $ApiBin)) {
        Write-Step "build nozywallet-api (release)"
        cargo build -p nozywallet-api --release
        if ($LASTEXITCODE -ne 0) { Write-Fail "api build" }
    }
    Write-Ok "api binary: $ApiBin"

    $lwdHost = ([uri]$LightwalletdUrl).Host
    $lwdUp = $false
    try {
        $tcp = Test-NetConnection -ComputerName $lwdHost -Port $LwdPort -WarningAction SilentlyContinue
        $lwdUp = $tcp.TcpTestSucceeded
    } catch { $lwdUp = $false }

    Write-Host "   LIGHTWALLETD_GRPC = $LightwalletdUrl" -ForegroundColor DarkGray
    if ($lwdUp) {
        Write-Ok "lightwalletd reachable on ${lwdHost}:${LwdPort}"
    } else {
        Write-Warn "lightwalletd not listening on ${lwdHost}:${LwdPort} (start with scripts/start-lightwalletd-wsl.ps1)"
    }

    $env:LIGHTWALLETD_GRPC = $LightwalletdUrl
    $reuseApi = $false
    if ($ApiPort -eq 0) {
        if (Test-ApiAlreadyRunning -Port 3000) {
            $ApiPort = 3000
            $reuseApi = $true
            Write-Ok "reusing api-server already on port 3000 (stop run-nozy-api to let smoke own the port)"
        } else {
            $ApiPort = Resolve-ApiPort -Preferred 0
        }
    } else {
        $ApiPort = Resolve-ApiPort -Preferred $ApiPort
    }

    if (-not $reuseApi) {
        Write-Step "start api-server on port $ApiPort"
        $env:NOZY_HTTP_PORT = "$ApiPort"
        $apiProc = Start-Process -FilePath $ApiBin -WorkingDirectory $RepoRoot -PassThru -WindowStyle Hidden
        Start-Sleep -Seconds 3
        if ($apiProc.HasExited) {
            Write-Fail "api-server exited immediately on port $ApiPort (check NOZY_HTTPS_PORT / logs)"
        }
    } else {
        Write-Step "use existing api-server on port $ApiPort"
    }

    $base = "http://127.0.0.1:$ApiPort"
    try {
        Write-Step "GET /api/lwd/chain-tip"
        $tip = Invoke-RestMethod -Uri "$base/api/lwd/chain-tip" -Method Get -TimeoutSec 30
        Write-Ok "chain_tip=$($tip.chain_tip)"
        $lwdUp = $true
    } catch {
        $err = $_.ErrorDetails.Message
        if ($err -match 'LWD_GRPC|9067|transport error') {
            Write-Ok "chain-tip unreachable lightwalletd (expected): $err"
        } else {
            Write-Fail "chain-tip: $err"
        }
    }

    Write-Step "POST /api/lwd/sync/compact-to-tip"
    # Do not use start_floor=1 when LWD is up: that streams millions of blocks to tip and times out.
    $syncStartFloor = 1
    if ($lwdUp -and $null -ne $tip.chain_tip) {
        $tipH = [long]$tip.chain_tip
        $window = 32
        $syncStartFloor = [Math]::Max(1, $tipH - $window)
        Write-Host "   smoke sync window: start_floor=$syncStartFloor (~$($tipH - $syncStartFloor + 1) blocks near tip)" -ForegroundColor DarkGray
    }
    $body = @{
        db_path = $SmokeDb
        start_floor = $syncStartFloor
        persist_progress_every = 32
        lightwalletd_url = $LightwalletdUrl
    } | ConvertTo-Json
    try {
        $sync = Invoke-RestMethod -Uri "$base/api/lwd/sync/compact-to-tip" -Method Post `
            -ContentType "application/json" -Body $body -TimeoutSec 120
        Write-Ok "already_at_tip=$($sync.already_at_tip) blocks_written=$($sync.blocks_written) range=$($sync.range_start_effective)-$($sync.range_end) tip=$($sync.chain_tip)"
        if (-not $lwdUp) { Write-Warn "sync succeeded but port 9067 was not open earlier - verify lightwalletd URL" }
    } catch {
        $status = $_.Exception.Response.StatusCode.value__
        $err = $_.ErrorDetails.Message
        if ($err -match '"code"\s*:\s*"LWD_GRPC"') {
            Write-Ok "compact-to-tip returned LWD_GRPC + HTTP $status (API wiring OK; start lightwalletd for live sync)"
            if ($LiveSync) {
                Write-Fail "LiveSync requested but lightwalletd unavailable"
            }
        } else {
            $hint = ""
            if (-not $status -and $lwdUp -and $syncStartFloor -eq 1) {
                $hint = " (likely timed out syncing from block 1; re-run after smoke script update)"
            } elseif (-not $status) {
                $hint = " (connection closed or timed out; check api-server console)"
            }
            Write-Fail "compact-to-tip HTTP $status : $err$hint"
        }
    }

    Write-Step "popup vitest (extension)"
    Push-Location "browser-extension\wasm-core\popup"
    npm run test --silent 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) { Pop-Location; Write-Fail "popup vitest" }
    Pop-Location
    Write-Ok "popup tests passed"

    Write-Host ""
    Write-Host "Zeaking LWD smoke finished." -ForegroundColor Green
    if (-not $lwdUp) {
        Write-Host "For full sync-to-tip: start lightwalletd in WSL (scripts/start-lightwalletd-wsl.ps1), then re-run with -LiveSync." -ForegroundColor DarkYellow
    }
} finally {
    if ($apiProc -and -not $apiProc.HasExited -and -not $reuseApi) {
        Stop-Process -Id $apiProc.Id -Force -ErrorAction SilentlyContinue
    }
    Pop-Location
}
