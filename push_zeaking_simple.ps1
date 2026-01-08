# Simple script to push zeaking to separate repo
# This works by cloning the target repo, copying files, and pushing

$ErrorActionPreference = "Continue"
$targetRepo = "https://github.com/Lowo88/Zeaking.git"
$tempDir = Join-Path $env:TEMP "Zeaking-push-$(Get-Date -Format 'yyyyMMddHHmmss')"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Pushing zeaking to GitHub" -ForegroundColor Cyan
Write-Host "Target: $targetRepo" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Clean up old temp directory
if (Test-Path $tempDir) {
    Write-Host "Cleaning up old temp directory..." -ForegroundColor Yellow
    Remove-Item -Recurse -Force $tempDir
}

# Step 2: Clone the target repository
Write-Host "Step 1: Cloning target repository..." -ForegroundColor Green
try {
    $cloneOutput = git clone $targetRepo $tempDir 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Failed to clone repository" -ForegroundColor Red
        Write-Host $cloneOutput -ForegroundColor Red
        Write-Host ""
        Write-Host "Possible issues:" -ForegroundColor Yellow
        Write-Host "  1. Repository doesn't exist yet - create it on GitHub first" -ForegroundColor Yellow
        Write-Host "  2. Authentication required - configure git credentials" -ForegroundColor Yellow
        Write-Host "  3. No internet connection" -ForegroundColor Yellow
        exit 1
    }
    Write-Host "✓ Repository cloned successfully" -ForegroundColor Green
} catch {
    Write-Host "ERROR: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

# Step 3: Copy zeaking files
Write-Host ""
Write-Host "Step 2: Copying zeaking files..." -ForegroundColor Green
$zeakingSource = Join-Path $PSScriptRoot "zeaking"
if (-not (Test-Path $zeakingSource)) {
    Write-Host "ERROR: zeaking directory not found at $zeakingSource" -ForegroundColor Red
    exit 1
}

# Remove everything in temp repo except .git
Get-ChildItem $tempDir -Exclude .git | Remove-Item -Recurse -Force

# Copy all zeaking files
Copy-Item -Recurse -Force "$zeakingSource\*" $tempDir
Write-Host "✓ Files copied successfully" -ForegroundColor Green

# Step 4: Commit and push
Write-Host ""
Write-Host "Step 3: Committing changes..." -ForegroundColor Green
Push-Location $tempDir
try {
    git add -A
    $commitOutput = git commit -m "Add zeaking indexing library" 2>&1
    Write-Host $commitOutput
    
    Write-Host ""
    Write-Host "Step 4: Pushing to GitHub..." -ForegroundColor Green
    
    # Try master first
    $pushOutput = git push origin master --force 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Master branch failed, trying main..." -ForegroundColor Yellow
        $pushOutput = git push origin main --force 2>&1
    }
    
    Write-Host $pushOutput
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "========================================" -ForegroundColor Green
        Write-Host "✓ SUCCESS! Code pushed to GitHub" -ForegroundColor Green
        Write-Host "Check: $targetRepo" -ForegroundColor Cyan
        Write-Host "========================================" -ForegroundColor Green
    } else {
        Write-Host ""
        Write-Host "========================================" -ForegroundColor Red
        Write-Host "✗ PUSH FAILED" -ForegroundColor Red
        Write-Host "========================================" -ForegroundColor Red
        Write-Host ""
        Write-Host "Error output:" -ForegroundColor Yellow
        Write-Host $pushOutput -ForegroundColor Red
        Write-Host ""
        Write-Host "Common issues:" -ForegroundColor Yellow
        Write-Host "  - Authentication: Configure git credentials" -ForegroundColor Yellow
        Write-Host "  - Permissions: Make sure you have push access" -ForegroundColor Yellow
        Write-Host "  - Network: Check your internet connection" -ForegroundColor Yellow
        exit 1
    }
} finally {
    Pop-Location
    # Clean up
    if (Test-Path $tempDir) {
        Write-Host ""
        Write-Host "Cleaning up temp directory..." -ForegroundColor Gray
        Remove-Item -Recurse -Force $tempDir -ErrorAction SilentlyContinue
    }
}
