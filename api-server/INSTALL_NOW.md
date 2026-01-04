# 🚀 Install Build Tools Now

## Quick Install (Using winget - Built into Windows)

You have `winget` available! Here's the fastest way:

### Install Visual Studio Build Tools

**Run this command in PowerShell (as Administrator):**

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools --override "--quiet --add Microsoft.VisualStudio.Workload.VCTools"
```

**Or if that doesn't work, try:**

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools
```

Then manually select "Desktop development with C++" when the installer opens.

### After Installation

1. **Close and restart your terminal/PowerShell** (IMPORTANT!)
2. **Verify installation:**
   ```powershell
   where link.exe
   # Should show: C:\Program Files\Microsoft Visual Studio\...
   ```
3. **Build the API server:**
   ```powershell
   cd api-server
   cargo build
   ```

## Alternative: Manual Download

If winget doesn't work:

1. **Download:** https://visualstudio.microsoft.com/downloads/
2. **Scroll to:** "Tools for Visual Studio"
3. **Download:** "Build Tools for Visual Studio 2022"
4. **Install:** Select "Desktop development with C++"
5. **Restart terminal**
6. **Build:** `cargo build`

## ⚠️ Important Notes

- Installation takes 10-20 minutes
- Requires ~2-3 GB disk space
- **Must restart terminal after installation**
- VS Code alone is NOT sufficient (need Build Tools)

## Verify It Worked

After restarting terminal:
```powershell
rustup show
# Should show: stable-x86_64-pc-windows-msvc

cd api-server
cargo build
# Should compile successfully!
```

