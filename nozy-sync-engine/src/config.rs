//! Operator configuration (env + CLI).

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::Parser;

use crate::error::{IndexerError, IndexerResult};

#[derive(Debug, Clone, Parser)]
#[command(
    name = "nozy-sync-engine",
    about = "Nozy Sync Engine — Zebra/Zakura → CompactTxStreamer for Zeaking (issue #274)"
)]
pub struct IndexerConfig {
    /// Zebra-family JSON-RPC URL (Zebrad or Zakura).
    #[arg(long, env = "ZEBRA_RPC_URL", default_value = "http://127.0.0.1:8232")]
    pub rpc_url: String,

    /// Alias for rpc_url (optional override).
    #[arg(long, env = "NOZY_SYNC_ENGINE_RPC_URL")]
    pub indexer_rpc_url: Option<String>,

    /// gRPC listen address for CompactTxStreamer.
    #[arg(long, env = "NOZY_SYNC_ENGINE_BIND", default_value = "127.0.0.1:9067")]
    pub bind: SocketAddr,

    /// SQLite path for compact blocks.
    #[arg(
        long,
        env = "NOZY_SYNC_ENGINE_DB",
        default_value = "nozy_sync_engine_compact.sqlite"
    )]
    pub db_path: PathBuf,

    /// Expected chain name (`main`, `test`, `regtest`). Empty = accept whatever the node reports.
    #[arg(long, env = "NOZY_SYNC_ENGINE_NETWORK", default_value = "")]
    pub network: String,

    /// Ingest poll interval in milliseconds.
    #[arg(long, env = "NOZY_SYNC_ENGINE_POLL_MS", default_value_t = 2_000)]
    pub poll_ms: u64,

    /// Start ingest from this height when the store is empty.
    #[arg(long, env = "NOZY_SYNC_ENGINE_START_HEIGHT", default_value_t = 0)]
    pub start_height: u64,

    /// When the store is empty, backfill at most this many blocks from tip.
    #[arg(long, env = "NOZY_SYNC_ENGINE_BACKFILL", default_value_t = 500)]
    pub backfill: u64,

    /// Log filter (RUST_LOG override).
    #[arg(long, env = "RUST_LOG", default_value = "info")]
    pub rust_log: String,

    /// Skip the Zeaking ASCII splash (CI, systemd, piped logs).
    #[arg(long, env = "NOZY_SYNC_ENGINE_NO_BANNER")]
    pub no_banner: bool,

    /// Print the pixel splash and exit (no node / gRPC). Use this to preview the logo.
    #[arg(long)]
    pub print_banner: bool,
}

impl IndexerConfig {
    pub fn effective_rpc_url(&self) -> &str {
        self.indexer_rpc_url
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(self.rpc_url.as_str())
    }

    pub fn validate(&self) -> IndexerResult<()> {
        let url = self.effective_rpc_url().trim();
        if url.is_empty() {
            return Err(IndexerError::Config(
                "ZEBRA_RPC_URL / NOZY_SYNC_ENGINE_RPC_URL is empty".into(),
            ));
        }
        if self.poll_ms == 0 {
            return Err(IndexerError::Config("poll_ms must be > 0".into()));
        }
        Ok(())
    }
}
