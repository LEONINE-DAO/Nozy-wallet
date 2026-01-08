#!/bin/bash
# Bash script to start API server for desktop client development

echo "🚀 Starting NozyWallet API Server for Desktop Client..."
echo ""

# Check if running from project root
if [ ! -d "api-server" ]; then
    echo "❌ Error: Please run this script from the project root directory"
    exit 1
fi

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Cargo is not installed. Please install Rust first."
    exit 1
fi

cd api-server

# Build if needed
if [ ! -f "target/release/nozywallet-api" ]; then
    echo "📦 Building API server..."
    cargo build --release
    if [ $? -ne 0 ]; then
        echo "❌ Build failed!"
        exit 1
    fi
fi

echo ""
echo "✅ API Server Configuration:"
echo "   URL: http://localhost:3000"
echo "   Health Check: http://localhost:3000/health"
echo ""
echo "📱 Desktop Client Configuration:"
echo "   Set API_URL=http://localhost:3000 in your desktop client"
echo "   Or use the API client examples from DESKTOP_CLIENT_INTEGRATION.md"
echo ""
echo "🌐 Starting server..."
echo ""

# Run the server
cargo run --release
