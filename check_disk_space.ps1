# Check disk space usage

Write-Host "Checking C:\ drive usage..." -ForegroundColor Cyan
Write-Host ""

# Check user directories
Write-Host "Top directories in C:\Users\user (excluding Nozy-wallet):" -ForegroundColor Yellow
Get-ChildItem "C:\Users\user" -Directory -ErrorAction SilentlyContinue | 
    Where-Object { $_.Name -ne "Nozy-wallet" } |
    ForEach-Object {
        $size = (Get-ChildItem $_.FullName -Recurse -ErrorAction SilentlyContinue | 
                 Measure-Object -Property Length -Sum -ErrorAction SilentlyContinue).Sum
        $sizeGB = [math]::Round($size / 1GB, 2)
        if ($sizeGB -gt 0.1) {
            [PSCustomObject]@{
                Name = $_.Name
                "Size (GB)" = $sizeGB
            }
        }
    } | Sort-Object "Size (GB)" -Descending | Format-Table -AutoSize

Write-Host ""
Write-Host "Checking common large locations:" -ForegroundColor Yellow
Write-Host ""

# Check Program Files
$pf = (Get-ChildItem "C:\Program Files" -Recurse -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum / 1GB
Write-Host "Program Files: $([math]::Round($pf, 2)) GB"

# Check Program Files (x86)
$pfx86 = (Get-ChildItem "C:\Program Files (x86)" -Recurse -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum / 1GB
Write-Host "Program Files (x86): $([math]::Round($pfx86, 2)) GB"

# Check Windows folder
$win = (Get-ChildItem "C:\Windows" -Recurse -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum / 1GB
Write-Host "Windows: $([math]::Round($win, 2)) GB"

# Check AppData
$appdata = (Get-ChildItem "C:\Users\user\AppData" -Recurse -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum / 1GB
Write-Host "AppData: $([math]::Round($appdata, 2)) GB"

# Check Downloads
if (Test-Path "C:\Users\user\Downloads") {
    $dl = (Get-ChildItem "C:\Users\user\Downloads" -Recurse -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum / 1GB
    Write-Host "Downloads: $([math]::Round($dl, 2)) GB"
}

# Check Documents
if (Test-Path "C:\Users\user\Documents") {
    $docs = (Get-ChildItem "C:\Users\user\Documents" -Recurse -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum / 1GB
    Write-Host "Documents: $([math]::Round($docs, 2)) GB"
}

# Check Temp folders
$temp = (Get-ChildItem "C:\Windows\Temp" -Recurse -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum / 1GB
Write-Host "Windows\Temp: $([math]::Round($temp, 2)) GB"

$usertemp = (Get-ChildItem "$env:TEMP" -Recurse -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum / 1GB
Write-Host "User Temp: $([math]::Round($usertemp, 2)) GB"
