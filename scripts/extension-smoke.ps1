Param()

$ErrorActionPreference = "Stop"

Write-Host "== Nozy Extension Smoke ==" -ForegroundColor Cyan

function Invoke-Step {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][scriptblock]$Action
    )
    Write-Host ""
    Write-Host "-> $Name" -ForegroundColor Yellow
    & $Action
    Write-Host "<- $Name OK" -ForegroundColor Green
}

Push-Location $PSScriptRoot\..
try {
    Invoke-Step "Worker utility tests" {
        node --test "browser-extension/background/tx-utils.test.mjs" "browser-extension/background/mobile-sync.test.mjs" "browser-extension/background/tx-lifecycle.test.mjs"
    }

    Invoke-Step "Popup typecheck + build" {
        Push-Location "browser-extension/popup"
        try {
            npm run typecheck
            npm run build
        } finally {
            Pop-Location
        }
    }

    Invoke-Step "WASM core host compile check" {
        Push-Location "browser-extension/wasm-core"
        try {
            cargo check
        } finally {
            Pop-Location
        }
    }

    Invoke-Step "WASM core unit tests" {
        Push-Location "browser-extension/wasm-core"
        try {
            cargo test --lib
        } finally {
            Pop-Location
        }
    }

    Invoke-Step "WASM target compile check" {
        Push-Location "browser-extension/wasm-core"
        try {
            if (-not (rustup target list --installed | Select-String -Pattern "^wasm32-unknown-unknown$")) {
                rustup target add wasm32-unknown-unknown
            }
            $env:CARGO_BUILD_RUSTC = (rustup which --toolchain stable rustc)
            rustup run stable cargo check --target wasm32-unknown-unknown
        } finally {
            Remove-Item Env:\CARGO_BUILD_RUSTC -ErrorAction SilentlyContinue
            Pop-Location
        }
    }

    Write-Host ""
    Write-Host "All automated smoke checks passed." -ForegroundColor Green
    Write-Host "Note: browser UI click-flow smoke still requires browser-extension install context." -ForegroundColor DarkYellow
}
finally {
    Pop-Location
}

