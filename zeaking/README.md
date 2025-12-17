# Zeaking - Fast Local Blockchain Indexing System

Zeaking is a fast, local blockchain indexing system for Zcash. It provides efficient indexing of blocks, transactions, and Orchard actions to enable quick queries without constantly hitting the blockchain node.

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

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
zeaking = { path = "../zeaking" }  # Or from crates.io when published
async-trait = "0.1"
```

## License

MIT

