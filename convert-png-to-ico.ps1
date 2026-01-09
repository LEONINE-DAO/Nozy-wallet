# PowerShell script to convert PNG to ICO for Windows icons
param(
    [string]$InputFile = "assets\logo.png",
    [string]$OutputFile = "assets\icon.ico"
)

Write-Host "Converting PNG to ICO..." -ForegroundColor Green
Write-Host "Input:  $InputFile"
Write-Host "Output: $OutputFile"

if (-not (Test-Path $InputFile)) {
    Write-Host "Error: Input file not found: $InputFile" -ForegroundColor Red
    exit 1
}

try {
    Add-Type -AssemblyName System.Drawing
    
    $inputPath = (Resolve-Path $InputFile).Path
    $pngImage = [System.Drawing.Image]::FromFile($inputPath)
    
    $outputDir = Split-Path -Parent $OutputFile
    if ($outputDir -and -not (Test-Path $outputDir)) {
        New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
    }
    
    $iconSize = 256
    $resized = New-Object System.Drawing.Bitmap($iconSize, $iconSize)
    $graphics = [System.Drawing.Graphics]::FromImage($resized)
    $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $graphics.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
    
    $graphics.DrawImage($pngImage, 0, 0, $iconSize, $iconSize)
    $graphics.Dispose()
    
    $outputPath = $OutputFile
    if (-not [System.IO.Path]::IsPathRooted($outputPath)) {
        $outputPath = Join-Path (Get-Location).Path $OutputFile
    }
    
    $tempPng = [System.IO.Path]::GetTempFileName() + ".png"
    $resized.Save($tempPng, [System.Drawing.Imaging.ImageFormat]::Png)
    
    $pngData = [System.IO.File]::ReadAllBytes($tempPng)
    $pngSize = $pngData.Length
    
    $icoStream = [System.IO.File]::Create($outputPath)
    $writer = New-Object System.IO.BinaryWriter($icoStream)
    
    $writer.Write([byte]0)
    $writer.Write([byte]0)
    $writer.Write([byte]1)
    $writer.Write([byte]0)
    $writer.Write([byte]1)
    $writer.Write([byte]0)
    
    $writer.Write([byte]0)
    $writer.Write([byte]0)
    $writer.Write([byte]0)
    $writer.Write([byte]0)
    $writer.Write([UInt16]1)
    $writer.Write([UInt16]32)
    
    $dataOffset = 22
    $writer.Write([UInt32]$dataOffset)
    $writer.Write([UInt32]$pngSize)
    
    $writer.Write($pngData)
    
    $writer.Close()
    $icoStream.Close()
    
    $resized.Dispose()
    $pngImage.Dispose()
    Remove-Item $tempPng -Force
    
    Write-Host "Successfully converted to ICO!" -ForegroundColor Green
    Write-Host "Output file: $outputPath"
    
    if (Test-Path $outputPath) {
        $fileInfo = Get-Item $outputPath
        Write-Host "File size: $($fileInfo.Length) bytes"
    }
} catch {
    Write-Host "Error converting image: $_" -ForegroundColor Red
    Write-Host "Alternative: Use an online converter at:" -ForegroundColor Yellow
    Write-Host "https://convertio.co/png-ico/" -ForegroundColor Cyan
    Write-Host "https://cloudconvert.com/png-to-ico" -ForegroundColor Cyan
    exit 1
}
