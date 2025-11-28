# PowerShell script to diagnose and fix cargo connection errors

Write-Host "Diagnosing Cargo Connection Issues..." -ForegroundColor Cyan
Write-Host ""

# Check network connectivity to crates.io
Write-Host "1. Testing network connectivity to crates.io..." -ForegroundColor Yellow
$cratesIoTest = Test-NetConnection -ComputerName "crates.io" -Port 443 -InformationLevel Quiet -WarningAction SilentlyContinue
if ($cratesIoTest) {
    Write-Host "   [OK] Can reach crates.io" -ForegroundColor Green
} else {
    Write-Host "   [ERROR] Cannot reach crates.io - network issue detected" -ForegroundColor Red
    Write-Host "   [TIP] Check your internet connection or firewall settings" -ForegroundColor Yellow
}

# Check if cargo is configured
Write-Host ""
Write-Host "2. Checking Cargo configuration..." -ForegroundColor Yellow
$cargoConfig = "$env:USERPROFILE\.cargo\config.toml"
if (Test-Path $cargoConfig) {
    Write-Host "   [OK] Cargo config found at: $cargoConfig" -ForegroundColor Green
    Get-Content $cargoConfig | Select-Object -First 10
} else {
    Write-Host "   [INFO] No custom Cargo config found (using defaults)" -ForegroundColor Cyan
}

# Check if dependencies are already cached
Write-Host ""
Write-Host "3. Checking cached dependencies..." -ForegroundColor Yellow
$cargoCache = "$env:USERPROFILE\.cargo\registry\cache"
if (Test-Path $cargoCache) {
    $cacheSize = (Get-ChildItem $cargoCache -Recurse -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum / 1MB
    Write-Host "   [OK] Cargo cache found: $([math]::Round($cacheSize, 2)) MB" -ForegroundColor Green
    Write-Host "   [TIP] You can try building in offline mode if network fails" -ForegroundColor Yellow
} else {
    Write-Host "   [WARN] No cached dependencies found" -ForegroundColor Yellow
}

# Check for proxy settings
Write-Host ""
Write-Host "4. Checking proxy settings..." -ForegroundColor Yellow
$proxyVars = @("HTTP_PROXY", "HTTPS_PROXY", "http_proxy", "https_proxy", "ALL_PROXY", "all_proxy")
$hasProxy = $false
foreach ($var in $proxyVars) {
    $value = [Environment]::GetEnvironmentVariable($var, "User")
    if ($value) {
        Write-Host "   [INFO] $var = $value" -ForegroundColor Cyan
        $hasProxy = $true
    }
}
if (-not $hasProxy) {
    Write-Host "   [OK] No proxy configured (direct connection)" -ForegroundColor Green
}

# Try to fetch a test crate
Write-Host ""
Write-Host "5. Testing cargo registry access..." -ForegroundColor Yellow
$testFetch = cargo fetch --manifest-path Cargo.toml 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "   [OK] Cargo can fetch dependencies" -ForegroundColor Green
} else {
    Write-Host "   [ERROR] Cargo fetch failed" -ForegroundColor Red
    Write-Host "   Error output:" -ForegroundColor Red
    $testFetch | Select-Object -First 10
}

Write-Host ""
Write-Host "Troubleshooting Steps:" -ForegroundColor Cyan
Write-Host ""
Write-Host "If you are getting connection errors, try these solutions:" -ForegroundColor White
Write-Host ""
Write-Host "1. Use offline mode (if dependencies are cached):" -ForegroundColor Yellow
Write-Host "   cargo build --release --offline" -ForegroundColor White
Write-Host ""
Write-Host "2. Clear cargo cache and retry:" -ForegroundColor Yellow
Write-Host "   cargo clean" -ForegroundColor White
Write-Host "   cargo build --release" -ForegroundColor White
Write-Host ""
Write-Host "3. Configure cargo to use a registry mirror (if behind firewall):" -ForegroundColor Yellow
Write-Host "   Create: $env:USERPROFILE\.cargo\config.toml" -ForegroundColor White
Write-Host "   Add:" -ForegroundColor White
Write-Host "   [registry]" -ForegroundColor White
Write-Host "   default = rsproxy" -ForegroundColor White
Write-Host ""
Write-Host "   [registries.rsproxy]" -ForegroundColor White
Write-Host "   index = https://rsproxy.cn/crates.io-index" -ForegroundColor White
Write-Host ""
Write-Host "4. Increase timeout (if slow connection):" -ForegroundColor Yellow
Write-Host "   Set environment variable: CARGO_NET_TIMEOUT=300" -ForegroundColor White
Write-Host ""
Write-Host "5. Check firewall/antivirus settings" -ForegroundColor Yellow
Write-Host "   Make sure cargo.exe is allowed through firewall" -ForegroundColor White
Write-Host ""
