# Start lightwalletd in WSL (backend: zebrad on 127.0.0.1:8232 inside WSL).
# gRPC listens on 0.0.0.0:9067 so Windows Nozy api-server can reach it via WSL IP.
#
# First-time setup (run once inside WSL):
#   sudo apt install -y golang-go protobuf-compiler
#   git clone https://github.com/zcash/lightwalletd ~/lightwalletd
#   cd ~/lightwalletd && make && make install
#   mkdir -p ~/.config && touch ~/.config/zcash.conf
#   mkdir -p ~/.cache/lightwalletd
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File scripts\start-lightwalletd-wsl.ps1 -Status
#   powershell -ExecutionPolicy Bypass -File scripts\start-lightwalletd-wsl.ps1
#   powershell -ExecutionPolicy Bypass -File scripts\start-lightwalletd-wsl.ps1 -Stop

param(
    [string]$Distro = "Ubuntu",
    [int]$GrpcPort = 9067,
    [switch]$Status,
    [switch]$Stop
)

$ErrorActionPreference = "Stop"

function Invoke-Wsl([string]$Cmd) {
    wsl -d $Distro -- bash -lc $Cmd
}

$LwdBin = '$HOME/go/bin/lightwalletd'
$DataDir = '$HOME/.cache/lightwalletd'
$Conf = '$HOME/.config/zcash.conf'
$Log = '$HOME/.cache/lightwalletd/lightwalletd.log'
$Listen = "0.0.0.0:${GrpcPort}"

if ($Stop) {
    Invoke-Wsl "pkill -f 'lightwalletd.*${GrpcPort}' || pkill -x lightwalletd || true"
    Write-Host "Stopped lightwalletd (if running)." -ForegroundColor Yellow
    exit 0
}

if ($Status) {
    Write-Host "=== WSL lightwalletd process ===" -ForegroundColor Cyan
    Invoke-Wsl "pgrep -a lightwalletd || echo '(not running)'"
    Write-Host "`n=== gRPC port $GrpcPort ===" -ForegroundColor Cyan
    Invoke-Wsl "ss -ltn | grep ':${GrpcPort}' || echo '(not listening)'"
    $ip = (Invoke-Wsl "hostname -I | awk '{print `$1}'").Trim()
    if ($ip) {
        Write-Host "`nFrom Windows use: LIGHTWALLETD_GRPC=http://${ip}:${GrpcPort}" -ForegroundColor Green
    }
    exit 0
}

$check = @"
test -x $LwdBin || { echo 'MISSING: run make install in ~/lightwalletd'; exit 1; }
if [[ ! -s $Conf ]] || ! grep -q '^rpcpassword=' $Conf 2>/dev/null; then
  mkdir -p ~/.config
  printf '%s\n' '# zebrad ignores rpcuser/rpcpassword when enable_cookie_auth=false' \
    'rpcuser=zebra' 'rpcpassword=local' 'rpcport=8232' 'rpcbind=127.0.0.1' > $Conf
fi
mkdir -p $DataDir
pgrep -x lightwalletd >/dev/null && { echo 'already running'; exit 0; }
nohup $LwdBin --grpc-bind-addr $Listen --http-bind-addr 127.0.0.1:8080 \
  --zcash-conf-path $Conf --data-dir $DataDir --log-file $Log \
  --no-tls-very-insecure >> $Log 2>&1 &
sleep 2
pgrep -a lightwalletd || { tail -20 $Log; exit 1; }
"@

Write-Host "Starting lightwalletd in WSL ($Distro) on $Listen ..." -ForegroundColor Cyan
Invoke-Wsl $check
& $PSScriptRoot\start-lightwalletd-wsl.ps1 -Status
