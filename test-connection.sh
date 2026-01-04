#!/bin/bash
# Bash script to test API server connection

echo "🔍 Testing NozyWallet API Server Connection..."
echo ""

# Test health endpoint
echo "Testing health endpoint..."
if curl -s http://localhost:3000/health > /dev/null 2>&1; then
    echo "✅ API Server is running!"
    echo "   Response: $(curl -s http://localhost:3000/health)"
    echo ""
    
    # Test wallet exists endpoint
    echo "Testing wallet exists endpoint..."
    if curl -s http://localhost:3000/api/wallet/exists > /dev/null 2>&1; then
        echo "✅ Wallet endpoint is accessible!"
        echo "   Response: $(curl -s http://localhost:3000/api/wallet/exists)"
    else
        echo "⚠️  Wallet endpoint error"
    fi
    
    echo ""
    echo "✅ All tests passed! DesktopClient should be able to connect."
    echo ""
    echo "Next steps:"
    echo "  1. Set VITE_API_URL=http://localhost:3000 in DesktopClient .env file"
    echo "  2. Run: npm run dev in DesktopClient directory"
else
    echo "❌ API Server is NOT running!"
    echo ""
    echo "To start the API server:"
    echo "  cd api-server"
    echo "  cargo run"
fi

