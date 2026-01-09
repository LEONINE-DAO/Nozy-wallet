# Pull and push NozyWallet improvements

Write-Host "=== Pulling latest changes from GitHub ===" -ForegroundColor Cyan
git pull origin master

if ($LASTEXITCODE -eq 0) {
    Write-Host "`n=== Pull successful! Pushing changes... ===" -ForegroundColor Green
    git push origin master
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "`n=== ✅ Push successful! ===" -ForegroundColor Green
        Write-Host "Your changes are now on GitHub: https://github.com/LEONINE-DAO/Nozy-wallet" -ForegroundColor Cyan
    } else {
        Write-Host "`n=== ❌ Push failed ===" -ForegroundColor Red
        Write-Host "Check the error message above for details." -ForegroundColor Yellow
    }
} else {
    Write-Host "`n=== ❌ Pull failed ===" -ForegroundColor Red
    Write-Host "There may be merge conflicts. Check the error message above." -ForegroundColor Yellow
}
