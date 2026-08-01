# Build nozy-ffi for mobile targets and optionally generate UniFFI bindings.
# Requires: Rust, and for Android: NDK + cargo-ndk (see nozy-ffi/README.md).

param(
    [ValidateSet("host", "android", "ios")]
    [string]$Target = "host",
    [ValidateSet("none", "kotlin", "swift")]
    [string]$Bindgen = "none"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Push-Location $Root

try {
    switch ($Target) {
        "host" {
            cargo build -p nozy-ffi --release
            $lib = Join-Path $Root "target\release\nozy_ffi.dll"
            if (-not (Test-Path $lib)) {
                $lib = Join-Path $Root "target\release\libnozy_ffi.so"
            }
            Write-Host "Built: $lib"
        }
        "android" {
            Write-Host "Android cross-compile requires cargo-ndk and NDK targets."
            cargo ndk -t arm64-v8a -t x86_64 build -p nozy-ffi --release
            $lib = Join-Path $Root "target\aarch64-linux-android\release\libnozy_ffi.so"
            $jni = Join-Path $Root "nozy-mobile\modules\nozy-wallet\android\src\main\jniLibs"
            if (Test-Path $lib) {
                New-Item -ItemType Directory -Force -Path (Join-Path $jni "arm64-v8a") | Out-Null
                Copy-Item $lib (Join-Path $jni "arm64-v8a\libnozy_ffi.so") -Force
                Write-Host "Copied arm64-v8a libnozy_ffi.so"
            }
            $lib64 = Join-Path $Root "target\x86_64-linux-android\release\libnozy_ffi.so"
            if (Test-Path $lib64) {
                New-Item -ItemType Directory -Force -Path (Join-Path $jni "x86_64") | Out-Null
                Copy-Item $lib64 (Join-Path $jni "x86_64\libnozy_ffi.so") -Force
                Write-Host "Copied x86_64 libnozy_ffi.so"
            }
        }
        "ios" {
            Write-Host "iOS builds require macOS + Xcode Rust targets."
            exit 1
        }
    }

    if ($Bindgen -ne "none" -and (Test-Path $lib)) {
        $bindgen = Get-Command uniffi-bindgen -ErrorAction SilentlyContinue
        if (-not $bindgen) {
            Write-Host "Install: cargo install uniffi_bindgen --locked --version 0.28.0"
            exit 1
        }
        $outDir = if ($Bindgen -eq "kotlin") {
            Join-Path $Root "nozy-mobile\modules\nozy-wallet\android\src\main\java\uniffi\nozy_ffi"
        } else {
            Join-Path $Root "nozy-mobile\modules\nozy-wallet\ios\bindings\swift"
        }
        New-Item -ItemType Directory -Force -Path $outDir | Out-Null
        uniffi-bindgen generate --library $lib --language $Bindgen --out-dir $outDir
        Write-Host "Generated $Bindgen bindings in $outDir"
    }
}
finally {
    Pop-Location
}
