# Tauri Integration Example Files

This directory contains example configuration files for integrating Tauri into the NozyWallet-landing repository.

## Files

- **`src-tauri/Cargo.toml.example`** - Rust backend dependencies configuration
- **`src-tauri/tauri.conf.json.example`** - Tauri application configuration
- **`package.json.updates.example`** - Package.json script updates
- **`vite.config.ts.example`** - Vite configuration for Tauri

## Usage

1. **Copy `Cargo.toml.example`** to `src-tauri/Cargo.toml` in your landing page repository
2. **Copy `tauri.conf.json.example`** to `src-tauri/tauri.conf.json`
3. **Update your `package.json`** with the scripts from `package.json.updates.example`
4. **Update your `vite.config.ts`** with settings from `vite.config.ts.example`

## Important Notes

### Cargo.toml Configuration

Choose the appropriate NozyWallet dependency option in `Cargo.toml`:

**Option 1: Relative Path** (if repos are in same workspace)
```toml
nozy = { path = "../../Nozy-wallet" }
```

**Option 2: Git Dependency** (recommended for separate repos)
```toml
nozy = { git = "https://github.com/LEONINE-DAO/Nozy-wallet.git", branch = "main" }
```

**Option 3: Published Crate** (after publishing to crates.io)
```toml
nozy = "2.1.0"
```

### Rust Backend Implementation

For the complete Rust backend implementation (main.rs, commands modules, error handling), see:
- **`TAURI_IMPLEMENTATION.md`** in the Nozy-wallet repository
- This file contains complete code for all wallet operations

### TypeScript Frontend Integration

For TypeScript API client code, see:
- **`TAURI_IMPLEMENTATION.md`** in the Nozy-wallet repository
- Sections on "Frontend Integration" and "TypeScript Types"

## Next Steps

1. Follow **`LANDING_PAGE_TAURI_INTEGRATION.md`** for complete setup guide
2. Initialize Tauri: `npx tauri init`
3. Copy these example files to appropriate locations
4. Implement Rust backend (see `TAURI_IMPLEMENTATION.md`)
5. Create TypeScript API client (see `TAURI_IMPLEMENTATION.md`)
6. Test with: `npm run tauri:dev`

## Resources

- **Setup Guide**: `LANDING_PAGE_TAURI_INTEGRATION.md`
- **Complete Implementation**: `TAURI_IMPLEMENTATION.md`
- **Migration Guide**: `TAURI_MIGRATION_GUIDE.md`
- **Tauri Docs**: https://tauri.app/

