# Password Reset Helper Script
# This script helps you reset your wallet password by restoring from mnemonic

Write-Host "🔐 NozyWallet Password Reset Helper" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan
Write-Host ""

Write-Host "⚠️  IMPORTANT: You need your mnemonic phrase to reset the password!" -ForegroundColor Yellow
Write-Host ""

$hasMnemonic = Read-Host "Do you have your 12/24 word mnemonic phrase? (y/n)"

if ($hasMnemonic -ne "y" -and $hasMnemonic -ne "Y") {
    Write-Host ""
    Write-Host "❌ Without your mnemonic phrase, the wallet cannot be recovered." -ForegroundColor Red
    Write-Host "   The wallet is encrypted and requires either:" -ForegroundColor Red
    Write-Host "   1. The correct password, OR" -ForegroundColor Red
    Write-Host "   2. The mnemonic phrase to restore" -ForegroundColor Red
    Write-Host ""
    Write-Host "💡 If you lost both, the wallet and funds are permanently lost." -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "✅ Great! Let's restore your wallet with a new password." -ForegroundColor Green
Write-Host ""
Write-Host "📝 Steps:" -ForegroundColor Cyan
Write-Host "   1. You'll be prompted for your mnemonic phrase"
Write-Host "   2. You'll be prompted for a NEW password (or leave empty for no password)"
Write-Host "   3. Your wallet will be restored with the new password"
Write-Host ""

$confirm = Read-Host "Ready to proceed? (y/n)"

if ($confirm -ne "y" -and $confirm -ne "Y") {
    Write-Host "Cancelled." -ForegroundColor Yellow
    exit 0
}

Write-Host ""
Write-Host "🔄 Running restore command..." -ForegroundColor Cyan
Write-Host ""

# Run the restore command
& ".\target\release\nozy.exe" restore

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "✅ Password reset successful!" -ForegroundColor Green
    Write-Host "   Your wallet now uses the new password you set." -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "❌ Restore failed. Please check:" -ForegroundColor Red
    Write-Host "   - Mnemonic phrase is correct (12 or 24 words)" -ForegroundColor Red
    Write-Host "   - No typos in the mnemonic" -ForegroundColor Red
    Write-Host "   - Wallet file is not corrupted" -ForegroundColor Red
}
