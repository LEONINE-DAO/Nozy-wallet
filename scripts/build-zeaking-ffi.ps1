# Build zeaking-ffi for mobile targets and optionally generate UniFFI bindings.
# Requires: Rust, protoc, and for Android: NDK + cargo-ndk (see zeaking-ffi/README.md).

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
            cargo build -p zeaking-ffi --release
            $lib = Join-Path $Root "target\release\zeaking_ffi.dll"
            if (-not (Test-Path $lib)) {
                $lib = Join-Path $Root "target\release\libzeaking_ffi.so"
            }
            Write-Host "Built: $lib"
        }
        "android" {
            Write-Host "Android cross-compile requires cargo-ndk and NDK targets."
            Write-Host "Example: cargo ndk -t arm64-v8a build -p zeaking-ffi --release"
            cargo ndk -t arm64-v8a build -p zeaking-ffi --release
            $lib = Join-Path $Root "target\aarch64-linux-android\release\libzeaking_ffi.so"
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
            Join-Path $Root "mobile\android\app\src\main\java\uniffi\zeaking_ffi"
        } else {
            Join-Path $Root "mobile\ios\bindings\swift"
        }
        New-Item -ItemType Directory -Force -Path $outDir | Out-Null
        uniffi-bindgen generate --library $lib --language $Bindgen --out-dir $outDir
        Write-Host "Generated $Bindgen bindings in $outDir"
    }
}
finally {
    Pop-Location
}
