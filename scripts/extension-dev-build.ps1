Param(
    [switch]$SkipPopup
)

$ErrorActionPreference = "Stop"

# Prefer rustup + LLVM over a standalone MSVC Rust install (wasm32 stdlib lives in rustup).
$env:PATH = @(
    "$env:USERPROFILE\.cargo\bin",
    "C:\Program Files\LLVM\bin",
    $env:PATH
) -join ";"

$repoRoot = Split-Path -Parent $PSScriptRoot
$wasmCore = Join-Path $repoRoot "browser-extension\wasm-core"
$popup = Join-Path $wasmCore "popup"

Write-Host "== Nozy extension dev build ==" -ForegroundColor Cyan

if (-not (Get-Command wasm-pack -ErrorAction SilentlyContinue)) {
    Write-Host "Installing wasm-pack 0.12.1 (compatible with common stable toolchains)..." -ForegroundColor Yellow
    & cargo install wasm-pack --version 0.12.1 --locked
}

$installedTargets = & rustup target list --installed
if ($installedTargets -notmatch "^wasm32-unknown-unknown$") {
    & rustup target add wasm32-unknown-unknown
}

Push-Location $wasmCore
try {
    Write-Host "-> wasm-pack build" -ForegroundColor Yellow
    & wasm-pack build --target web --out-dir ../wasm/pkg --release
    if ($LASTEXITCODE -ne 0) { throw "wasm-pack failed ($LASTEXITCODE)" }
} finally {
    Pop-Location
}

if (-not $SkipPopup) {
    Push-Location $popup
    try {
        Write-Host "-> popup npm ci + build" -ForegroundColor Yellow
        & npm ci
        if ($LASTEXITCODE -ne 0) { throw "npm ci failed ($LASTEXITCODE)" }
        & npm run build
        if ($LASTEXITCODE -ne 0) { throw "popup build failed ($LASTEXITCODE)" }
    } finally {
        Pop-Location
    }
}

Write-Host ""
Write-Host "Ready to load unpacked:" -ForegroundColor Green
Write-Host "  $((Join-Path $repoRoot 'browser-extension'))" -ForegroundColor White
Write-Host "Chrome: chrome://extensions -> Developer mode -> Load unpacked" -ForegroundColor DarkYellow
