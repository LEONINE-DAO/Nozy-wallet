# Check what's inside AppData

Write-Host "Checking AppData folder breakdown:" -ForegroundColor Cyan
Write-Host ""

$appdataPath = "C:\Users\user\AppData"

Get-ChildItem $appdataPath -Directory -ErrorAction SilentlyContinue | 
    ForEach-Object {
        $size = (Get-ChildItem $_.FullName -Recurse -ErrorAction SilentlyContinue | 
                 Measure-Object -Property Length -Sum -ErrorAction SilentlyContinue).Sum
        $sizeGB = [math]::Round($size / 1GB, 2)
        if ($sizeGB -gt 0.5) {
            [PSCustomObject]@{
                Folder = $_.Name
                "Size (GB)" = $sizeGB
                Path = $_.FullName
            }
        }
    } | Sort-Object "Size (GB)" -Descending | Format-Table -AutoSize

Write-Host ""
Write-Host "Checking common AppData subdirectories:" -ForegroundColor Yellow

# Check Local
$local = (Get-ChildItem "$appdataPath\Local" -Recurse -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum -ErrorAction SilentlyContinue).Sum / 1GB
Write-Host "AppData\Local: $([math]::Round($local, 2)) GB"

# Check Roaming
$roaming = (Get-ChildItem "$appdataPath\Roaming" -Recurse -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum -ErrorAction SilentlyContinue).Sum / 1GB
Write-Host "AppData\Roaming: $([math]::Round($roaming, 2)) GB"

# Check LocalLow
$locallow = (Get-ChildItem "$appdataPath\LocalLow" -Recurse -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum -ErrorAction SilentlyContinue).Sum / 1GB
Write-Host "AppData\LocalLow: $([math]::Round($locallow, 2)) GB"
