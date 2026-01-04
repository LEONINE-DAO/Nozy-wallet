# Windows Build Setup Guide

## Problem

You're getting one of these errors:
- `error: linker 'link.exe' not found` (MSVC toolchain)
- `error: failed to find tool "gcc.exe"` (GNU toolchain)

Both toolchains require additional tools on Windows.

## Solution Options

### Option 1: Install Visual Studio Build Tools (For MSVC) ⭐ Recommended

**Why:** Better Windows integration, faster builds, no PATH issues

**Steps:**

1. **Download Visual Studio Build Tools:**
   - Go to: https://visualstudio.microsoft.com/downloads/
   - Scroll down to "Tools for Visual Studio"
   - Download "Build Tools for Visual Studio 2022"

2. **Install with C++ workload:**
   - Run the installer
   - Select "Desktop development with C++" workload
   - Click "Install"
   - Wait for installation (may take 10-20 minutes)

3. **Restart your terminal** (important!)

4. **Verify:**
   ```powershell
   rustup show
   # Should show: stable-x86_64-pc-windows-msvc
   
   cd api-server
   cargo build
   ```

**Alternative: Install via Chocolatey (faster):**
```powershell
choco install visualstudio2022buildtools --params "--add Microsoft.VisualStudio.Workload.VCTools"
```

### Option 2: Install MinGW-w64 (For GNU Toolchain)

**Why:** Smaller download, lighter weight

**Steps:**

1. **Install MinGW-w64:**
   
   **Option A: Using MSYS2 (Recommended)**
   ```powershell
   # Download from: https://www.msys2.org/
   # After installing MSYS2, run in MSYS2 terminal:
   pacman -S mingw-w64-x86_64-gcc
   ```
   
   **Option B: Using Chocolatey**
   ```powershell
   choco install mingw
   ```
   
   **Option C: Direct Download**
   - Download from: https://www.mingw-w64.org/downloads/
   - Extract to `C:\mingw-w64`
   - Add `C:\mingw-w64\bin` to PATH

2. **Add to PATH:**
   ```powershell
   # Add to system PATH (replace with your actual path):
   # C:\msys64\mingw64\bin  (if using MSYS2)
   # Or C:\mingw-w64\bin    (if direct install)
   ```

3. **Switch to GNU toolchain:**
   ```powershell
   rustup default stable-x86_64-pc-windows-gnu
   ```

4. **Verify:**
   ```powershell
   gcc --version
   # Should show gcc version
   
   cd api-server
   cargo build
   ```

### Option 3: Use Pre-built Binary (Quickest)

If you just want to run the API server without building:

1. **Download from releases:**
   - Check GitHub releases for pre-built Windows binaries
   - Or ask someone to build it for you

2. **Or use Docker:**
   ```powershell
   docker build -t nozy-api-server ./api-server
   docker run -p 3000:3000 nozy-api-server
   ```

## Quick Decision Guide

**Choose MSVC if:**
- ✅ You want the best Windows experience
- ✅ You don't mind a larger download (2-3 GB)
- ✅ You plan to do more Rust development

**Choose MinGW if:**
- ✅ You want a smaller download
- ✅ You're comfortable with PATH configuration
- ✅ You prefer open-source tools

## After Installation

Once you have the tools installed:

```powershell
# Verify toolchain
rustup show

# Build API server
cd api-server
cargo build --release

# Run API server
cargo run
```

## Troubleshooting

### "link.exe not found" after installing Build Tools
- **Restart your terminal** (required!)
- Close and reopen VS Code/PowerShell
- Verify: `where link.exe` should show path

### "gcc.exe not found" after installing MinGW
- Check PATH: `$env:PATH -split ';' | Select-String mingw`
- Restart terminal
- Verify: `where gcc` should show path

### Still having issues?
- Check Rust installation: `rustup show`
- Try clean build: `cargo clean && cargo build`
- Check for conflicting toolchains

## Recommended Setup

For most users, I recommend:
1. **Install Visual Studio Build Tools** (Option 1)
2. **Use MSVC toolchain** (already set)
3. **Restart terminal**
4. **Build should work**

This gives you the best experience on Windows.

