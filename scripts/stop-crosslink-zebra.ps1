# Stop Crosslink Season zebrad systemd user service in WSL.
$ErrorActionPreference = "Stop"

$script = @'
systemctl --user stop zebrad-crosslink.service 2>/dev/null || true
# Also clear any leftover one-shot process.
pkill -f 'zebra-crosslink/zebrad.toml' 2>/dev/null || true
sleep 1
if systemctl --user is-active --quiet zebrad-crosslink.service; then
  echo "STILL_ACTIVE"
  systemctl --user --no-pager status zebrad-crosslink.service || true
  exit 1
fi
echo "STOPPED"
ss -ltn | grep -E '18232|18233' || echo "ports free"
'@

Write-Host "Stopping Crosslink zebrad..."
wsl.exe -d Ubuntu -- bash -lc $script
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host "Crosslink stopped."
