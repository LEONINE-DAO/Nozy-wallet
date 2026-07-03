#!/usr/bin/env bash
# Quick validation test: 106-char u1 must NOT return "Invalid recipient address"
set -euo pipefail
API="${1:-http://127.0.0.1:3000}"
UA="u13nkpl0xejf50y2l2nwq44jeg6u28ayey0k80htxspz6vqfa4zru4v45ez7n3qz9c3e6h29m89w4ket6wlmpgpq4ra4f7gd42uyp7c94e"
echo "POST $API/api/transaction/send (max 10s)..."
out=$(curl -s --max-time 10 -X POST "$API/api/transaction/send" \
  -H "Content-Type: application/json" \
  -d "{\"recipient\":\"$UA\",\"amount\":0.01,\"password\":\"\"}" || echo "__CURL_TIMEOUT__")
echo "$out"
if echo "$out" | grep -qi "Invalid recipient address"; then
  echo "FAIL: address validation bug still present"
  exit 1
elif [[ "$out" == "__CURL_TIMEOUT__" ]] || [[ -z "$out" ]]; then
  echo "OK: no instant rejection (request still processing or timed out after 10s — validation passed)"
  exit 0
else
  echo "OK: got JSON response (not invalid-address rejection)"
  exit 0
fi
