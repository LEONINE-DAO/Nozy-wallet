#!/usr/bin/env bash
# Start nozywallet-api (zeaking HTTP) from WSL/Linux — same binary as Windows PowerShell script.
#
# Usage (from Nozy-wallet clone):
#   bash scripts/run-nozy-api.sh
#   bash scripts/run-nozy-api.sh 3000    # pin HTTP port
#
# Env:
#   LIGHTWALLETD_GRPC  default http://127.0.0.1:9067 (WSL: lightwalletd on same machine)
#   NOZY_HTTP_PORT      if set, skips auto-pick
#
# Note: run-nozy-api.ps1 is Windows PowerShell only; in WSL bash use this file, not *.ps1.
# WSL: install protoc before first build: sudo apt install protobuf-compiler

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ ! -f "$ROOT/api-server/Cargo.toml" ]]; then
  echo "run-nozy-api.sh: not a Nozy-wallet repo root (missing api-server/). Computed ROOT=$ROOT" >&2
  echo "Fix: cd to your Nozy-wallet clone, then: bash scripts/run-nozy-api.sh" >&2
  echo "Do not run from the Zebrad-only repo — it has no scripts/run-nozy-api.sh." >&2
  exit 1
fi
export LIGHTWALLETD_GRPC="${LIGHTWALLETD_GRPC:-http://127.0.0.1:9067}"

tcp_port_busy() {
  local p="$1"
  # Something accepting TCP on 127.0.0.1:p => busy for our purposes
  bash -c "echo >/dev/tcp/127.0.0.1/${p}" &>/dev/null
}

pick_free_port() {
  local p
  for p in $(seq 3000 3100); do
    if ! tcp_port_busy "$p"; then
      echo "$p"
      return 0
    fi
  done
  echo "run-nozy-api.sh: no free port in 3000-3100" >&2
  return 1
}

if [[ -n "${NOZY_HTTP_PORT:-}" ]]; then
  export NOZY_HTTP_PORT
elif [[ "${1:-}" =~ ^[0-9]+$ ]]; then
  export NOZY_HTTP_PORT="$1"
  shift
else
  NOZY_HTTP_PORT="$(pick_free_port)"
  export NOZY_HTTP_PORT
  if [[ "$NOZY_HTTP_PORT" != "3000" ]]; then
    echo "Using NOZY_HTTP_PORT=$NOZY_HTTP_PORT (3000 busy). Extension companion baseUrl must match." >&2
  fi
fi

echo "LIGHTWALLETD_GRPC=$LIGHTWALLETD_GRPC"
echo "NOZY_HTTP_PORT=$NOZY_HTTP_PORT"
echo "Listening: http://127.0.0.1:$NOZY_HTTP_PORT"
cd "$ROOT"
exec cargo run -p nozywallet-api --release "$@"
