# NozyWallet Desktop Client

Tauri-based desktop application for NozyWallet.

##  Quick Start

### Prerequisites

- **Rust 1.70+** - Install from [rustup.rs](https://rustup.rs/)
- **Node.js 18+** - Install from [nodejs.org](https://nodejs.org/)
- **Tauri CLI** - Will be installed automatically or run: `cargo install tauri-cli`

### Installation

1. **Install frontend dependencies:**
   ```bash
   cd desktop-client
   npm install
   ```

2. **Run in development mode:**
   ```bash
   # From desktop-client directory
   cargo tauri dev
   ```

   This will:
   - Build the Rust backend
   - Start the React dev server on port 5173
   - Launch the desktop app window

### Production Build

```bash
# Regenerate app icons from landing/src/assets/logo.png (Windows)
npm run icons

cargo tauri build
```

Outputs will be in `src-tauri/target/release/`:
- **Windows**: `.exe` installer
- **macOS**: `.dmg` file
- **Linux**: `.AppImage` or `.deb`

##  Project Structure

```
desktop-client/
├── src-tauri/          # Tauri backend (Rust)
│   ├── src/
│   │   ├── main.rs     # Tauri app entry point
│   │   ├── error.rs    # Error handling
│   │   └── commands/   # Tauri commands (API)
│   │       ├── mod.rs
│   │       ├── wallet.rs
│   │       ├── address.rs
│   │       ├── sync.rs
│   │       ├── transaction.rs
│   │       ├── config.rs
│   │       ├── proving.rs
│   │       ├── notes.rs
│   │       └── lwd.rs        # lightwalletd / zeaking::lwd compact sync
│   ├── Cargo.toml      # Rust dependencies
│   ├── build.rs        # Build script
│   └── tauri.conf.json # Tauri configuration
├── src/                # React frontend (to be created)
└── package.json        # Node dependencies (to be created)
```

## lightwalletd + Zeaking (Chrome / Edge companion)

When **Nozy desktop** or **`nozywallet-api`** is running, extensions can sync compact blocks without raw gRPC in the browser.

**Tauri commands** (invoke from the webview frontend):

| Command | Purpose |
|---------|---------|
| `lwd_get_info` | `{ lightwalletdUrl?: string }` → chain name, tip height |
| `lwd_chain_tip` | optional URL → tip height |
| `lwd_sync_compact` | `{ start, end?, lightwalletdUrl?, dbPath?, resume? }` → range + blocks written |
| `lwd_sync_compact_to_tip` | `{ lightwalletdUrl?, dbPath?, startFloor?, persistProgressEvery? }` → tip + `alreadyAtTip` + range stats |

Default lightwalletd URL: env `LIGHTWALLETD_GRPC` or `http://127.0.0.1:9067`.

**HTTP API** (if you run `nozywallet-api`): see [`api-server`](../api-server) routes `/api/lwd/*`.

**Browser extension (Chrome / Edge):** [`browser-extension/COMPANION.md`](../browser-extension/COMPANION.md) — `host_permissions` for `http://127.0.0.1:3000/*` and service-worker methods `companion_status`, `companion_lwd_*` (including `companion_lwd_sync_compact_to_tip`).

**Mobile:** [`zeaking-ffi`](../zeaking-ffi) — UniFFI bindings for the same `zeaking::lwd` calls.

## 🔧 Configuration

The Tauri backend uses the NozyWallet library from the parent directory (`../../`).

To use a different location or Git repository, edit `src-tauri/Cargo.toml`:

```toml
# Local path (current setup)
nozy = { path = "../../" }

# Or from Git
# nozy = { git = "https://github.com/LEONINE-DAO/Nozy-wallet.git", branch = "master" }
```

## Shielded send (desktop UI)

The Send screen uses the same path as the CLI: **scan spendable Orchard notes**, **build a ZIP-225 v5 transaction** (Halo2 proof + signatures), then **`sendrawtransaction`** on your **Zebra** JSON-RPC URL from config.

**Before you send**

1. **Zebra** reachable at the URL in Settings (default `http://127.0.0.1:8232`).
2. **Sync** the wallet so balances and notes (including Orchard incremental witness data used at spend time) are up to date.
3. Recipient must be a **unified address with an Orchard receiver** (`u1…`).

**What to expect**

- The first shielded send in a session can take **several minutes** while the Orchard proving key is built and the proof is generated; the UI shows a loading toast explaining this.
- `walletApi.sendTransaction` throws if the backend returns `success: false` (e.g. insufficient funds, invalid address, or RPC error).

Run the app: from `desktop-client`, `npm install` then `cargo tauri dev`.

## Ironwood migration (desktop UI)

The **Ironwood** nav tab wraps the same core path as CLI `nozy ironwood plan|migrate|broadcast`:

1. **Plan migration** — save ZIP 318 schedule  
2. **Start migration** — prebuild next turnstile (when readiness is `ready-to-prebuild`)  
3. **Broadcast** — submit in-window (needs local Zebrad / Tor / Advanced attestation)

**Testnet check:** Ironwood testnet wallet profile + WSL Zebrad with Ironwood RPC; sync to tip; open Ironwood tab. If status says note split required, run `nozy ironwood split` in the CLI first (split UI is not in this MVP).

## Keystone hardware wallet (mainnet)

Air-gapped Orchard signing via **PCZT** + QR (`zcash-pczt` UR frames).

**In the app:** **Settings → Keystone** (pair UFVK, enable) → **Send** (prepare → sign on device → broadcast).

**Docs:** [Keystone Hardware Wallet](../book/src/security/keystone-hardware-wallet.md) in the Nozy book (also linked from FAQ).

Requirements: Zcash **mainnet** wallet config, synced node, Keystone on mainnet with matching keys. Recipients must be `u1…` Orchard unified addresses.

## 📝 Available Tauri Commands

All commands are exposed to the frontend via Tauri's invoke system.

### Wallet Operations
- `wallet_exists()` - Check if wallet exists
- `create_wallet(password?)` - Create new wallet
- `restore_wallet(mnemonic, password)` - Restore wallet
- `unlock_wallet(password)` - Unlock wallet
- `get_wallet_status()` - Get wallet status

### Address Operations
- `generate_address(password?, account?, index?)` - Generate Orchard address

### Balance & Sync
- `get_balance()` - Get current balance
- `sync_wallet(start_height?, end_height?, zebra_url?, password?)` - Sync wallet

### Transactions
- `send_transaction(recipient, amount, memo?, zebra_url?, password?)` - Send ZEC
- `estimate_fee(zebra_url?)` - Estimate transaction fee
- `get_transaction_history()` - Local sent txs (`status`, `confirmations`, `block_height`, etc.)
- `get_transaction(txid)` - Same fields for one tx

### Keystone (mainnet PCZT)
- `get_keystone_status()` - Enabled, UFVK paired, network, pending send
- `set_keystone_enabled(enabled, device_label?)` - Toggle Keystone signing
- `export_keystone_ufvk(password?)` - Export Orchard UFVK for device pairing
- `keystone_prepare_send(recipient, amount, ...)` - Build proved PCZT + UR frames
- `keystone_complete_send(pczt_hex | ur_frames, broadcast?)` - Broadcast signed tx

### Configuration
- `get_config()` - Get configuration
- `set_zebra_url(url)` - Set Zebra URL
- `test_zebra_connection(zebra_url?)` - Test Zebra connection

### Proving Parameters
- `check_proving_status()` - Check proving parameters status
- `download_proving_parameters()` - Download proving parameters

### Notes
- `get_notes()` - Get wallet notes

##  Frontend Integration

Example TypeScript/React usage:

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// Check if wallet exists
const walletInfo = await invoke<{exists: boolean, has_password: boolean}>('wallet_exists');

// Create wallet
const mnemonic = await invoke<string>('create_wallet', {
  request: { password: 'my-password' }
});

// Get balance
const balance = await invoke<{success: boolean, balance_zec: number, message: string}>('get_balance');

// Send transaction
const result = await invoke<{success: boolean, txid?: string, message: string}>('send_transaction', {
  request: {
    recipient: 'u1...',
    amount: 0.1,
    memo: 'Hello',
    password: 'my-password'
  }
});
```

##  Troubleshooting

### Build Errors

**"Cannot find crate `nozy`"**
- Ensure you're in the `desktop-client` directory
- Check that `src-tauri/Cargo.toml` has the correct path to NozyWallet
- Run `cargo build` from `src-tauri/` to verify dependencies

**"Failed to start dev server"**
- Ensure Node.js 18+ is installed
- Run `npm install` in the frontend directory
- Check that port 5173 is available

### Runtime Errors

**"Wallet not found"**
- Create a wallet first using `create_wallet` or `restore_wallet`
- Check wallet file exists in the default location

**"Cannot connect to node" / NET_001**
- Use the **NozyWallet desktop window** (taskbar), not a browser tab at `http://localhost:5173`
- Ensure Zebrad is running: `Get-Process zebrad`
- Test RPC: `nozy test-zebra` from repo root (or Settings → Network → Test Connection)
- **Windows:** Zebrad reads config from `%LOCALAPPDATA%\zebrad.toml` (not Roaming). RPC must be enabled:

  ```toml
  [rpc]
  listen_addr = "127.0.0.1:18232"
  enable_cookie_auth = false
  ```

- Port **8232** may be used by Windows IP Helper on some PCs — pick another port (e.g. **18232**) and set the same URL in wallet config
- Full writeup: [`docs/issues/bugs/2026-06-desktop-pre-release-debug-session.md`](../docs/issues/bugs/2026-06-desktop-pre-release-debug-session.md)

**Send stuck on "Checking sync status…"**
- Often **lightwalletd not running** on `127.0.0.1:9067`; status probe can block until timeout
- Ensure Zebra tip is **not behind** wallet `last_scan_height` (node still syncing after restart)
- See session doc Issue 2 and Issue 3 in the link above

**"Failed to connect to Zebra"** (legacy message)
- Same as NET_001 — start `zebrad start` with Local config, verify with `nozy test-zebra`

##  Next Steps

1. **Create Frontend**: Set up React/TypeScript frontend in `src/` directory
2. **Add UI Components**: Build wallet UI using the Tauri commands
3. **Test**: Test all wallet operations
4. **Build**: Create production builds for all platforms

##  Related Documentation

- [Tauri Documentation](https://tauri.app/)
- [NozyWallet Main Repository](../README.md)
- [Desktop App Setup Guide](../DESKTOP_APP_SETUP.md)

