// Secret Network integration module
// Provides Secret Network wallet functionality with SNIP-20 token support (Shade tokens)
// This entire module is only compiled when the "secret-network" feature is enabled

pub mod rpc_client;
pub mod wallet;
pub mod snip20;
pub mod transaction;
pub mod transaction_history;

pub use rpc_client::SecretRpcClient;
pub use wallet::SecretWallet;
pub use snip20::Snip20Token;
pub use transaction::{SecretTransactionBuilder, AccountInfo};
pub use transaction_history::{SecretTransactionStorage, SecretTransactionRecord, SecretTransactionStatus};
