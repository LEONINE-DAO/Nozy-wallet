# 🦓 NozyWallet Enhancement Roadmap

##  Current Status
✅ **Completed**: Basic RPC integration, transaction building framework, testing tools
🔄 **In Progress**: Real blockchain data integration
⏳ **Pending**: Production-ready features

##  Priority 1: Core Blockchain Integration

### 1. Real Note Commitment Extraction
**Current**: Using placeholder commitment bytes
**Needed**: Extract real note commitments from orchard crate
```rust
// Instead of:
let note_commitment_bytes = [0u8; 32];

// Need:
let note_commitment = spendable_note.orchard_note.note.commitment();
let note_commitment_bytes = note_commitment.to_bytes(); // Real implementation
```

### 2. Proper Unified Address Parsing
**Current**: Using placeholder addresses
**Needed**: Parse real unified addresses from strings
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
**Current**: Using deterministic placeholder paths
**Needed**: Construct real Merkle paths from blockchain data
```rust
// Instead of:
let merkle_hashes: [MerkleHashOrchard; 32] = [MerkleHashOrchard::from_cmx(&note_cmx); 32];

// Need:
let real_auth_path = zebra_client.get_real_authentication_path(position, anchor).await?;
let merkle_hashes = convert_to_merkle_hashes(real_auth_path);
```

##  Priority 2: Transaction Processing

### 4. Bundle Authorization and Signing
**Current**: Bundle remains in InProgress state
**Needed**: Complete authorization and signing process
```rust
// Current:
let (mut bundle, metadata) = builder.build::<i64>(&mut rng)?;

// Needed:
let authorized_bundle = bundle.authorize(&spending_keys)?;
let signed_bundle = authorized_bundle.sign(&mut rng)?;
let serialized_transaction = signed_bundle.to_bytes();
```

### 5. Real Transaction Broadcasting
**Current**: Placeholder transaction data
**Needed**: Broadcast real transactions to network
```rust
// Current:
let mut serialized_transaction = Vec::new();
serialized_transaction.extend_from_slice(&amount_zatoshis.to_le_bytes());

// Needed:
let txid = zebra_client.broadcast_transaction(&serialized_transaction).await?;
```

##  Priority 3: Note Management

### 6. Real Note Scanning and Decryption
**Current**: Placeholder note scanning
**Needed**: Scan blockchain for real notes and decrypt them
```rust
// Needed:
pub async fn scan_real_notes(&self, start_height: u32, end_height: u32) -> NozyResult<Vec<SpendableNote>> {
    // Scan blocks for Orchard actions
    // Decrypt notes using viewing keys
    // Extract spendable notes
}
```

### 7. Note Storage and Persistence
**Current**: Basic note storage
**Needed**: Robust note persistence with encryption
```rust
// Needed:
pub struct SecureNoteStorage {
    encrypted_notes: HashMap<String, EncryptedNote>,
    note_index: BTreeMap<u32, Vec<String>>, // height -> note_ids
    spending_keys: EncryptedKeyStore,
}
```

##  Priority 4: Security and User Experience

### 8. Enhanced Security Features
**Current**: Basic key management
**Needed**: Production-grade security
- Password-protected wallet files
- Secure key derivation (PBKDF2/Argon2)
- Hardware wallet support
- Multi-signature support

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

##  Priority 5: Advanced Features

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

##  Priority 6: User Interfaces & Platforms

### 15. Desktop GUI Application
**Status**:  **Migration in Progress** - Switching from Electron to Tauri
**Priority**: High - Makes NozyWallet accessible to non-technical users

**Current Status:**
- ✅ Electron prototype exists: [NozyWallet-DesktopClient](https://github.com/LEONINE-DAO/NozyWallet-DesktopClient)
- 🔄 **Migrating to Tauri** for better security, performance, and Rust integration
- 📋 See [TAURI_MIGRATION_GUIDE.md](TAURI_MIGRATION_GUIDE.md) for complete migration guide

**Why Tauri?**
- ✅ **Native Rust Integration** - Direct access to NozyWallet backend (no API server needed)
- ✅ **Smaller Binaries** - ~5-10MB vs Electron's 100MB+ (10-20x smaller!)
- ✅ **Better Security** - Isolated processes, smaller attack surface, no Node.js
- ✅ **Better Performance** - Native Rust code, lower memory footprint, faster startup
- ✅ **Zcash Ecosystem Alignment** - Rust is the language of Zcash (Zebra, zcashd, etc.)
- ✅ **Cross-Platform** - Windows, macOS, Linux support with native look and feel

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
1. ✅ Create Tauri project structure
2. 🔄 Migrate React frontend to use Tauri commands
3. ⏳ Replace HTTP API calls with direct Rust function calls
4. ⏳ Test on all platforms (Windows, macOS, Linux)
5. ⏳ Update CI/CD for Tauri builds
6. ⏳ Remove Electron dependencies

**Contributor Needs:**
- Frontend developers (React/TypeScript) - Migrate components to Tauri API
- Rust developers - Create Tauri commands and integrate NozyWallet core
- UI/UX designers - Polish interface during migration
- Testers - Test on Windows, macOS, Linux

** See [TAURI_MIGRATION_GUIDE.md](TAURI_MIGRATION_GUIDE.md) for complete step-by-step migration guide**

### 16. Mobile Applications
**Status**:  Planned
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
**Status**:  In Progress (API server exists)
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
**Status**:  Planned
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

##  Implementation Strategy

### Phase 1: Core Integration (Weeks 1-2)
1. ✅ Real note commitment extraction
2. ✅ Proper address parsing
3. ✅ Real Merkle path construction

### Phase 2: Transaction Processing (Weeks 3-4)
1. ✅ Bundle authorization and signing
2. ✅ Real transaction broadcasting
3. ✅ Transaction confirmation tracking

### Phase 3: Note Management (Weeks 5-6)
1. ✅ Real note scanning
2. ✅ Secure note storage
3. ✅ Note synchronization

### Phase 4: Security & UX (Weeks 7-8)
1. ✅ Enhanced security features
2. ✅ Improved error handling
3. ✅ Better CLI interface

### Phase 5: Advanced Features (Weeks 9-12)
1. ✅ Multi-address support
2. ✅ Transaction history
3. ✅ Network monitoring
4. ✅ Backup and recovery

##  Immediate Next Steps

1. **Start with Priority 1**: Fix the core blockchain integration issues
2. **Test with real data**: Use actual Zebra node data instead of placeholders
3. **Implement proper error handling**: Make the system robust for production use
4. **Add security features**: Protect user funds and private keys

##  Success Metrics

- ✅ All placeholder data replaced with real blockchain data
- ✅ Transactions can be built, signed, and broadcast successfully
- ✅ Notes can be scanned, decrypted, and spent
- ✅ Wallet can be secured with passwords and proper key management
- ✅ CLI provides excellent user experience
- ✅ System handles errors gracefully and provides helpful feedback

---

**Current Focus**: Replace all placeholder implementations with real blockchain data integration to make NozyWallet fully functional with actual Zcash transactions.
