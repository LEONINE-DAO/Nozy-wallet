pub mod address_book;
pub mod benchmarks;
pub mod block_parser;
pub mod bridge;
pub mod cache;
pub mod cli_helpers;
pub mod config;
pub mod error;
pub mod groth16_prover_simple;
pub mod grpc_client;
pub mod hd_wallet;
pub mod input_validation;
pub mod key_management;
pub mod local_analytics;
pub mod monero;
pub mod monero_zk_verifier;
pub mod note_index;
pub mod note_sync;
pub mod notes;
pub mod nu6_1_check;
pub mod orchard_tx;
pub mod paths;
pub mod privacy;
pub mod privacy_network;
pub mod privacy_ui;
pub mod progress;
pub mod proving;
pub mod rpc_test;
pub mod safe_display;
#[cfg(feature = "secret-network")]
pub mod secret;
pub mod secret_keys;
pub mod storage;
pub mod swap;
#[cfg(test)]
pub mod tests;
pub mod transaction_builder;
pub mod transaction_history;
pub mod transaction_tracker;
pub mod transactions;
pub mod zeaking_adapter;
pub mod zebra_integration;

pub use address_book::{AddressBook, AddressEntry};
pub use block_parser::BlockParser;
pub use bridge::{
    AddressTracker, ChurnManager, ChurnRecommendation, PrivacyCheckResult, PrivacyValidator,
    StoredSwap, SwapStorage,
};
pub use cli_helpers::estimate_transaction_fee;
pub use config::{load_config, save_config, update_last_scan_height, WalletConfig};
pub use config::{BackendKind, Protocol};
pub use error::{NozyError, NozyResult};
pub use hd_wallet::HDWallet;
pub use monero::{
    MoneroRpcClient, MoneroTransactionRecord, MoneroTransactionStatus, MoneroTransactionStorage,
    MoneroWallet,
};
pub use monero_zk_verifier::{MoneroZkVerifier, VerificationLevel, VerificationResult};
pub use note_index::NoteIndex;
pub use note_sync::{NoteSyncManager, SyncResult};
pub use notes::{NoteScanResult, NoteScanner, OrchardNote, SerializableOrchardNote, SpendableNote};
pub use paths::{
    get_wallet_config_dir, get_wallet_config_path, get_wallet_data_dir, get_wallet_data_path,
};
pub use rpc_test::RpcTester;
#[cfg(feature = "secret-network")]
pub use secret::{
    SecretRpcClient, SecretTransactionRecord, SecretTransactionStatus, SecretTransactionStorage,
    SecretWallet, Snip20Token,
};
pub use secret_keys::{
    SecretDerivationPath, SecretKeyDerivation, SecretKeyPair, SECRET_ADDRESS_PREFIX,
    SECRET_COIN_TYPE,
};
pub use storage::{WalletData, WalletStorage};
pub use swap::{SwapDirection, SwapEngine, SwapRequest, SwapResponse, SwapService, SwapStatus};
pub use transaction_builder::ZcashTransactionBuilder;
pub use transaction_history::{
    SentTransactionRecord, TransactionStatus, TransactionType, TransactionView,
};
pub use transaction_tracker::TransactionConfirmationTracker;
pub use transactions::{SignedTransaction, TransactionBuilder, TransactionDetails};
pub use zeaking::{IndexStats, IndexedBlock, IndexedTransaction, Zeaking};
pub use zeaking_adapter::{ZebraBlockParser, ZebraBlockSource};
pub use zebra_integration::ZebraClient;
