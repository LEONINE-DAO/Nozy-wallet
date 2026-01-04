# Check what's in OneDrive/Evms folder

$evmsPath = "$env:USERPROFILE\OneDrive\Evms"

if (Test-Path $evmsPath) {
    Write-Host "=== Contents of OneDrive/Evms ===" -ForegroundColor Cyan
    
    $items = Get-ChildItem -Path $evmsPath -ErrorAction SilentlyContinue
    foreach ($item in $items) {
        if ($item.PSIsContainer) {
            $size = (Get-ChildItem -Path $item.FullName -Recurse -ErrorAction SilentlyContinue | 
                     Measure-Object -Property Length -Sum).Sum
            $sizeGB = [math]::Round($size / 1GB, 2)
            Write-Host "  [DIR] $($item.Name) - $sizeGB GB" -ForegroundColor Yellow
        } else {
            $sizeMB = [math]::Round($item.Length / 1MB, 2)
            if ($sizeMB -gt 1) {
                Write-Host "  [FILE] $($item.Name) - $sizeMB MB" -ForegroundColor Yellow
            }
        }
    }
    
    $totalSize = (Get-ChildItem -Path $evmsPath -Recurse -ErrorAction SilentlyContinue | 
                  Measure-Object -Property Length -Sum).Sum
    $totalGB = [math]::Round($totalSize / 1GB, 2)
    Write-Host ""
    Write-Host "Total size: $totalGB GB" -ForegroundColor Cyan
} else {
    Write-Host "OneDrive/Evms folder not found" -ForegroundColor Green
}

