# PowerShell start script for NozyWallet API Server

Write-Host "🚀 Starting NozyWallet API Server..." -ForegroundColor Green
Write-Host ""

# Check if running from project root
if (-not (Test-Path "api-server")) {
    Write-Host "❌ Error: Please run this script from the project root directory" -ForegroundColor Red
    exit 1
}

Set-Location api-server

# Check if cargo is installed
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Error: Cargo is not installed. Please install Rust first." -ForegroundColor Red
    exit 1
}

# Build if needed
if (-not (Test-Path "target/release/nozywallet-api.exe")) {
    Write-Host "📦 Building API server..." -ForegroundColor Yellow
    cargo build --release
}

# Run the server
Write-Host "✅ Starting server on http://0.0.0.0:3000" -ForegroundColor Green
Write-Host "📱 Mobile app should connect to: http://localhost:3000/api" -ForegroundColor Cyan
Write-Host ""
cargo run --release

