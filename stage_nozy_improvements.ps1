# Stage only NozyWallet core improvements (excluding Monero/Secret Network)

Write-Host "📦 Staging NozyWallet Core Improvements Only..." -ForegroundColor Cyan
Write-Host ""

# Stage core improvement files
Write-Host "✅ Staging core improvement files..." -ForegroundColor Green
git add src/error.rs
git add src/progress.rs
git add src/input_validation.rs
git add src/cache.rs
git add src/lib.rs

Write-Host ""
Write-Host "⚠️  For src/main.rs, you need to manually review and stage only Nozy improvements:" -ForegroundColor Yellow
Write-Host "   Run: git add -p src/main.rs" -ForegroundColor Yellow
Write-Host "   Then select only the Nozy improvements (error handling, progress, validation)" -ForegroundColor Yellow
Write-Host "   Skip: Monero, Secret, Swap command handlers" -ForegroundColor Yellow
Write-Host ""

Write-Host "📋 Files staged so far:" -ForegroundColor Cyan
git status --short

Write-Host ""
Write-Host "💡 Next steps:" -ForegroundColor Cyan
Write-Host "   1. Review staged files: git status" -ForegroundColor White
Write-Host "   2. Stage main.rs selectively: git add -p src/main.rs" -ForegroundColor White
Write-Host "   3. Review diff: git diff --cached" -ForegroundColor White
Write-Host "   4. Commit: git commit -m 'feat: Core NozyWallet improvements'" -ForegroundColor White
