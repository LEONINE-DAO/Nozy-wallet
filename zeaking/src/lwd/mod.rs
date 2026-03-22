//! [lightwalletd](https://github.com/zcash/lightwalletd) gRPC client, compact-block SQLite cache, and
//! [`crate::BlockSource`](crate::traits::BlockSource) for Zebrad-backed setups (no `z_getauthpath`).

#[allow(clippy::all)]
pub mod proto {
    tonic::include_proto!("cash.z.wallet.sdk.rpc");
}

mod block_source;
mod client;
mod store;
mod sync;

pub use block_source::LightwalletdBlockSource;
pub use client::{connect_lightwalletd, LwdClient};
pub use store::LwdCompactStore;
pub use sync::{chain_tip_height, sync_compact_range};
