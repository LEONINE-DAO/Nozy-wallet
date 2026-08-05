# Smoke-test nozywallet-api (companion) against a running instance.
#
# Usage:
#   .\api-server\scripts\smoke-companion.ps1
#   .\api-server\scripts\smoke-companion.ps1 -BaseUrl http://127.0.0.1:3000
#   $env:NOZY_API_KEY = 'secret'; .\api-server\scripts\smoke-companion.ps1
#
# Exit codes:
#   0 - /health, /api/wallet/exists, and /api/config succeeded
#   1 - a hard probe failed
#
# Soft failures (LWD) print WARN and do not fail the script.

param(
    [string]$BaseUrl = "http://127.0.0.1:3000",
    [string]$ApiKey = $env:NOZY_API_KEY
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
    $headers = @{}
    if (-not [string]::IsNullOrWhiteSpace($ApiKey)) {
        $headers["X-API-Key"] = $ApiKey
    }
    try {
        $resp = Invoke-WebRequest -Uri $url -Method GET -Headers $headers -UseBasicParsing -TimeoutSec 10
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
if (-not (Invoke-Probe -Path "/api/wallet/exists")) {
    $failedHard = $true
}
if (-not (Invoke-Probe -Path "/api/config")) {
    $failedHard = $true
}

Invoke-Probe -Path "/api/lwd/info" -Soft | Out-Null
Invoke-Probe -Path "/api/lwd/chain-tip" -Soft | Out-Null

Write-Host ""
if ($failedHard) {
    Write-Host "Smoke FAILED: one or more hard probes did not succeed."
    exit 1
}
Write-Host "Smoke PASSED: hard probes OK (soft LWD probes reported above)."
exit 0
