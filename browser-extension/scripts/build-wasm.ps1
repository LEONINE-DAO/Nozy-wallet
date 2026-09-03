Param(
    [switch]$Release = $true
)

$ErrorActionPreference = "Stop"

$llvmBin = "C:\Program Files\LLVM\bin"
if (Test-Path $llvmBin) {
    $env:PATH = "$llvmBin;$env:USERPROFILE\.cargo\bin;" + $env:PATH
    $env:CC_wasm32_unknown_unknown = "clang"
    Set-Item -Path "env:CC_wasm32-unknown-unknown" -Value "clang"
} else {
    Write-Host "LLVM not found at $llvmBin" -ForegroundColor Yellow
    Write-Host "Install: winget install -e --id LLVM.LLVM" -ForegroundColor Yellow
    if (-not (Get-Command clang -ErrorAction SilentlyContinue)) {
        throw "clang is required to build secp256k1-sys for wasm32-unknown-unknown"
    }
}

$env:PATH = "$env:USERPROFILE\.cargo\bin;" + $env:PATH

Push-Location $PSScriptRoot\..\wasm-core
try {
    $targetList = & rustup target list --installed
    if ($targetList -notmatch "^wasm32-unknown-unknown$") {
        & rustup target add wasm32-unknown-unknown
    }

    $profile = if ($Release) { "--release" } else { "" }
    Write-Host "Building WASM (this may take several minutes on first compile)..." -ForegroundColor Cyan
    if ($Release) {
        & wasm-pack build --target web --out-dir ../wasm/pkg --release
    } else {
        & wasm-pack build --target web --out-dir ../wasm/pkg
    }
    if ($LASTEXITCODE -ne 0) { throw "wasm-pack failed ($LASTEXITCODE)" }
    Write-Host "WASM build OK -> browser-extension/wasm/pkg" -ForegroundColor Green
}
finally {
    Pop-Location
}
