# Build nozy-ffi for mobile targets and optionally generate UniFFI bindings.
# Requires: Rust, and for Android: NDK + cargo-ndk (see nozy-ffi/README.md).
# Android cross-compile needs rustup's rustc (android std). If `cargo` is the
# standalone "Rust MSVC" install, prepend `$env:USERPROFILE\.cargo\bin` to PATH.

param(
    [ValidateSet("host", "android", "ios")]
    [string]$Target = "host",
    [ValidateSet("none", "kotlin", "swift")]
    [string]$Bindgen = "none"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$TargetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $Root "target" }
Push-Location $Root

try {
    switch ($Target) {
        "host" {
            cargo build -p nozy-ffi --release
            $lib = Join-Path $TargetDir "release\nozy_ffi.dll"
            if (-not (Test-Path $lib)) {
                $lib = Join-Path $TargetDir "release\libnozy_ffi.so"
            }
            Write-Host "Built: $lib"
        }
        "android" {
            Write-Host "Android cross-compile requires cargo-ndk and NDK targets."
            cargo ndk -t arm64-v8a -t x86_64 build -p nozy-ffi --release
            $lib = Join-Path $TargetDir "aarch64-linux-android\release\libnozy_ffi.so"
            $jni = Join-Path $Root "nozy-mobile\modules\nozy-wallet\android\src\main\jniLibs"
            if (Test-Path $lib) {
                New-Item -ItemType Directory -Force -Path (Join-Path $jni "arm64-v8a") | Out-Null
                Copy-Item $lib (Join-Path $jni "arm64-v8a\libnozy_ffi.so") -Force
                Write-Host "Copied arm64-v8a libnozy_ffi.so"
            }
            $lib64 = Join-Path $TargetDir "x86_64-linux-android\release\libnozy_ffi.so"
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
        $outDir = if ($Bindgen -eq "kotlin") {
            # bindgen writes uniffi/<crate>.kt under this root (package uniffi.nozy_ffi)
            Join-Path $Root "nozy-mobile\modules\nozy-wallet\android\src\main\java"
        } else {
            Join-Path $Root "nozy-mobile\modules\nozy-wallet\ios\bindings\swift"
        }
        New-Item -ItemType Directory -Force -Path $outDir | Out-Null
        cargo run -p nozy-ffi --features bindgen --bin uniffi-bindgen -- generate --library $lib --language $Bindgen --out-dir $outDir
        Write-Host "Generated $Bindgen bindings in $outDir"
    }
}
finally {
    Pop-Location
}
