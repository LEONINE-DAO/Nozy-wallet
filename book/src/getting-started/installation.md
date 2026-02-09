# Installation

NozyWallet can be installed in several ways depending on your needs and technical level.

## Desktop Application (Recommended for Most Users)

The easiest way to use NozyWallet is through our desktop application, which provides a beautiful graphical interface.

### Windows

1. Download the latest installer from [GitHub Releases](https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest)
2. Run the `.exe` installer
3. Follow the installation wizard
4. Launch NozyWallet from your Start menu

### macOS

1. Download the latest `.dmg` file from [GitHub Releases](https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest)
2. Open the `.dmg` file
3. Drag NozyWallet to your Applications folder
4. Launch NozyWallet from Applications

### Linux

1. Download the latest `.AppImage` or `.deb` package from [GitHub Releases](https://github.com/LEONINE-DAO/Nozy-wallet/releases/latest)
2. For `.deb` packages: `sudo dpkg -i nozywallet_*.deb`
3. For `.AppImage`: Make it executable and run:
   ```bash
   chmod +x NozyWallet-*.AppImage
   ./NozyWallet-*.AppImage
   ```

## Command-Line Interface (CLI)

For advanced users, developers, or automated systems, you can use the CLI version.

### Prerequisites

- **Rust 1.70+** - Install from [rustup.rs](https://rustup.rs/)
- **Zebra RPC node** - Running on `http://127.0.0.1:8232` (or configure custom URL)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/LEONINE-DAO/Nozy-wallet.git
cd Nozy-wallet

# Build the release version
cargo build --release

# The binary will be in target/release/nozy (or nozy.exe on Windows)
```

### Install with Cargo

```bash
# Install directly from GitHub
cargo install --git https://github.com/LEONINE-DAO/Nozy-wallet.git

# Or install from a local clone
cd Nozy-wallet
cargo install --path .
```

## Setting Up Zebra Node

NozyWallet requires a connection to a Zebra RPC node to interact with the Zcash blockchain.

### Option 1: Use Public Node (Easiest)

NozyWallet can connect to a public Zebra node. Configure the node URL in your wallet settings:
- Default: `http://127.0.0.1:8232`
- Public node: `https://zec.leoninedao.org:443` (if available)

### Option 2: Run Your Own Node (Most Private)

For maximum privacy, run your own Zebra node:

```bash
# Install Zebra (Zcash Foundation's node implementation)
# See: https://zebra.zfnd.org/user/install.html

# Run Zebra with RPC enabled
zebrad start --rpc-bind-addr 127.0.0.1:8232
```

## Verifying Installation

After installation, verify everything works:

### Desktop App

1. Launch NozyWallet
2. The setup wizard should appear
3. You're ready to create your first wallet!

### CLI

```bash
# Check version
nozy --version

# Check if wallet exists
nozy status

# If no wallet, create one
nozy new
```

## Next Steps

Once installed, proceed to [Creating Your First Wallet](creating-wallet.md) to get started!
