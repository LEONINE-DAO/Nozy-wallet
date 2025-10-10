# Quick Send ZEC Script
# This script helps you send ZEC from your NozyWallet

Write-Host "🚀 NozyWallet - Quick Send ZEC" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Check Zebra connection
Write-Host "📡 Step 1: Checking Zebra node connection..." -ForegroundColor Yellow
cargo run --bin quick_test 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Failed to connect to Zebra node" -ForegroundColor Red
    Write-Host "   Make sure Zebra is running at http://localhost:18232" -ForegroundColor Red
    exit 1
}
Write-Host ""

# Step 2: Show wallet info
Write-Host "👛 Step 2: Checking wallet..." -ForegroundColor Yellow
Write-Host "   Run this command to see your wallet info:" -ForegroundColor Gray
Write-Host "   cargo run --bin nozy -- info" -ForegroundColor White
Write-Host ""

# Step 3: List notes
Write-Host "📝 Step 3: Checking for spendable notes..." -ForegroundColor Yellow
Write-Host "   Run this command to see your notes:" -ForegroundColor Gray
Write-Host "   cargo run --bin nozy -- list-notes" -ForegroundColor White
Write-Host ""

# Step 4: Explain send command
Write-Host "💸 Step 4: Send ZEC Command" -ForegroundColor Yellow
Write-Host "   Use this command to send:" -ForegroundColor Gray
Write-Host "   cargo run --bin nozy -- send --to <ADDRESS> --amount <ZATOSHIS>" -ForegroundColor White
Write-Host ""

# Examples
Write-Host "📖 Examples:" -ForegroundColor Cyan
Write-Host "   Send 0.001 ZEC (100,000 zatoshis):" -ForegroundColor Gray
Write-Host "   cargo run --bin nozy -- send --to u1test123... --amount 100000" -ForegroundColor White
Write-Host ""
Write-Host "   Send 0.01 ZEC (1,000,000 zatoshis):" -ForegroundColor Gray
Write-Host "   cargo run --bin nozy -- send --to u1test123... --amount 1000000" -ForegroundColor White
Write-Host ""

# Conversion table
Write-Host "💰 Amount Conversion:" -ForegroundColor Cyan
Write-Host "   0.0001 ZEC = 10,000 zatoshis" -ForegroundColor Gray
Write-Host "   0.001 ZEC  = 100,000 zatoshis" -ForegroundColor Gray
Write-Host "   0.01 ZEC   = 1,000,000 zatoshis" -ForegroundColor Gray
Write-Host "   0.1 ZEC    = 10,000,000 zatoshis" -ForegroundColor Gray
Write-Host "   1.0 ZEC    = 100,000,000 zatoshis" -ForegroundColor Gray
Write-Host ""

Write-Host "✅ Ready to send ZEC!" -ForegroundColor Green
Write-Host "   Follow the steps above to send your first transaction." -ForegroundColor Green

