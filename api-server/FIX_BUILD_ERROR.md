# Fixing Build Error: Missing gcc.exe

## Problem

When building the API server, you may encounter:
```
error occurred in cc-rs: failed to find tool "gcc.exe": program not found
```

This happens because the `ring` crate requires a C compiler, and the GNU toolchain requires MinGW-w64.

## Solution: Use MSVC Toolchain (Recommended for Windows)

The easiest fix is to use the MSVC toolchain instead of GNU:

```powershell
# Switch to MSVC toolchain
rustup default stable-x86_64-pc-windows-msvc

# Add MSVC target (if not already installed)
rustup target add x86_64-pc-windows-msvc

# Now build should work
cd api-server
cargo build
```

## Alternative: Install MinGW-w64

If you prefer to use the GNU toolchain:

1. **Install MinGW-w64:**
   - Download from: https://www.mingw-w64.org/downloads/
   - Or use MSYS2: https://www.msys2.org/
   - Or use Chocolatey: `choco install mingw`

2. **Add to PATH:**
   - Add MinGW `bin` directory to your PATH
   - Restart terminal

3. **Verify:**
   ```powershell
   gcc --version
   ```

## Verify Your Setup

```powershell
# Check active toolchain
rustup show

# Should show: stable-x86_64-pc-windows-msvc (for MSVC)
# Or: stable-x86_64-pc-windows-gnu (for GNU with MinGW)
```

## Why MSVC is Recommended

- ✅ No additional tools needed (comes with Rust on Windows)
- ✅ Better Windows integration
- ✅ Faster builds
- ✅ Better IDE support

## After Fixing

Once the build works, you can run:
```powershell
cd api-server
cargo run
```

The server will start on `http://localhost:3000`



