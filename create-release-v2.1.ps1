# NozyWallet v2.1.0 Release Script

Write-Host "🚀 Creating NozyWallet v2.1.0 Release" -ForegroundColor Cyan
Write-Host ""

if (-not (Test-Path .git)) {
    Write-Host "❌ Error: Not in a git repository!" -ForegroundColor Red
    exit 1
}

$status = git status --porcelain
if ($status) {
    Write-Host "⚠️  Warning: You have uncommitted changes:" -ForegroundColor Yellow
    Write-Host $status
    Write-Host ""
    $response = Read-Host "Continue anyway? (y/N)"
    if ($response -ne "y" -and $response -ne "Y") {
        Write-Host "Aborted." -ForegroundColor Yellow
        exit 1
    }
}

$existingTag = git tag -l "v2.1.0"
if ($existingTag) {
    Write-Host "⚠️  Warning: Tag v2.1.0 already exists!" -ForegroundColor Yellow
    $response = Read-Host "Delete and recreate? (y/N)"
    if ($response -eq "y" -or $response -eq "Y") {
        Write-Host "Deleting existing tag..." -ForegroundColor Yellow
        git tag -d v2.1.0
        git push origin :refs/tags/v2.1.0
    } else {
        Write-Host "Aborted." -ForegroundColor Yellow
        exit 1
    }
}

Write-Host "📝 Creating tag v2.1.0..." -ForegroundColor Cyan
$tagMessage = 'NozyWallet v2.1.0 Release

Major Features:
- NU 6.1 Support
- Crosslink Backend Integration
- REST API Server
- Security Hardening (93 unwrap() calls removed)
- Deterministic Scanning Tests
- Note Indexing System
- Secret Network Support
- Cross-Chain Swap Framework'

git tag -a v2.1.0 -m $tagMessage

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Error: Failed to create tag!" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Tag created successfully!" -ForegroundColor Green
Write-Host ""

Write-Host "📤 Pushing tag to GitHub..." -ForegroundColor Cyan
git push origin v2.1.0

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Error: Failed to push tag!" -ForegroundColor Red
    Write-Host "You may need to push manually: git push origin v2.1.0" -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "✅ Tag pushed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "🎉 Release workflow triggered!" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "1. Monitor workflow: https://github.com/LEONINE-DAO/Nozy-wallet/actions" -ForegroundColor White
Write-Host "2. Check release: https://github.com/LEONINE-DAO/Nozy-wallet/releases" -ForegroundColor White
Write-Host "3. The release will be created automatically when builds complete" -ForegroundColor White
Write-Host ""
Write-Host "The workflow will build:" -ForegroundColor Cyan
Write-Host "  • CLI binaries (Linux, Windows, macOS Intel, macOS ARM)" -ForegroundColor White
Write-Host "  • API server binaries (Linux, Windows, macOS Intel, macOS ARM)" -ForegroundColor White
Write-Host "  • Desktop client installers (from NozyWallet-DesktopClient repo)" -ForegroundColor White
Write-Host "  • SHA256 hashes for all binaries" -ForegroundColor White
Write-Host ""
Write-Host "Note: Desktop client will be cloned from:" -ForegroundColor Yellow
Write-Host "  https://github.com/LEONINE-DAO/NozyWallet-DesktopClient" -ForegroundColor Cyan
Write-Host ""

