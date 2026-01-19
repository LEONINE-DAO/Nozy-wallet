#!/bin/bash
# Run cargo-chec for parallel cargo checks
# cargo-chec runs multiple cargo checks in parallel and outputs JSON

echo "=== Running cargo-chec ==="
echo ""

if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust first."
    exit 1
fi

if ! cargo chec --version &> /dev/null; then
    echo "cargo-chec not found. Installing..."
    cargo install cargo-chec
    if [ $? -ne 0 ]; then
        echo "❌ Failed to install cargo-chec"
        exit 1
    fi
    echo "✅ cargo-chec installed successfully"
    echo ""
fi

echo "Running cargo chec (parallel checks)..."
echo "This will run: cargo check, cargo clippy, cargo fmt, and cargo test"
echo ""

# Run cargo-chec
cargo chec

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ All checks passed!"
else
    echo ""
    echo "⚠️  Some checks failed. Review output above."
fi

echo ""
echo "=== Complete ==="
echo ""
echo "Note: cargo-chec outputs JSON by default."
echo "For human-readable output, use: cargo chec --pretty"
