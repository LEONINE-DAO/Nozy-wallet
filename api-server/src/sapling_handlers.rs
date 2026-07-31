//! Quiet Sapling legacy HTTP handlers (status / scan / shield).
//! Same core path as CLI `nozy sapling` and desktop Tauri commands.

use axum::{extract::Json, http::StatusCode, response::Json as ResponseJson};
use serde::{Deserialize, Serialize};

use crate::handlers::{error_response_with_code, load_wallet_with_password};
use nozy::fee_policy::PilotSendOptions;
use nozy::paths::get_wallet_data_dir;
use nozy::{
    build_sapling_shield_to_self, derive_sapling_account_keys, load_config, load_sapling_notes,
    sapling_note_has_rseed, sapling_note_ready_to_shield, sapling_shield_fee_zatoshis,
    sapling_unspent_balance_zatoshis, save_sapling_notes, scan_sapling_wallet_from_compact_store,
    ZebraClient,
};

fn compact_db_path() -> std::path::PathBuf {
    get_wallet_data_dir().join("lwd_compact.sqlite")
}

fn network_type(config: &nozy::WalletConfig) -> zcash_protocol::consensus::NetworkType {
    if config.network.eq_ignore_ascii_case("testnet") {
        zcash_protocol::consensus::NetworkType::Test
    } else {
        zcash_protocol::consensus::NetworkType::Main
    }
}

fn map_err(e: nozy::NozyError) -> (StatusCode, ResponseJson<serde_json::Value>) {
    error_response_with_code(
        StatusCode::BAD_REQUEST,
        e.user_friendly_message(),
        "SAPLING",
    )
}

#[derive(Debug, Serialize)]
pub struct SaplingStatusResponse {
    pub unspent_notes: usize,
    pub with_rseed: usize,
    pub ready_to_shield: usize,
    pub unspent_zatoshis: u64,
    pub unspent_zec: f64,
    pub fee_zatoshis: u64,
    pub fee_zec: f64,
    pub has_legacy_balance: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct SaplingScanRequest {
    pub password: Option<String>,
    pub start_floor: Option<u64>,
    pub full: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct SaplingScanResponse {
    pub blocks_scanned: u64,
    pub outputs_seen: u64,
    pub notes_discovered: u64,
    pub notes_marked_spent: u64,
    pub range_start: u64,
    pub range_end: u64,
    pub unspent_zatoshis: u64,
    pub unspent_notes: usize,
    pub message: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct SaplingShieldRequest {
    pub password: Option<String>,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default)]
    pub no_broadcast: bool,
}

#[derive(Debug, Serialize)]
pub struct SaplingShieldResponse {
    pub dry_run: bool,
    pub broadcast: bool,
    pub txid: Option<String>,
    pub shielded_value_zatoshis: Option<u64>,
    pub fee_zatoshis: u64,
    pub expiry_height: Option<u32>,
    pub candidate_notes: usize,
    pub candidate_zatoshis: u64,
    pub message: String,
}

pub async fn get_sapling_status(
) -> Result<ResponseJson<SaplingStatusResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let notes = load_sapling_notes().unwrap_or_default();
    let unspent: Vec<_> = notes.iter().filter(|n| !n.spent).collect();
    let with_rseed = unspent
        .iter()
        .filter(|n| sapling_note_has_rseed(n))
        .count();
    let ready = unspent
        .iter()
        .filter(|n| sapling_note_ready_to_shield(n))
        .count();
    let bal = sapling_unspent_balance_zatoshis(&notes);
    let fee = sapling_shield_fee_zatoshis();
    let message = if ready > 0 {
        "Legacy funds ready to move into your shielded balance.".to_string()
    } else if with_rseed > 0 {
        "Legacy funds found — sync compact blocks, then move into shielded balance.".to_string()
    } else if bal > 0 {
        "Legacy notes need a rescan before they can be moved.".to_string()
    } else {
        "No legacy shielded balance.".to_string()
    };
    Ok(ResponseJson(SaplingStatusResponse {
        unspent_notes: unspent.len(),
        with_rseed,
        ready_to_shield: ready,
        unspent_zatoshis: bal,
        unspent_zec: bal as f64 / 100_000_000.0,
        fee_zatoshis: fee,
        fee_zec: fee as f64 / 100_000_000.0,
        has_legacy_balance: bal > 0,
        message,
    }))
}

pub async fn scan_sapling(
    Json(body): Json<SaplingScanRequest>,
) -> Result<ResponseJson<SaplingScanResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let (wallet, _) = load_wallet_with_password(body.password).await.map_err(|e| {
        error_response_with_code(StatusCode::UNAUTHORIZED, e, "WALLET_AUTH")
    })?;
    let seed = wallet.get_mnemonic_object().to_seed("");
    let db = compact_db_path();
    let store = zeaking::lwd::LwdCompactStore::open(&db).map_err(|e| {
        error_response_with_code(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("open compact store: {e}"),
            "LWD_STORAGE",
        )
    })?;
    let (notes, scan) = scan_sapling_wallet_from_compact_store(
        &seed,
        &store,
        body.start_floor,
        body.full.unwrap_or(false),
    )
    .map_err(map_err)?;
    let unspent_zatoshis = sapling_unspent_balance_zatoshis(&notes);
    let unspent_notes = notes.iter().filter(|n| !n.spent).count();
    Ok(ResponseJson(SaplingScanResponse {
        blocks_scanned: scan.blocks_scanned,
        outputs_seen: scan.outputs_seen,
        notes_discovered: scan.notes_discovered,
        notes_marked_spent: scan.notes_marked_spent,
        range_start: scan.range_start,
        range_end: scan.range_end,
        unspent_zatoshis,
        unspent_notes,
        message: format!(
            "Scanned {} block(s); {} legacy note(s) unspent.",
            scan.blocks_scanned, unspent_notes
        ),
    }))
}

