# Zebra Node RPC Compatibility Test
Write-Host "🔍 Zebra Node RPC Compatibility Test" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan
Write-Host

# Default URL
$ZEBRA_URL = if ($args.Count -gt 0) { $args[0] } else { "http://127.0.0.1:8232" }

Write-Host "Testing Zebra node at: $ZEBRA_URL" -ForegroundColor Yellow
Write-Host

# Test 1: Basic connectivity with curl
Write-Host "1️⃣ Testing basic RPC connectivity with curl..." -ForegroundColor Green

try {
    $response = Invoke-RestMethod -Uri $ZEBRA_URL -Method Post -ContentType "application/json" -Body '{"jsonrpc": "2.0", "method": "getblockcount", "params": [], "id":1}' -ErrorAction Stop
    
    if ($response.result -ne $null) {
        Write-Host "✅ RPC endpoint is accessible" -ForegroundColor Green
        Write-Host "   Block count: $($response.result)" -ForegroundColor White
        
        if ($response.result -gt 0) {
            Write-Host "   ✅ Node is synchronized" -ForegroundColor Green
        } else {
            Write-Host "   ⚠️  Node is starting up (0 blocks)" -ForegroundColor Yellow
        }
    } else {
        Write-Host "   ❌ Invalid RPC response" -ForegroundColor Red
        Write-Host $response
    }
} catch {
    Write-Host "❌ Cannot connect to RPC endpoint" -ForegroundColor Red
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host
    Write-Host "Troubleshooting:" -ForegroundColor Yellow
    Write-Host "1. Is Zebra running? Check with: Get-Process | Where-Object {$_.ProcessName -like '*zebra*'}"
    Write-Host "2. Is RPC enabled? Check ~/.config/zebrad.toml for:"
    Write-Host "   [rpc]"
    Write-Host "   listen_addr = `"127.0.0.1:8232`""
    Write-Host "3. Is port 8232 open? Check with: netstat -an | findstr 8232"
    exit 1
}

Write-Host

# Test 2: Test additional RPC methods
Write-Host "2️⃣ Testing additional RPC methods..." -ForegroundColor Green

$methods = @("getnetworkinfo", "getmempoolinfo", "gettxoutsetinfo")

foreach ($method in $methods) {
    Write-Host "   Testing $method... " -NoNewline
    try {
        $response = Invoke-RestMethod -Uri $ZEBRA_URL -Method Post -ContentType "application/json" -Body "{\"jsonrpc\": \"2.0\", \"method\": \"$method\", \"params\": [], \"id\":1}" -ErrorAction Stop
        if ($response.result -ne $null) {
            Write-Host "✅" -ForegroundColor Green
        } else {
            Write-Host "❌" -ForegroundColor Red
        }
    } catch {
        Write-Host "❌" -ForegroundColor Red
    }
}

Write-Host

# Test 3: Run Rust-based tests
Write-Host "3️⃣ Running Rust-based compatibility tests..." -ForegroundColor Green
Write-Host "   Building and running quick test..." -ForegroundColor White

try {
    $result = cargo run --bin quick_test $ZEBRA_URL 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✅ Rust tests passed" -ForegroundColor Green
    } else {
        Write-Host "   ❌ Rust tests failed" -ForegroundColor Red
        Write-Host "   Run manually: cargo run --bin quick_test $ZEBRA_URL" -ForegroundColor Yellow
    }
} catch {
    Write-Host "   ❌ Rust tests failed" -ForegroundColor Red
    Write-Host "   Run manually: cargo run --bin quick_test $ZEBRA_URL" -ForegroundColor Yellow
}

Write-Host

# Test 4: Run comprehensive diagnostic
Write-Host "4️⃣ Running comprehensive diagnostic..." -ForegroundColor Green
Write-Host "   Building and running diagnostic tool..." -ForegroundColor White

try {
    $result = cargo run --bin diagnose_zebra $ZEBRA_URL 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✅ Comprehensive tests passed" -ForegroundColor Green
    } else {
        Write-Host "   ❌ Comprehensive tests failed" -ForegroundColor Red
        Write-Host "   Run manually: cargo run --bin diagnose_zebra $ZEBRA_URL" -ForegroundColor Yellow
    }
} catch {
    Write-Host "   ❌ Comprehensive tests failed" -ForegroundColor Red
    Write-Host "   Run manually: cargo run --bin diagnose_zebra $ZEBRA_URL" -ForegroundColor Yellow
}

Write-Host
Write-Host "🎉 Zebra node compatibility test completed!" -ForegroundColor Cyan
Write-Host
Write-Host "If all tests passed, your Zebra node is compatible with NozyWallet!" -ForegroundColor Green
Write-Host "If tests failed, check the troubleshooting tips above." -ForegroundColor Yellow
