# Library API

The `nozy` crate is the wallet library. HTTP for the extension and local apps is a separate binary: [api-server README](../../api-server/README.md). CLI commands: [CLI usage](../guides/cli-usage.md).

These snippets show the intended call shape. Amounts are zatoshis (1 ZEC = 100_000_000 zatoshis). A typical ZIP-317 2-action fee is 10_000 zatoshis.

## Core types

```rust
use nozy::{HDWallet, ZebraClient};

let wallet = HDWallet::new()?;
wallet.set_password("my_secure_password")?;
let address = wallet.generate_orchard_address(0, 0)?;
let client = ZebraClient::new("http://127.0.0.1:8232".to_string());
```

## Scan, build, broadcast

```rust
use nozy::{
    HDWallet, ZebraClient, OrchardTransactionBuilder, ZebraJsonRpcOrchardWitnessProvider,
    NoteScanner, WalletStorage,
};

#[tokio::main]
async fn main() -> nozy::NozyResult<()> {
    let storage = WalletStorage::with_xdg_dir();
    let wallet = storage.load_wallet("password").await?;

    let zebra_client = ZebraClient::new("http://127.0.0.1:8232".to_string());
    let tip_height = zebra_client.get_block_count().await?;

    let mut scanner = NoteScanner::new(wallet, zebra_client.clone());
    let start_height = tip_height.saturating_sub(10_000);
    let (result, spendable_notes) = scanner
        .scan_notes(Some(start_height), Some(tip_height))
        .await?;

    let amount_zatoshis = 50_000_000; // 0.5 ZEC
    let fee_zatoshis = 10_000;
    let total_needed = amount_zatoshis + fee_zatoshis;
    if result.total_balance < total_needed {
        return Err(nozy::NozyError::InsufficientFunds(format!(
            "Need {} zatoshis, have {}",
            total_needed, result.total_balance
        )));
    }

    let mut builder = OrchardTransactionBuilder::new_async(true).await?;
    let built = builder
        .build_single_spend(
            &zebra_client,
            &ZebraJsonRpcOrchardWitnessProvider,
            &spendable_notes,
            "u1recipientaddress...",
            amount_zatoshis,
            fee_zatoshis,
            Some(b"Payment memo"),
        )
        .await?;

    // Broadcast only after an explicit user confirm on mainnet.
    let tx_hex = hex::encode(&built.raw_transaction);
    let network_txid = zebra_client.broadcast_transaction(&tx_hex).await?;
    println!("sent {network_txid} (ZIP-244: {})", built.txid);
    Ok(())
}
```

## Errors

```rust
use nozy::NozyError;

match result {
    Ok(value) => println!("Success: {value:?}"),
    Err(NozyError::NetworkError(msg)) => eprintln!("Network error: {msg}"),
    Err(NozyError::AddressParsing(msg)) => eprintln!("Address error: {msg}"),
    Err(NozyError::InsufficientFunds(msg)) => eprintln!("Insufficient funds: {msg}"),
    Err(e) => eprintln!("Error: {e}"),
}
```

## Notes and storage

```rust
use nozy::{NoteScanner, WalletStorage};

let mut scanner = NoteScanner::new(wallet, zebra_client);
let (result, spendable_notes) = scanner
    .scan_notes(Some(2_000_000), Some(2_100_000))
    .await?;

for note in &spendable_notes {
    let value = note.orchard_note.note.value().inner();
    println!("Note value: {:.8} ZEC", value as f64 / 100_000_000.0);
}

let storage = WalletStorage::with_xdg_dir();
storage.save_wallet(&wallet, "password").await?;
let _loaded = storage.load_wallet("password").await?;
storage.create_backup("backups").await?;
```

Parallel scan knobs: [performance](performance.md). Witness rules: [`ZEBRAD_SHIELDED_SEND_LIMIT.md`](../../ZEBRAD_SHIELDED_SEND_LIMIT.md).
