# PowerShell script to start API server for desktop client development

Write-Host "🚀 Starting NozyWallet API Server for Desktop Client..." -ForegroundColor Green
Write-Host ""

# Check if running from project root
if (-not (Test-Path "api-server")) {
    Write-Host "❌ Error: Please run this script from the project root directory" -ForegroundColor Red
    exit 1
}

# Check if cargo is installed
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Error: Cargo is not installed. Please install Rust first." -ForegroundColor Red
    exit 1
}

Set-Location api-server

# Build if needed
if (-not (Test-Path "target/release/nozywallet-api.exe")) {
    Write-Host "📦 Building API server..." -ForegroundColor Yellow
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Build failed!" -ForegroundColor Red
        exit 1
    }
}

Write-Host ""
Write-Host "✅ API Server Configuration:" -ForegroundColor Green
Write-Host "   URL: http://localhost:3000" -ForegroundColor Cyan
Write-Host "   Health Check: http://localhost:3000/health" -ForegroundColor Cyan
Write-Host ""
Write-Host "📱 Desktop Client Configuration:" -ForegroundColor Yellow
Write-Host "   Set API_URL=http://localhost:3000 in your desktop client" -ForegroundColor Cyan
Write-Host "   Or use the API client examples from DESKTOP_CLIENT_INTEGRATION.md" -ForegroundColor Cyan
Write-Host ""
Write-Host "🌐 Starting server..." -ForegroundColor Green
Write-Host ""

# Run the server
cargo run --release
