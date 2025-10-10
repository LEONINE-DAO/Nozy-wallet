#!/bin/bash

echo "🔍 Zebra Node RPC Compatibility Test"
echo "===================================="
echo

# Default URL
ZEBRA_URL=${1:-"http://127.0.0.1:8232"}

echo "Testing Zebra node at: $ZEBRA_URL"
echo

# Test 1: Basic connectivity with curl
echo "1️⃣ Testing basic RPC connectivity with curl..."
curl -s -H 'content-type: application/json' \
     --data-binary '{"jsonrpc": "2.0", "method": "getblockcount", "params": [], "id":1}' \
     "$ZEBRA_URL" > /tmp/zebra_test.json

if [ $? -eq 0 ]; then
    echo "✅ RPC endpoint is accessible"
    
    # Check if we got a valid response
    if grep -q "result" /tmp/zebra_test.json; then
        BLOCK_COUNT=$(cat /tmp/zebra_test.json | grep -o '"result":[0-9]*' | grep -o '[0-9]*')
        echo "   Block count: $BLOCK_COUNT"
        
        if [ "$BLOCK_COUNT" -gt 0 ]; then
            echo "   ✅ Node is synchronized"
        else
            echo "   ⚠️  Node is starting up (0 blocks)"
        fi
    else
        echo "   ❌ Invalid RPC response"
        cat /tmp/zebra_test.json
    fi
else
    echo "❌ Cannot connect to RPC endpoint"
    echo
    echo "Troubleshooting:"
    echo "1. Is Zebra running? Check with: ps aux | grep zebrad"
    echo "2. Is RPC enabled? Check ~/.config/zebrad.toml for:"
    echo "   [rpc]"
    echo "   listen_addr = \"127.0.0.1:8232\""
    echo "3. Is port 8232 open? Check with: netstat -tlnp | grep 8232"
    exit 1
fi

echo

# Test 2: Test additional RPC methods
echo "2️⃣ Testing additional RPC methods..."

methods=("getnetworkinfo" "getmempoolinfo" "gettxoutsetinfo")

for method in "${methods[@]}"; do
    echo -n "   Testing $method... "
    curl -s -H 'content-type: application/json' \
         --data-binary "{\"jsonrpc\": \"2.0\", \"method\": \"$method\", \"params\": [], \"id\":1}" \
         "$ZEBRA_URL" | grep -q "result" && echo "✅" || echo "❌"
done

echo

# Test 3: Run Rust-based tests
echo "3️⃣ Running Rust-based compatibility tests..."
echo "   Building and running quick test..."

if cargo run --bin quick_test "$ZEBRA_URL" 2>/dev/null; then
    echo "   ✅ Rust tests passed"
else
    echo "   ❌ Rust tests failed"
    echo "   Run manually: cargo run --bin quick_test $ZEBRA_URL"
fi

echo

# Test 4: Run comprehensive diagnostic
echo "4️⃣ Running comprehensive diagnostic..."
echo "   Building and running diagnostic tool..."

if cargo run --bin diagnose_zebra "$ZEBRA_URL" 2>/dev/null; then
    echo "   ✅ Comprehensive tests passed"
else
    echo "   ❌ Comprehensive tests failed"
    echo "   Run manually: cargo run --bin diagnose_zebra $ZEBRA_URL"
fi

echo
echo "🎉 Zebra node compatibility test completed!"
echo
echo "If all tests passed, your Zebra node is compatible with NozyWallet!"
echo "If tests failed, check the troubleshooting tips above."
