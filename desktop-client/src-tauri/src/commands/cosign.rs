use crate::error::TauriError;
use crate::session::load_session_wallet;
use nozy::{
    build_keystone_send_pczt, estimate_transaction_fee_for_send, extract_signed_tx_from_pczt_bytes,
    load_config, mark_wallet_notes_spent_from_spendables,
    orchard_spending_key_from_wallet, prepared_send_from_build, scan_notes_for_sending,
    select_single_spend_note,
    sign_pczt_orchard_spends, transaction_history::{SentTransactionRecord, SentTransactionStorage},
    KeystonePreparedSend, KeystoneWalletConfig, PilotSendOptions, NOZY_WALLET_PRIORITY_FEE,
    PILOT_EXPIRY_DELTA_BLOCKS, ZebraClient, ZebraJsonRpcOrchardWitnessProvider,
};
use serde::{Deserialize, Serialize};
use tauri::command;
use zcash_protocol::consensus::NetworkType;

fn network_from_config() -> NetworkType {
    let config = load_config();
    if config.network == "testnet" {
        NetworkType::Test
    } else {
        NetworkType::Main
    }
}

#[derive(Debug, Deserialize)]
pub struct PrepareCosignRequest {
    pub recipient: String,
    pub amount: f64,
    pub memo: Option<String>,
    pub password: Option<String>,
    pub zebra_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PrepareCosignResponse {
    pub request: KeystonePreparedSend,
    pub ur_frames: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SignCosignRequest {
    pub pczt_hex: String,
    pub password: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SignCosignResponse {
    pub pczt_hex: String,
}

#[derive(Debug, Deserialize)]
pub struct CompleteCosignSendRequest {
    pub pczt_hex: String,
    pub recipient: String,
    pub amount: f64,
    pub memo: Option<String>,
    pub password: Option<String>,
    pub zebra_url: Option<String>,
}

#[command]
pub async fn prepare_cosign_request(
    request: PrepareCosignRequest,
) -> Result<PrepareCosignResponse, TauriError> {
    let config = load_config();
    let expected_testnet = config.network.eq_ignore_ascii_case("testnet");
    let recipient = request.recipient.trim().to_string();
    let prefix_ok = if expected_testnet {
        recipient.starts_with("utest1")
    } else {
        recipient.starts_with("u1")
    };
    if !prefix_ok || recipient.len() < 78 {
        return Err(TauriError::from(
            if expected_testnet {
                "Invalid recipient address. Testnet co-sign sends require a valid shielded address (utest1...).".to_string()
            } else {
                "Invalid recipient address. Mainnet co-sign sends require a valid shielded address (u1...).".to_string()
            },
        ));
    }
    if request.amount <= 0.0 {
        return Err(TauriError::from("Amount must be greater than 0.".to_string()));
    }

    let zebra_url = request
        .zebra_url
        .unwrap_or_else(|| config.zebra_url.clone());
    let wallet = load_session_wallet(request.password.as_deref()).await?;
    let spendable_notes = scan_notes_for_sending(wallet.clone(), &zebra_url)
        .await
        .map_err(|e| TauriError::from(e.to_string()))?;

    let amount_zatoshis = (request.amount * 100_000_000.0) as u64;
    let zebra_client = ZebraClient::from_config(&config);
    let pilot = PilotSendOptions {
        priority: NOZY_WALLET_PRIORITY_FEE,
        expiry_delta_blocks: PILOT_EXPIRY_DELTA_BLOCKS,
    };
    let memo_preview = request
        .memo
        .as_ref()
        .map(|m| m.trim().as_bytes())
        .filter(|b| !b.is_empty());
    let fee_zatoshis =
        estimate_transaction_fee_for_send(&zebra_client, memo_preview, pilot.priority).await;

    let witness_provider = ZebraJsonRpcOrchardWitnessProvider;
    let keystone = KeystoneWalletConfig::default();
    let network = network_from_config();
    let build = build_keystone_send_pczt(
        &zebra_client,
        &witness_provider,
        &wallet,
        &keystone,
        &spendable_notes,
        &recipient,
        amount_zatoshis,
        fee_zatoshis,
        memo_preview,
        pilot,
        network,
    )
    .await
    .map_err(|e| TauriError::from(e.to_string()))?;

    let prepared = prepared_send_from_build(&recipient, amount_zatoshis, fee_zatoshis, &build);
    let ur_frames =
        nozy::encode_pczt_ur_frames(&build.pczt_bytes, nozy::DEFAULT_UR_FRAGMENT_SIZE)
            .map_err(|e| TauriError::from(e.to_string()))?;

    Ok(PrepareCosignResponse {
        request: prepared,
        ur_frames,
    })
}

#[command]
pub async fn sign_cosign_request(
    request: SignCosignRequest,
) -> Result<SignCosignResponse, TauriError> {
    let pczt_bytes = hex::decode(request.pczt_hex.trim())
        .map_err(|e| TauriError::from(format!("Invalid PCZT hex: {e}")))?;
    let wallet = load_session_wallet(request.password.as_deref()).await?;
    let spending_key = orchard_spending_key_from_wallet(&wallet)
        .map_err(|e| TauriError::from(e.to_string()))?;
    let signed = sign_pczt_orchard_spends(&pczt_bytes, &spending_key)
        .map_err(|e| TauriError::from(e.to_string()))?;

    Ok(SignCosignResponse {
        pczt_hex: hex::encode(signed),
    })
}

#[command]
pub async fn complete_cosign_send(
    request: CompleteCosignSendRequest,
) -> Result<super::transaction::SendTransactionResponse, TauriError> {
    let pczt_bytes = hex::decode(request.pczt_hex.trim())
        .map_err(|e| TauriError::from(format!("Invalid PCZT hex: {e}")))?;
    let extracted = extract_signed_tx_from_pczt_bytes(&pczt_bytes)
        .map_err(|e| TauriError::from(e.to_string()))?;

    let config = load_config();
    let zebra_url = request
        .zebra_url
        .unwrap_or_else(|| config.zebra_url.clone());
    let zebra_client = ZebraClient::from_config(&config);
    let raw_hex = hex::encode(&extracted.raw_transaction);
    let txid = zebra_client
        .broadcast_transaction(&raw_hex)
        .await
        .map_err(|e| TauriError::from(e.to_string()))?;

    let wallet = load_session_wallet(request.password.as_deref()).await?;
    let spendable_notes = scan_notes_for_sending(wallet.clone(), &zebra_url)
        .await
        .map_err(|e| TauriError::from(e.to_string()))?;
    let amount_zatoshis = (request.amount * 100_000_000.0) as u64;
    let pilot = PilotSendOptions {
        priority: NOZY_WALLET_PRIORITY_FEE,
        expiry_delta_blocks: PILOT_EXPIRY_DELTA_BLOCKS,
    };
    let memo_preview = request
        .memo
        .as_ref()
        .map(|m| m.trim().as_bytes())
        .filter(|b| !b.is_empty());
    let fee_zatoshis =
        estimate_transaction_fee_for_send(&zebra_client, memo_preview, pilot.priority).await;
    let chain_tip = zebra_client.get_best_block_height().await.unwrap_or(0);
    let expiry_height = chain_tip.saturating_add(PILOT_EXPIRY_DELTA_BLOCKS);

    if let Ok(spent_note) = select_single_spend_note(&spendable_notes, amount_zatoshis, fee_zatoshis)
    {
        let _ = mark_wallet_notes_spent_from_spendables(std::slice::from_ref(spent_note), Some(&txid));
        if let Ok(tx_storage) = SentTransactionStorage::new() {
            let spent_note_ids = vec![hex::encode(spent_note.orchard_note.nullifier.to_bytes())];
            let memo_bytes = request
                .memo
                .as_ref()
                .map(|m| m.trim().as_bytes().to_vec())
                .filter(|b| !b.is_empty());
            let mut tx_record = SentTransactionRecord::new_pilot(
                txid.clone(),
                request.recipient.clone(),
                amount_zatoshis,
                fee_zatoshis,
                memo_bytes,
                spent_note_ids,
                pilot.priority,
                expiry_height,
            );
            tx_record.mark_broadcast();
            let _ = tx_storage.save_transaction(tx_record);
        }
    }

    Ok(super::transaction::SendTransactionResponse {
        success: true,
        txid: Some(txid.clone()),
        message: format!("Co-signed transaction broadcast successfully! TXID: {txid}"),
    })
}
