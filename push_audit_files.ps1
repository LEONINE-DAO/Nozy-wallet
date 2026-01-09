# Push Security Audit Files to GitHub

Write-Host "=== Pushing Security Audit Files to GitHub ===" -ForegroundColor Cyan
Write-Host ""

# Check if files exist
Write-Host "1. Checking files..." -ForegroundColor Yellow
$files = @("SELF_AUDIT_RESULTS.md", "SELF_SECURITY_AUDIT_GUIDE.md", "SECURITY_AUDIT_ANNOUNCEMENT.md", "run_self_audit.ps1")
foreach ($file in $files) {
    if (Test-Path $file) {
        Write-Host "   ✅ $file exists" -ForegroundColor Green
    } else {
        Write-Host "   ❌ $file NOT FOUND" -ForegroundColor Red
        exit 1
    }
}

# Add files with force (to override .gitignore)
Write-Host "`n2. Adding files to git..." -ForegroundColor Yellow
git add -f SELF_AUDIT_RESULTS.md SELF_SECURITY_AUDIT_GUIDE.md SECURITY_AUDIT_ANNOUNCEMENT.md run_self_audit.ps1 .gitignore
if ($LASTEXITCODE -ne 0) {
    Write-Host "   ❌ Failed to add files" -ForegroundColor Red
    exit 1
}
Write-Host "   ✅ Files added" -ForegroundColor Green

# Check status
Write-Host "`n3. Checking git status..." -ForegroundColor Yellow
git status --short

# Commit
Write-Host "`n4. Committing files..." -ForegroundColor Yellow
$commitMsg = @"
Add security self-audit results and community announcement

- Complete self-security audit of NozyWallet
- 93 unwrap() calls identified across 18 files (remediation plan included)
- No critical security vulnerabilities found
- No unsafe code blocks in production
- No hardcoded secrets found
- Community announcement and transparency commitment
- Automated audit script for future use

Remediation plan:
- Phase 1: Fix high-priority files (note_storage, transaction_history, address_book)
- Phase 2: Fix remaining production files
- Phase 3: Verification and testing
- Phase 4: Professional audit (future)

See SELF_AUDIT_RESULTS.md for complete findings.
"@

git commit -m $commitMsg
if ($LASTEXITCODE -ne 0) {
    Write-Host "   ❌ Failed to commit" -ForegroundColor Red
    exit 1
}
Write-Host "   ✅ Files committed" -ForegroundColor Green

# Push
Write-Host "`n5. Pushing to GitHub..." -ForegroundColor Yellow
git push origin master
if ($LASTEXITCODE -ne 0) {
    Write-Host "   ❌ Failed to push" -ForegroundColor Red
    Write-Host "   Try: git pull origin master first" -ForegroundColor Yellow
    exit 1
}
Write-Host "   ✅ Pushed successfully!" -ForegroundColor Green

Write-Host "`n=== Complete ===" -ForegroundColor Cyan
Write-Host "Files are now available on GitHub:" -ForegroundColor White
Write-Host "  - SELF_AUDIT_RESULTS.md" -ForegroundColor White
Write-Host "  - SELF_SECURITY_AUDIT_GUIDE.md" -ForegroundColor White
Write-Host "  - SECURITY_AUDIT_ANNOUNCEMENT.md" -ForegroundColor White
Write-Host "  - run_self_audit.ps1" -ForegroundColor White
