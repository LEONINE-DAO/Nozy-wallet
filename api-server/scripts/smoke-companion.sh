#!/usr/bin/env bash
# Smoke-test nozywallet-api (companion) against a running instance.
#
# Usage:
#   ./api-server/scripts/smoke-companion.sh
#   BASE_URL=http://127.0.0.1:3000 ./api-server/scripts/smoke-companion.sh
#   NOZY_API_KEY=secret ./api-server/scripts/smoke-companion.sh
#
# Exit codes:
#   0 - /health, /api/wallet/exists, and /api/config succeeded
#   1 - a hard probe failed
#
# Soft failures (LWD) print WARN and do not fail the script.

set -u
BASE_URL="${BASE_URL:-http://127.0.0.1:3000}"
BASE_URL="${BASE_URL%/}"
failed_hard=0

# Auto-generated companion_api_key (loopback default). CI unsets NOZY_API_KEY.
if [[ -z "${NOZY_API_KEY:-}" ]]; then
  search_root="${XDG_DATA_HOME:-${HOME:-}/.local/share}"
  if [[ -d "$search_root" ]]; then
    found_key=$(find "$search_root" -name companion_api_key -type f 2>/dev/null | head -n 1 || true)
    if [[ -n "${found_key}" ]]; then
      NOZY_API_KEY=$(tr -d '[:space:]' < "$found_key")
      echo "Using companion API key from ${found_key}"
    fi
  fi
fi

CURL_AUTH=()
if [[ -n "${NOZY_API_KEY:-}" ]]; then
  CURL_AUTH=(-H "X-API-Key: ${NOZY_API_KEY}")
fi

probe() {
  local path="$1"
  local soft="${2:-0}"
  local url="${BASE_URL}${path}"
  local code
  code=$(curl -sS "${CURL_AUTH[@]}" -o /tmp/nozy-smoke-body.$$ -w "%{http_code}" --connect-timeout 5 --max-time 10 "$url" 2>/tmp/nozy-smoke-err.$$ || true)
  if [[ "$code" =~ ^2 ]]; then
    echo "OK   ${path}  (${code})"
    return 0
  fi
  local err
  err=$(cat /tmp/nozy-smoke-err.$$ 2>/dev/null || true)
  if [[ "$soft" == "1" ]]; then
    echo "WARN ${path}  (soft fail: http=${code} ${err})"
    return 0
  fi
  echo "FAIL ${path}  (http=${code} ${err})"
  return 1
}

cleanup() {
  rm -f /tmp/nozy-smoke-body.$$ /tmp/nozy-smoke-err.$$
}
trap cleanup EXIT

echo "Smoke companion API at ${BASE_URL}"
echo ""

if ! probe "/health" 0; then
  failed_hard=1
fi

# No chain required — these must work on a fresh boot.
if ! probe "/api/wallet/exists" 0; then
  failed_hard=1
fi
if ! probe "/api/config" 0; then
  failed_hard=1
fi

# Soft: need lightwalletd.
probe "/api/lwd/info" 1
probe "/api/lwd/chain-tip" 1

echo ""
if [[ "$failed_hard" -ne 0 ]]; then
  echo "Smoke FAILED: one or more hard probes did not succeed."
  exit 1
fi
echo "Smoke PASSED: hard probes OK (soft LWD probes reported above)."
exit 0
