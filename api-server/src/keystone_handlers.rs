use axum::{http::StatusCode, response::Json as ResponseJson, Json};
use serde::{Deserialize, Serialize};
use zcash_protocol::consensus::NetworkType;

use crate::handlers::{error_response, load_wallet_with_password, validate_amount};

fn network_from_config(config: &nozy::WalletConfig) -> NetworkType {
    if config.network == "testnet" {
        NetworkType::Test
    } else {
        NetworkType::Main
    }
}

fn keystone_mainnet_error() -> (StatusCode, ResponseJson<serde_json::Value>) {
    error_response(
        StatusCode::BAD_REQUEST,
        "Keystone is configured for Zcash mainnet only. Set wallet network to mainnet in config.",
    )
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

pub async fn keystone_status(
) -> Result<ResponseJson<KeystoneStatusResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::{load_config, load_pending_send};

    let config = load_config();
    let pending_send = load_pending_send().ok().flatten().is_some();

    Ok(ResponseJson(KeystoneStatusResponse {
        enabled: config.keystone.enabled,
        device_label: config.keystone.device_label.clone(),
        has_ufvk: config.keystone.ufvk.as_ref().is_some_and(|s| !s.is_empty()),
        pending_send,
        network: config.network.clone(),
    }))
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

pub async fn keystone_enable(
    Json(payload): Json<KeystoneEnableRequest>,
) -> Result<ResponseJson<KeystoneEnableResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::{load_config, save_config};

    let config = load_config();
    if payload.enabled && config.network == "testnet" {
        return Err(keystone_mainnet_error());
    }

    let mut config = config;
    config.keystone.enabled = payload.enabled;
    if let Some(label) = payload.device_label {
        let trimmed = label.trim();
        config.keystone.device_label = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        };
    }
    save_config(&config).map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save Keystone config: {e}"),
        )
    })?;

    Ok(ResponseJson(KeystoneEnableResponse {
        success: true,
        enabled: payload.enabled,
    }))
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

pub async fn keystone_export_ufvk(
    Json(payload): Json<KeystoneExportUfvkRequest>,
) -> Result<ResponseJson<KeystoneExportUfvkResponse>, (StatusCode, ResponseJson<serde_json::Value>)>
{
    use nozy::{export_ufvk_from_wallet, load_config, save_config, validate_ufvk};

    let (wallet, _storage) = load_wallet_with_password(payload.password)
        .await
        .map_err(|e| error_response(StatusCode::UNAUTHORIZED, e))?;

    let config = load_config();
    if config.network == "testnet" {
        return Err(keystone_mainnet_error());
    }
    let network = network_from_config(&config);
    let ufvk = export_ufvk_from_wallet(&wallet, network).map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to export UFVK: {e}"),
        )
    })?;
    validate_ufvk(&ufvk, network).map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Exported UFVK validation failed: {e}"),
        )
    })?;

    let mut config = load_config();
    config.keystone.ufvk = Some(ufvk.clone());
    save_config(&config).map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save UFVK to config: {e}"),
        )
    })?;

    Ok(ResponseJson(KeystoneExportUfvkResponse {
        success: true,
        ufvk,
    }))
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pczt_hex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ur_frames: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ur_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

