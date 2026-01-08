# Check status and push NozyWallet improvements

Write-Host "=== Checking Git Status ===" -ForegroundColor Cyan
$status = git status --porcelain
if ($status) {
    Write-Host "Uncommitted changes:" -ForegroundColor Yellow
    Write-Host $status
} else {
    Write-Host "No uncommitted changes" -ForegroundColor Green
}

Write-Host "`n=== Checking if files are tracked ===" -ForegroundColor Cyan
$files = @("src/progress.rs", "src/cache.rs", "src/input_validation.rs")
foreach ($file in $files) {
    if (Test-Path $file) {
        $tracked = git ls-files $file
        if ($tracked) {
            Write-Host "✓ $file is tracked" -ForegroundColor Green
        } else {
            Write-Host "✗ $file is NOT tracked - needs to be added!" -ForegroundColor Red
        }
    }
}

Write-Host "`n=== Recent Local Commits ===" -ForegroundColor Cyan
git log --oneline -5

Write-Host "`n=== Recent Remote Commits ===" -ForegroundColor Cyan
git fetch origin 2>&1 | Out-Null
git log origin/master --oneline -5

Write-Host "`n=== Unpushed Commits ===" -ForegroundColor Cyan
$unpushed = git log origin/master..HEAD --oneline
if ($unpushed) {
    Write-Host "Commits to push:" -ForegroundColor Yellow
    Write-Host $unpushed
} else {
    Write-Host "No unpushed commits" -ForegroundColor Yellow
}

Write-Host "`n=== Checking if files need to be staged ===" -ForegroundColor Cyan
$needsAdd = $false
foreach ($file in $files) {
    if (Test-Path $file) {
        $tracked = git ls-files $file
        if (-not $tracked) {
            Write-Host "⚠ $file needs to be added!" -ForegroundColor Red
            $needsAdd = $true
        }
    }
}

if ($needsAdd) {
    Write-Host "`n=== Staging files ===" -ForegroundColor Cyan
    git add src/error.rs
    git add src/progress.rs
    git add src/input_validation.rs
    git add src/cache.rs
    git add src/lib.rs
    Write-Host "Files staged. You may need to stage main.rs with: git add -p src/main.rs" -ForegroundColor Yellow
}

Write-Host "`n=== Attempting Push ===" -ForegroundColor Cyan
git push origin master 2>&1
