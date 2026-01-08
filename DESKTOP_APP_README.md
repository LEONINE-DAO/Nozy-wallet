# 🖥️ NozyWallet Desktop App

A beautiful desktop application for NozyWallet built with Tauri (Rust + React).

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Node.js 18+ (install from [nodejs.org](https://nodejs.org/))
- Zebra RPC node running on `http://127.0.0.1:8232`

### Installation

1. **Install Tauri CLI** (if not already installed):
   ```bash
   cargo install tauri-cli
   ```

2. **Install frontend dependencies**:
   ```bash
   cd frontend
   npm install
   ```

3. **Build and run**:
   ```bash
   # From the project root
   cargo tauri dev
   ```

   This will:
   - Build the Rust backend
   - Start the React development server
   - Launch the desktop app

### Production Build

```bash
cargo tauri build
```

This creates installers in `src-tauri/target/release/`:
- Windows: `.exe` installer
- macOS: `.dmg` file
- Linux: `.AppImage` or `.deb`

## 📁 Project Structure

```
NozyWallet/
├── src-tauri/          # Tauri backend (Rust)
│   ├── src/
│   │   ├── main.rs     # Tauri app entry point
│   │   └── commands.rs # Tauri commands (API)
│   ├── Cargo.toml      # Rust dependencies
│   └── tauri.conf.json # Tauri configuration
├── frontend/            # React frontend
│   ├── src/
│   │   ├── App.tsx     # Main app component
│   │   └── pages/      # Page components
│   ├── package.json    # Node dependencies
│   └── vite.config.ts  # Vite configuration
└── src/                # Existing NozyWallet library
```

## 🎨 Features

### ✅ Implemented

- **Interactive Setup Wizard**: Guided first-time setup
- **Wallet Creation/Restore**: Create new wallet or restore from seed phrase
- **Dashboard**: View balance and wallet status
- **Send ZEC**: Send transactions with memo support
- **Receive ZEC**: Generate and display receiving addresses
- **Settings**: Configure Zebra node and check proving parameters
- **Auto-routing**: Automatically redirects to setup if no wallet exists

### 🚧 Coming Soon

- QR code generation for addresses
- Transaction history
- Address book
- Dark theme
- Real-time balance updates

## 🔧 Development

### Adding New Tauri Commands

1. Add the command function in `src-tauri/src/commands.rs`:
   ```rust
   #[tauri::command]
   pub async fn my_command(param: String) -> Result<String, String> {
       // Implementation
       Ok("result".to_string())
   }
   ```

2. Register it in `src-tauri/src/main.rs`:
   ```rust
   .invoke_handler(tauri::generate_handler![
       // ... existing commands
       my_command,
   ])
   ```

3. Call it from the frontend:
   ```typescript
   import { invoke } from "@tauri-apps/api/tauri";
   const result = await invoke("my_command", { param: "value" });
   ```

### Frontend Development

The frontend uses:
- **React 18** with TypeScript
- **React Router** for navigation
- **Vite** for fast development
- **CSS** for styling (no framework, keeping it simple)

## 🐛 Troubleshooting

### "Failed to connect to Zebra node"

1. Make sure Zebra is running:
   ```bash
   zebrad start --rpc.bind-addr 127.0.0.1:8232
   ```

2. Test the connection in Settings
3. Update the Zebra URL if needed

### "Proving parameters not found"

1. Go to Settings
2. Click "Download Proving Parameters"
3. Wait for download to complete (~500MB)

### Build Errors

- Make sure all dependencies are installed
- Try `cargo clean` and rebuild
- Check Rust and Node.js versions

## 📝 Notes

- The desktop app uses the same wallet data as the CLI (`wallet_data/` directory)
- You can use both CLI and desktop app with the same wallet
- All wallet operations are handled by the existing NozyWallet Rust library

## 🎯 Next Steps

1. Add QR code generation for addresses
2. Implement transaction history
3. Add address book/contacts
4. Improve error handling and user feedback
5. Add dark theme support
6. Implement real-time sync notifications