pub async fn keystone_prepare_send(
    Json(payload): Json<KeystonePrepareSendRequest>,
) -> Result<ResponseJson<KeystonePrepareResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    use nozy::cli_helpers::{is_zebra_unavailable_error, scan_notes_for_sending};
    use nozy::{
        build_keystone_send_pczt, encode_pczt_ur_frames, estimate_transaction_fee_for_send,
        load_config, prepared_send_from_build, save_pending_send, PilotSendOptions, ZebraClient,
        ZebraJsonRpcOrchardWitnessProvider, DEFAULT_UR_FRAGMENT_SIZE, PILOT_EXPIRY_DELTA_BLOCKS,
        UR_TYPE_ZCASH_PCZT,
    };

    let config = load_config();
    if config.network == "testnet" {
        return Ok(ResponseJson(KeystonePrepareResponse {
            success: false,
            message: Some(
                "Keystone is configured for Zcash mainnet only. Set wallet network to mainnet in config."
                    .to_string(),
            ),
            summary: None,
            action_count: None,
            pczt_hex: None,
            ur_frames: None,
            ur_type: None,
        }));
    }

    if !payload.recipient.starts_with("u1") || payload.recipient.len() < 78 {
        return Ok(ResponseJson(KeystonePrepareResponse {
            success: false,
            message: Some(
                "Invalid recipient. Use a mainnet Orchard unified address (u1…).".to_string(),
            ),
            summary: None,
            action_count: None,
            pczt_hex: None,
            ur_frames: None,
            ur_type: None,
        }));
    }

    if !validate_amount(payload.amount) {
        return Ok(ResponseJson(KeystonePrepareResponse {
            success: false,
            message: Some("Invalid amount. Must be greater than 0.".to_string()),
            summary: None,
            action_count: None,
            pczt_hex: None,
            ur_frames: None,
            ur_type: None,
        }));
    }

    let zebra_url = payload
        .zebra_url
        .unwrap_or_else(|| config.zebra_url.clone());

    let (wallet, _storage) = load_wallet_with_password(payload.password)
        .await
        .map_err(|e| error_response(StatusCode::UNAUTHORIZED, e))?;

    let spendable_notes = match scan_notes_for_sending(wallet.clone(), &zebra_url).await {
        Ok(notes) => notes,
        Err(e) => {
            let msg = e.to_string();
            if is_zebra_unavailable_error(&msg) {
                return Err(error_response(
                    StatusCode::SERVICE_UNAVAILABLE,
                    format!("Zebra node unavailable during note scan: {msg}"),
                ));
            }
            return Err(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to scan notes: {msg}"),
            ));
        }
    };

    let amount_zatoshis = (payload.amount * 100_000_000.0) as u64;
    let zebra_client = ZebraClient::from_config_with_url(&config, Some(&zebra_url));
    let pilot = PilotSendOptions {
        priority: payload.priority,
        expiry_delta_blocks: PILOT_EXPIRY_DELTA_BLOCKS,
    };
    let memo_preview = payload
        .memo
        .as_ref()
        .map(|m| m.trim().as_bytes())
        .filter(|b| !b.is_empty());
    let fee_zatoshis =
        estimate_transaction_fee_for_send(&zebra_client, memo_preview, pilot.priority).await;

    nozy::warm_orchard_proving_key();

    let witness_provider = ZebraJsonRpcOrchardWitnessProvider;
    let network = network_from_config(&config);
    let build = build_keystone_send_pczt(
        &zebra_client,
        &witness_provider,
        &wallet,
        &config.keystone,
        &spendable_notes,
        &payload.recipient,
        amount_zatoshis,
        fee_zatoshis,
        memo_preview,
        pilot,
        network,
    )
    .await
    .map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to build Keystone PCZT: {e}"),
        )
    })?;

    let prepared =
        prepared_send_from_build(&payload.recipient, amount_zatoshis, fee_zatoshis, &build);
    save_pending_send(&prepared).map_err(|e| {
        error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save pending send: {e}"),
        )
    })?;

    let ur_frames =
        encode_pczt_ur_frames(&build.pczt_bytes, DEFAULT_UR_FRAGMENT_SIZE).map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to encode UR frames: {e}"),
            )
        })?;

    Ok(ResponseJson(KeystonePrepareResponse {
        success: true,
        summary: Some(prepared.summary.clone()),
        action_count: Some(prepared.action_count),
        pczt_hex: Some(prepared.pczt_hex.clone()),
        ur_frames: Some(ur_frames),
        ur_type: Some(UR_TYPE_ZCASH_PCZT.to_string()),
        message: None,
    }))
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub txid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub broadcast: Option<bool>,
}

pub async fn keystone_complete_send(
    Json(payload): Json<KeystoneCompleteSendRequest>,
) -> Result<ResponseJson<KeystoneCompleteSendResponse>, (StatusCode, ResponseJson<serde_json::Value>)>
{
    use nozy::cli_helpers::scan_notes_for_sending;
    use nozy::transaction_history::{SentTransactionRecord, SentTransactionStorage};
    use nozy::{
        clear_pending_send, decode_pczt_ur_frames, extract_signed_tx_from_pczt_bytes, load_config,
        load_pending_send, mark_wallet_notes_spent_from_spendables, select_single_spend_note,
        ZebraClient,
    };
    use nozy::{NOZY_WALLET_PRIORITY_FEE, PILOT_EXPIRY_DELTA_BLOCKS};

    let config = load_config();
    if config.network == "testnet" {
        return Err(keystone_mainnet_error());
    }

    let pczt_bytes = if let Some(frames) = payload.ur_frames.filter(|f| !f.is_empty()) {
        decode_pczt_ur_frames(&frames).map_err(|e| {
            error_response(
                StatusCode::BAD_REQUEST,
                format!("Failed to decode UR frames: {e}"),
            )
        })?
    } else if let Some(hex_str) = payload.pczt_hex.filter(|s| !s.trim().is_empty()) {
        hex::decode(hex_str.trim()).map_err(|e| {
            error_response(StatusCode::BAD_REQUEST, format!("Invalid PCZT hex: {e}"))
        })?
    } else {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            "Provide pczt_hex or ur_frames from Keystone",
        ));
    };

    let extracted = extract_signed_tx_from_pczt_bytes(&pczt_bytes).map_err(|e| {
        error_response(
            StatusCode::BAD_REQUEST,
            format!("Failed to extract signed transaction: {e}"),
        )
    })?;

    if !payload.broadcast {
        let _ = clear_pending_send();
        return Ok(ResponseJson(KeystoneCompleteSendResponse {
            success: true,
            txid: Some(extracted.txid.clone()),
            broadcast: Some(false),
        }));
    }

    let zebra_url = payload
        .zebra_url
        .unwrap_or_else(|| config.zebra_url.clone());
    let zebra_client = ZebraClient::from_config_with_url(&config, Some(&zebra_url));
    let raw_hex = hex::encode(&extracted.raw_transaction);
    let txid = zebra_client
        .broadcast_transaction(&raw_hex)
        .await
        .map_err(|e| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Broadcast failed: {e}"),
            )
        })?;

    let pending = load_pending_send().ok().flatten();
    if let Some(prepared) = pending {
        if let Ok((wallet, _)) = load_wallet_with_password(None).await {
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

    Ok(ResponseJson(KeystoneCompleteSendResponse {
        success: true,
        txid: Some(txid),
        broadcast: Some(true),
    }))
}
