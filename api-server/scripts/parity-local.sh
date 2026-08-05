#!/usr/bin/env bash
# Local non-chain parity helper: exercise companion routes that do not need Zebrad/LWD.
#
# Prerequisites: nozywallet-api listening on BASE_URL (default http://127.0.0.1:3000).
# Optional: NOZY_API_KEY if the server requires auth.
#
# Does NOT create/restore wallets (avoids writing seeds in CI). Chain-backed
# rows in PARITY.md remain manual field sign-off.
#
# Usage:
#   ./api-server/scripts/parity-local.sh
#   BASE_URL=http://127.0.0.1:3000 ./api-server/scripts/parity-local.sh

set -u
BASE_URL="${BASE_URL:-http://127.0.0.1:3000}"
BASE_URL="${BASE_URL%/}"
failed=0
CURL_AUTH=()
if [[ -n "${NOZY_API_KEY:-}" ]]; then
  CURL_AUTH=(-H "X-API-Key: ${NOZY_API_KEY}")
fi

check() {
  local method="$1"
  local path="$2"
  local expect="$3"
  local url="${BASE_URL}${path}"
  local code
  if [[ "$method" == "GET" ]]; then
    code=$(curl -sS "${CURL_AUTH[@]}" -o /tmp/nozy-parity-body.$$ -w "%{http_code}" --connect-timeout 5 --max-time 15 "$url" 2>/tmp/nozy-parity-err.$$ || true)
  else
    code=$(curl -sS "${CURL_AUTH[@]}" -X "$method" -H "Content-Type: application/json" -d "${4:-{}}" -o /tmp/nozy-parity-body.$$ -w "%{http_code}" --connect-timeout 5 --max-time 15 "$url" 2>/tmp/nozy-parity-err.$$ || true)
  fi
  if [[ "$code" == "$expect" ]] || [[ "$expect" == "2xx" && "$code" =~ ^2 ]]; then
    echo "OK   ${method} ${path}  (${code})"
    return 0
  fi
  echo "FAIL ${method} ${path}  (got ${code}, want ${expect})"
  failed=1
  return 1
}

cleanup() {
  rm -f /tmp/nozy-parity-body.$$ /tmp/nozy-parity-err.$$
}
trap cleanup EXIT

echo "Local (non-chain) companion parity probes at ${BASE_URL}"
echo ""

check GET /health 2xx
check GET /api/wallet/exists 2xx
check GET /api/config 2xx
# Status may 500 with no wallet / no notes — treat as reachable if we get an HTTP response.
code=$(curl -sS "${CURL_AUTH[@]}" -o /tmp/nozy-parity-body.$$ -w "%{http_code}" --connect-timeout 5 --max-time 15 "${BASE_URL}/api/wallet/status" 2>/dev/null || true)
if [[ "$code" =~ ^[245] ]]; then
  echo "OK   GET /api/wallet/status  (${code})  [shape probe]"
else
  echo "FAIL GET /api/wallet/status  (got ${code})"
  failed=1
fi
# Fee estimate may 200 with defaults or 4xx/5xx if node missing — accept both shapes as "reachable".
code=$(curl -sS "${CURL_AUTH[@]}" -o /tmp/nozy-parity-body.$$ -w "%{http_code}" --connect-timeout 5 --max-time 15 "${BASE_URL}/api/transaction/fee-estimate" 2>/dev/null || true)
if [[ "$code" =~ ^[245] ]]; then
  echo "OK   GET /api/transaction/fee-estimate  (${code})  [shape probe]"
else
  echo "FAIL GET /api/transaction/fee-estimate  (got ${code})"
  failed=1
fi

echo ""
if [[ "$failed" -ne 0 ]]; then
  echo "Local parity FAILED. See PARITY.md for full CLI↔API field sign-off."
  exit 1
fi
echo "Local parity PASSED (non-chain). Sign off chain rows in PARITY.md against your node."
exit 0
