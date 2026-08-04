#!/usr/bin/env bash
# Smoke-test nozywallet-api (companion) against a running instance.
#
# Usage:
#   ./api-server/scripts/smoke-companion.sh
#   BASE_URL=http://127.0.0.1:3000 ./api-server/scripts/smoke-companion.sh
#
# Exit codes:
#   0 - /health succeeded (other probes may soft-fail)
#   1 - /health failed or unreachable
#
# Soft failures (LWD / optional routes) print WARN and do not fail the script.

set -u
BASE_URL="${BASE_URL:-http://127.0.0.1:3000}"
BASE_URL="${BASE_URL%/}"
failed_hard=0

probe() {
  local path="$1"
  local soft="${2:-0}"
  local url="${BASE_URL}${path}"
  local code
  code=$(curl -sS -o /tmp/nozy-smoke-body.$$ -w "%{http_code}" --connect-timeout 5 --max-time 10 "$url" 2>/tmp/nozy-smoke-err.$$ || true)
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

probe "/api/lwd/info" 1
probe "/api/lwd/chain-tip" 1
probe "/api/wallet/exists" 1
probe "/api/config" 1

echo ""
if [[ "$failed_hard" -ne 0 ]]; then
  echo "Smoke FAILED: /health did not succeed."
  exit 1
fi
echo "Smoke PASSED: /health OK (soft probes reported above)."
exit 0