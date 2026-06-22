use crate::error::{NozyError, NozyResult};
use crate::fee_policy::{estimate_orchard_send_fee_zatoshis, PilotSendOptions};
use crate::{load_config, HDWallet, NoteScanner, WalletStorage, ZebraClient};
use dialoguer::Password;

/// Sum unspent note values from cached `notes.json` (no Zebra RPC).
pub fn cached_unspent_balance_zatoshis() -> NozyResult<u64> {
    use crate::notes::{load_wallet_notes, wallet_unspent_balance_zatoshis};

    Ok(wallet_unspent_balance_zatoshis(&load_wallet_notes()?))
}

pub fn format_insufficient_funds_message(
    available_zat: u64,
    amount_zat: u64,
    fee_zat: u64,
) -> String {
    let need = amount_zat.saturating_add(fee_zat);
    format!(
        "Insufficient funds: need {:.8} ZEC (amount + fee), have {:.8} ZEC",
        need as f64 / 100_000_000.0,
        available_zat as f64 / 100_000_000.0,
    )
}

pub fn is_insufficient_funds_error(err: &str) -> bool {
    err.contains("Insufficient Orchard funds") || err.contains("Insufficient funds")
}

pub fn is_zebra_unavailable_error(err: &str) -> bool {
    err.contains("Failed to connect to Zebra")
        || err.contains("Connection failed to")
        || err.contains("Is Zebra running")
        || err.contains("Remote Zebra RPC blocked")
        || err.contains("privacy proxy configuration is invalid")
}

/// Structured connect-phase code for API clients (sync, test-zebra, send).
pub fn zebra_connect_api_code(err: &str) -> &'static str {
    let m = err.to_ascii_lowercase();
    if m.contains("remote zebra rpc blocked") || m.contains("privacy policy active") {
        "PRIVACY_POLICY_BLOCKED"
    } else if m.contains("privacy proxy") || m.contains("invalid privacy proxy") {
        "TOR_PROXY_UNREACHABLE"
    } else if m.contains("failed to connect to zebra")
        || m.contains("connection failed to")
        || m.contains("is zebra running")
    {
        "ZEBRA_RPC_UNREACHABLE"
    } else {
        "ZEBRA_RPC_ERROR"
    }
}

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
    use crate::notes::load_spendable_notes_from_wallet;
    use crate::paths::get_wallet_data_dir;
    use crate::wallet_sync::MAINNET_DEFAULT_SCAN_START;

    // Fast path: reuse persisted notes + witnesses from sync (witness catch-up at spend time).
    if let Ok(cached) = load_spendable_notes_from_wallet(&wallet) {
        if !cached.is_empty() {
            return Ok(cached);
        }
    }

    let mut config = load_config();
    config.zebra_url = zebra_url.to_string();
    let zebra_client = ZebraClient::from_config(&config);
    let tip_height = zebra_client.get_block_count().await?;

    // Fallback scan only when cache is empty or notes lack witnesses.
    const SEND_SCAN_REWIND_BLOCKS: u32 = 100;
    const SEND_SCAN_WIDE_FALLBACK_BLOCKS: u32 = 500_000;
    const SEND_SCAN_MAINNET_FLOOR: u32 = 3_276_000;

    let cached_notes = crate::notes::load_wallet_notes().unwrap_or_default();
    let start_height = if let Some(earliest) = cached_notes
        .iter()
        .filter(|n| !n.spent)
        .map(|n| n.block_height)
        .min()
    {
        earliest.saturating_sub(SEND_SCAN_REWIND_BLOCKS)
    } else if let Some(last) = config.last_scan_height {
        last.saturating_add(1)
            .saturating_sub(SEND_SCAN_REWIND_BLOCKS)
    } else {
        tip_height
            .saturating_sub(SEND_SCAN_WIDE_FALLBACK_BLOCKS)
            .max(if config.network == "testnet" {
                1
            } else {
                MAINNET_DEFAULT_SCAN_START
            })
            .min(SEND_SCAN_MAINNET_FLOOR)
    }
    .min(tip_height);

    let notes_path = get_wallet_data_dir().join("notes.json");
    let mut note_scanner = if notes_path.exists() {
        NoteScanner::with_index_file(wallet, zebra_client.clone(), &notes_path)?
    } else {
        NoteScanner::new(wallet, zebra_client.clone())
    };
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

    let transaction = tx_builder
        .build_and_broadcast_send_transaction(
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
        use crate::privacy_ui::show_privacy_badge;
        show_privacy_badge();
        println!("✅ Transaction broadcast successfully!");
        println!("🆔 Network TXID: {}", transaction.txid);

        let tx_storage = SentTransactionStorage::new()?;
        use orchard::keys::FullViewingKey;
        let spent_note_ids: Vec<String> = spendable_notes
            .iter()
            .map(|note| {
                let fvk = FullViewingKey::from(&note.spending_key);
                hex::encode(note.orchard_note.note.nullifier(&fvk).to_bytes())
            })
            .collect();

        if let Err(e) = crate::notes::mark_wallet_notes_spent_from_spendables(spendable_notes) {
            eprintln!("Warning: could not mark spent notes locally: {e}");
        }

        let mut tx_record = SentTransactionRecord::new_pilot(
            transaction.txid.clone(),
            recipient.to_string(),
            amount_zatoshis,
            fee_zatoshis,
            memo.map(|m| m.to_vec()),
            spent_note_ids,
            pilot.priority,
            transaction.expiry_height,
        );
        tx_record.mark_broadcast();
        tx_storage.save_transaction(tx_record)?;

        println!("📝 Transaction saved to history - will track confirmations");

        Ok(transaction.txid)
    } else {
        Err(NozyError::InvalidOperation(
            "Broadcasting disabled".to_string(),
        ))
    }
}

pub fn handle_insufficient_funds_error(error: &NozyError) {
    if is_insufficient_funds_error(&error.to_string()) {
        println!("💡 You don't have enough ZEC to send this amount.");
        println!("   Run 'sync' to update your balance.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_insufficient_funds_message_includes_amounts() {
        let msg = format_insufficient_funds_message(0, 1_000_000, 10_000);
        assert!(msg.contains("Insufficient funds"));
        assert!(msg.contains("0.00000000"));
        assert!(msg.contains("0.01010000"));
    }

    #[test]
    fn classifies_zebra_and_insufficient_errors() {
        assert!(is_zebra_unavailable_error(
            "Failed to connect to Zebra node(s) [http://127.0.0.1:8232]"
        ));
        assert!(is_insufficient_funds_error(
            "Insufficient Orchard funds: need 0.01000000 ZEC, have 0.00000000 ZEC"
        ));
    }
}
