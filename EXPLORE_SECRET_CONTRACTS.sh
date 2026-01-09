#!/bin/bash
# Script to explore Secret Network contract examples

set -e

echo "🔍 Exploring Secret Network Contract Examples"
echo "=============================================="
echo ""

# Create exploration directory
EXPLORE_DIR="secret-contracts-exploration"
mkdir -p "$EXPLORE_DIR"
cd "$EXPLORE_DIR"

echo "📦 Cloning Secret Network repositories..."
echo ""

# 1. Clone secret-template (starter template)
echo "1️⃣  Cloning secret-template..."
if [ ! -d "secret-template" ]; then
    git clone https://github.com/scrtlabs/secret-template.git
    echo "   ✅ Cloned secret-template"
else
    echo "   ℹ️  secret-template already exists"
fi
echo ""

# 2. Clone secret-contracts (official examples)
echo "2️⃣  Cloning secret-contracts..."
if [ ! -d "secret-contracts" ]; then
    git clone https://github.com/scrtlabs/secret-contracts.git
    echo "   ✅ Cloned secret-contracts"
else
    echo "   ℹ️  secret-contracts already exists"
fi
echo ""

# 3. Clone secret-toolkit (utilities)
echo "3️⃣  Cloning secret-toolkit..."
if [ ! -d "secret-toolkit" ]; then
    git clone https://github.com/scrtlabs/secret-toolkit.git
    echo "   ✅ Cloned secret-toolkit"
else
    echo "   ℹ️  secret-toolkit already exists"
fi
echo ""

# 4. Clone secret.js (JavaScript SDK)
echo "4️⃣  Cloning secret.js..."
if [ ! -d "secret.js" ]; then
    git clone https://github.com/scrtlabs/secret.js.git
    echo "   ✅ Cloned secret.js"
else
    echo "   ℹ️  secret.js already exists"
fi
echo ""

echo "📚 Repository Structure:"
echo "========================"
echo ""
echo "secret-template/          - Starter template for new contracts"
echo "secret-contracts/         - Official contract examples"
echo "secret-toolkit/           - Utility library"
echo "secret.js/                - JavaScript SDK"
echo ""

echo "🎯 Key Contracts to Study:"
echo "=========================="
echo ""
echo "1. secret-template/src/    - Basic contract structure"
echo "2. secret-contracts/snip20-reference-impl/ - Privacy token example"
echo "3. secret-contracts/snip721-reference-impl/ - NFT example"
echo "4. secret-toolkit/         - Common utilities"
echo ""

echo "📖 Next Steps:"
echo "=============="
echo ""
echo "1. Explore secret-template/src/ to understand basic structure"
echo "2. Study secret-contracts/snip20-reference-impl/ for privacy patterns"
echo "3. Review secret-toolkit/ for utilities"
echo "4. Check secret.js/examples/ for integration patterns"
echo ""

echo "✅ Exploration setup complete!"
echo ""
echo "Location: $(pwd)"
echo ""
echo "Start exploring:"
echo "  cd secret-template && cat README.md"
echo "  cd ../secret-contracts && ls -la"
echo ""

