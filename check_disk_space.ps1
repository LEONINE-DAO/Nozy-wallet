# Quick disk space and cleanup verification script

Write-Host "=== Disk Space Check ===" -ForegroundColor Cyan
$drive = Get-PSDrive C
$usedGB = [math]::Round($drive.Used / 1GB, 2)
$freeGB = [math]::Round($drive.Free / 1GB, 2)
$totalGB = [math]::Round(($drive.Used + $drive.Free) / 1GB, 2)
$percentUsed = [math]::Round(($drive.Used / ($drive.Used + $drive.Free)) * 100, 1)

Write-Host "C: Drive Status:" -ForegroundColor Yellow
Write-Host "  Total: $totalGB GB" -ForegroundColor White
Write-Host "  Used:  $usedGB GB ($percentUsed%)" -ForegroundColor $(if ($percentUsed -gt 95) { "Red" } else { "Green" })
Write-Host "  Free:  $freeGB GB" -ForegroundColor Green
Write-Host ""

Write-Host "=== Checking for Node/Bot Data ===" -ForegroundColor Cyan

$locations = @(
    @{Path="$env:USERPROFILE\OneDrive\Evms\zebra-node"; Name="Zebra node (OneDrive)"},
    @{Path="$env:USERPROFILE\zebra"; Name="Zebra node"},
    @{Path="$env:USERPROFILE\zebrad"; Name="Zebrad"},
    @{Path="$env:USERPROFILE\.zebra"; Name="Zebra (hidden)"},
    @{Path="$env:USERPROFILE\crypto-price-bot"; Name="Atom bot"},
    @{Path="$env:USERPROFILE\levana-trading-bot"; Name="ETH bot"},
    @{Path="$env:USERPROFILE\.juno"; Name="Juno node"},
    @{Path="$env:USERPROFILE\juno"; Name="Juno"}
)

foreach ($loc in $locations) {
    if (Test-Path $loc.Path) {
        $size = (Get-ChildItem -Path $loc.Path -Recurse -ErrorAction SilentlyContinue | 
                 Measure-Object -Property Length -Sum).Sum
        $sizeGB = [math]::Round($size / 1GB, 2)
        if ($sizeGB -gt 0.01) {
            Write-Host "  FOUND: $($loc.Name) - $sizeGB GB" -ForegroundColor Red
        } else {
            Write-Host "  Found: $($loc.Name) - < 0.01 GB (tiny)" -ForegroundColor Yellow
        }
    } else {
        Write-Host "  OK: $($loc.Name) - Not found" -ForegroundColor Green
    }
}

Write-Host ""
Write-Host "=== OneDrive Recycle Bin Check ===" -ForegroundColor Cyan
$recycleBin = "$env:USERPROFILE\OneDrive\Evms"
if (Test-Path $recycleBin) {
    $items = Get-ChildItem -Path $recycleBin -Recurse -ErrorAction SilentlyContinue
    if ($items) {
        $size = ($items | Measure-Object -Property Length -Sum).Sum
        $sizeGB = [math]::Round($size / 1GB, 2)
        Write-Host "  OneDrive/Evms folder exists with $sizeGB GB" -ForegroundColor Yellow
    } else {
        Write-Host "  OneDrive/Evms folder is empty" -ForegroundColor Green
    }
} else {
    Write-Host "  OneDrive/Evms folder not found" -ForegroundColor Green
}

