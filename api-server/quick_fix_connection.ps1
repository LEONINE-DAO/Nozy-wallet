# Quick fix script for cargo connection errors
# This script tries multiple solutions in order

param(
    [switch]$Offline,
    [switch]$Clean
)

Write-Host "Quick Fix for Cargo Connection Errors" -ForegroundColor Cyan
Write-Host ""

if ($Offline) {
    Write-Host "Attempting offline build..." -ForegroundColor Yellow
    cargo build --release --offline
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Success! Build completed offline." -ForegroundColor Green
        exit 0
    } else {
        Write-Host "Offline build failed. Trying other solutions..." -ForegroundColor Yellow
    }
}

if ($Clean) {
    Write-Host "Cleaning cargo cache..." -ForegroundColor Yellow
    cargo clean
}

Write-Host "Setting increased timeout..." -ForegroundColor Yellow
$env:CARGO_NET_TIMEOUT = "300"

Write-Host "Attempting build with increased timeout..." -ForegroundColor Yellow
cargo build --release

if ($LASTEXITCODE -eq 0) {
    Write-Host "Success! Build completed." -ForegroundColor Green
} else {
    Write-Host "Build still failing. Try running: .\fix_connection_error.ps1" -ForegroundColor Red
    Write-Host "Or check CONNECTION_ERROR_FIX.md for more solutions." -ForegroundColor Yellow
}

