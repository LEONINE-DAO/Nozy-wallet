# PowerShell test script for Shade Protocol integration

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Shade Protocol Integration Tests" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

$TestsPassed = 0
$TestsFailed = 0

function Test-Command {
    param(
        [string]$TestName,
        [string]$Command
    )
    
    Write-Host -NoNewline "Testing: $TestName... "
    
    try {
        $result = Invoke-Expression $Command 2>&1
        if ($LASTEXITCODE -eq 0 -or $?) {
            Write-Host "✓ PASSED" -ForegroundColor Green
            $script:TestsPassed++
            return $true
        } else {
            Write-Host "✗ FAILED" -ForegroundColor Red
            $script:TestsFailed++
            return $false
        }
    } catch {
        Write-Host "✗ FAILED" -ForegroundColor Red
        $script:TestsFailed++
        return $false
    }
}

# Test 1: Compilation
Write-Host "1. Compilation Tests" -ForegroundColor Yellow
Write-Host "-------------------"
Test-Command "Code compilation" "cargo check --lib"
Test-Command "Binary compilation" "cargo build --bin nozy"
Write-Host ""

# Test 2: CLI Commands
Write-Host "2. CLI Command Tests" -ForegroundColor Yellow
Write-Host "-------------------"
Test-Command "Shade command exists" "cargo run --bin nozy -- shade --help"
Test-Command "List tokens command" "cargo run --bin nozy -- shade list-tokens"
Write-Host ""

# Test 3: Module Structure
Write-Host "3. Module Structure Tests" -ForegroundColor Yellow
Write-Host "-------------------"
Test-Command "Secret module exists" "Test-Path src/secret/mod.rs"
Test-Command "Secret keys module exists" "Test-Path src/secret_keys.rs"
Test-Command "SNIP-20 module exists" "Test-Path src/secret/snip20.rs"
Test-Command "Transaction module exists" "Test-Path src/secret/transaction.rs"
Test-Command "RPC client exists" "Test-Path src/secret/rpc_client.rs"
Test-Command "Wallet module exists" "Test-Path src/secret/wallet.rs"
Write-Host ""

# Test 4: Documentation
Write-Host "4. Documentation Tests" -ForegroundColor Yellow
Write-Host "-------------------"
Test-Command "Integration docs exist" "Test-Path SHADE_INTEGRATION.md"
Test-Command "Notes exist" "Test-Path SHADE_INTEGRATION_NOTES.md"
Test-Command "Checklist exists" "Test-Path GITHUB_PUSH_CHECKLIST.md"
Test-Command "Summary exists" "Test-Path IMPLEMENTATION_SUMMARY.md"
Write-Host ""

# Test 5: Dependencies
Write-Host "5. Dependency Tests" -ForegroundColor Yellow
Write-Host "-------------------"
$cargoToml = Get-Content Cargo.toml -Raw
Test-Command "secp256k1 dependency" { $cargoToml -match "secp256k1" }
Test-Command "cosmrs dependency" { $cargoToml -match "cosmrs" }
Test-Command "tendermint dependency" { $cargoToml -match "tendermint" }
Test-Command "ripemd dependency" { $cargoToml -match "ripemd" }
Write-Host ""

# Summary
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Test Summary" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "Passed: $TestsPassed" -ForegroundColor Green
Write-Host "Failed: $TestsFailed" -ForegroundColor $(if ($TestsFailed -eq 0) { "Green" } else { "Red" })
Write-Host ""

if ($TestsFailed -eq 0) {
    Write-Host "All tests passed! ✓" -ForegroundColor Green
    exit 0
} else {
    Write-Host "Some tests failed. Please review." -ForegroundColor Red
    exit 1
}
