#!/usr/bin/env bash
set -euo pipefail

# Usage (from Nozy-wallet repo):
#   bash scripts/run-nozy-wsl.sh
# Optional env:
#   NOZY_DESKTOP_DIR=~/projects/Nozy-wallet/desktop-client
#   ZEBRA_RPC_LISTEN=0.0.0.0:8232
#   RPC_WAIT_SECONDS=90

NOZY_DESKTOP_DIR="${NOZY_DESKTOP_DIR:-$HOME/projects/Nozy-wallet/desktop-client}"
ZEBRA_RPC_LISTEN="${ZEBRA_RPC_LISTEN:-0.0.0.0:8232}"
RPC_WAIT_SECONDS="${RPC_WAIT_SECONDS:-90}"

echo "== Nozy WSL launcher =="
echo "Desktop dir: $NOZY_DESKTOP_DIR"

if pgrep -af '^zebrad start' >/dev/null; then
  echo "Zebra: already running"
else
  echo "Zebra: starting..."
  source "$HOME/.cargo/env" 2>/dev/null || true
  mkdir -p "$HOME/.cache/zebra"
  nohup env ZEBRA_RPC__LISTEN_ADDR="$ZEBRA_RPC_LISTEN" ZEBRA_RPC__ENABLE_COOKIE_AUTH=false zebrad start >"$HOME/.cache/zebra/zebrad-run.log" 2>&1 < /dev/null &
fi

echo "Waiting for Zebra RPC on 127.0.0.1:8232 ..."
rpc_up=0
for ((i=0; i<RPC_WAIT_SECONDS; i++)); do
  if python3 - <<'PY'
import socket
s = socket.socket()
s.settimeout(1)
ok = False
try:
    s.connect(("127.0.0.1", 8232))
    ok = True
except Exception:
    pass
finally:
    s.close()
raise SystemExit(0 if ok else 1)
PY
  then
    rpc_up=1
    break
  fi
  sleep 1
done

if [[ "$rpc_up" -eq 1 ]]; then
  echo "Zebra RPC: OPEN"
else
  echo "WARNING: RPC did not open in ${RPC_WAIT_SECONDS}s."
  echo "Check log: $HOME/.cache/zebra/zebrad-run.log"
fi

echo "Cleaning stale desktop processes..."
pkill -f 'node .*vite|cargo tauri dev|nozy-wallet-desktop' >/dev/null 2>&1 || true
sleep 1

echo "Launching Nozy desktop..."
source "$HOME/.cargo/env" 2>/dev/null || true
cd "$NOZY_DESKTOP_DIR"
export LIBGL_ALWAYS_SOFTWARE=1
export WEBKIT_DISABLE_DMABUF_RENDERER=1
export GDK_BACKEND=x11
export WINIT_UNIX_BACKEND=x11
cargo tauri dev
