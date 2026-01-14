# Cleanup script for NozyWallet repository
# This removes build artifacts, dependencies, and temporary files

Write-Host "Starting cleanup of NozyWallet repository..." -ForegroundColor Green
Write-Host ""

$totalFreed = 0

# Function to get directory size
function Get-DirectorySize {
    param($path)
    if (Test-Path $path) {
        $size = (Get-ChildItem -Path $path -Recurse -ErrorAction SilentlyContinue | 
                 Measure-Object -Property Length -Sum -ErrorAction SilentlyContinue).Sum
        return [math]::Round($size / 1MB, 2)
    }
    return 0
}

# Function to safely remove directory
function Remove-DirectorySafely {
    param($path, $description)
    if (Test-Path $path) {
        $size = Get-DirectorySize $path
        Write-Host "  Removing $description ($size MB)..." -ForegroundColor Yellow
        Remove-Item -Path $path -Recurse -Force -ErrorAction SilentlyContinue
        $script:totalFreed += $size
        Write-Host "  Removed $description" -ForegroundColor Green
        return $size
    } else {
        Write-Host "  $description not found, skipping" -ForegroundColor Gray
        return 0
    }
}

# Function to safely remove file
function Remove-FileSafely {
    param($path, $description)
    if (Test-Path $path) {
        $size = [math]::Round((Get-Item $path).Length / 1MB, 2)
        Write-Host "  Removing $description ($size MB)..." -ForegroundColor Yellow
        Remove-Item -Path $path -Force -ErrorAction SilentlyContinue
        $script:totalFreed += $size
        Write-Host "  Removed $description" -ForegroundColor Green
        return $size
    } else {
        Write-Host "  $description not found, skipping" -ForegroundColor Gray
        return 0
    }
}

Write-Host "Removing build artifacts..." -ForegroundColor Cyan
Remove-DirectorySafely "target" "Rust build artifacts (target/)"
Remove-DirectorySafely "landing/dist" "Landing page build output"
Remove-DirectorySafely "landing/docs" "Landing page docs output"
Remove-DirectorySafely "desktop-client/dist" "Desktop client build output"
Remove-DirectorySafely "book/book" "mdBook build output"

Write-Host ""
Write-Host "Removing node_modules (can be reinstalled with npm install)..." -ForegroundColor Cyan
Remove-DirectorySafely "landing/node_modules" "Landing page node_modules"
Remove-DirectorySafely "desktop-client/node_modules" "Desktop client node_modules"

Write-Host ""
Write-Host "Removing temporary files..." -ForegroundColor Cyan
Remove-FileSafely "temp_notes_part1.rs" "Temporary notes file"
Remove-FileSafely "temp_notes_part2.rs" "Temporary notes file"
Remove-FileSafely "test_unwrap_fixes.rs" "Test file"
Remove-FileSafely "COMMIT_MESSAGE.txt" "Commit message file"

# Remove log files
Get-ChildItem -Path . -Filter "*.log" -Recurse -ErrorAction SilentlyContinue | ForEach-Object {
    $size = [math]::Round($_.Length / 1MB, 2)
    Write-Host "  Removing log file: $($_.Name) ($size MB)..." -ForegroundColor Yellow
    Remove-Item $_.FullName -Force -ErrorAction SilentlyContinue
    $script:totalFreed += $size
}

# Remove backup files
Get-ChildItem -Path . -Filter "*BACKUP*" -Recurse -ErrorAction SilentlyContinue | ForEach-Object {
    $size = [math]::Round($_.Length / 1MB, 2)
    Write-Host "  Removing backup file: $($_.Name) ($size MB)..." -ForegroundColor Yellow
    Remove-Item $_.FullName -Force -ErrorAction SilentlyContinue
    $script:totalFreed += $size
}

Write-Host ""
Write-Host "Cleanup complete!" -ForegroundColor Green
Write-Host "Total space freed: $([math]::Round($totalFreed, 2)) MB" -ForegroundColor Green
Write-Host ""

if ($totalFreed -gt 100) {
    Write-Host "Tip: Run npm install in landing/ and desktop-client/ to restore dependencies" -ForegroundColor Cyan
    Write-Host "Tip: Run cargo build to rebuild Rust artifacts when needed" -ForegroundColor Cyan
}
