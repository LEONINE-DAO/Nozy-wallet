# Fix PATH to use Visual Studio linker instead of Git's link.exe

Write-Host "🔧 Fixing PATH for Visual Studio Build Tools..." -ForegroundColor Cyan
Write-Host ""

# Find Visual Studio Build Tools installation
$vsPaths = @(
    "C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC",
    "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC",
    "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC",
    "C:\Program Files (x86)\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC"
)

$linkerPath = $null
foreach ($basePath in $vsPaths) {
    if (Test-Path $basePath) {
        $msvcDirs = Get-ChildItem $basePath -Directory | Sort-Object Name -Descending
        foreach ($msvcDir in $msvcDirs) {
            $linkerPath = Join-Path $msvcDir "bin\Hostx64\x64\link.exe"
            if (Test-Path $linkerPath) {
                Write-Host "✅ Found Visual Studio linker!" -ForegroundColor Green
                Write-Host "   Path: $linkerPath" -ForegroundColor Gray
                break
            }
        }
        if ($linkerPath) { break }
    }
}

if (-not $linkerPath) {
    Write-Host "❌ Visual Studio linker not found!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please ensure Visual Studio Build Tools is installed with C++ workload." -ForegroundColor Yellow
    Write-Host "Or use the Developer Command Prompt:" -ForegroundColor Yellow
    Write-Host "  Start Menu > Visual Studio 2022 > Developer Command Prompt for VS 2022" -ForegroundColor White
    exit 1
}

# Get the bin directory
$vsBinDir = Split-Path $linkerPath

Write-Host ""
Write-Host "📝 To fix PATH, run this command:" -ForegroundColor Yellow
Write-Host ""
Write-Host '$env:PATH = "$vsBinDir;$env:PATH"' -ForegroundColor White
Write-Host ""
Write-Host "Or add this to your PATH permanently:" -ForegroundColor Yellow
Write-Host "  $vsBinDir" -ForegroundColor White
Write-Host ""
Write-Host "Or use Developer Command Prompt (easiest):" -ForegroundColor Cyan
Write-Host "  Start Menu > Visual Studio 2022 > Developer Command Prompt for VS 2022" -ForegroundColor White
Write-Host "  Then run: cd api-server && cargo build" -ForegroundColor White


