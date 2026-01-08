#!/bin/bash
echo "🧪 Testing Tauri Desktop Client Setup"
echo "====================================="

# Test 1: Check if Tauri CLI is available
echo "1. Checking Tauri CLI..."
if ! command -v cargo-tauri &> /dev/null; then
    echo "❌ Tauri CLI not found. Install with: cargo install tauri-cli"
    exit 1
fi
echo "✅ Tauri CLI found"

# Test 2: Check Rust compilation
echo "2. Testing Rust backend compilation..."
cd src-tauri
if ! cargo check > /dev/null 2>&1; then
    echo "❌ Rust backend compilation failed"
    exit 1
fi
echo "✅ Rust backend compiles successfully"
cd ..

# Test 3: Check Node.js dependencies
echo "3. Checking Node.js dependencies..."
if ! npm list @tauri-apps/api > /dev/null 2>&1; then
    echo "❌ @tauri-apps/api not installed"
    exit 1
fi
echo "✅ Node.js dependencies installed"

# Test 4: Check Vite build
echo "4. Testing Vite build..."
if ! npm run build > /dev/null 2>&1; then
    echo "❌ Vite build failed"
    exit 1
fi
echo "✅ Vite build successful"

# Test 5: Check Tauri configuration
echo "5. Checking Tauri configuration..."
if [ ! -f "src-tauri/tauri.conf.json" ]; then
    echo "❌ tauri.conf.json not found"
    exit 1
fi
if ! grep -q '"identifier"' src-tauri/tauri.conf.json; then
    echo "❌ identifier not found in tauri.conf.json"
    exit 1
fi
echo "✅ Tauri configuration valid"

echo ""
echo "🎉 All tests passed! Ready to run:"
echo "   cargo tauri dev"
echo ""
echo "This will start the beautiful NozyWallet desktop app!"
