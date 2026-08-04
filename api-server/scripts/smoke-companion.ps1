# Smoke-test nozywallet-api (companion) against a running instance.
#
# Usage:
#   .\api-server\scripts\smoke-companion.ps1
#   .\api-server\scripts\smoke-companion.ps1 -BaseUrl http://127.0.0.1:3000
#
# Exit codes:
#   0 - /health succeeded (other probes may soft-fail)
#   1 - /health failed or unreachable
#
# Soft failures (LWD / optional routes) print WARN and do not fail the script.

param(
    [string]$BaseUrl = "http://127.0.0.1:3000"
)

$ErrorActionPreference = "Continue"
$BaseUrl = $BaseUrl.TrimEnd("/")
$failedHard = $false

function Invoke-Probe {
    param(
        [string]$Path,
        [switch]$Soft
    )
    $url = "$BaseUrl$Path"
    try {
        $resp = Invoke-WebRequest -Uri $url -Method GET -UseBasicParsing -TimeoutSec 10
        Write-Host "OK   $Path  ($($resp.StatusCode))"
        return $true
    } catch {
        $msg = $_.Exception.Message
        if ($Soft) {
            Write-Host "WARN $Path  (soft fail: $msg)"
            return $false
        }
        Write-Host "FAIL $Path  ($msg)"
        return $false
    }
}

Write-Host "Smoke companion API at $BaseUrl"
Write-Host ""

if (-not (Invoke-Probe -Path "/health")) {
    $failedHard = $true
}

Invoke-Probe -Path "/api/lwd/info" -Soft | Out-Null
Invoke-Probe -Path "/api/lwd/chain-tip" -Soft | Out-Null
Invoke-Probe -Path "/api/wallet/exists" -Soft | Out-Null
Invoke-Probe -Path "/api/config" -Soft | Out-Null

Write-Host ""
if ($failedHard) {
    Write-Host "Smoke FAILED: /health did not succeed."
    exit 1
}
Write-Host "Smoke PASSED: /health OK (soft probes reported above)."
exit 0