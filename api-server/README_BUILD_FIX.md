# ⚠️ Build Error: Missing Build Tools

## Current Error
```
error: linker `link.exe` not found
```

## Quick Fix (Choose One)

### 🚀 Option 1: Install via Chocolatey (Fastest)

If you have Chocolatey:
```powershell
choco install visualstudio2022buildtools --params "--add Microsoft.VisualStudio.Workload.VCTools" -y
```

Then **restart your terminal** and run:
```powershell
cd api-server
cargo build
```

### 📥 Option 2: Manual Download

1. Go to: https://visualstudio.microsoft.com/downloads/
2. Scroll to "Tools for Visual Studio"
3. Download "Build Tools for Visual Studio 2022"
4. Install with "Desktop development with C++" workload
5. **Restart terminal**
6. Run: `cargo build`

### 🔧 Option 3: Use MinGW (Alternative)

```powershell
# Install MinGW
choco install mingw

# Switch to GNU toolchain
rustup default stable-x86_64-pc-windows-gnu

# Build
cd api-server
cargo build
```

## ⚠️ Important: Restart Terminal

After installing build tools, **you MUST restart your terminal/PowerShell** for the changes to take effect!

## Verify Installation

After restarting:
```powershell
# Check if link.exe is available
where link.exe

# Should show path like: C:\Program Files\Microsoft Visual Studio\...
```

## Still Having Issues?

1. Make sure you restarted your terminal
2. Check PATH includes Visual Studio tools
3. Try: `cargo clean && cargo build`
4. See detailed guide: `WINDOWS_BUILD_SETUP.md`


