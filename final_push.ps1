# Final push script for NozyWallet improvements

Write-Host "=== Step 1: Stashing README.md ===" -ForegroundColor Cyan
git stash push -m "README.md local changes" README.md 2>&1
Write-Host "README.md stashed" -ForegroundColor Green

Write-Host "`n=== Step 2: Staging core improvement files ===" -ForegroundColor Cyan
git add src/error.rs
git add src/progress.rs
git add src/input_validation.rs
git add src/cache.rs
git add src/lib.rs

$staged = git diff --cached --name-only
Write-Host "Staged files:" -ForegroundColor Yellow
$staged | ForEach-Object { Write-Host "  ✓ $_" -ForegroundColor Green }

Write-Host "`n=== Step 3: Checking main.rs ===" -ForegroundColor Cyan
$mainStaged = git diff --cached --name-only | Select-String "main.rs"
if ($mainStaged) {
    Write-Host "✓ main.rs is already staged" -ForegroundColor Green
} else {
    Write-Host "⚠ main.rs needs to be staged" -ForegroundColor Yellow
    Write-Host "Run: git add -p src/main.rs" -ForegroundColor Yellow
    Write-Host "Or to stage all main.rs changes: git add src/main.rs" -ForegroundColor Yellow
    $answer = Read-Host "Stage all main.rs changes now? (y/n)"
    if ($answer -eq "y") {
        git add src/main.rs
        Write-Host "✓ main.rs staged" -ForegroundColor Green
    }
}

Write-Host "`n=== Step 4: Committing ===" -ForegroundColor Cyan
$commitMsg = @"
feat: Core NozyWallet improvements

- Add missing error types (InsufficientFunds, InvalidInput)
- Add recovery suggestions for better error handling
- Fix mutex error handling (no more unwrap on locks)
- Fix IO unwrap calls with proper error handling
- Add progress indicators for sync and transaction operations
- Add input validation utilities for Zcash addresses and amounts
- Add generic caching system for API calls
- Enhance error messages with actionable recovery steps
"@

git commit -m $commitMsg
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Commit successful!" -ForegroundColor Green
} else {
    Write-Host "✗ Commit failed" -ForegroundColor Red
    Write-Host "Check if there are changes to commit: git status" -ForegroundColor Yellow
    exit 1
}

Write-Host "`n=== Step 5: Pulling latest changes ===" -ForegroundColor Cyan
git pull origin master
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠ Pull had issues" -ForegroundColor Yellow
    Write-Host "You may need to resolve conflicts manually" -ForegroundColor Yellow
}

Write-Host "`n=== Step 6: Pushing to GitHub ===" -ForegroundColor Cyan
git push origin master
if ($LASTEXITCODE -eq 0) {
    Write-Host "`n✅ SUCCESS! Changes pushed to GitHub!" -ForegroundColor Green
    Write-Host "Check: https://github.com/LEONINE-DAO/Nozy-wallet" -ForegroundColor Cyan
    
    Write-Host "`n=== Restoring README.md ===" -ForegroundColor Cyan
    git stash pop 2>&1 | Out-Null
    Write-Host "✓ README.md restored" -ForegroundColor Green
} else {
    Write-Host "`nPush failed - check error message above" -ForegroundColor Red
}
