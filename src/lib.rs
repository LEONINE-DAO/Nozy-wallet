// ============================================================
// WASM-safe modules (available on all targets)
// ============================================================
pub mod error;
pub mod traits;
pub mod groth16_prover_simple;
pub mod hd_wallet;
pub mod input_validation;
pub mod key_management;
pub mod privacy;
pub mod safe_display;
#[cfg(not(target_arch = "wasm32"))]
pub mod secret_keys;
pub mod transactions;

// ============================================================
// Native-only modules (filesystem, networking, CLI, threading)
// ============================================================
#[cfg(not(target_arch = "wasm32"))]
pub mod address_book;
#[cfg(not(target_arch = "wasm32"))]
pub mod benchmarks;
#[cfg(not(target_arch = "wasm32"))]
pub mod block_parser;
#[cfg(not(target_arch = "wasm32"))]
pub mod bridge;
#[cfg(not(target_arch = "wasm32"))]
pub mod cache;
#[cfg(not(target_arch = "wasm32"))]
pub mod cli_helpers;
#[cfg(not(target_arch = "wasm32"))]
pub mod config;
#[cfg(not(target_arch = "wasm32"))]
pub mod grpc_client;
#[cfg(not(target_arch = "wasm32"))]
pub mod local_analytics;
#[cfg(not(target_arch = "wasm32"))]
pub mod monero;
#[cfg(not(target_arch = "wasm32"))]
pub mod monero_zk_verifier;
#[cfg(not(target_arch = "wasm32"))]
pub mod note_index;
#[cfg(not(target_arch = "wasm32"))]
pub mod note_sync;
#[cfg(not(target_arch = "wasm32"))]
pub mod notes;
#[cfg(not(target_arch = "wasm32"))]
pub mod nu6_1_check;
#[cfg(not(target_arch = "wasm32"))]
pub mod orchard_tx;
#[cfg(not(target_arch = "wasm32"))]
pub mod paths;
#[cfg(not(target_arch = "wasm32"))]
pub mod privacy_network;
#[cfg(not(target_arch = "wasm32"))]
pub mod privacy_ui;
#[cfg(not(target_arch = "wasm32"))]
pub mod progress;
#[cfg(not(target_arch = "wasm32"))]
pub mod proving;
#[cfg(not(target_arch = "wasm32"))]
pub mod rpc_test;
#[cfg(feature = "secret-network")]
pub mod secret;
#[cfg(not(target_arch = "wasm32"))]
pub mod storage;
#[cfg(not(target_arch = "wasm32"))]
pub mod swap;
#[cfg(test)]
pub mod tests;
#[cfg(not(target_arch = "wasm32"))]
pub mod transaction_builder;
#[cfg(not(target_arch = "wasm32"))]
pub mod transaction_history;
#[cfg(not(target_arch = "wasm32"))]
pub mod transaction_tracker;
#[cfg(not(target_arch = "wasm32"))]
pub mod zeaking_adapter;
#[cfg(not(target_arch = "wasm32"))]
pub mod zebra_integration;

// ============================================================
// WASM-safe re-exports
// ============================================================
pub use error::{NozyError, NozyResult};
pub use hd_wallet::HDWallet;
pub use transactions::{SignedTransaction, TransactionBuilder, TransactionDetails};

// ============================================================
// Native-only re-exports
// ============================================================
#[cfg(not(target_arch = "wasm32"))]
pub use address_book::{AddressBook, AddressEntry};
#[cfg(not(target_arch = "wasm32"))]
pub use block_parser::BlockParser;
#[cfg(not(target_arch = "wasm32"))]
pub use bridge::{
    AddressTracker, ChurnManager, ChurnRecommendation, PrivacyCheckResult, PrivacyValidator,
    StoredSwap, SwapStorage,
};
#[cfg(not(target_arch = "wasm32"))]
pub use cli_helpers::estimate_transaction_fee;
#[cfg(not(target_arch = "wasm32"))]
pub use config::{load_config, save_config, update_last_scan_height, WalletConfig};
#[cfg(not(target_arch = "wasm32"))]
pub use config::{BackendKind, Protocol};
#[cfg(not(target_arch = "wasm32"))]
pub use monero::{
    MoneroRpcClient, MoneroTransactionRecord, MoneroTransactionStatus, MoneroTransactionStorage,
    MoneroWallet,
};
#[cfg(not(target_arch = "wasm32"))]
pub use monero_zk_verifier::{MoneroZkVerifier, VerificationLevel, VerificationResult};
#[cfg(not(target_arch = "wasm32"))]
pub use note_index::NoteIndex;
#[cfg(not(target_arch = "wasm32"))]
pub use note_sync::{NoteSyncManager, SyncResult};
#[cfg(not(target_arch = "wasm32"))]
pub use notes::{NoteScanResult, NoteScanner, OrchardNote, SerializableOrchardNote, SpendableNote};
#[cfg(not(target_arch = "wasm32"))]
pub use paths::{
    get_wallet_config_dir, get_wallet_config_path, get_wallet_data_dir, get_wallet_data_path,
};
#[cfg(not(target_arch = "wasm32"))]
pub use rpc_test::RpcTester;
#[cfg(feature = "secret-network")]
pub use secret::{
    SecretRpcClient, SecretTransactionRecord, SecretTransactionStatus, SecretTransactionStorage,
    SecretWallet, Snip20Token,
};
#[cfg(not(target_arch = "wasm32"))]
pub use secret_keys::{
    SecretDerivationPath, SecretKeyDerivation, SecretKeyPair, SECRET_ADDRESS_PREFIX,
    SECRET_COIN_TYPE,
};
#[cfg(not(target_arch = "wasm32"))]
pub use storage::{WalletData, WalletStorage};
#[cfg(not(target_arch = "wasm32"))]
pub use swap::{SwapDirection, SwapEngine, SwapRequest, SwapResponse, SwapService, SwapStatus};
#[cfg(not(target_arch = "wasm32"))]
pub use transaction_builder::ZcashTransactionBuilder;
#[cfg(not(target_arch = "wasm32"))]
pub use transaction_history::{
    SentTransactionRecord, TransactionStatus, TransactionType, TransactionView,
};
#[cfg(not(target_arch = "wasm32"))]
pub use transaction_tracker::TransactionConfirmationTracker;
#[cfg(not(target_arch = "wasm32"))]
pub use zeaking::{IndexStats, IndexedBlock, IndexedTransaction, Zeaking};
#[cfg(not(target_arch = "wasm32"))]
pub use zeaking_adapter::{ZebraBlockParser, ZebraBlockSource};
#[cfg(not(target_arch = "wasm32"))]
pub use zebra_integration::ZebraClient;
