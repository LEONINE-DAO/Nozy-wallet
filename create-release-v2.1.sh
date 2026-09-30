

echo "🚀 Creating NozyWallet v2.1.0 Release"
echo ""

if [ ! -d .git ]; then
    echo "❌ Error: Not in a git repository!"
    exit 1
fi

if [ -n "$(git status --porcelain)" ]; then
    echo "⚠️  Warning: You have uncommitted changes:"
    git status --short
    echo ""
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 1
    fi
fi

if git rev-parse v2.1.0 >/dev/null 2>&1; then
    echo "⚠️  Warning: Tag v2.1.0 already exists!"
    read -p "Delete and recreate? (y/N) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Deleting existing tag..."
        git tag -d v2.1.0
        git push origin :refs/tags/v2.1.0
    else
        echo "Aborted."
        exit 1
    fi
fi

echo "📝 Creating tag v2.1.0..."
git tag -a v2.1.0 -m "NozyWallet v2.1.0 Release

Major Features:
- NU 6.1 Support
- Crosslink Backend Integration
- REST API Server
- Security Hardening (93 unwrap() calls removed)
- Deterministic Scanning Tests
- Note Indexing System
- Secret Network Support
- Cross-Chain Swap Framework"

if [ $? -ne 0 ]; then
    echo "❌ Error: Failed to create tag!"
    exit 1
fi

echo "✅ Tag created successfully!"
echo ""

echo "📤 Pushing tag to GitHub..."
git push origin v2.1.0

if [ $? -ne 0 ]; then
    echo "❌ Error: Failed to push tag!"
    echo "You may need to push manually: git push origin v2.1.0"
    exit 1
fi

echo ""
echo "✅ Tag pushed successfully!"
echo ""
echo "🎉 Release workflow triggered!"
echo ""
echo "Next steps:"
echo "1. Monitor workflow: https://github.com/LEONINE-DAO/Nozy-wallet/actions"
echo "2. Check release: https://github.com/LEONINE-DAO/Nozy-wallet/releases"
echo "3. The release will be created automatically when builds complete"
echo ""
echo "The workflow will build:"
echo "  • CLI binaries (Linux, Windows, macOS Intel, macOS ARM)"
echo "  • API server binaries (Linux, Windows, macOS Intel, macOS ARM)"
echo "  • Desktop client installers (from NozyWallet-DesktopClient repo)"
echo "  • SHA256 hashes for all binaries"
echo ""
echo "Note: Desktop client will be cloned from:"
echo "  https://github.com/LEONINE-DAO/NozyWallet-DesktopClient"
echo ""

