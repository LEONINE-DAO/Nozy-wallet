# NozyWallet Enhancement Roadmap

## Current Status
**Completed**: Real blockchain RPC integration, transaction building framework, note decryption, testing tools
**Completed**: Real blockchain data integration with Zebra RPC
**In Progress**: Production-ready features, desktop app (Tauri migration)
**Pending**: Mobile apps, hardware wallet support

## Priority 1: Core Blockchain Integration - COMPLETED

### 1. Real Note Commitment Extraction
**Status**: **IMPLEMENTED** - Real note commitments extracted from orchard crate
```rust
// Instead of:
let note_commitment_bytes = [0u8; 32];

// Need:
let note_commitment = spendable_note.orchard_note.note.commitment();
let note_commitment_bytes = note_commitment.to_bytes(); // Real implementation
```

### 2. Proper Unified Address Parsing
**Status**: **IMPLEMENTED** - Real unified address parsing from strings
```rust
// Instead of:
let recipient_orchard_address = OrchardAddress::from_raw_address_bytes(&[0u8; 43]).unwrap();

// Need:
let (_, recipient) = zcash_address::unified::Address::decode(recipient_address)?;
let orchard_receiver = recipient.receivers().iter()
    .find(|r| matches!(r, zcash_address::unified::Receiver::Orchard(_)));
// Extract real Orchard address from unified address
```

### 3. Real Merkle Path Construction
**Status**: **IMPLEMENTED** - Real Merkle paths constructed from blockchain data via Zebra RPC
```rust
// Instead of:
let merkle_hashes: [MerkleHashOrchard; 32] = [MerkleHashOrchard::from_cmx(&note_cmx); 32];

// Need:
let real_auth_path = zebra_client.get_real_authentication_path(position, anchor).await?;
let merkle_hashes = convert_to_merkle_hashes(real_auth_path);
```

## Priority 2: Transaction Processing

### 4. Bundle Authorization and Signing
**Status**: **IN PROGRESS** - Bundle authorization framework implemented, signing integration in progress
```rust
// Current:
let (mut bundle, metadata) = builder.build::<i64>(&mut rng)?;

// Needed:
let authorized_bundle = bundle.authorize(&spending_keys)?;
let signed_bundle = authorized_bundle.sign(&mut rng)?;
let serialized_transaction = signed_bundle.to_bytes();
```

### 5. Real Transaction Broadcasting
**Status**: **IMPLEMENTED** - Real transaction broadcasting via Zebra RPC (`broadcast_transaction`, `send_raw_transaction`)
```rust
// Current:
let mut serialized_transaction = Vec::new();
serialized_transaction.extend_from_slice(&amount_zatoshis.to_le_bytes());

// Needed:
let txid = zebra_client.broadcast_transaction(&serialized_transaction).await?;
```

## Priority 3: Note Management

### 6. Real Note Scanning and Decryption
**Status**: **IMPLEMENTED** - Real note scanning and decryption with full Orchard action parsing and zcash_note_encryption integration
```rust
// Needed:
pub async fn scan_real_notes(&self, start_height: u32, end_height: u32) -> NozyResult<Vec<SpendableNote>> {
    // Scan blocks for Orchard actions
    // Decrypt notes using viewing keys
    // Extract spendable notes
}
```

### 7. Note Storage and Persistence
**Status**: **IMPLEMENTED** - Note storage implemented with `NoteStorage`, `StoredNote`, and transaction tracking
```rust
// Needed:
pub struct SecureNoteStorage {
    encrypted_notes: HashMap<String, EncryptedNote>,
    note_index: BTreeMap<u32, Vec<String>>, // height -> note_ids
    spending_keys: EncryptedKeyStore,
}
```

## Priority 4: Security and User Experience

### 8. Enhanced Security Features
**Status**: 
- **COMPLETED**: Password-protected wallet files
- **COMPLETED**: Argon2 key derivation
- **PENDING**: Hardware wallet support
- **PENDING**: Multi-signature support

### 9. Improved Error Handling
**Current**: Basic error messages
**Needed**: Comprehensive error handling
```rust
// Needed:
pub enum NozyError {
    Network(NetworkError),
    Cryptographic(CryptoError),
    Blockchain(BlockchainError),
    User(UserError),
    // ... with detailed error context
}
```

### 10. Enhanced CLI Interface
**Current**: Basic CLI commands
**Needed**: Rich user experience
- Interactive transaction building
- Real-time balance updates
- Transaction history
- Address book management
- Configuration management

## Priority 5: Advanced Features

### 11. Multi-Address Support
- Generate multiple addresses
- Address labeling and management
- Balance aggregation across addresses

### 12. Transaction History
- Complete transaction history
- Transaction details and status
- Export capabilities

### 13. Network Monitoring
- Real-time network status
- Fee estimation
- Mempool monitoring

### 14. Backup and Recovery
- Wallet backup/restore
- Seed phrase recovery
- Multi-device sync

## Priority 6: User Interfaces & Platforms

### 15. Desktop GUI Application
**Status**: **Migration in Progress** - Switching from Electron to Tauri
**Priority**: High - Makes NozyWallet accessible to non-technical users

