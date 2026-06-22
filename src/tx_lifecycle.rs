//! Dynamic-fee pilot lifecycle: expiry detection, balance release, and speed-up rebuild.

use crate::cli_helpers::scan_notes_for_sending;
use crate::config::load_config;
use crate::error::{NozyError, NozyResult};
use crate::fee_policy::{
    estimate_orchard_send_fee_zatoshis, PilotSendOptions, PILOT_EXPIRY_DELTA_BLOCKS,
};
use crate::hd_wallet::HDWallet;
use crate::notes::mark_wallet_notes_spent_from_spendables;
use crate::transaction_builder::ZcashTransactionBuilder;
use crate::transaction_history::{
    SentTransactionRecord, SentTransactionStorage, TransactionStatus,
};
use crate::zebra_integration::ZebraClient;
use orchard::keys::FullViewingKey;

/// Expire unmined pilot txs past `expiry_height` and release their pending note locks.
pub async fn expire_stale_pending_transactions(zebra_client: &ZebraClient) -> NozyResult<usize> {
    let tx_storage = SentTransactionStorage::new()?;
    tx_storage
        .check_expired_pending_transactions(zebra_client)
        .await
}

/// Rebuild a brand-new priority transaction after the original expired (not a rebroadcast).
pub async fn speed_up_transaction(
    wallet: HDWallet,
    zebra_url: &str,
    original_txid: &str,
) -> NozyResult<String> {
    let tx_storage = SentTransactionStorage::new()?;
    let mut config = load_config();
    if config.zebra_url != zebra_url {
        config.zebra_url = zebra_url.to_string();
    }
    let zebra_client = ZebraClient::from_config(&config);

    tx_storage
        .check_expired_pending_transactions(&zebra_client)
        .await?;

    let original = tx_storage
        .get_transaction_record(original_txid)
        .ok_or_else(|| {
            NozyError::InvalidOperation(format!(
                "Transaction {original_txid} not found in wallet history"
            ))
        })?;

    if original.status == TransactionStatus::Confirmed {
        return Err(NozyError::InvalidOperation(
            "Transaction already confirmed; speed-up is not applicable".to_string(),
        ));
    }

    if original.status == TransactionStatus::Pending {
        if let Some(expiry) = original.expiry_height {
            let tip = zebra_client.get_block_count().await?;
            if tip <= expiry {
                return Err(NozyError::InvalidOperation(format!(
                    "Transaction has not expired yet (expiry height {expiry}, tip {tip})"
                )));
            }
            if zebra_client.check_transaction_exists(original_txid).await? {
                return Err(NozyError::InvalidOperation(
                    "Transaction is pending on chain; wait for confirmation instead of speed-up"
                        .to_string(),
                ));
            }
            tx_storage.mark_transaction_expired(original_txid)?;
            let _ = crate::notes::release_wallet_notes_by_nullifier_hex(&original.spent_note_ids);
        } else {
            return Err(NozyError::InvalidOperation(
                "Only pilot transactions with expiry can be sped up".to_string(),
            ));
        }
    } else if original.status != TransactionStatus::Expired {
        return Err(NozyError::InvalidOperation(format!(
            "Speed-up requires an expired transaction (current status: {:?})",
            original.status
        )));
    }

    let refreshed = tx_storage
        .get_transaction_record(original_txid)
        .ok_or_else(|| NozyError::InvalidOperation("Transaction record disappeared".to_string()))?;
    if refreshed.status != TransactionStatus::Expired {
        return Err(NozyError::InvalidOperation(
            "Transaction must be expired before speed-up".to_string(),
        ));
    }

    let spendable_notes = scan_notes_for_sending(wallet, zebra_url).await?;
    if spendable_notes.is_empty() {
        return Err(NozyError::InvalidOperation(
            "No spendable notes available for speed-up; sync the wallet first".to_string(),
        ));
    }

    let memo = refreshed.memo.as_deref();
    let pilot = PilotSendOptions {
        priority: true,
        expiry_delta_blocks: PILOT_EXPIRY_DELTA_BLOCKS,
    };
    let fee_zatoshis = estimate_orchard_send_fee_zatoshis(memo, true);
    let total_needed = refreshed.amount_zatoshis.saturating_add(fee_zatoshis);
    let available: u64 = spendable_notes.iter().map(|n| n.orchard_note.value).sum();
    if available < total_needed {
        return Err(NozyError::InvalidOperation(format!(
            "Insufficient funds for speed-up: need {:.8} ZEC, have {:.8} ZEC",
            total_needed as f64 / 100_000_000.0,
            available as f64 / 100_000_000.0,
        )));
    }

    let mut tx_builder = ZcashTransactionBuilder::new();
    tx_builder.set_zebra_url(zebra_url);
    tx_builder.enable_mainnet_broadcast();

    let transaction = tx_builder
        .build_and_broadcast_send_transaction(
            &zebra_client,
            &spendable_notes,
            &refreshed.recipient_address,
            refreshed.amount_zatoshis,
            fee_zatoshis,
            memo,
            pilot,
        )
        .await?;

    let network_txid = transaction.txid.clone();

    let spent_note_ids: Vec<String> = spendable_notes
        .iter()
        .map(|note| {
            let fvk = FullViewingKey::from(&note.spending_key);
            hex::encode(note.orchard_note.note.nullifier(&fvk).to_bytes())
        })
        .collect();

    if let Err(e) = mark_wallet_notes_spent_from_spendables(&spendable_notes) {
        eprintln!("Warning: could not mark spent notes locally after speed-up: {e}");
    }

    let mut tx_record = SentTransactionRecord::new_speed_up(
        network_txid.clone(),
        &refreshed,
        fee_zatoshis,
        transaction.expiry_height,
        spent_note_ids,
    );
    tx_record.mark_broadcast();
    tx_storage.save_transaction(tx_record)?;

    Ok(network_txid)
}
