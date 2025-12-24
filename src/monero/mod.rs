// Monero integration module
// Provides Monero wallet functionality with privacy proxy support

pub mod rpc_client;
pub mod wallet;
pub mod transaction_history;

pub use rpc_client::MoneroRpcClient;
pub use wallet::MoneroWallet;
pub use transaction_history::{MoneroTransactionStorage, MoneroTransactionRecord, MoneroTransactionStatus};
