#!/usr/bin/env bash
set -euo pipefail

LOG="/home/lowo/crosslink-season1-v13/logs/zebrad.log"
PIDFILE="/home/lowo/crosslink-season1-v13/zebrad.pid"

echo "=== tail log ==="
tail -50 "$LOG" 2>/dev/null || echo "no log yet"

echo "=== ports ==="
ss -ltn | awk 'NR==1 || /:18232|:18233/'

echo "=== process ==="
if [[ -f "$PIDFILE" ]]; then
  pid="$(cat "$PIDFILE")"
  ps -p "$pid" -o pid,cmd || echo "pid $pid not running"
else
  echo "no pidfile"
fi

echo "=== rpc ==="
curl -sS -H 'Content-Type: application/json' \
  --data '{"jsonrpc":"2.0","method":"getblockcount","params":[],"id":1}' \
  http://127.0.0.1:18232/ || echo "RPC failed"
