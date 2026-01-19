# NozyWallet Security Audit Script

Write-Host "=== NozyWallet Self-Security Audit ===" -ForegroundColor Cyan
Write-Host ""

Write-Host "1. Checking for cargo-audit..." -ForegroundColor Yellow
$auditInstalled = cargo audit --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "   ⚠️  cargo-audit not installed. Install with: cargo install cargo-audit" -ForegroundColor Yellow
} else {
    Write-Host "   ✅ cargo-audit installed" -ForegroundColor Green
    Write-Host "   Running dependency audit..." -ForegroundColor Cyan
    cargo audit
}

Write-Host "`n1.5. Running cargo-chec (parallel checks)..." -ForegroundColor Yellow
$checInstalled = cargo chec --version 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "   ⚠️  cargo-chec not installed. Install with: cargo install cargo-chec" -ForegroundColor Yellow
    Write-Host "   Skipping cargo-chec checks..." -ForegroundColor Yellow
} else {
    Write-Host "   ✅ cargo-chec installed" -ForegroundColor Green
    Write-Host "   Running parallel checks (check, clippy, fmt, test)..." -ForegroundColor Cyan
    cargo chec 2>&1 | Select-Object -First 30
}

Write-Host "`n2. Running cargo clippy (security checks)..." -ForegroundColor Yellow
cargo clippy -- -D warnings 2>&1 | Select-Object -First 50

Write-Host "`n3. Searching for unsafe code patterns..." -ForegroundColor Yellow
$unsafeCount = (Select-String -Path "src\*.rs" -Pattern "unsafe" -Recurse).Count
Write-Host "   Found $unsafeCount unsafe blocks" -ForegroundColor $(if ($unsafeCount -eq 0) { "Green" } else { "Yellow" })

Write-Host "`n4. Searching for unwrap() calls..." -ForegroundColor Yellow
$unwrapCount = (Select-String -Path "src\*.rs" -Pattern "\.unwrap\(\)" -Recurse).Count
Write-Host "   Found $unwrapCount unwrap() calls" -ForegroundColor $(if ($unwrapCount -lt 10) { "Green" } elseif ($unwrapCount -lt 50) { "Yellow" } else { "Red" })

Write-Host "`n5. Searching for panic! calls..." -ForegroundColor Yellow
$panicCount = (Select-String -Path "src\*.rs" -Pattern "panic!" -Recurse).Count
Write-Host "   Found $panicCount panic! calls" -ForegroundColor $(if ($panicCount -eq 0) { "Green" } else { "Yellow" })

Write-Host "`n6. Searching for expect() calls..." -ForegroundColor Yellow
$expectCount = (Select-String -Path "src\*.rs" -Pattern "\.expect\(" -Recurse).Count
Write-Host "   Found $expectCount expect() calls" -ForegroundColor $(if ($expectCount -lt 20) { "Green" } else { "Yellow" })

Write-Host "`n7. Checking for hardcoded secrets..." -ForegroundColor Yellow
$secrets = Select-String -Path "src\*.rs" -Pattern "(password|secret|key|token)\s*=\s*[\"']" -Recurse -CaseSensitive:$false | Where-Object { $_.Line -notmatch "//" -and $_.Line -notmatch "test" }
if ($secrets) {
    Write-Host "   ⚠️  Found potential hardcoded secrets:" -ForegroundColor Red
    $secrets | ForEach-Object { Write-Host "      $($_.Filename):$($_.LineNumber)" -ForegroundColor Yellow }
} else {
    Write-Host "   ✅ No obvious hardcoded secrets found" -ForegroundColor Green
}

Write-Host "`n8. Running tests..." -ForegroundColor Yellow
cargo test --lib 2>&1 | Select-Object -Last 20

Write-Host "`n9. Building release..." -ForegroundColor Yellow
cargo build --release 2>&1 | Select-Object -Last 10

Write-Host "`n=== Audit Summary ===" -ForegroundColor Cyan
Write-Host "Unsafe blocks: $unsafeCount" -ForegroundColor White
Write-Host "Unwrap calls: $unwrapCount" -ForegroundColor White
Write-Host "Panic calls: $panicCount" -ForegroundColor White
Write-Host "Expect calls: $expectCount" -ForegroundColor White

Write-Host "`n⚠️  Note: This is a basic self-audit. Professional security audit recommended for production." -ForegroundColor Yellow
Write-Host "   See SELF_SECURITY_AUDIT_GUIDE.md for detailed audit process." -ForegroundColor Cyan
