# Run cargo-audit for dependency vulnerability scanning

Write-Host "=== Running cargo-audit ===" -ForegroundColor Cyan
Write-Host ""

# Check if cargo-audit is installed
if (-not (Get-Command cargo-audit -ErrorAction SilentlyContinue)) {
    Write-Host "cargo-audit not found. Installing..." -ForegroundColor Yellow
    cargo install cargo-audit
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Failed to install cargo-audit" -ForegroundColor Red
        exit 1
    }
    Write-Host "cargo-audit installed successfully" -ForegroundColor Green
    Write-Host ""
}

Write-Host "Running cargo audit..." -ForegroundColor Yellow
cargo audit

if ($LASTEXITCODE -eq 0) {
    Write-Host "`n✅ No vulnerabilities found!" -ForegroundColor Green
} else {
    Write-Host "`n⚠️  Vulnerabilities found. Review output above." -ForegroundColor Yellow
}

Write-Host "`n=== Complete ===" -ForegroundColor Cyan
