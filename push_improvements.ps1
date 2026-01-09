# Push NozyWallet core improvements to GitHub

Write-Host "=== Step 1: Checking if files exist ===" -ForegroundColor Cyan
$files = @("src/progress.rs", "src/cache.rs", "src/input_validation.rs")
foreach ($file in $files) {
    if (Test-Path $file) {
        Write-Host "✓ $file exists" -ForegroundColor Green
    } else {
        Write-Host "✗ $file NOT FOUND!" -ForegroundColor Red
        exit 1
    }
}

Write-Host "`n=== Step 2: Staging files ===" -ForegroundColor Cyan
git add src/error.rs
git add src/progress.rs
git add src/input_validation.rs
git add src/cache.rs
git add src/lib.rs
Write-Host "Core files staged" -ForegroundColor Green

Write-Host "`n=== Step 3: Checking main.rs ===" -ForegroundColor Cyan
Write-Host "You need to stage main.rs changes manually:" -ForegroundColor Yellow
Write-Host "Run: git add -p src/main.rs" -ForegroundColor Yellow
Write-Host "Then select only the Nozy improvements (skip Monero/Shade/Swap)" -ForegroundColor Yellow

$staged = git diff --cached --name-only
if ($staged -match "main.rs") {
    Write-Host "✓ main.rs is staged" -ForegroundColor Green
} else {
    Write-Host "⚠ main.rs is NOT staged yet" -ForegroundColor Yellow
}

Write-Host "`n=== Step 4: Committing ===" -ForegroundColor Cyan
$commitMsg = @"
feat: Core NozyWallet improvements

- Add missing error types (InsufficientFunds, InvalidInput)
- Add recovery suggestions for better error handling
- Fix mutex error handling (no more unwrap() on locks)
- Fix IO unwrap() calls with proper error handling
- Add progress indicators for sync and transaction operations
- Add input validation utilities for Zcash addresses and amounts
- Add generic caching system for API calls
- Enhance error messages with actionable recovery steps
"@

git commit -m $commitMsg
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Commit successful" -ForegroundColor Green
} else {
    Write-Host "✗ Commit failed - check if there are changes to commit" -ForegroundColor Red
    exit 1
}

Write-Host "`n=== Step 5: Pulling latest changes ===" -ForegroundColor Cyan
git pull origin master
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠ Pull had issues - you may need to resolve conflicts" -ForegroundColor Yellow
}

Write-Host "`n=== Step 6: Pushing to GitHub ===" -ForegroundColor Cyan
git push origin master
if ($LASTEXITCODE -eq 0) {
    Write-Host "`n✅ SUCCESS! Changes pushed to GitHub" -ForegroundColor Green
    Write-Host "Check: https://github.com/LEONINE-DAO/Nozy-wallet" -ForegroundColor Cyan
} else {
    Write-Host "`n✗ Push failed - check error message above" -ForegroundColor Red
}
