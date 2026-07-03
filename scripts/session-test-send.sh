#!/usr/bin/env bash
# Quick session prep: validate long u1 address + optional send test
set -euo pipefail

API="${1:-http://127.0.0.1:3000}"
REPORTER_UA="u13nkpl0xejf50y2l2nwq44jeg6u28ayey0k80htxspz6vqfa4zru4v45ez7n3qz9c3e6h29m89w4ket6wlmpgpq4ra4f7gd42uyp7c94e"

echo "=== health ==="
curl -s "$API/health"
echo

echo "=== wallet exists ==="
curl -s "$API/api/wallet/exists"
echo

echo "=== lwd info ==="
curl -s "$API/api/lwd/info"
echo

echo "=== address validation test (reporter 106-char u1) ==="
curl -s -X POST "$API/api/transaction/send" \
  -H "Content-Type: application/json" \
  -d "{\"recipient\":\"$REPORTER_UA\",\"amount\":0.01,\"password\":\"\"}"
echo
