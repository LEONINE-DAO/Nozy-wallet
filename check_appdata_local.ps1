# Check what's inside AppData\Local (the biggest culprit!)

Write-Host "Top directories in AppData\Local:" -ForegroundColor Cyan
Write-Host ""

$localPath = "C:\Users\user\AppData\Local"

Get-ChildItem $localPath -Directory -ErrorAction SilentlyContinue | 
    ForEach-Object {
        $size = (Get-ChildItem $_.FullName -Recurse -ErrorAction SilentlyContinue | 
                 Measure-Object -Property Length -Sum -ErrorAction SilentlyContinue).Sum
        $sizeGB = [math]::Round($size / 1GB, 2)
        if ($sizeGB -gt 0.5) {
            [PSCustomObject]@{
                Folder = $_.Name
                "Size (GB)" = $sizeGB
            }
        }
    } | Sort-Object "Size (GB)" -Descending | Format-Table -AutoSize
