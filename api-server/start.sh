#!/bin/bash

# Start script for NozyWallet API Server

echo "🚀 Starting NozyWallet API Server..."
echo ""

# Check if running from project root
if [ ! -d "api-server" ]; then
    echo "❌ Error: Please run this script from the project root directory"
    exit 1
fi

cd api-server

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Cargo is not installed. Please install Rust first."
    exit 1
fi

# Build if needed
if [ ! -f "target/release/nozywallet-api" ]; then
    echo "📦 Building API server..."
    cargo build --release
fi

# Run the server
echo "✅ Starting server on http://0.0.0.0:3000"
echo "📱 Mobile app should connect to: http://localhost:3000/api"
echo ""
cargo run --release

