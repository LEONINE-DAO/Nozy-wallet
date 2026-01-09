// Secret Network integration module
// Provides Secret Network wallet functionality with SNIP-20 token support (Shade tokens)
// This entire module is only compiled when the "secret-network" feature is enabled

pub mod rpc_client;
pub mod snip20;
pub mod transaction;
pub mod transaction_history;
pub mod wallet;

pub use rpc_client::SecretRpcClient;
pub use snip20::Snip20Token;
pub use transaction::{AccountInfo, SecretTransactionBuilder};
pub use transaction_history::{
    SecretTransactionRecord, SecretTransactionStatus, SecretTransactionStorage,
};
pub use wallet::SecretWallet;
