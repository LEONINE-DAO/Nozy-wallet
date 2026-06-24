//! Live verification for send-readiness: witness lag, warm proving, stale-send guard.
//!
//! Usage:
//!   ZEBRA_RPC_URL=http://172.20.199.206:8232 cargo run --release --bin test_send_readiness

use nozy::{
    ensure_witness_fresh_for_send, load_config, load_wallet_notes, max_witness_lag_blocks,
    orchard_witness_lag_blocks, warm_orchard_proving_key, witness_lag_from_stored_tip,
    WalletStorage, ZebraClient, MAX_SEND_WITNESS_LAG_BLOCKS, WITNESS_CATCHUP_PARALLEL_BLOCKS,
};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("== Nozy send-readiness live check ==\n");

    let config = load_config();
    let zebra = ZebraClient::from_config(&config);
    let chain_tip = zebra.get_best_block_height().await?;
    println!("Chain tip: {chain_tip}");
    println!("Zebra URL: {}", config.zebra_url);
    println!(
        "Witness lag threshold: {MAX_SEND_WITNESS_LAG_BLOCKS} blocks; parallel catch-up batch: {WITNESS_CATCHUP_PARALLEL_BLOCKS}\n"
    );

    let storage = WalletStorage::with_xdg_dir();
    let wallet = storage.load_wallet("").await?;
    let notes_path = nozy::paths::get_wallet_data_dir().join("notes.json");
    println!("Wallet data: {}", notes_path.display());

    let serialized = load_wallet_notes()?;
    let max_lag_serialized = serialized
        .iter()
        .filter(|n| !n.spent)
        .map(|n| witness_lag_from_stored_tip(n.orchard_witness_tip_height, chain_tip))
        .max()
        .unwrap_or(0);
    println!(
        "Serialized notes: {} total, max witness lag (unspent): {max_lag_serialized} blocks",
        serialized.len()
    );

    let notes = nozy::load_spendable_notes_from_wallet(&wallet)?;
    println!("Spendable notes loaded: {}", notes.len());

    let max_lag = max_witness_lag_blocks(&notes, chain_tip);
    println!("Max witness lag across notes: {max_lag} blocks");

    for (i, note) in notes.iter().enumerate() {
        let lag = orchard_witness_lag_blocks(note, chain_tip);
        let stored = note.orchard_witness_tip_height;
        let value_zec = note.orchard_note.value as f64 / 100_000_000.0;
        println!("  note[{i}] value={value_zec:.8} ZEC witness_tip={stored:?} lag={lag} blocks");
    }

    // Simulate Gilmore-style stale wallet (3381308 witness vs current tip).
    let gilmore_lag = witness_lag_from_stored_tip(Some(3381308), chain_tip);
    println!("\nSimulated lag (witness tip 3381308 vs tip {chain_tip}): {gilmore_lag} blocks");
    if gilmore_lag > MAX_SEND_WITNESS_LAG_BLOCKS {
        println!("  => Would BLOCK send (expected for stale wallet before sync)");
    }

    if let Some(spend_note) = notes
        .iter()
        .filter(|n| !n.orchard_note.spent)
        .max_by_key(|n| n.orchard_note.value)
    {
        print!("\nensure_witness_fresh_for_send (largest unspent note): ");
        match ensure_witness_fresh_for_send(spend_note, chain_tip) {
            Ok(()) => println!("PASS — witness fresh enough to send"),
            Err(e) => {
                let msg = e.to_string();
                println!("BLOCKED (expected if stale)");
                println!("  {msg}");
                if nozy::is_witness_stale_for_send_error(&msg) {
                    println!("  => stale error classification: OK");
                }
            }
        }
    }

    println!("\nOrchard proving warm-up:");
    let t0 = Instant::now();
    warm_orchard_proving_key();
    println!(
        "  first call:  {:?} (builds cached proving key)",
        t0.elapsed()
    );
    let t1 = Instant::now();
    warm_orchard_proving_key();
    println!("  second call: {:?} (should be near-instant)", t1.elapsed());

    if max_lag > MAX_SEND_WITNESS_LAG_BLOCKS || max_lag_serialized > MAX_SEND_WITNESS_LAG_BLOCKS {
        let effective = max_lag.max(max_lag_serialized);
        println!(
            "\nOverall: STALE (lag {effective} blocks) — sync to tip before send (`nozy sync --to-tip` or POST /api/sync)."
        );
        std::process::exit(2);
    }

    println!("\nOverall: READY — witness lag within send threshold.");
    Ok(())
}
