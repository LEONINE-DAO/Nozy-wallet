#!/bin/bash
# Test script for Shade integration

echo "Testing Shade Protocol Integration..."
echo "======================================"
echo ""

echo "1. Testing list-tokens command..."
cargo run --bin nozy -- shade list-tokens
echo ""

echo "2. Testing info command (SHD token)..."
cargo run --bin nozy -- shade info --token secret1qfql357amn448duf5gvp9gr48sxx9tsnhupu3d
echo ""

echo "3. Testing info command (SILK token)..."
cargo run --bin nozy -- shade info --token secret1fl449muk5yq8dlad7a22nje4p5d2pnsgymhjfd
echo ""

echo "✅ All tests completed!"
echo ""
echo "Note: Balance checks require a valid Secret Network address."
echo "Example: cargo run --bin nozy -- shade balance --address secret1your-address-here"
