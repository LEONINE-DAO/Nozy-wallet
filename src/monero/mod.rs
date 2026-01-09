// Monero integration module
// Provides Monero wallet functionality with privacy proxy support

pub mod rpc_client;
pub mod transaction_history;
pub mod wallet;

pub use rpc_client::MoneroRpcClient;
pub use transaction_history::{
    MoneroTransactionRecord, MoneroTransactionStatus, MoneroTransactionStorage,
};
pub use wallet::MoneroWallet;
