# Zeaking - Fast Local Blockchain Indexing System

Zeaking is a fast, local blockchain indexing system for Zcash. It provides efficient indexing of blocks, transactions, and Orchard actions to enable quick queries without constantly hitting the blockchain node.

**Canonical sync layer for Nozy:** lightwalletd gRPC, compact SQLite cache, and related wallet wiring should go through **`zeaking`** (and the thin surfaces below — Tauri, `api-server`, `zeaking-ffi`, extension → companion). Avoid duplicating sync logic in JavaScript or Kotlin/Swift except as UI glue.

## Features

- **Block Indexing**: Index blocks by height, hash, time, size
- **Transaction Indexing**: Index transactions by txid, block height, nullifiers
- **Orchard Action Indexing**: Track Orchard spends and outputs
- **Fast Queries**: Query indexed data without network calls
- **Persistent Storage**: Save/load index from disk
- **Trait-based API**: Works with any blockchain backend

## Usage

### Basic Example

```rust
use zeaking::{Zeaking, BlockSource, DefaultBlockParser};
use zeaking::error::ZeakingResult;
use zeaking::types::BlockData;

struct MyBlockClient;

#[async_trait::async_trait]
impl BlockSource for MyBlockClient {
    async fn get_block(&self, height: u32) -> ZeakingResult<BlockData> {
        todo!()
    }
    
    async fn get_block_hash(&self, height: u32) -> ZeakingResult<String> {
        todo!()
    }
    
    async fn get_block_count(&self) -> ZeakingResult<u32> {
        todo!()
    }
}

let indexer = Zeaking::new(
    "/path/to/index".into(),
    MyBlockClient,
    DefaultBlockParser
).await?;

indexer.index_range(1000, 2000).await?;

let block = indexer.get_block(1500)?;
let tx = indexer.get_transaction("txid...")?;
```

## Architecture

Zeaking uses a trait-based design to work with any blockchain backend:

- **`BlockSource`**: Trait for fetching block data from your node
- **`BlockParser`**: Trait for parsing block data (optional, has default implementation)
- **`Zeaking`**: Main indexer that uses these traits

## lightwalletd (Zebrad + compact blocks)

### Build prerequisites (`lightwalletd` feature)

The **`build.rs`** step compiles vendored `.proto` files and needs Google’s **`protoc`** on your `PATH` (or set **`PROTOC`** to its full path).

- **Ubuntu / Debian / WSL:** `sudo apt install protobuf-compiler`
- **macOS:** `brew install protobuf`
- **Windows:** install [protobuf releases](https://github.com/protocolbuffers/protobuf/releases) and ensure `protoc.exe` is on `PATH`, or set `PROTOC`.

If `protoc` is missing, `cargo` panics with *Could not find `protoc`*.

Enable the **`lightwalletd`** Cargo feature for gRPC to [lightwalletd](https://github.com/zcash/lightwalletd) (backed by Zebrad), SQLite storage of compact blocks, and a [`BlockSource`](src/traits.rs) implementation designed for compact-block sync and local witness derivation.

**Shielded sends (Orchard):** Nozy derives **Merkle witnesses** locally (`incrementalmerkletree`) using `z_gettreestate` checkpoints and Orchard commitments in **chain order**. Use **`orchard_cmx_bytes_from_compact_block`** on cached compact blobs to replay the note commitment tree and cross-check roots against RPC (`nozy::orchard_chain_tree`). Historical limits are documented in **`ZEBRAD_SHIELDED_SEND_LIMIT.md`**.

```toml
zeaking = { path = "../zeaking", features = ["lightwalletd"] }
```

API (under `zeaking::lwd`):

- `connect_lightwalletd`, `LwdClient` — gRPC connection
- `LwdCompactStore` — SQLite (`compact_blocks`, `sync_meta`; extend for witnesses as needed)
- `sync_compact_range`, `chain_tip_height` — download a height range into the store
- `LightwalletdBlockSource` — use with `Zeaking::new(..., LightwalletdBlockSource::connect(uri).await?, ...)` to index from compact data

Protos are vendored under `proto/` (Zcash MIT license).

### Nozy integration (built)

- **Tauri** ([`desktop-client/src-tauri`](../desktop-client/src-tauri)): commands `lwd_get_info`, `lwd_chain_tip`, `lwd_sync_compact` (see `commands/lwd.rs`). Env default: `LIGHTWALLETD_GRPC` (fallback `http://127.0.0.1:9067`). Compact DB: `wallet_data/lwd_compact.sqlite` unless `db_path` is passed.
- **HTTP API** ([`api-server`](../api-server)): `GET /api/lwd/info`, `GET /api/lwd/chain-tip`, `POST /api/lwd/sync/compact` — same env; use from Chrome/Edge extensions with `host_permissions` for your API host (e.g. `http://127.0.0.1:3000`). See [`browser-extension/COMPANION.md`](../browser-extension/COMPANION.md).
- **Mobile UniFFI** ([`zeaking-ffi`](../zeaking-ffi)): `lwd_get_info`, `lwd_chain_tip`, `lwd_sync_compact` — same semantics as Tauri/API; generate Kotlin/Swift with `uniffi-bindgen` from the built `cdylib` (see that crate’s README).

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
zeaking = { path = "../zeaking" }  # Or from crates.io when published
async-trait = "0.1"
```

## License

MIT

