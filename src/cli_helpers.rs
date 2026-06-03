use crate::error::{NozyError, NozyResult};
use crate::fee_policy::{estimate_orchard_send_fee_zatoshis, PilotSendOptions};
use crate::{load_config, HDWallet, NoteScanner, WalletStorage, ZebraClient};
use dialoguer::Password;

/// ZIP-317 client-side fee for an Orchard send (Zebrad does not implement `estimatefee`).
pub async fn estimate_transaction_fee_for_send(
    _zebra_client: &ZebraClient,
    memo: Option<&[u8]>,
    priority: bool,
) -> u64 {
    let fee_zatoshis = estimate_orchard_send_fee_zatoshis(memo, priority);
    let label = if priority {
        "priority (×4)"
    } else {
        "standard"
    };
    println!(
        "💰 Estimated fee ({label}): {:.8} ZEC ({fee_zatoshis} zats, ZIP-317)",
        fee_zatoshis as f64 / 100_000_000.0,
    );
    fee_zatoshis
}

/// Back-compat: standard (non-priority) ZIP-317 estimate.
pub async fn estimate_transaction_fee(zebra_client: &ZebraClient) -> u64 {
    estimate_transaction_fee_for_send(zebra_client, None, false).await
}

pub async fn load_wallet() -> NozyResult<(HDWallet, WalletStorage)> {
    use crate::paths::get_wallet_data_dir;
    let storage = WalletStorage::with_xdg_dir();

    let wallet_path = get_wallet_data_dir().join("wallet.dat");
    if !wallet_path.exists() {
        return Err(NozyError::Storage(
            "No wallet found. Use 'nozy new' or 'nozy restore' to create a wallet first."
                .to_string(),
        ));
    }

    if let Ok(wallet) = storage.load_wallet("").await {
        return Ok((wallet, storage));
    }

    let password = Password::new()
        .with_prompt("Enter wallet password")
        .interact()
        .map_err(|e| NozyError::InvalidOperation(format!("Password input error: {}", e)))?;

    let wallet = storage.load_wallet(&password).await?;
    Ok((wallet, storage))
}

pub async fn scan_notes_for_sending(
    wallet: HDWallet,
    zebra_url: &str,
) -> NozyResult<Vec<crate::SpendableNote>> {
    let mut config = load_config();
    config.zebra_url = zebra_url.to_string();
    let zebra_client = ZebraClient::from_config(&config);
    let tip_height = zebra_client.get_block_count().await?;

    // Sending should bias for reliability over speed:
    // - If we have a last scan height, rewind enough to recover from stale/incomplete local state.
    // - Otherwise use a wide fallback that still avoids scanning from genesis.
    const SEND_SCAN_REWIND_BLOCKS: u32 = 50_000;
    const SEND_SCAN_WIDE_FALLBACK_BLOCKS: u32 = 500_000;
    const SEND_SCAN_MAINNET_FLOOR: u32 = 3_276_000;
    let start_height = if let Some(last) = config.last_scan_height {
        last.saturating_sub(SEND_SCAN_REWIND_BLOCKS)
    } else {
        tip_height
            .saturating_sub(SEND_SCAN_WIDE_FALLBACK_BLOCKS)
            .min(SEND_SCAN_MAINNET_FLOOR)
    }
    .min(tip_height);

    let mut note_scanner = NoteScanner::new(wallet, zebra_client.clone());
    let (_result, spendable) = note_scanner
        .scan_notes(Some(start_height), Some(tip_height))
        .await?;
    Ok(spendable)
}

pub async fn build_and_broadcast_transaction(
    zebra_client: &ZebraClient,
    spendable_notes: &[crate::SpendableNote],
    recipient: &str,
    amount_zatoshis: u64,
    fee_zatoshis: Option<u64>,
    memo: Option<&[u8]>,
    enable_broadcast: bool,
    zebra_url: &str,
    pilot: PilotSendOptions,
) -> NozyResult<String> {
    let fee_zatoshis = if let Some(fee) = fee_zatoshis {
        fee
    } else {
        estimate_transaction_fee_for_send(zebra_client, memo, pilot.priority).await
    };
    use crate::transaction_history::{SentTransactionRecord, SentTransactionStorage};
    use crate::ZcashTransactionBuilder;

    let mut tx_builder = ZcashTransactionBuilder::new();
    tx_builder.set_zebra_url(zebra_url);

    if enable_broadcast {
        tx_builder.enable_mainnet_broadcast();
    }

    let tip_height = zebra_client.get_best_block_height().await?;
    let expiry_height = tip_height.saturating_add(pilot.expiry_delta_blocks);

    let transaction = tx_builder
        .build_send_transaction(
            zebra_client,
            spendable_notes,
            recipient,
            amount_zatoshis,
            fee_zatoshis,
            memo,
            pilot,
        )
        .await?;

    use crate::privacy_ui::show_transaction_privacy_summary;
    show_transaction_privacy_summary();

    if enable_broadcast {
        match tx_builder
            .broadcast_transaction(zebra_client, &transaction)
            .await
        {
            Ok(network_txid) => {
                use crate::privacy_ui::show_privacy_badge;
                show_privacy_badge();
                println!("✅ Transaction broadcast successfully!");
                println!("🆔 Network TXID: {}", network_txid);

                let tx_storage = SentTransactionStorage::new()?;
                let spent_note_ids: Vec<String> = spendable_notes
                    .iter()
                    .map(|note| hex::encode(note.orchard_note.nullifier.to_bytes()))
                    .collect();

                let mut tx_record = SentTransactionRecord::new_pilot(
                    network_txid.clone(),
                    recipient.to_string(),
                    amount_zatoshis,
                    fee_zatoshis,
                    memo.map(|m| m.to_vec()),
                    spent_note_ids,
                    pilot.priority,
                    expiry_height,
                );
                tx_record.mark_broadcast();
                tx_storage.save_transaction(tx_record)?;

                println!("📝 Transaction saved to history - will track confirmations");

                Ok(network_txid)
            }
            Err(e) => Err(e),
        }
    } else {
        Err(NozyError::InvalidOperation(
            "Broadcasting disabled".to_string(),
        ))
    }
}

pub fn handle_insufficient_funds_error(error: &NozyError) {
    if error.to_string().contains("Insufficient funds") {
        println!("💡 You don't have enough ZEC to send this amount.");
        println!("   Run 'sync' to update your balance.");
    }
}
