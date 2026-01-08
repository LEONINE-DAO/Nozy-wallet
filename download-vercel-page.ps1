# PowerShell script to download Vercel landing page and assets

$vercelUrl = "https://nozy-wallet.vercel.app/"
$outputDir = "vercel-landing"
$htmlFile = "$outputDir\index.html"

Write-Host "📥 Downloading landing page from Vercel..." -ForegroundColor Cyan

# Create output directory
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
New-Item -ItemType Directory -Force -Path "$outputDir\assets" | Out-Null

# Download HTML
Write-Host "Downloading HTML..." -ForegroundColor Yellow
try {
    Invoke-WebRequest -Uri $vercelUrl -OutFile $htmlFile -UseBasicParsing
    Write-Host "✅ HTML downloaded to $htmlFile" -ForegroundColor Green
} catch {
    Write-Host "❌ Failed to download HTML: $_" -ForegroundColor Red
    exit 1
}

# Extract and download assets (CSS, JS, images)
Write-Host "`n📦 Extracting asset URLs..." -ForegroundColor Cyan
$htmlContent = Get-Content $htmlFile -Raw

# Find CSS files
$cssMatches = [regex]::Matches($htmlContent, 'href=["'']([^"'']*\.css[^"'']*)["'']')
Write-Host "Found $($cssMatches.Count) CSS files" -ForegroundColor Yellow
foreach ($match in $cssMatches) {
    $cssUrl = $match.Groups[1].Value
    if ($cssUrl -notlike "http*") {
        $cssUrl = "https://nozy-wallet.vercel.app$cssUrl"
    }
    $cssFile = Split-Path -Leaf $cssUrl
    Write-Host "  Downloading: $cssFile" -ForegroundColor Gray
    try {
        Invoke-WebRequest -Uri $cssUrl -OutFile "$outputDir\assets\$cssFile" -UseBasicParsing
    } catch {
        Write-Host "    ⚠️  Failed: $_" -ForegroundColor Yellow
    }
}

# Find JS files
$jsMatches = [regex]::Matches($htmlContent, 'src=["'']([^"'']*\.js[^"'']*)["'']')
Write-Host "Found $($jsMatches.Count) JS files" -ForegroundColor Yellow
foreach ($match in $jsMatches) {
    $jsUrl = $match.Groups[1].Value
    if ($jsUrl -notlike "http*") {
        $jsUrl = "https://nozy-wallet.vercel.app$jsUrl"
    }
    $jsFile = Split-Path -Leaf $jsUrl
    Write-Host "  Downloading: $jsFile" -ForegroundColor Gray
    try {
        Invoke-WebRequest -Uri $jsUrl -OutFile "$outputDir\assets\$jsFile" -UseBasicParsing
    } catch {
        Write-Host "    ⚠️  Failed: $_" -ForegroundColor Yellow
    }
}

# Find image files
$imgMatches = [regex]::Matches($htmlContent, 'src=["'']([^"'']*\.(png|jpg|jpeg|svg|gif|webp)[^"'']*)["'']')
Write-Host "Found $($imgMatches.Count) image files" -ForegroundColor Yellow
foreach ($match in $imgMatches) {
    $imgUrl = $match.Groups[1].Value
    if ($imgUrl -notlike "http*") {
        $imgUrl = "https://nozy-wallet.vercel.app$imgUrl"
    }
    $imgFile = Split-Path -Leaf $imgUrl
    Write-Host "  Downloading: $imgFile" -ForegroundColor Gray
    try {
        Invoke-WebRequest -Uri $imgUrl -OutFile "$outputDir\assets\$imgFile" -UseBasicParsing
    } catch {
        Write-Host "    ⚠️  Failed: $_" -ForegroundColor Yellow
    }
}

Write-Host "`n✅ Download complete!" -ForegroundColor Green
Write-Host "Files saved to: $outputDir" -ForegroundColor Cyan
Write-Host "`nNext steps:" -ForegroundColor Yellow
Write-Host "1. Review the files in $outputDir" -ForegroundColor White
Write-Host "2. Copy index.html to repo root (replace existing)" -ForegroundColor White
Write-Host "3. Copy assets to assets/ folder" -ForegroundColor White
Write-Host "4. Update paths in index.html if needed" -ForegroundColor White
Write-Host "5. Commit and push" -ForegroundColor White

