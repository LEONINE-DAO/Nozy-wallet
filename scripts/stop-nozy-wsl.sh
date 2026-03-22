#!/usr/bin/env bash
set -euo pipefail

# Usage:
#   bash scripts/stop-nozy-wsl.sh
#   bash scripts/stop-nozy-wsl.sh --keep-zebra

KEEP_ZEBRA=0
if [[ "${1:-}" == "--keep-zebra" ]]; then
  KEEP_ZEBRA=1
fi

echo "Stopping Nozy desktop processes..."
pkill -f 'node .*vite|cargo tauri dev|nozy-wallet-desktop' >/dev/null 2>&1 || true

if [[ "$KEEP_ZEBRA" -eq 0 ]]; then
  echo "Stopping Zebra..."
  pkill -f '^zebrad start' >/dev/null 2>&1 || true
else
  echo "Keeping Zebra running."
fi

sleep 1
if pgrep -af 'node .*vite|cargo tauri dev|nozy-wallet-desktop' >/dev/null; then
  echo "Desktop: RUNNING"
else
  echo "Desktop: STOPPED"
fi

if pgrep -af '^zebrad start' >/dev/null; then
  echo "Zebra: RUNNING"
else
  echo "Zebra: STOPPED"
fi
