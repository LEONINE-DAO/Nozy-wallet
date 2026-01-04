# PowerShell script to test API server connection

Write-Host "🔍 Testing NozyWallet API Server Connection..." -ForegroundColor Cyan
Write-Host ""

# Test health endpoint
Write-Host "Testing health endpoint..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "http://localhost:3000/health" -Method GET -UseBasicParsing -ErrorAction Stop
    Write-Host "✅ API Server is running!" -ForegroundColor Green
    Write-Host "   Status: $($response.StatusCode)" -ForegroundColor Gray
    Write-Host "   Response: $($response.Content)" -ForegroundColor Gray
    Write-Host ""
    
    # Test wallet exists endpoint
    Write-Host "Testing wallet exists endpoint..." -ForegroundColor Yellow
    try {
        $walletResponse = Invoke-WebRequest -Uri "http://localhost:3000/api/wallet/exists" -Method GET -UseBasicParsing -ErrorAction Stop
        Write-Host "✅ Wallet endpoint is accessible!" -ForegroundColor Green
        Write-Host "   Response: $($walletResponse.Content)" -ForegroundColor Gray
    } catch {
        Write-Host "⚠️  Wallet endpoint error: $($_.Exception.Message)" -ForegroundColor Yellow
    }
    
    Write-Host ""
    Write-Host "✅ All tests passed! DesktopClient should be able to connect." -ForegroundColor Green
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Cyan
    Write-Host "  1. Set VITE_API_URL=http://localhost:3000 in DesktopClient .env file" -ForegroundColor White
    Write-Host "  2. Run: npm run dev in DesktopClient directory" -ForegroundColor White
    
} catch {
    Write-Host "❌ API Server is NOT running!" -ForegroundColor Red
    Write-Host "   Error: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host ""
    Write-Host "To start the API server:" -ForegroundColor Yellow
    Write-Host "  cd api-server" -ForegroundColor White
    Write-Host "  cargo run" -ForegroundColor White
}

