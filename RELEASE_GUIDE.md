# NozyWallet Release Guide

This guide explains how to create and publish releases with pre-built binaries and installers.

## Prerequisites

- Rust toolchain installed
- Node.js 18+ installed (for desktop app)
- GitHub repository access
- (Optional) Code signing certificates for Windows/macOS

## Release Process

### 1. Update Version

Update the version in:
- `Cargo.toml` (main package)
- `src-tauri/Cargo.toml` (desktop app)
- `src-tauri/tauri.conf.json` (desktop app version)

```bash
# Example: Update to v0.2.1
# Edit Cargo.toml: version = "0.2.1"
# Edit src-tauri/Cargo.toml: version = "0.2.1"
# Edit src-tauri/tauri.conf.json: "version": "0.2.1"
```

### 2. Create Release Tag

```bash
git add .
git commit -m "Release v0.2.1"
git tag -a v0.2.1 -m "Release v0.2.1"
git push origin main
git push origin v0.2.1
```

### 3. Automated Build (Recommended)

The GitHub Actions workflow will automatically build all binaries and installers when you push a tag:

```bash
git tag -a v0.2.1 -m "Release v0.2.1"
git push origin v0.2.1
```

This triggers `.github/workflows/release.yml` which:
- Builds CLI binaries for Windows, macOS (Intel + ARM), Linux
- Builds desktop installers (.exe, .dmg, .deb)
- Generates SHA256 hashes
- Creates a GitHub release with all artifacts

### 4. Manual Build (Alternative)

If you need to build locally:

#### Linux/macOS:
```bash
chmod +x scripts/build-release.sh
./scripts/build-release.sh 0.2.1 all
```

#### Windows:
```powershell
.\scripts\build-release.ps1 -Version 0.2.1 -Platform all
```

This creates `releases/v0.2.1/` with all binaries and hashes.

### 5. Verify Builds

Check that all files are present:
- CLI binaries for each platform
- Desktop installers
- Hash files (`.sha256` files)
- Combined `HASHES.txt` file

### 6. Test Installers

Before publishing, test the installers:
- **Windows**: Run the `.exe` installer
- **macOS**: Mount the `.dmg` and test installation
- **Linux**: Install the `.deb` package

### 7. Create GitHub Release

If using automated builds, the release is created automatically. Otherwise:

1. Go to GitHub → Releases → Draft a new release
2. Select the tag (e.g., `v0.2.1`)
3. Upload all files from `releases/v0.2.1/`
4. Include `HASHES.txt` in the release
5. Write release notes
6. Publish

### 8. Update Downloads Page

The downloads page (`downloads/index.html`) automatically fetches the latest release from GitHub API. No manual update needed unless you want to customize the page.

To manually update:
1. Edit `downloads/index.html`
2. Commit and push
3. GitHub Pages will deploy automatically

## Build Targets

### CLI Binaries
- `x86_64-pc-windows-msvc` → Windows 64-bit
- `x86_64-apple-darwin` → macOS Intel
- `aarch64-apple-darwin` → macOS Apple Silicon
- `x86_64-unknown-linux-gnu` → Linux 64-bit

### Desktop Installers
- Windows: `.exe` installer (NSIS)
- macOS: `.dmg` disk image
- Linux: `.deb` package, `.AppImage`, `.rpm`

## Hash Verification

All releases include SHA256 hashes for verification:

### Windows:
```powershell
certutil -hashfile nozy-windows.exe SHA256
```

### macOS/Linux:
```bash
shasum -a 256 nozy-macos
```

Compare the output with the hash in `HASHES.txt` or on the downloads page.

## Code Signing (Optional)

For production releases, consider code signing:

### Windows:
- Requires a code signing certificate
- Set `TAURI_PRIVATE_KEY` and `TAURI_KEY_PASSWORD` secrets in GitHub
- Configure in `src-tauri/tauri.conf.json`

### macOS:
- Requires Apple Developer certificate
- Set `TAURI_PRIVATE_KEY` and `TAURI_KEY_PASSWORD` secrets
- Configure in `src-tauri/tauri.conf.json`

## Troubleshooting

### Build Fails
- Check Rust version: `rustc --version` (should be 1.70+)
- Check Node.js version: `node --version` (should be 18+)
- Clean build: `cargo clean && cargo build --release`

### Installer Issues
- Ensure Tauri CLI is installed: `cargo install tauri-cli`
- Check frontend dependencies: `cd frontend && npm install`
- Verify `tauri.conf.json` configuration

### Hash Mismatch
- Re-download the file
- Check for file corruption
- Verify the hash file matches the binary

## Release Checklist

- [ ] Update version numbers
- [ ] Run tests: `cargo test`
- [ ] Build locally and test
- [ ] Create and push tag
- [ ] Wait for GitHub Actions to complete
- [ ] Verify all artifacts are present
- [ ] Test installers on each platform
- [ ] Verify hashes match
- [ ] Update release notes
- [ ] Publish release
- [ ] Verify downloads page updates

## Next Steps

After release:
1. Announce on social media
2. Update documentation
3. Monitor for issues
4. Plan next release

---

**Questions?** Open an issue on GitHub or check the documentation.
