#!/bin/bash
# Permanent setup for Git Bash to work with Rust/MSVC
# This adds the fix to your ~/.bashrc so it runs automatically

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FIX_SCRIPT="$SCRIPT_DIR/fix-git-bash-path.sh"

# Check if fix script exists
if [ ! -f "$FIX_SCRIPT" ]; then
    echo "❌ fix-git-bash-path.sh not found!"
    exit 1
fi

# Make fix script executable
chmod +x "$FIX_SCRIPT"

# Add to .bashrc if not already there
BASHRC="$HOME/.bashrc"

if [ ! -f "$BASHRC" ]; then
    touch "$BASHRC"
fi

# Check if already added
if grep -q "fix-git-bash-path.sh" "$BASHRC"; then
    echo "✅ Already configured in .bashrc"
else
    echo "" >> "$BASHRC"
    echo "# Fix PATH for Visual Studio MSVC linker (for Rust builds)" >> "$BASHRC"
    echo "source \"$FIX_SCRIPT\"" >> "$BASHRC"
    echo "✅ Added to .bashrc"
fi

echo ""
echo "✅ Setup complete!"
echo ""
echo "The PATH fix will now run automatically when you open Git Bash."
echo ""
echo "To test, open a new Git Bash window and run:"
echo "  which link.exe"
echo "  # Should show Visual Studio linker path"
echo ""
echo "Then you can build Rust projects:"
echo "  cd api-server"
echo "  cargo build"

