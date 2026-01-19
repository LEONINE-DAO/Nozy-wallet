# Run cargo-chec for parallel cargo checks
# cargo-chec runs multiple cargo checks in parallel and outputs JSON

Write-Host "=== Running cargo-chec ===" -ForegroundColor Cyan
Write-Host ""

$checInstalled = cargo chec --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "cargo-chec not found. Installing..." -ForegroundColor Yellow
    cargo install cargo-chec
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Failed to install cargo-chec" -ForegroundColor Red
        Write-Host "Make sure you have Rust and Cargo installed" -ForegroundColor Yellow
        exit 1
    }
    Write-Host "cargo-chec installed successfully" -ForegroundColor Green
    Write-Host ""
}

Write-Host "Running cargo chec (parallel checks)..." -ForegroundColor Yellow
Write-Host "This will run: cargo check, cargo clippy, cargo fmt, and cargo test" -ForegroundColor Cyan
Write-Host ""

cargo chec

if ($LASTEXITCODE -eq 0) {
    Write-Host "`n✅ All checks passed!" -ForegroundColor Green
} else {
    Write-Host "`n⚠️  Some checks failed. Review output above." -ForegroundColor Yellow
}

Write-Host "`n=== Complete ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "Note: cargo-chec outputs JSON by default." -ForegroundColor Cyan
Write-Host "For human-readable output, use: cargo chec --pretty" -ForegroundColor Cyan