**Current Status:**
- Electron prototype exists: [NozyWallet-DesktopClient](https://github.com/LEONINE-DAO/NozyWallet-DesktopClient)
- **Migrating to Tauri** for better security, performance, and Rust integration
- See [TAURI_MIGRATION_GUIDE.md](TAURI_MIGRATION_GUIDE.md) for complete migration guide

**Why Tauri?**
- **Native Rust Integration** - Direct access to NozyWallet backend (no API server needed)
- **Smaller Binaries** - ~5-10MB vs Electron's 100MB+ (10-20x smaller!)
- **Better Security** - Isolated processes, smaller attack surface, no Node.js
- **Better Performance** - Native Rust code, lower memory footprint, faster startup
- **Zcash Ecosystem Alignment** - Rust is the language of Zcash (Zebra, zcashd, etc.)
- **Cross-Platform** - Windows, macOS, Linux support with native look and feel

**Goals:**
- Cross-platform desktop application (Windows, macOS, Linux)
- Beautiful, privacy-focused user interface
- Real-time balance and transaction updates
- Visual transaction builder
- Address book with labels
- Transaction history viewer
- Settings and configuration UI
- **Direct Rust integration** - No API server overhead

**Technology Stack:**
- **Backend**: Tauri (Rust) - Direct integration with NozyWallet core
- **Frontend**: React 18 + Vite 5 + Tailwind CSS 4 (existing from Electron version)
- **Icons**: Solar Icons (existing)
- **State**: Zustand (existing)
- **Data Fetching**: TanStack Query v5 (existing)

**Migration Plan:**
1. Create Tauri project structure
2. Migrate React frontend to use Tauri commands
3. Replace HTTP API calls with direct Rust function calls
4. Test on all platforms (Windows, macOS, Linux)
5. Update CI/CD for Tauri builds
6. Remove Electron dependencies

**Contributor Needs:**
- Frontend developers (React/TypeScript) - Migrate components to Tauri API
- Rust developers - Create Tauri commands and integrate NozyWallet core
- UI/UX designers - Polish interface during migration
- Testers - Test on Windows, macOS, Linux

**See [TAURI_MIGRATION_GUIDE.md](TAURI_MIGRATION_GUIDE.md) for complete step-by-step migration guide**

### 16. Mobile Applications
**Status**: Planned
**Priority**: High - Essential for on-the-go privacy

**Goals:**
- Native iOS application
- Native Android application
- QR code scanning for addresses
- Biometric authentication
- Push notifications for transactions
- Mobile-optimized UI/UX

**Technology Options:**
- React Native (shared codebase)
- Flutter (Dart)
- Native Swift/Kotlin with Rust core

**Contributor Needs:**
- Mobile developers (iOS/Android)
- React Native or Flutter developers
- Mobile UI/UX designers

### 17. Web Interface
**Status**: In Progress (API server exists)
**Priority**: Medium - Leverages existing API server

**Goals:**
- Browser-based wallet interface
- Connect to existing API server
- Full wallet functionality via web
- Responsive design for mobile browsers

**Contributor Needs:**
- Frontend developers
- Web UI/UX designers
- API integration developers

### 18. Multi-Account Management
**Status**: Planned
**Priority**: Medium - Useful for power users

**Goals:**
- Manage multiple wallets from one interface
- Switch between accounts easily
- Aggregate balances across accounts
- Account labeling and organization
- Separate transaction histories per account

**Contributor Needs:**
- Backend developers for account management
- UI developers for account switching interface

## Implementation Strategy

### Phase 1: Core Integration - COMPLETED
1. Real note commitment extraction - **DONE**
2. Proper address parsing - **DONE**
3. Real Merkle path construction - **DONE**
4. Real Zebra RPC integration - **DONE**

### Phase 2: Transaction Processing - IN PROGRESS
1. Bundle authorization and signing - **Framework ready, integration in progress**
2. Real transaction broadcasting - **DONE** (via Zebra RPC)
3. Transaction confirmation tracking - **DONE**

### Phase 3: Note Management - COMPLETED
1. Real note scanning - **DONE** (NoteScanner with real blockchain data)
2. Secure note storage - **DONE** (NoteStorage implementation)
3. Note synchronization - **DONE** (sync commands and workflow)
4. Real note decryption - **DONE** (full zcash_note_encryption integration)

### Phase 4: Security & UX - MOSTLY COMPLETED
1. Enhanced security features - **DONE** (Argon2, password protection, encrypted storage)
2. Improved error handling - **In progress**
3. Better CLI interface - **DONE** (comprehensive CLI commands)

### Phase 5: Advanced Features - IN PROGRESS
1. Multi-address support - **DONE** (address generation)
2. Transaction history - **In progress**
3. Network monitoring - **DONE** (Zebra connection testing)
4. Backup and recovery - **DONE** (mnemonic restore)

## Immediate Next Steps

1. **COMPLETED**: Core blockchain integration with real Zebra RPC
2. **COMPLETED**: Real blockchain data integration - all placeholder data replaced
3. **IN PROGRESS**: Complete transaction signing and broadcasting workflow
4. **IN PROGRESS**: Desktop app (Tauri migration)
5. **PENDING**: Mobile applications
6. **PENDING**: Hardware wallet support

## Success Metrics

- **ACHIEVED**: All placeholder data replaced with real blockchain data
- **ACHIEVED**: Real Zebra RPC integration fully functional
- **ACHIEVED**: Real note scanning, decryption, and parsing
- **ACHIEVED**: Real transaction building framework
- **ACHIEVED**: Wallet secured with passwords and Argon2 key derivation
- **ACHIEVED**: Real Merkle path construction from blockchain
- **ACHIEVED**: Real unified address parsing
- **IN PROGRESS**: Complete transaction authorization and signing
- **PENDING**: Production-ready error handling improvements
- **PENDING**: Hardware wallet integration

---

**Current Focus**: Complete Tauri desktop app migration and finalize transaction signing workflow to make NozyWallet fully production-ready for end users.
