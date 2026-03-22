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
| `lwd_sync_compact` | `{ start, end?, lightwalletdUrl?, dbPath? }` → blocks written |

Default lightwalletd URL: env `LIGHTWALLETD_GRPC` or `http://127.0.0.1:9067`.

**HTTP API** (if you run `nozywallet-api`): see [`api-server`](../api-server) routes `/api/lwd/*`.

**Browser extension (Chrome / Edge):** [`browser-extension/COMPANION.md`](../browser-extension/COMPANION.md) — `host_permissions` for `http://127.0.0.1:3000/*` and service-worker methods `companion_status`, `companion_lwd_*`.

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
- `get_transaction_history()` - Get transaction history
- `get_transaction(txid)` - Get specific transaction

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

**"Failed to connect to Zebra"**
- Ensure Zebra node is running: `zebrad start --rpc.bind-addr 127.0.0.1:8232`
- Check Zebra URL in configuration
- Test connection using `test_zebra_connection`

##  Next Steps

1. **Create Frontend**: Set up React/TypeScript frontend in `src/` directory
2. **Add UI Components**: Build wallet UI using the Tauri commands
3. **Test**: Test all wallet operations
4. **Build**: Create production builds for all platforms

##  Related Documentation

- [Tauri Documentation](https://tauri.app/)
- [NozyWallet Main Repository](../README.md)
- [Desktop App Setup Guide](../DESKTOP_APP_SETUP.md)

