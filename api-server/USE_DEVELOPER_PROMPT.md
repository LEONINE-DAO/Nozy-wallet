# ✅ Quick Fix: Use Visual Studio Developer Command Prompt

## The Problem

Git Bash is finding Git's `link.exe` (Unix utility) instead of Visual Studio's linker. This causes build errors.

## Solution: Use Developer Command Prompt

The easiest fix is to use Visual Studio's Developer Command Prompt which has the correct PATH setup.

### Steps:

1. **Open Developer Command Prompt:**
   - Press `Win + S` (Windows Search)
   - Type: "Developer Command Prompt for VS 2022"
   - Click on it

2. **Navigate to your project:**
   ```cmd
   cd C:\Users\user\Nozy-wallet\api-server
   ```

3. **Build:**
   ```cmd
   cargo build
   ```

4. **Run:**
   ```cmd
   cargo run
   ```

## Alternative: Fix PATH in Current Terminal

If you want to keep using your current terminal:

1. **Find Visual Studio linker:**
   - Usually at: `C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\<version>\bin\Hostx64\x64\link.exe`
   - Or search for "link.exe" in `C:\Program Files\Microsoft Visual Studio`

2. **Add to PATH (temporarily):**
   ```bash
   # Get the bin directory (replace with actual path)
   export PATH="/c/Program Files/Microsoft Visual Studio/2022/BuildTools/VC/Tools/MSVC/14.xx.xxxxx/bin/Hostx64/x64:$PATH"
   
   # Then build
   cargo build
   ```

## Recommended: Always Use Developer Command Prompt

For Rust development on Windows with MSVC, it's best to:
- Use "Developer Command Prompt for VS 2022" 
- Or use PowerShell with VS environment loaded
- This ensures correct PATH and environment variables

## Verify It Works

After using Developer Command Prompt:
```cmd
where link.exe
# Should show: C:\Program Files\Microsoft Visual Studio\...\link.exe

cargo build
# Should compile successfully!
```

