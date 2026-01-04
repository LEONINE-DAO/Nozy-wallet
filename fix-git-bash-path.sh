#!/bin/bash
# Fix PATH for Git Bash to work with Visual Studio MSVC linker
# This script adds Visual Studio linker to PATH before Git's link.exe

# Find Visual Studio linker
VS_LINKER_PATHS=(
    "/c/Program Files/Microsoft Visual Studio/2022/BuildTools/VC/Tools/MSVC"
    "/c/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC"
    "/c/Program Files/Microsoft Visual Studio/18/Insiders/VC/Tools/MSVC"
    "/c/Program Files (x86)/Microsoft Visual Studio/2022/BuildTools/VC/Tools/MSVC"
)

VS_LINKER_DIR=""

for base_path in "${VS_LINKER_PATHS[@]}"; do
    if [ -d "$base_path" ]; then
        # Find the latest MSVC version
        latest_version=$(ls -1 "$base_path" 2>/dev/null | sort -V | tail -1)
        if [ -n "$latest_version" ]; then
            linker_path="$base_path/$latest_version/bin/Hostx64/x64"
            if [ -f "$linker_path/link.exe" ]; then
                VS_LINKER_DIR="$linker_path"
                break
            fi
        fi
    fi
done

if [ -z "$VS_LINKER_DIR" ]; then
    echo "❌ Visual Studio linker not found!"
    echo "Please ensure Visual Studio Build Tools is installed."
    exit 1
fi

echo "✅ Found Visual Studio linker at: $VS_LINKER_DIR"

# Remove Git's usr/bin from PATH (where link.exe conflicts)
export PATH=$(echo "$PATH" | tr ':' '\n' | grep -v "Git/usr/bin" | tr '\n' ':' | sed 's/:$//')

# Add Visual Studio linker to the beginning of PATH
export PATH="$VS_LINKER_DIR:$PATH"

echo "✅ PATH fixed! Visual Studio linker is now first in PATH."
echo ""
echo "Verify:"
which link.exe
echo ""
echo "Now you can run: cargo build"

