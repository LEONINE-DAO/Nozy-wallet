param(
    [string]$Distro = "Ubuntu",
    [string]$DesktopPath = "~/projects/Nozy-wallet/desktop-client",
    [string]$ZebraRpcListen = "0.0.0.0:8232",
    [int]$RpcWaitSeconds = 90,
    [switch]$SkipDesktop,
    [switch]$SkipCleanup
)

$ErrorActionPreference = "Stop"

function Invoke-WslBash {
    param([string]$Command)
    wsl -d $Distro -- bash -lc $Command
}

Write-Host "== Nozy launcher ==" -ForegroundColor Cyan
Write-Host "Distro: $Distro"

# 1) Ensure zebrad is running
$zebraState = (Invoke-WslBash "pgrep -af '^zebrad start' >/dev/null && echo RUNNING || echo STOPPED").Trim()
if ($zebraState -eq "RUNNING") {
    Write-Host "Zebra: already running" -ForegroundColor Green
} else {
    Write-Host "Zebra: starting..." -ForegroundColor Yellow
    # Cookie auth off so lightwalletd / Nozy JSON-RPC clients work without Zebra cookie files.
    Invoke-WslBash "source ~/.cargo/env 2>/dev/null; mkdir -p ~/.cache/zebra; nohup env ZEBRA_RPC__LISTEN_ADDR=$ZebraRpcListen ZEBRA_RPC__ENABLE_COOKIE_AUTH=false zebrad start > ~/.cache/zebra/zebrad-run.log 2>&1 < /dev/null &"
}

# 2) Wait for RPC port
$rpcUp = $false
for ($i = 0; $i -lt $RpcWaitSeconds; $i++) {
    $state = (Invoke-WslBash "python3 - <<'PY'
import socket
s = socket.socket()
s.settimeout(1)
try:
    s.connect(('127.0.0.1', 8232))
    print('OPEN')
except Exception:
    print('CLOSED')
finally:
    s.close()
PY").Trim()

    if ($state -eq "OPEN") {
        $rpcUp = $true
        break
    }
    Start-Sleep -Seconds 1
}

if ($rpcUp) {
    Write-Host "Zebra RPC: 127.0.0.1:8232 is OPEN" -ForegroundColor Green
} else {
    Write-Warning "Zebra RPC did not open within $RpcWaitSeconds seconds. Check ~/.cache/zebra/zebrad-run.log in WSL."
}

if ($SkipDesktop) {
    Write-Host "SkipDesktop set; exiting after Zebra checks."
    exit 0
}

# 3) Cleanup stale desktop processes that commonly block port 5173
if (-not $SkipCleanup) {
    Write-Host "Cleaning stale Nozy desktop processes..." -ForegroundColor Yellow
    Invoke-WslBash "pkill -f 'node .*vite|cargo tauri dev|nozy-wallet-desktop' >/dev/null 2>&1 || true"
    Start-Sleep -Seconds 1
}

# 4) Launch Nozy desktop in foreground
Write-Host "Launching Nozy desktop..." -ForegroundColor Cyan
Write-Host "Tip: use the desktop window, not localhost:5173 in a browser."
Invoke-WslBash "source ~/.cargo/env 2>/dev/null; cd $DesktopPath; export LIBGL_ALWAYS_SOFTWARE=1 WEBKIT_DISABLE_DMABUF_RENDERER=1 GDK_BACKEND=x11 WINIT_UNIX_BACKEND=x11; cargo tauri dev"
