#!/bin/bash
# Test script for password functionality

echo "🧪 Testing Password Functionality"
echo "================================="
echo ""

# Clean up any existing test wallet
rm -rf ~/.local/share/nozy/wallet.dat 2>/dev/null || true
rm -rf wallet_data/wallet.dat 2>/dev/null || true

echo "Test 1: Create wallet WITH password"
echo "------------------------------------"
echo "When prompted, enter password: testpass123"
cargo run --bin nozy new <<EOF
y
testpass123
testpass123
EOF

if [ $? -eq 0 ]; then
    echo "✅ Test 1 PASSED: Wallet created with password"
else
    echo "❌ Test 1 FAILED: Could not create wallet"
    exit 1
fi

echo ""
echo "Test 2: Load wallet with CORRECT password"
echo "----------------------------------------"
echo "When prompted, enter password: testpass123"
cargo run --bin nozy balance <<EOF
testpass123
EOF

if [ $? -eq 0 ]; then
    echo "✅ Test 2 PASSED: Wallet loaded with correct password"
else
    echo "❌ Test 2 FAILED: Could not load wallet with correct password"
    exit 1
fi

echo ""
echo "Test 3: Load wallet with WRONG password (should fail)"
echo "----------------------------------------------------"
echo "When prompted, enter password: wrongpass"
cargo run --bin nozy balance <<EOF
wrongpass
EOF

if [ $? -ne 0 ]; then
    echo "✅ Test 3 PASSED: Wallet correctly rejected wrong password"
else
    echo "❌ Test 3 FAILED: Wallet should have rejected wrong password"
    exit 1
fi

echo ""
echo "🎉 All password tests passed!"

