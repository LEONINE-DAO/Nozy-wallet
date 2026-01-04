# Quick Install Script for Visual Studio Build Tools
# This script helps you install the required build tools for Rust on Windows

Write-Host "🔧 NozyWallet API Server - Build Tools Setup" -ForegroundColor Cyan
Write-Host ""

# Check if Chocolatey is installed
$hasChoco = Get-Command choco -ErrorAction SilentlyContinue

if ($hasChoco) {
    Write-Host "✅ Chocolatey detected!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Installing Visual Studio Build Tools via Chocolatey..." -ForegroundColor Yellow
    Write-Host "This will install the C++ build tools required for Rust." -ForegroundColor Gray
    Write-Host ""
    
    $confirm = Read-Host "Continue? (Y/N)"
    if ($confirm -eq "Y" -or $confirm -eq "y") {
        Write-Host ""
        Write-Host "Installing... This may take 10-20 minutes." -ForegroundColor Yellow
        choco install visualstudio2022buildtools --params "--add Microsoft.VisualStudio.Workload.VCTools --quiet" -y
        Write-Host ""
        Write-Host "✅ Installation complete!" -ForegroundColor Green
        Write-Host ""
        Write-Host "⚠️  IMPORTANT: Restart your terminal/PowerShell after installation!" -ForegroundColor Yellow
        Write-Host ""
        Write-Host "Then run:" -ForegroundColor Cyan
        Write-Host "  cd api-server" -ForegroundColor White
        Write-Host "  cargo build" -ForegroundColor White
    }
} else {
    Write-Host "❌ Chocolatey not found" -ForegroundColor Red
    Write-Host ""
    Write-Host "Option 1: Install Chocolatey first (recommended)" -ForegroundColor Yellow
    Write-Host "  Run PowerShell as Administrator, then:" -ForegroundColor White
    Write-Host "  Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Option 2: Manual installation" -ForegroundColor Yellow
    Write-Host "  1. Download: https://visualstudio.microsoft.com/downloads/" -ForegroundColor White
    Write-Host "  2. Scroll to 'Tools for Visual Studio'" -ForegroundColor White
    Write-Host "  3. Download 'Build Tools for Visual Studio 2022'" -ForegroundColor White
    Write-Host "  4. Run installer and select 'Desktop development with C++'" -ForegroundColor White
    Write-Host "  5. Restart your terminal" -ForegroundColor White
    Write-Host ""
    Write-Host "Option 3: Use MinGW (alternative)" -ForegroundColor Yellow
    Write-Host "  choco install mingw" -ForegroundColor White
    Write-Host "  rustup default stable-x86_64-pc-windows-gnu" -ForegroundColor White
}

