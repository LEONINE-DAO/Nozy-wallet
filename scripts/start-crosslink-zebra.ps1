# Start Crosslink Season zebrad in WSL as a background systemd user service.
# Survives Cursor close; auto-restarts on crash.
$ErrorActionPreference = "Stop"

$script = @'
set -euo pipefail
UNIT="$HOME/.config/systemd/user/zebrad-crosslink.service"
if [[ ! -f "$UNIT" ]]; then
  echo "Service not installed. Run: bash /mnt/c/Users/User/NozyWallet/scripts/setup-crosslink-node.sh"
  exit 1
fi
systemctl --user daemon-reload
systemctl --user enable zebrad-crosslink.service >/dev/null
systemctl --user start zebrad-crosslink.service
sleep 2
systemctl --user --no-pager --lines=8 status zebrad-crosslink.service || true
curl -sS --max-time 5 \
  --data '{"jsonrpc":"2.0","id":1,"method":"getblockcount","params":[]}' \
  -H 'content-type: application/json' \
  http://127.0.0.1:18232 || echo "RPC warming up..."
echo
'@

Write-Host "Starting Crosslink zebrad (systemd user service)..."
wsl.exe -d Ubuntu -- bash -lc $script
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host "Crosslink is running in the background (independent of Cursor)."
Write-Host "RPC from Windows: http://172.20.199.206:18232  (or current WSL IP)"
