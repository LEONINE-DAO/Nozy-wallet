# PowerShell build script for Windows

param(
    [string]$Version = "0.2.0",
    [string]$Platform = "all"
)

Write-Host "🚀 Building NozyWallet Release v$Version" -ForegroundColor Green
Write-Host "Platform: $Platform"

$ReleaseDir = "releases\v$Version"
New-Item -ItemType Directory -Force -Path $ReleaseDir | Out-Null

function Build-CLI {
    param(
        [string]$Target,
        [string]$BinaryName
    )
    
    Write-Host "Building CLI for $Target..." -ForegroundColor Green
    
    cargo build --release --target $Target --bin nozy
    
    Copy-Item "target\$Target\release\$BinaryName" "$ReleaseDir\nozy-$Target.exe"
    
    $hash = (Get-FileHash -Path "$ReleaseDir\nozy-$Target.exe" -Algorithm SHA256).Hash
    "$hash  nozy-$Target.exe" | Out-File -FilePath "$ReleaseDir\nozy-$Target.exe.sha256" -Encoding ASCII
    
    Write-Host "✓ Built $Target" -ForegroundColor Green
}

function Build-Desktop {
    Write-Host "Desktop build disabled - src-tauri removed" -ForegroundColor Yellow
    Write-Host "Skipping desktop app build..." -ForegroundColor Yellow
}

switch ($Platform) {
    "windows" {
        Build-CLI "x86_64-pc-windows-msvc" "nozy.exe"
    }
    "desktop" {
        Build-Desktop
    }
    "all" {
        Write-Host "Building all platforms..." -ForegroundColor Yellow
        Build-CLI "x86_64-pc-windows-msvc" "nozy.exe"
        # Desktop build disabled - src-tauri removed
        # Build-Desktop
    }
    default {
        Write-Host "Unknown platform: $Platform" -ForegroundColor Red
        Write-Host "Usage: .\build-release.ps1 [version] [platform]"
        Write-Host "Platforms: windows, desktop, all"
        exit 1
    }
}

Write-Host "Generating hashes file..." -ForegroundColor Green
"# NozyWallet v$Version - SHA256 Hashes" | Out-File -FilePath "$ReleaseDir\HASHES.txt" -Encoding ASCII
"" | Out-File -FilePath "$ReleaseDir\HASHES.txt" -Append -Encoding ASCII
"Generated: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss UTC')" | Out-File -FilePath "$ReleaseDir\HASHES.txt" -Append -Encoding ASCII
"" | Out-File -FilePath "$ReleaseDir\HASHES.txt" -Append -Encoding ASCII

Get-ChildItem -Path $ReleaseDir -Filter "*.sha256" | ForEach-Object {
    Get-Content $_.FullName | Out-File -FilePath "$ReleaseDir\HASHES.txt" -Append -Encoding ASCII
}

Write-Host "✓ Release build complete!" -ForegroundColor Green
Write-Host "Files are in: $ReleaseDir" -ForegroundColor Green

