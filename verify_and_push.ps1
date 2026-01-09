# Verify and push NozyWallet core improvements

Write-Host "=== Checking Git Status ===" -ForegroundColor Cyan
git status --short

Write-Host "`n=== Recent Commits ===" -ForegroundColor Cyan
git log --oneline -5

Write-Host "`n=== Checking if new files are tracked ===" -ForegroundColor Cyan
$files = @("src/progress.rs", "src/cache.rs", "src/input_validation.rs")
foreach ($file in $files) {
    if (Test-Path $file) {
        $tracked = git ls-files $file
        if ($tracked) {
            Write-Host "✓ $file is tracked" -ForegroundColor Green
        } else {
            Write-Host "✗ $file is NOT tracked - needs to be added" -ForegroundColor Red
        }
    } else {
        Write-Host "✗ $file does not exist" -ForegroundColor Red
    }
}

Write-Host "`n=== Checking local vs remote ===" -ForegroundColor Cyan
$local = git rev-parse HEAD
$remote = git rev-parse origin/master 2>$null
if ($LASTEXITCODE -eq 0) {
    if ($local -eq $remote) {
        Write-Host "✓ Local and remote are in sync" -ForegroundColor Green
    } else {
        Write-Host "⚠ Local and remote differ" -ForegroundColor Yellow
        Write-Host "Local:  $local" -ForegroundColor White
        Write-Host "Remote: $remote" -ForegroundColor White
        Write-Host "`nUnpushed commits:" -ForegroundColor Yellow
        git log origin/master..HEAD --oneline
    }
} else {
    Write-Host "⚠ Could not fetch remote status" -ForegroundColor Yellow
}

Write-Host "`n=== Attempting to push ===" -ForegroundColor Cyan
git push origin master 2>&1
