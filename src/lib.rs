// ============================================================
// WASM-safe modules (available on all targets)
// ============================================================
pub mod error;
pub mod fee_policy;
pub mod groth16_prover_simple;
pub mod hd_wallet;
pub mod input_validation;
pub mod key_management;
pub mod privacy;
pub mod safe_display;
#[cfg(feature = "native")]
pub mod secret_keys;
pub mod traits;
pub mod transactions;
pub mod version_info;

// ============================================================
// Native-only modules (filesystem, networking, CLI, threading)
// ============================================================
#[cfg(feature = "native")]
pub mod address_book;
#[cfg(feature = "native")]
pub mod benchmarks;
#[cfg(feature = "native")]
pub mod block_parser;
#[cfg(feature = "native")]
pub mod bridge;
#[cfg(feature = "native")]
pub mod cache;
#[cfg(feature = "native")]
pub mod cli_helpers;
#[cfg(feature = "native")]
pub mod config;
#[cfg(feature = "native")]
pub mod grpc_client;
#[cfg(feature = "native")]
pub mod local_analytics;
#[cfg(feature = "native")]
pub mod monero;
#[cfg(feature = "native")]
pub mod monero_zk_verifier;
#[cfg(feature = "native")]
pub mod note_index;
#[cfg(feature = "native")]
pub mod note_sync;
#[cfg(feature = "native")]
pub mod notes;
#[cfg(feature = "native")]
pub mod nu6_1_check;
#[cfg(feature = "native")]
pub mod orchard_chain_tree;
#[cfg(feature = "native")]
pub mod orchard_tree_codec;
#[cfg(feature = "native")]
pub mod orchard_tx;
#[cfg(feature = "native")]
pub mod orchard_witness;
#[cfg(feature = "native")]
pub mod paths;
#[cfg(feature = "native")]
pub mod privacy_network;
#[cfg(feature = "native")]
pub mod privacy_ui;
#[cfg(feature = "native")]
pub mod progress;
#[cfg(feature = "native")]
pub mod proving;
#[cfg(feature = "native")]
pub mod rpc_test;
#[cfg(feature = "secret-network")]
pub mod secret;
#[cfg(feature = "native")]
pub mod storage;
#[cfg(feature = "native")]
pub mod swap;
#[cfg(feature = "native")]
pub mod sync_status;
#[cfg(test)]
pub mod tests;
#[cfg(feature = "native")]
pub mod transaction_builder;
#[cfg(feature = "native")]
pub mod transaction_history;
#[cfg(feature = "native")]
pub mod transaction_tracker;
#[cfg(feature = "native")]
pub mod zeaking_adapter;
#[cfg(feature = "native")]
pub mod zebra_integration;
#[cfg(feature = "native")]
pub mod zebra_tree_rpc;

// ============================================================
// WASM-safe re-exports
// ============================================================
pub use error::{NozyError, NozyResult};
pub use fee_policy::{
    estimate_orchard_send_fee_zatoshis, OrchardSendFeeShape, PilotSendOptions,
    PILOT_EXPIRY_DELTA_BLOCKS, PRIORITY_MULTIPLIER,
};
pub use hd_wallet::HDWallet;
pub use transactions::{SignedTransaction, TransactionBuilder, TransactionDetails};
pub use version_info::{RELEASE_CODENAME, RELEASE_VERSION, VERSION_DISPLAY};

// ============================================================
// Native-only re-exports
// ============================================================
#[cfg(feature = "native")]
pub use address_book::{AddressBook, AddressEntry};
#[cfg(feature = "native")]
pub use block_parser::BlockParser;
#[cfg(feature = "native")]
pub use bridge::{
    AddressTracker, ChurnManager, ChurnRecommendation, PrivacyCheckResult, PrivacyValidator,
    StoredSwap, SwapStorage,
};
#[cfg(feature = "native")]
pub use cli_helpers::{
    cached_unspent_balance_zatoshis, estimate_transaction_fee, estimate_transaction_fee_for_send,
    format_insufficient_funds_message, is_insufficient_funds_error, is_zebra_unavailable_error,
    scan_notes_for_sending,
};
#[cfg(feature = "native")]
pub use config::{load_config, save_config, update_last_scan_height, WalletConfig};
#[cfg(feature = "native")]
pub use config::{BackendKind, Protocol};
#[cfg(feature = "native")]
pub use monero::{
    MoneroRpcClient, MoneroTransactionRecord, MoneroTransactionStatus, MoneroTransactionStorage,
    MoneroWallet,
};
#[cfg(feature = "native")]
pub use monero_zk_verifier::{MoneroZkVerifier, VerificationLevel, VerificationResult};
#[cfg(feature = "native")]
pub use note_index::NoteIndex;
#[cfg(feature = "native")]
pub use note_sync::{NoteSyncManager, SyncResult};
#[cfg(feature = "native")]
pub use notes::{NoteScanResult, NoteScanner, OrchardNote, SerializableOrchardNote, SpendableNote};
#[cfg(feature = "native")]
pub use orchard_tx::{
    select_single_spend_note, OrchardBuiltSpend, OrchardTransactionBuilder, OrchardWitnessProvider,
    ZebraJsonRpcOrchardWitnessProvider,
};
#[cfg(feature = "native")]
pub use paths::{
    get_wallet_config_dir, get_wallet_config_path, get_wallet_data_dir, get_wallet_data_path,
};
#[cfg(feature = "native")]
pub use rpc_test::RpcTester;
#[cfg(feature = "secret-network")]
pub use secret::{
    SecretRpcClient, SecretTransactionRecord, SecretTransactionStatus, SecretTransactionStorage,
    SecretWallet, Snip20Token,
};
#[cfg(feature = "native")]
pub use secret_keys::{
    SecretDerivationPath, SecretKeyDerivation, SecretKeyPair, SECRET_ADDRESS_PREFIX,
    SECRET_COIN_TYPE,
};
#[cfg(feature = "native")]
pub use storage::{WalletData, WalletStorage};
#[cfg(feature = "native")]
pub use swap::{SwapDirection, SwapEngine, SwapRequest, SwapResponse, SwapService, SwapStatus};
#[cfg(feature = "native")]
pub use sync_status::{gather_sync_status, print_sync_status, resolve_lightwalletd_url};
#[cfg(feature = "native")]
pub use transaction_builder::ZcashTransactionBuilder;
#[cfg(feature = "native")]
pub use transaction_history::{
    SentTransactionRecord, TransactionStatus, TransactionType, TransactionView,
};
#[cfg(feature = "native")]
pub use transaction_tracker::TransactionConfirmationTracker;
#[cfg(feature = "native")]
pub use zeaking::{IndexStats, IndexedBlock, IndexedTransaction, Zeaking};
#[cfg(feature = "native")]
pub use zeaking_adapter::{ZebraBlockParser, ZebraBlockSource};
#[cfg(feature = "native")]
pub use zebra_integration::ZebraClient;
