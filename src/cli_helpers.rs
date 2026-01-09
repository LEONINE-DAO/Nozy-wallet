use crate::error::{NozyError, NozyResult};
use crate::{load_config, HDWallet, NoteScanner, WalletStorage, ZebraClient};
use dialoguer::Password;

pub async fn estimate_transaction_fee(zebra_client: &ZebraClient) -> u64 {
    const DEFAULT_FEE_ZATOSHIS: u64 = 10_000;

    match zebra_client.get_fee_estimate().await {
        Ok(estimate) => {
            if let Some(fee_value) = estimate.get("fee") {
                if let Some(fee_f64) = fee_value.as_f64() {
                    let fee_zatoshis = (fee_f64 * 100_000_000.0) as u64;
                    if fee_zatoshis > 0 {
                        println!("💰 Estimated fee: {:.8} ZEC ({})", fee_f64, fee_zatoshis);
                        return fee_zatoshis;
                    }
                } else if let Some(fee_u64) = fee_value.as_u64() {
                    println!(
                        "💰 Estimated fee: {:.8} ZEC ({})",
                        fee_u64 as f64 / 100_000_000.0,
                        fee_u64
                    );
                    return fee_u64;
                }
            }

            for field in &["feerate", "feeRate", "fee_rate"] {
                if let Some(fee_value) = estimate.get(*field) {
                    if let Some(fee_f64) = fee_value.as_f64() {
                        let fee_zatoshis = (fee_f64 * 100_000_000.0) as u64;
                        if fee_zatoshis > 0 {
                            println!("💰 Estimated fee: {:.8} ZEC ({})", fee_f64, fee_zatoshis);
                            return fee_zatoshis;
                        }
                    }
                }
            }

            println!(
                "⚠️  Could not parse fee estimate, using default: {:.8} ZEC",
                DEFAULT_FEE_ZATOSHIS as f64 / 100_000_000.0
            );
            DEFAULT_FEE_ZATOSHIS
        }
        Err(e) => {
            println!(
                "⚠️  Fee estimation failed: {}, using default: {:.8} ZEC",
                e,
                DEFAULT_FEE_ZATOSHIS as f64 / 100_000_000.0
            );
            DEFAULT_FEE_ZATOSHIS
        }
    }
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
    let tip_height = match zebra_client.get_block_count().await {
        Ok(h) => h,
        Err(_e) => 3_066_071,
    };
    let start_height = tip_height.saturating_sub(10_000);
    let mut note_scanner = NoteScanner::new(wallet, zebra_client.clone());

    match note_scanner
        .scan_notes(Some(start_height), Some(tip_height))
        .await
    {
        Ok((_result, spendable)) => Ok(spendable),
        Err(_e) => Ok(Vec::new()),
    }
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
) -> NozyResult<()> {
    let fee_zatoshis = if let Some(fee) = fee_zatoshis {
        fee
    } else {
        estimate_transaction_fee(zebra_client).await
    };
    use crate::transaction_history::{SentTransactionRecord, SentTransactionStorage};
    use crate::ZcashTransactionBuilder;

    let mut tx_builder = ZcashTransactionBuilder::new();
    tx_builder.set_zebra_url(zebra_url);

    if enable_broadcast {
        tx_builder.enable_mainnet_broadcast();
    }

    let transaction = tx_builder
        .build_send_transaction(
            zebra_client,
            spendable_notes,
            recipient,
            amount_zatoshis,
            fee_zatoshis,
            memo,
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

                let mut tx_record = SentTransactionRecord::new(
                    network_txid.clone(),
                    recipient.to_string(),
                    amount_zatoshis,
                    fee_zatoshis,
                    memo.map(|m| m.to_vec()),
                    spent_note_ids,
                );
                tx_record.mark_broadcast();
                tx_storage.save_transaction(tx_record)?;

                println!("📝 Transaction saved to history - will track confirmations");
            }
            Err(e) => {
                return Err(e);
            }
        }
    } else {
        return Err(NozyError::InvalidOperation(
            "Broadcasting disabled".to_string(),
        ));
    }

    Ok(())
}

pub fn handle_insufficient_funds_error(error: &NozyError) {
    if error.to_string().contains("Insufficient funds") {
        println!("💡 You don't have enough ZEC to send this amount.");
        println!("   Run 'sync' to update your balance.");
    }
}
