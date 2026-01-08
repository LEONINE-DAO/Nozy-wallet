# Script to push zeaking code to separate repository
# https://github.com/Lowo88/Zeaking

$ErrorActionPreference = "Stop"

Write-Host "Checking git remotes..." -ForegroundColor Cyan
git remote -v

Write-Host "`nChecking if zeaking directory is committed..." -ForegroundColor Cyan
$commit = git log --oneline --all -- zeaking/ | Select-Object -First 1
if ($commit) {
    Write-Host "Found commit: $commit" -ForegroundColor Green
} else {
    Write-Host "Zeaking directory not found in commits. Adding and committing..." -ForegroundColor Yellow
    git add zeaking/
    git commit -m "Add zeaking indexing library"
}

Write-Host "`nCreating subtree branch..." -ForegroundColor Cyan
git subtree split --prefix=zeaking -b zeaking-export 2>&1 | Write-Host

Write-Host "`nPushing to remote repository..." -ForegroundColor Cyan
Write-Host "Trying master branch..." -ForegroundColor Yellow
$result = git push zeaking zeaking-export:master --force 2>&1
$result | Write-Host

if ($LASTEXITCODE -ne 0) {
    Write-Host "`nMaster branch failed, trying main branch..." -ForegroundColor Yellow
    $result = git push zeaking zeaking-export:main --force 2>&1
    $result | Write-Host
}

if ($LASTEXITCODE -eq 0) {
    Write-Host "`n✓ Push successful!" -ForegroundColor Green
    Write-Host "Check https://github.com/Lowo88/Zeaking to verify." -ForegroundColor Cyan
} else {
    Write-Host "`n✗ Push failed. Error code: $LASTEXITCODE" -ForegroundColor Red
    Write-Host "Make sure:" -ForegroundColor Yellow
    Write-Host "  1. The repository exists at https://github.com/Lowo88/Zeaking" -ForegroundColor Yellow
    Write-Host "  2. You have push access to the repository" -ForegroundColor Yellow
    Write-Host "  3. Your git credentials are configured" -ForegroundColor Yellow
}
