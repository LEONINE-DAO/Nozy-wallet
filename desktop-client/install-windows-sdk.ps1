# Install Windows SDK for Rust MSVC toolchain
# This script helps install the Windows SDK

Write-Host "🔧 Windows SDK Installation Guide" -ForegroundColor Cyan
Write-Host ""

# Check if Windows SDK is already installed
$sdkPaths = @(
    "C:\Program Files (x86)\Windows Kits\10\Lib",
    "C:\Program Files\Windows Kits\10\Lib"
)

$sdkFound = $false
foreach ($path in $sdkPaths) {
    if (Test-Path $path) {
        Write-Host "✅ Windows SDK found at: $path" -ForegroundColor Green
        $versions = Get-ChildItem $path -Directory -ErrorAction SilentlyContinue | Sort-Object Name -Descending | Select-Object -First 1
        if ($versions) {
            Write-Host "   Latest version: $($versions.Name)" -ForegroundColor Green
            $sdkFound = $true
        }
    }
}

if (-not $sdkFound) {
    Write-Host "❌ Windows SDK not found!" -ForegroundColor Red
    Write-Host ""
    Write-Host "📥 To install Windows SDK:" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Option 1: Install via Visual Studio Installer (Recommended)" -ForegroundColor Cyan
    Write-Host "   1. Open Visual Studio Installer"
    Write-Host "   2. Click 'Modify' on your Visual Studio installation"
    Write-Host "   3. Go to 'Individual components' tab"
    Write-Host "   4. Search for 'Windows SDK'"
    Write-Host "   5. Check 'Windows 10 SDK' or 'Windows 11 SDK' (latest version)"
    Write-Host "   6. Click 'Modify' to install"
    Write-Host ""
    Write-Host "Option 2: Download Windows SDK Standalone" -ForegroundColor Cyan
    Write-Host "   1. Visit: https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/"
    Write-Host "   2. Download Windows 11 SDK (or Windows 10 SDK)"
    Write-Host "   3. Run the installer"
    Write-Host ""
    Write-Host "Option 3: Install via winget (if available)" -ForegroundColor Cyan
    Write-Host "   winget install Microsoft.WindowsSDK.10.0"
    Write-Host ""
    
    # Try winget if available
    $wingetAvailable = Get-Command winget -ErrorAction SilentlyContinue
    if ($wingetAvailable) {
        Write-Host "🚀 Attempting to install via winget..." -ForegroundColor Yellow
        Write-Host ""
        winget install Microsoft.WindowsSDK.10.0 --accept-package-agreements --accept-source-agreements
    }
} else {
    Write-Host ""
    Write-Host "✅ Windows SDK is installed!" -ForegroundColor Green
    Write-Host ""
    Write-Host "🔄 Switching back to MSVC toolchain..." -ForegroundColor Yellow
    rustup default stable-x86_64-pc-windows-msvc
    Write-Host ""
    Write-Host "✅ Done! You can now compile the Tauri backend." -ForegroundColor Green
}
