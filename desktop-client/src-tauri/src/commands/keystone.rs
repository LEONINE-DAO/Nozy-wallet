use crate::error::TauriError;
use crate::session::load_session_wallet;
use nozy::{
    build_keystone_send_pczt, clear_pending_send, decode_pczt_ur_frames,
    encode_pczt_ur_frames, estimate_transaction_fee_for_send, export_ufvk_from_wallet,
    extract_signed_tx_from_pczt_bytes, load_config, load_pending_send, mark_wallet_notes_spent_from_spendables,
    prepared_send_from_build, save_config, save_pending_send, scan_notes_for_sending,
    select_single_spend_note, validate_ufvk, ZebraClient, ZebraJsonRpcOrchardWitnessProvider,
    DEFAULT_UR_FRAGMENT_SIZE, KeystonePreparedSend, PilotSendOptions, NOZY_WALLET_PRIORITY_FEE,
    PILOT_EXPIRY_DELTA_BLOCKS, UR_TYPE_ZCASH_PCZT,
};
use nozy::transaction_history::{SentTransactionRecord, SentTransactionStorage};
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

fn ensure_keystone_mainnet() -> Result<(), TauriError> {
    let config = load_config();
    if config.network == "testnet" {
        return Err(TauriError::from(
            "Keystone is configured for Zcash mainnet only. Set wallet network to mainnet in config."
                .to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct KeystoneStatusResponse {
    pub enabled: bool,
    pub device_label: Option<String>,
    pub has_ufvk: bool,
    pub pending_send: bool,
    /// Wallet network from config (`mainnet` or `testnet`).
    pub network: String,
}

#[command]
pub async fn get_keystone_status() -> Result<KeystoneStatusResponse, TauriError> {
    let config = load_config();
    let pending_send = load_pending_send()
        .map_err(|e| TauriError::from(e.to_string()))?
        .is_some();

    Ok(KeystoneStatusResponse {
        enabled: config.keystone.enabled,
        device_label: config.keystone.device_label.clone(),
        has_ufvk: config
            .keystone
            .ufvk
            .as_ref()
            .is_some_and(|s| !s.is_empty()),
        pending_send,
        network: config.network.clone(),
    })
}

#[derive(Debug, Deserialize)]
pub struct KeystoneEnableRequest {
    pub enabled: bool,
    pub device_label: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct KeystoneEnableResponse {
    pub success: bool,
    pub enabled: bool,
}

#[command]
pub async fn set_keystone_enabled(
    request: KeystoneEnableRequest,
) -> Result<KeystoneEnableResponse, TauriError> {
    if request.enabled {
        ensure_keystone_mainnet()?;
    }
    let mut config = load_config();
    config.keystone.enabled = request.enabled;
    if let Some(label) = request.device_label {
        let trimmed = label.trim();
        config.keystone.device_label = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        };
    }
    save_config(&config).map_err(|e| TauriError::from(e.to_string()))?;

    Ok(KeystoneEnableResponse {
        success: true,
        enabled: request.enabled,
    })
}

#[derive(Debug, Deserialize)]
pub struct KeystoneExportUfvkRequest {
    pub password: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct KeystoneExportUfvkResponse {
    pub success: bool,
    pub ufvk: String,
}

#[command]
pub async fn export_keystone_ufvk(
    request: KeystoneExportUfvkRequest,
) -> Result<KeystoneExportUfvkResponse, TauriError> {
    ensure_keystone_mainnet()?;
    let wallet = load_session_wallet(request.password.as_deref()).await?;
    let network = network_from_config();
    let ufvk = export_ufvk_from_wallet(&wallet, network)
        .map_err(|e| TauriError::from(e.to_string()))?;
    validate_ufvk(&ufvk, network).map_err(|e| TauriError::from(e.to_string()))?;

    let mut config = load_config();
    config.keystone.ufvk = Some(ufvk.clone());
    save_config(&config).map_err(|e| TauriError::from(e.to_string()))?;

    Ok(KeystoneExportUfvkResponse {
        success: true,
        ufvk,
    })
}

#[derive(Debug, Deserialize)]
pub struct KeystonePrepareSendRequest {
    pub recipient: String,
    pub amount: f64,
    pub memo: Option<String>,
    #[serde(default = "default_true")]
    pub priority: bool,
    pub password: Option<String>,
    pub zebra_url: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize)]
pub struct KeystonePrepareResponse {
    pub success: bool,
    pub summary: Option<String>,
    pub action_count: Option<u32>,
    pub pczt_hex: Option<String>,
    pub ur_frames: Option<Vec<String>>,
    pub ur_type: Option<String>,
    pub message: Option<String>,
}

#[command]
pub async fn keystone_prepare_send(
    request: KeystonePrepareSendRequest,
) -> Result<KeystonePrepareResponse, TauriError> {
    if let Err(e) = ensure_keystone_mainnet() {
        return Ok(KeystonePrepareResponse {
            success: false,
            message: Some(e.message),
            summary: None,
            action_count: None,
            pczt_hex: None,
            ur_frames: None,
            ur_type: None,
        });
    }
    if !request.recipient.starts_with("u1") || request.recipient.len() < 78 {
        return Ok(KeystonePrepareResponse {
            success: false,
            message: Some(
                "Invalid recipient. Use a mainnet Orchard unified address (u1…).".to_string(),
            ),
            summary: None,
            action_count: None,
            pczt_hex: None,
            ur_frames: None,
            ur_type: None,
        });
    }
    if request.amount <= 0.0 {
        return Ok(KeystonePrepareResponse {
            success: false,
            message: Some("Amount must be greater than 0.".to_string()),
            summary: None,
            action_count: None,
            pczt_hex: None,
            ur_frames: None,
            ur_type: None,
        });
    }

    let config = load_config();
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
        priority: request.priority,
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
    let network = network_from_config();
    let build = build_keystone_send_pczt(
        &zebra_client,
        &witness_provider,
        &wallet,
        &config.keystone,
        &spendable_notes,
        &request.recipient,
        amount_zatoshis,
        fee_zatoshis,
        memo_preview,
        pilot,
        network,
    )
    .await
    .map_err(|e| TauriError::from(e.to_string()))?;

    let prepared = prepared_send_from_build(
        &request.recipient,
        amount_zatoshis,
        fee_zatoshis,
        &build,
    );
    save_pending_send(&prepared).map_err(|e| TauriError::from(e.to_string()))?;
    let ur_frames =
        encode_pczt_ur_frames(&build.pczt_bytes, DEFAULT_UR_FRAGMENT_SIZE)
            .map_err(|e| TauriError::from(e.to_string()))?;

    Ok(KeystonePrepareResponse {
        success: true,
        summary: Some(prepared.summary.clone()),
        action_count: Some(prepared.action_count),
        pczt_hex: Some(prepared.pczt_hex.clone()),
        ur_frames: Some(ur_frames),
        ur_type: Some(UR_TYPE_ZCASH_PCZT.to_string()),
        message: None,
    })
}

#[derive(Debug, Deserialize)]
pub struct KeystoneCompleteSendRequest {
    pub pczt_hex: Option<String>,
    pub ur_frames: Option<Vec<String>>,
    #[serde(default = "default_true")]
    pub broadcast: bool,
    pub zebra_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct KeystoneCompleteSendResponse {
    pub success: bool,
    pub txid: Option<String>,
    pub broadcast: Option<bool>,
}

#[command]
pub async fn keystone_complete_send(
    request: KeystoneCompleteSendRequest,
) -> Result<KeystoneCompleteSendResponse, TauriError> {
    ensure_keystone_mainnet()?;
    let pczt_bytes = if let Some(frames) = request.ur_frames.filter(|f| !f.is_empty()) {
        decode_pczt_ur_frames(&frames).map_err(|e| TauriError::from(e.to_string()))?
    } else if let Some(hex_str) = request.pczt_hex.filter(|s| !s.trim().is_empty()) {
        hex::decode(hex_str.trim())
            .map_err(|e| TauriError::from(format!("Invalid PCZT hex: {e}")))?
    } else {
        return Err(TauriError::from(
            "Provide pczt_hex or ur_frames from Keystone".to_string(),
        ));
    };

    let extracted = extract_signed_tx_from_pczt_bytes(&pczt_bytes)
        .map_err(|e| TauriError::from(e.to_string()))?;

    if !request.broadcast {
        let _ = clear_pending_send();
        return Ok(KeystoneCompleteSendResponse {
            success: true,
            txid: Some(extracted.txid.clone()),
            broadcast: Some(false),
        });
    }

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

    let pending: Option<KeystonePreparedSend> = load_pending_send()
        .map_err(|e| TauriError::from(e.to_string()))?;
    if let Some(prepared) = pending {
        if let Ok(wallet) = load_session_wallet(None).await {
            if let Ok(spendable_notes) = scan_notes_for_sending(wallet, &zebra_url).await {
                if let Ok(spent_note) = select_single_spend_note(
                    &spendable_notes,
                    prepared.amount_zatoshis,
                    prepared.fee_zatoshis,
                ) {
                    let _ = mark_wallet_notes_spent_from_spendables(
                        std::slice::from_ref(spent_note),
                        Some(&txid),
                    );
                    if let Ok(tx_storage) = SentTransactionStorage::new() {
                        let spent_note_ids =
                            vec![hex::encode(spent_note.orchard_note.nullifier.to_bytes())];
                        let mut tx_record = SentTransactionRecord::new_pilot(
                            txid.clone(),
                            prepared.recipient.clone(),
                            prepared.amount_zatoshis,
                            prepared.fee_zatoshis,
                            None,
                            spent_note_ids,
                            NOZY_WALLET_PRIORITY_FEE,
                            zebra_client
                                .get_best_block_height()
                                .await
                                .unwrap_or(0)
                                .saturating_add(PILOT_EXPIRY_DELTA_BLOCKS),
                        );
                        tx_record.mark_broadcast();
                        let _ = tx_storage.save_transaction(tx_record);
                    }
                }
            }
        }
    }

    let _ = clear_pending_send();

    Ok(KeystoneCompleteSendResponse {
        success: true,
        txid: Some(txid),
        broadcast: Some(true),
    })
}