pub async fn shield_sapling(
    Json(body): Json<SaplingShieldRequest>,
) -> Result<ResponseJson<SaplingShieldResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    let (wallet, _) = load_wallet_with_password(body.password.clone())
        .await
        .map_err(|e| error_response_with_code(StatusCode::UNAUTHORIZED, e, "WALLET_AUTH"))?;
    let mut notes = load_sapling_notes().unwrap_or_default();
    let fee = sapling_shield_fee_zatoshis();
    let candidates: Vec<_> = notes
        .iter()
        .filter(|n| !n.spent && sapling_note_has_rseed(n))
        .collect();
    let candidate_zatoshis: u64 = candidates.iter().map(|n| n.value).sum();
    let candidate_notes = candidates.len();

    if body.dry_run {
        return Ok(ResponseJson(SaplingShieldResponse {
            dry_run: true,
            broadcast: false,
            txid: None,
            shielded_value_zatoshis: None,
            fee_zatoshis: fee,
            expiry_height: None,
            candidate_notes,
            candidate_zatoshis,
            message: format!(
                "Dry run: {candidate_notes} note(s), {:.8} ZEC (fee ~{:.8}).",
                candidate_zatoshis as f64 / 100_000_000.0,
                fee as f64 / 100_000_000.0
            ),
        }));
    }

    if candidates.is_empty() {
        return Err(error_response_with_code(
            StatusCode::BAD_REQUEST,
            "No reconstructible legacy notes — sync compact blocks and scan first.",
            "SAPLING_NO_NOTES",
        ));
    }

    let lwd_url = std::env::var("LIGHTWALLETD_GRPC")
        .unwrap_or_else(|_| "http://127.0.0.1:9067".to_string());
    let db = compact_db_path();
    if let Some(parent) = db.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut client) = zeaking::lwd::connect_lightwalletd(&lwd_url).await {
        if let Ok(store) = zeaking::lwd::LwdCompactStore::open(&db) {
            let _ = zeaking::lwd::sync_compact_to_tip_with_options(
                &mut client,
                &store,
                zeaking::lwd::SyncCompactToTipOptions::default(),
            )
            .await;
            let seed = wallet.get_mnemonic_object().to_seed("");
            let _ = scan_sapling_wallet_from_compact_store(&seed, &store, None, false);
            notes = load_sapling_notes().unwrap_or_default();
        }
    }

    let store = zeaking::lwd::LwdCompactStore::open(&db).map_err(|e| {
        error_response_with_code(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("open compact store: {e}"),
            "LWD_STORAGE",
        )
    })?;
    let config = load_config();
    let zebra = ZebraClient::from_config(&config);
    let seed = wallet.get_mnemonic_object().to_seed("");
    let keys = derive_sapling_account_keys(&seed, 0, 0).map_err(map_err)?;
    let expiry = PilotSendOptions::for_send().expiry_delta_blocks;
    let network = network_type(&config);

    let built = build_sapling_shield_to_self(
        &zebra,
        &store,
        &seed,
        &keys.extsk,
        &mut notes,
        network,
        expiry,
    )
    .await
    .map_err(map_err)?;
    save_sapling_notes(&notes).map_err(map_err)?;

    if body.no_broadcast {
        return Ok(ResponseJson(SaplingShieldResponse {
            dry_run: false,
            broadcast: false,
            txid: Some(built.txid.clone()),
            shielded_value_zatoshis: Some(built.shielded_value_zatoshis),
            fee_zatoshis: built.fee_zatoshis,
            expiry_height: Some(built.expiry_height),
            candidate_notes,
            candidate_zatoshis,
            message: format!(
                "Built (not broadcast). TXID {}. Move {:.8} ZEC after fee.",
                built.txid,
                built.shielded_value_zatoshis as f64 / 100_000_000.0
            ),
        }));
    }

    let txid = zebra
        .broadcast_transaction_bytes(&built.raw_transaction)
        .await
        .map_err(map_err)?;
    if let Some(note) = notes
        .iter_mut()
        .find(|n| hex::encode(&n.nullifier_bytes) == built.spent_nullifier_hex)
    {
        note.spent = true;
        note.spent_in_txid = Some(txid.clone());
    }
    save_sapling_notes(&notes).map_err(map_err)?;

    Ok(ResponseJson(SaplingShieldResponse {
        dry_run: false,
        broadcast: true,
        txid: Some(txid.clone()),
        shielded_value_zatoshis: Some(built.shielded_value_zatoshis),
        fee_zatoshis: built.fee_zatoshis,
        expiry_height: Some(built.expiry_height),
        candidate_notes,
        candidate_zatoshis,
        message: format!("Moved legacy funds into shielded balance. Broadcast {txid}."),
    }))
}
