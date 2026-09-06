//! Nozy Sync Engine — Zebra-family ingest + LWD-compatible CompactTxStreamer for Zeaking.
//!
//! Tracking: https://github.com/LEONINE-DAO/Nozy-wallet/issues/274

pub mod banner;
#[rustfmt::skip]
mod banner_ansi;
pub mod compact;
pub mod config;
pub mod error;
pub mod ingest;
pub mod parity;
pub mod rpc;
pub mod serve;
pub mod store;
pub mod tree_sizes;
pub mod treestate;

#[allow(clippy::all)]
pub mod proto {
    tonic::include_proto!("cash.z.wallet.sdk.rpc");
}

pub use config::IndexerConfig;
pub use error::{IndexerError, IndexerResult};
pub use store::IndexerStore;
