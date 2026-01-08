# Simple push script - no fancy formatting

Write-Host "Stashing README.md..." -ForegroundColor Cyan
git stash push -m "README.md" README.md

Write-Host "`nStaging files..." -ForegroundColor Cyan
git add src/error.rs
git add src/progress.rs
git add src/input_validation.rs
git add src/cache.rs
git add src/lib.rs
git add src/main.rs

Write-Host "`nCommitting..." -ForegroundColor Cyan
git commit -m "feat: Core NozyWallet improvements" -m "Add missing error types" -m "Add recovery suggestions" -m "Fix mutex error handling" -m "Fix IO unwrap calls" -m "Add progress indicators" -m "Add input validation" -m "Add caching system" -m "Enhance error messages"

if ($LASTEXITCODE -eq 0) {
    Write-Host "Commit successful!" -ForegroundColor Green
    
    Write-Host "`nPulling latest changes..." -ForegroundColor Cyan
    git pull origin master
    
    Write-Host "`nPushing to GitHub..." -ForegroundColor Cyan
    git push origin master
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "`nSUCCESS! Pushed to GitHub" -ForegroundColor Green
        Write-Host "Check: https://github.com/LEONINE-DAO/Nozy-wallet" -ForegroundColor Cyan
    }
    
    Write-Host "`nRestoring README.md..." -ForegroundColor Cyan
    git stash pop
} else {
    Write-Host "Commit failed - check git status" -ForegroundColor Red
}
