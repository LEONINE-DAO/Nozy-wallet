#!/bin/bash
echo "Testing Tauri desktop client build..."

# Test if Tauri can compile the Rust backend
echo "Testing Rust backend compilation..."
cd src-tauri && cargo check
if [ $? -ne 0 ]; then
    echo "❌ Rust backend compilation failed"
    exit 1
fi
echo "✅ Rust backend compiled successfully"

# Test if npm packages are installed
echo "Testing frontend dependencies..."
cd ..
npm list @tauri-apps/api > /dev/null 2>&1
if [ $? -ne 0 ]; then
    echo "❌ Tauri API package not installed"
    exit 1
fi
echo "✅ Frontend dependencies are installed"

# Test if Vite can build
echo "Testing frontend build..."
npm run build
if [ $? -ne 0 ]; then
    echo "❌ Frontend build failed"
    exit 1
fi
echo "✅ Frontend built successfully"

echo "🎉 All tests passed! Ready for development."
