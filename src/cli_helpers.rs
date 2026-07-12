use crate::error::{NozyError, NozyResult};
use crate::fee_policy::{estimate_orchard_send_fee_zatoshis, PilotSendOptions};
use crate::{load_config, HDWallet, NoteScanner, WalletStorage, ZebraClient};
use dialoguer::Password;
pub fn cached_unspent_balance_zatoshis() -> NozyResult<u64> {
    use crate::notes::{load_wallet_notes, wallet_unspent_balance_zatoshis};

    Ok(wallet_unspent_balance_zatoshis(&load_wallet_notes()?))
}

/// Orchard shielded balance from cache, with pending outbound sends subtracted for spendable amount.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WalletBalanceSnapshot {
    pub confirmed_zatoshis: u64,
    pub pending_zatoshis: u64,
    pub available_zatoshis: u64,
    pub unspent_note_count: usize,
}

/// Load confirmed, pending, and available shielded balance from local wallet state.
pub fn wallet_balance_snapshot() -> NozyResult<WalletBalanceSnapshot> {
    use crate::notes::{
        load_wallet_notes, reconcile_wallet_spends_from_local_state,
        wallet_unspent_balance_zatoshis,
    };
    use crate::paths::get_wallet_data_dir;
    use crate::transaction_history::SentTransactionStorage;
    use std::collections::HashSet;

    let _ = reconcile_wallet_spends_from_local_state();

    let notes_path = get_wallet_data_dir().join("notes.json");
    let notes = if notes_path.exists() {
        load_wallet_notes()?
    } else {
        Vec::new()
    };

    let confirmed_zatoshis = wallet_unspent_balance_zatoshis(&notes);
    let unspent_note_count = notes.iter().filter(|note| !note.spent).count();
    let (pending_zatoshis, pending_tx_ids) = SentTransactionStorage::new()
        .map(|storage| {
            let pending = storage.get_pending_transactions();
            let total = pending
                .iter()
                .map(|tx| tx.amount_zatoshis.saturating_add(tx.fee_zatoshis))
                .sum::<u64>();
            let ids = pending.iter().map(|tx| tx.txid.clone()).collect::<Vec<_>>();
            (total, ids)
        })
        .unwrap_or((0, Vec::new()));
    let pending_txid_set: HashSet<&str> = pending_tx_ids.iter().map(String::as_str).collect();
    let pending_spent_input_zatoshis = notes
        .iter()
        .filter(|note| {
            note.spent
                && note
                    .spent_in_txid
                    .as_deref()
                    .is_some_and(|txid| pending_txid_set.contains(txid))
        })
        .map(|note| note.value)
        .sum::<u64>();
    let effective_confirmed_zatoshis =
        confirmed_zatoshis.saturating_add(pending_spent_input_zatoshis);
    let available_zatoshis = effective_confirmed_zatoshis.saturating_sub(pending_zatoshis);

    Ok(WalletBalanceSnapshot {
        confirmed_zatoshis: effective_confirmed_zatoshis,
        pending_zatoshis,
        available_zatoshis,
        unspent_note_count,
    })
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
    _priority: bool,
) -> u64 {
    let fee_zatoshis = estimate_orchard_send_fee_zatoshis(memo, true);
    println!(
        "💰 Estimated fee (priority ×4): {:.8} ZEC ({fee_zatoshis} zats, ZIP-317)",
        fee_zatoshis as f64 / 100_000_000.0,
    );
    fee_zatoshis
}

/// Back-compat alias; NozyWallet always uses the priority fee.
pub async fn estimate_transaction_fee(zebra_client: &ZebraClient) -> u64 {
    estimate_transaction_fee_for_send(zebra_client, None, true).await
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
    use crate::send_readiness::ensure_cached_witness_fresh_for_send;
    use crate::wallet_sync::MAINNET_DEFAULT_SCAN_START;

    let mut config = load_config();
    config.zebra_url = zebra_url.to_string();
    let zebra_client = ZebraClient::from_config(&config);
    let chain_tip = zebra_client.get_best_block_height().await?;
    let cached_notes = crate::notes::load_wallet_notes().unwrap_or_default();
    let ironwood_witness_incomplete = cached_notes.iter().any(|n| {
        !n.spent
            && n.pool == crate::shielded_pool::ShieldedPool::Ironwood
            && !n.witness_hex_for_pool().is_some_and(|w| !w.is_empty())
    });
    if !ironwood_witness_incomplete {
        ensure_cached_witness_fresh_for_send(&cached_notes, chain_tip)?;
    }

    // Fast path: reuse persisted notes + witnesses from sync (witness catch-up at spend time).
    if !ironwood_witness_incomplete {
        if let Ok(cached) = load_spendable_notes_from_wallet(&wallet) {
            if !cached.is_empty() {
                return Ok(cached);
            }
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
    let start_height = if ironwood_witness_incomplete {
        cached_notes
            .iter()
            .filter(|n| !n.spent && n.pool == crate::shielded_pool::ShieldedPool::Ironwood)
            .map(|n| n.block_height)
            .min()
            .unwrap_or(1)
            .saturating_sub(SEND_SCAN_REWIND_BLOCKS)
            .min(tip_height)
    } else if let Some(earliest) = cached_notes
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

    crate::orchard_tx::warm_orchard_proving_key();

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
        let spent_note_ids = if let Some(nullifier_hex) = &transaction.spent_nullifier_hex {
            if let Err(e) = crate::notes::mark_wallet_notes_spent_by_nullifier_hex(
                std::slice::from_ref(nullifier_hex),
                Some(&transaction.txid),
            ) {
                eprintln!("Warning: could not mark spent notes locally: {e}");
            }
            vec![nullifier_hex.clone()]
        } else {
            use orchard::keys::FullViewingKey;
            let spent_note = crate::orchard_tx::select_single_spend_note(
                spendable_notes,
                amount_zatoshis,
                fee_zatoshis,
            )?;
            let fvk = FullViewingKey::from(&spent_note.spending_key);
            let nullifier_hex =
                hex::encode(spent_note.orchard_note.note.nullifier(&fvk).to_bytes());
            if let Err(e) = crate::notes::mark_wallet_notes_spent_from_spendables(
                std::slice::from_ref(spent_note),
                Some(&transaction.txid),
            ) {
                eprintln!("Warning: could not mark spent notes locally: {e}");
            }
            vec![nullifier_hex]
        };

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
