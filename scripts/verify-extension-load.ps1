# Verify unpacked extension paths and background module exports (catches SW registration failures).
#
# Usage:
#   .\scripts\verify-extension-load.ps1

$ErrorActionPreference = "Stop"
$Root = Join-Path (Split-Path $PSScriptRoot -Parent) "browser-extension"

function Fail($msg) {
    Write-Host "FAIL: $msg" -ForegroundColor Red
    exit 1
}

function Pass($msg) {
    Write-Host "PASS: $msg" -ForegroundColor Green
}

$manifestPath = Join-Path $Root "manifest.json"
if (-not (Test-Path $manifestPath)) { Fail "Missing $manifestPath" }

$manifest = Get-Content $manifestPath -Raw | ConvertFrom-Json
$paths = @(
    $manifest.background.service_worker,
    $manifest.action.default_popup
)
foreach ($rel in $paths) {
    $full = Join-Path $Root ($rel -replace '/', '\')
    if (-not (Test-Path -LiteralPath $full)) {
        Fail "Manifest path missing: $rel"
    }
    Pass "Found $rel"
}

$wasmJs = Join-Path $Root "wasm\pkg\nozy_wasm.js"
$wasmBin = Join-Path $Root "wasm\pkg\nozy_wasm_bg.wasm"
foreach ($p in @($wasmJs, $wasmBin)) {
    if (-not (Test-Path $p)) {
        Fail "Missing WASM artifact (run scripts\extension-dev-build.ps1): $p"
    }
}
Pass "WASM pkg present"

$swPath = Join-Path $Root "background\service-worker.js"
$swText = Get-Content $swPath -Raw
if ($swText -match 'from\s+"\./rpc-utils\.js"') {
    $importBlock = [regex]::Match(
        $swText,
        'import\s*\{([^}]+)\}\s*from\s*"\./rpc-utils\.js"'
    ).Groups[1].Value
    $wanted = $importBlock -split ',' | ForEach-Object { $_.Trim() } | Where-Object { $_ }
    $rpcPath = Join-Path $Root "background\rpc-utils.js"
    $rpcText = Get-Content $rpcPath -Raw
    foreach ($name in $wanted) {
        if ($rpcText -notmatch "export\s+(async\s+)?function\s+$name\b") {
            Fail "rpc-utils.js does not export '$name' (service worker will fail to register). Save rpc-utils.js and Reload extension."
        }
    }
    Pass "rpc-utils exports match service-worker imports ($($wanted.Count) symbols)"
}

Write-Host ""
Write-Host "Extension folder ready to Load unpacked:" -ForegroundColor Cyan
Write-Host "  $Root"
Write-Host "Then chrome://extensions -> Reload on NozyWallet." -ForegroundColor DarkYellow
