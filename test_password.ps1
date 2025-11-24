# Test script for password functionality (PowerShell)

Write-Host "🧪 Testing Password Functionality" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan
Write-Host ""

# Clean up any existing test wallet
$walletPath = "$env:USERPROFILE\AppData\Roaming\Nozy\nozy\data\wallet.dat"
if (Test-Path $walletPath) {
    Remove-Item $walletPath -Force
    Write-Host "Cleaned up existing wallet" -ForegroundColor Yellow
}

Write-Host "Test 1: Create wallet WITH password" -ForegroundColor Green
Write-Host "------------------------------------" -ForegroundColor Green
Write-Host "When prompted:"
Write-Host "  - Do you want to set a password? -> y"
Write-Host "  - Enter password: testpass123"
Write-Host "  - Confirm password: testpass123"
Write-Host ""
Write-Host "Press Enter to start test 1..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")

cargo run --bin nozy new

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Test 1 PASSED: Wallet created with password" -ForegroundColor Green
} else {
    Write-Host "❌ Test 1 FAILED: Could not create wallet" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Test 2: Load wallet with CORRECT password" -ForegroundColor Green
Write-Host "----------------------------------------" -ForegroundColor Green
Write-Host "When prompted, enter password: testpass123"
Write-Host ""
Write-Host "Press Enter to start test 2..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")

cargo run --bin nozy balance

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Test 2 PASSED: Wallet loaded with correct password" -ForegroundColor Green
} else {
    Write-Host "❌ Test 2 FAILED: Could not load wallet with correct password" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Test 3: Load wallet with WRONG password (should fail)" -ForegroundColor Green
Write-Host "----------------------------------------------------" -ForegroundColor Green
Write-Host "When prompted, enter password: wrongpass"
Write-Host ""
Write-Host "Press Enter to start test 3..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")

cargo run --bin nozy balance

if ($LASTEXITCODE -ne 0) {
    Write-Host "✅ Test 3 PASSED: Wallet correctly rejected wrong password" -ForegroundColor Green
} else {
    Write-Host "❌ Test 3 FAILED: Wallet should have rejected wrong password" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "🎉 All password tests passed!" -ForegroundColor Green

