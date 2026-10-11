//! Gift cards anyone can buy with shielded ZEC and send to someone else.
//!
//! Buying pays a fresh Orchard address whose spending key is not part of the
//! buyer's seed. The claim code is that key plus the block height to scan from.
//! The recipient redeems it in their own Nozy whenever they choose; the sweep
//! pays their shielded address, and the network fee comes out of the card.
//! The code is shown once and is not written to disk. Funds never sit on the public API.

use crate::error::{NozyError, NozyResult};
use crate::hd_wallet::HDWallet;
use crate::notes::SerializableOrchardNote;
use crate::paths::get_wallet_data_dir;
use crate::zip321::build_payment_uri;
use chrono::Utc;
use orchard::keys::{DiversifierIndex, FullViewingKey, Scope, SpendingKey};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use zeroize::Zeroize;

/// First diversifier used for gift cards. Invoice diversifiers start at 1, so the
/// two features do not share a receive address on the Business account.
const DIVERSIFIER_BASE: u32 = 1_000_000;

const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// While set, note selection may spend only this backing nullifier.
static REDEEM_NULLIFIER: Mutex<Option<String>> = Mutex::new(None);
static STORE_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GiftCardStatus {
    Unfunded,
    Funded,
    Redeeming,
    Redeemed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredGiftCard {
    gift_id: String,
    status: GiftCardStatus,
    payment_address: String,
    amount_zatoshis: u64,
    amount_zec: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    diversifier_index: u32,
    account: u32,
    created_at: String,
    zcash_uri: String,
    code_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    backing_nullifier: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    funding_txid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    redeem_txid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    recipient: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    payout_zatoshis: Option<u64>,
}

/// Gift card metadata. The claim code is not stored and is not included here.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GiftCard {
    pub gift_id: String,
    pub status: GiftCardStatus,
    pub payment_address: String,
    pub amount_zatoshis: u64,
    pub amount_zec: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub diversifier_index: u32,
    pub account: u32,
    pub created_at: String,
    pub zcash_uri: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backing_nullifier: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub funding_txid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redeem_txid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipient: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payout_zatoshis: Option<u64>,
}

/// Create result. `claim_code` is returned once and is not written to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiftCardCreated {
    #[serde(flatten)]
    pub card: GiftCard,
    pub claim_code: String,
}

#[derive(Debug, Clone)]
pub struct CreateGiftCardParams {
    pub amount_zec: f64,
    pub label: Option<String>,
    pub payment_address: String,
    pub diversifier_index: u32,
    pub account: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct GiftStore {
    next_diversifier: u32,
    cards: Vec<StoredGiftCard>,
}

fn store_guard() -> std::sync::MutexGuard<'static, ()> {
    STORE_LOCK.lock().unwrap_or_else(|p| p.into_inner())
}

impl GiftStore {
    fn path() -> PathBuf {
        get_wallet_data_dir().join("gift_cards.json")
    }

    fn load() -> NozyResult<Self> {
        let path = Self::path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(&path)
            .map_err(|e| NozyError::Storage(format!("Failed to read gift cards: {e}")))?;
        serde_json::from_str(&raw)
            .map_err(|e| NozyError::Storage(format!("Corrupt gift card store: {e}")))
    }

    fn save(&self) -> NozyResult<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| NozyError::Storage(format!("Failed to create gift card dir: {e}")))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| NozyError::Storage(format!("Failed to serialize gift cards: {e}")))?;
        fs::write(&path, json)
            .map_err(|e| NozyError::Storage(format!("Failed to write gift cards: {e}")))?;
        Ok(())
    }
}

impl StoredGiftCard {
    fn to_public(&self) -> GiftCard {
        GiftCard {
            gift_id: self.gift_id.clone(),
            status: self.status,
            payment_address: self.payment_address.clone(),
            amount_zatoshis: self.amount_zatoshis,
            amount_zec: self.amount_zec,
            label: self.label.clone(),
            diversifier_index: self.diversifier_index,
            account: self.account,
            created_at: self.created_at.clone(),
            zcash_uri: self.zcash_uri.clone(),
            backing_nullifier: self.backing_nullifier.clone(),
            funding_txid: self.funding_txid.clone(),
            redeem_txid: self.redeem_txid.clone(),
            recipient: self.recipient.clone(),
            payout_zatoshis: self.payout_zatoshis,
        }
    }
}

/// Next diversifier index for a gift-card receive address (not persisted until create).
pub fn peek_next_diversifier() -> NozyResult<u32> {
    let _guard = store_guard();
    let store = GiftStore::load()?;
    Ok(DIVERSIFIER_BASE.saturating_add(store.next_diversifier))
}

pub fn create_gift_card(params: CreateGiftCardParams) -> NozyResult<GiftCardCreated> {
    let addr = params.payment_address.trim();
    if addr.starts_with('t') {
        return Err(NozyError::InvalidInput(
            "transparent ZEC addresses are not supported for gift cards; use a shielded UA (u1…)"
                .into(),
        ));
    }
    if !(params.amount_zec.is_finite() && params.amount_zec > 0.0) {
        return Err(NozyError::InvalidInput(
            "amount_zec must be a positive finite value".into(),
        ));
    }
    let amount_zatoshis = (params.amount_zec * 100_000_000.0).round() as u64;
    if amount_zatoshis == 0 {
        return Err(NozyError::InvalidInput(
            "amount_zec is too small (rounds to 0 zats)".into(),
        ));
    }
    let label = normalize_label(params.label)?;

    let mut raw = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut raw);
    let body = encode_crockford(&raw);
    raw.zeroize();
    let claim_code = format_claim_code(&body);
    let code_hash = hash_claim_code(&claim_code);

    let _guard = store_guard();
    let mut store = GiftStore::load()?;
    let now = Utc::now();
    let gift_id = format!("gift_{}", now.timestamp_millis());
    let memo = format!("gift:{gift_id}");
    let zcash_uri = build_payment_uri(addr, Some(params.amount_zec), Some(&memo))?;

    let relative = params.diversifier_index.saturating_sub(DIVERSIFIER_BASE);
    if relative >= store.next_diversifier {
        store.next_diversifier = relative.saturating_add(1);
    }

    let stored = StoredGiftCard {
        gift_id,
        status: GiftCardStatus::Unfunded,
        payment_address: params.payment_address,
        amount_zatoshis,
        amount_zec: params.amount_zec,
        label,
        diversifier_index: params.diversifier_index,
        account: params.account,
        created_at: now.to_rfc3339(),
        zcash_uri,
        code_hash,
        backing_nullifier: None,
        funding_txid: None,
        redeem_txid: None,
        recipient: None,
        payout_zatoshis: None,
    };
    let card = stored.to_public();
    store.cards.push(stored);
    store.save()?;
    Ok(GiftCardCreated { card, claim_code })
}

pub fn get_gift_card(id: &str) -> NozyResult<Option<GiftCard>> {
    let _guard = store_guard();
    let store = GiftStore::load()?;
    Ok(store
        .cards
        .iter()
        .find(|c| c.gift_id == id)
        .map(StoredGiftCard::to_public))
}

pub fn list_gift_cards(limit: usize) -> NozyResult<Vec<GiftCard>> {
    let _guard = store_guard();
    let store = GiftStore::load()?;
    let mut items: Vec<GiftCard> = store.cards.iter().map(StoredGiftCard::to_public).collect();
    items.reverse();
    items.truncate(limit.max(1));
    Ok(items)
}

pub fn cancel_gift_card(id: &str) -> NozyResult<GiftCard> {
    let _guard = store_guard();
    let mut store = GiftStore::load()?;
    let card = store
        .cards
        .iter_mut()
        .find(|c| c.gift_id == id)
        .ok_or_else(|| NozyError::InvalidOperation(format!("Gift card {id} not found")))?;
    match card.status {
        GiftCardStatus::Unfunded => {
            card.status = GiftCardStatus::Cancelled;
            card.backing_nullifier = None;
        }
        GiftCardStatus::Funded if card.backing_nullifier.is_some() => {
            card.status = GiftCardStatus::Cancelled;
            card.backing_nullifier = None;
        }
        GiftCardStatus::Funded => {
            return Err(NozyError::InvalidOperation(
                "This gift card was already paid. Redeem the claim code to spend it. Cancelling this record does not return the ZEC.".into(),
            ));
        }
        other => {
            return Err(NozyError::InvalidOperation(format!(
                "Cannot cancel a gift card that is {other:?}"
            )));
        }
    }
    let out = card.to_public();
    store.save()?;
    Ok(out)
}

/// After sync, lock the note that paid an unfunded card (address + exact amount).
pub fn match_notes_after_sync(notes: &[SerializableOrchardNote]) -> NozyResult<usize> {
    let _guard = store_guard();
    let mut store = GiftStore::load()?;
    let mut matched = 0usize;
    for card in &mut store.cards {
        if card.status != GiftCardStatus::Unfunded {
            continue;
        }
        let Some(raw) = orchard_raw_from_ua(&card.payment_address) else {
            continue;
        };
        let Some(note) = notes.iter().find(|n| {
            !n.spent
                && n.value == card.amount_zatoshis
                && n.address_bytes.as_slice() == raw.as_slice()
        }) else {
            continue;
        };
        card.status = GiftCardStatus::Funded;
        card.backing_nullifier = Some(hex::encode(&note.nullifier_bytes));
        card.funding_txid = Some(note.txid.clone());
        matched += 1;
    }
    if matched > 0 {
        store.save()?;
    }
    Ok(matched)
}

/// Snapshot used by note selection. Redeem arms an allowance so only the backing
/// note can be spent for that payout; otherwise funded and in-progress cards are locked.
pub struct SpendSelection {
    allow_only: Option<String>,
    reserved: Vec<String>,
}

impl SpendSelection {
    pub fn allows(&self, nullifier_hex: &str) -> bool {
        let hex_norm = nullifier_hex.trim().to_ascii_lowercase();
        if let Some(expected) = &self.allow_only {
            return expected == &hex_norm;
        }
        !self
            .reserved
            .iter()
            .any(|n| n.eq_ignore_ascii_case(&hex_norm))
    }
}

pub fn spend_selection() -> SpendSelection {
    let allow_only = {
        let slot = REDEEM_NULLIFIER.lock().unwrap_or_else(|p| p.into_inner());
        slot.clone()
    };
    if allow_only.is_some() {
        return SpendSelection {
            allow_only,
            reserved: Vec::new(),
        };
    }
    let Ok(_guard) = STORE_LOCK.lock() else {
        return SpendSelection {
            allow_only: None,
            reserved: Vec::new(),
        };
    };
    let Ok(store) = GiftStore::load() else {
        return SpendSelection {
            allow_only: None,
            reserved: Vec::new(),
        };
    };
    let reserved = store
        .cards
        .iter()
        .filter(|c| matches!(c.status, GiftCardStatus::Funded | GiftCardStatus::Redeeming))
        .filter_map(|c| c.backing_nullifier.clone())
        .collect();
    SpendSelection {
        allow_only: None,
        reserved,
    }
}

/// True when a normal send may select this note.
pub fn nullifier_spendable_for_selection(nullifier_hex: &str) -> bool {
    spend_selection().allows(nullifier_hex)
}

/// Pay `amount_zec` in shielded ZEC to a new gift-card address and return the claim code.
///
/// The buyer sends the code to the recipient. Redeem does not need this wallet to be online.
pub async fn buy_gift_card(
    wallet: &HDWallet,
    amount_zec: f64,
    label: Option<String>,
    zebra_url: &str,
) -> NozyResult<GiftCardCreated> {
    if !(amount_zec.is_finite() && amount_zec > 0.0) {
        return Err(NozyError::InvalidInput(
            "amount_zec must be a positive finite value".into(),
        ));
    }
    let amount_zatoshis = (amount_zec * 100_000_000.0).round() as u64;
    if amount_zatoshis == 0 {
        return Err(NozyError::InvalidInput(
            "amount_zec is too small (rounds to 0 zats)".into(),
        ));
    }
    let label = normalize_label(label)?;
    let network = gift_network();
    let (spending_key, mut raw_key) = ephemeral_spending_key()?;
    let payment_address = gift_unified_address(&spending_key, network)?;

    let spendable = match crate::cli_helpers::scan_notes_for_sending(wallet, zebra_url).await {
        Ok(notes) => notes,
        Err(e) => {
            raw_key.zeroize();
            return Err(e);
        }
    };
    let config = crate::config::load_config();
    let zebra =
        crate::zebra_integration::ZebraClient::from_config_with_url(&config, Some(zebra_url));
    let redeem_fee = crate::fee_policy::estimate_orchard_send_fee_zatoshis(None, true);
    if amount_zatoshis <= redeem_fee {
        raw_key.zeroize();
        let fee_zec = redeem_fee as f64 / 100_000_000.0;
        return Err(NozyError::InvalidInput(format!(
            "Gift card must be larger than the redeem fee ({fee_zec:.8} ZEC). That fee comes out of the card."
        )));
    }
    let memo = b"Nozy gift".as_slice();
    let send_result = crate::cli_helpers::build_and_broadcast_transaction(
        &zebra,
        &spendable,
        &payment_address,
        amount_zatoshis,
        None,
        Some(memo),
        true,
        zebra_url,
        crate::fee_policy::PilotSendOptions::for_send(),
    )
    .await;
    let txid = match send_result {
        Ok(txid) => txid,
        Err(e) => {
            raw_key.zeroize();
            return Err(e);
        }
    };
    let height = match zebra.get_best_block_height().await {
        Ok(height) => height,
        Err(_) => config.last_scan_height.unwrap_or(0),
    };
    let claim_code = encode_claim_code(&raw_key, height);
    raw_key.zeroize();

    let card = record_bought_card(BoughtGift {
        payment_address,
        amount_zatoshis,
        amount_zec,
        label,
        claim_code: claim_code.clone(),
        funding_txid: txid,
    })?;
    Ok(GiftCardCreated { card, claim_code })
}

struct BoughtGift {
    payment_address: String,
    amount_zatoshis: u64,
    amount_zec: f64,
    label: Option<String>,
    claim_code: String,
    funding_txid: String,
}

fn record_bought_card(bought: BoughtGift) -> NozyResult<GiftCard> {
    let _guard = store_guard();
    let mut store = GiftStore::load()?;
    let now = Utc::now();
    let stored = StoredGiftCard {
        gift_id: format!("gift_{}", now.timestamp_millis()),
        status: GiftCardStatus::Funded,
        payment_address: bought.payment_address,
        amount_zatoshis: bought.amount_zatoshis,
        amount_zec: bought.amount_zec,
        label: bought.label,
        diversifier_index: 0,
        account: 0,
        created_at: now.to_rfc3339(),
        zcash_uri: String::new(),
        code_hash: hash_claim_code(&bought.claim_code),
        backing_nullifier: None,
        funding_txid: Some(bought.funding_txid),
        redeem_txid: None,
        recipient: None,
        payout_zatoshis: None,
    };
    let card = stored.to_public();
    store.cards.push(stored);
    store.save()?;
    Ok(card)
}

/// Redeem a claim code into this wallet. The buyer does not have to be online.
pub async fn redeem_gift_card(
    wallet: &HDWallet,
    claim_code: &str,
    zebra_url: &str,
) -> NozyResult<GiftCard> {
    let (mut raw_key, height) = decode_claim_code(claim_code)?;
    let parsed = SpendingKey::from_bytes(raw_key);
    raw_key.zeroize();
    if !bool::from(parsed.is_some()) {
        return Err(NozyError::InvalidInput(
            "Gift card code was not recognized.".into(),
        ));
    }
    let spending_key = parsed.unwrap();
    let network = gift_network();
    let payment_address = gift_unified_address(&spending_key, network)?;
    let config = crate::config::load_config();
    let zebra =
        crate::zebra_integration::ZebraClient::from_config_with_url(&config, Some(zebra_url));
    let tip = zebra.get_best_block_height().await?;
    if height > tip {
        return Err(NozyError::InvalidOperation(
            "This gift card is not confirmed yet. Redeem it again after the payment is mined."
                .into(),
        ));
    }

    let scanner = crate::notes::NoteScanner::new(wallet, zebra.clone());
    let found = scanner
        .scan_for_spending_key(&spending_key, height, tip)
        .await?;
    let Some(note) = found.spendable.iter().max_by_key(|n| n.orchard_note.value) else {
        if found.spent_notes > 0 {
            return Err(NozyError::InvalidOperation(
                "This gift card has already been redeemed.".into(),
            ));
        }
        return Err(NozyError::InvalidOperation(
            "This gift card is not confirmed yet. Redeem it again after the payment is mined."
                .into(),
        ));
    };

    let fee = crate::fee_policy::estimate_orchard_send_fee_zatoshis(None, true);
    if note.orchard_note.value <= fee {
        return Err(NozyError::InvalidOperation(
            "This gift card is smaller than the network fee, so it cannot be redeemed.".into(),
        ));
    }
    let payout = note.orchard_note.value - fee;
    let account = config.active_orchard_account();
    let recipient = wallet.generate_orchard_address(account, 0, network)?;
    let txid = crate::cli_helpers::build_and_broadcast_transaction(
        &zebra,
        std::slice::from_ref(note),
        &recipient,
        payout,
        Some(fee),
        None,
        true,
        zebra_url,
        crate::fee_policy::PilotSendOptions::for_send(),
    )
    .await?;

    if let Some(card) = mark_redeemed_if_known(claim_code, &txid, payout)? {
        return Ok(card);
    }
    Ok(GiftCard {
        gift_id: format!("gift_redeemed_{}", Utc::now().timestamp_millis()),
        status: GiftCardStatus::Redeemed,
        payment_address,
        amount_zatoshis: note.orchard_note.value,
        amount_zec: note.orchard_note.value as f64 / 100_000_000.0,
        label: None,
        diversifier_index: 0,
        account,
        created_at: Utc::now().to_rfc3339(),
        zcash_uri: String::new(),
        backing_nullifier: None,
        funding_txid: None,
        redeem_txid: Some(txid),
        recipient: Some(recipient),
        payout_zatoshis: Some(payout),
    })
}

fn mark_redeemed_if_known(
    claim_code: &str,
    txid: &str,
    payout: u64,
) -> NozyResult<Option<GiftCard>> {
    let want = hash_claim_code(claim_code);
    let _guard = store_guard();
    let mut store = GiftStore::load()?;
    let Some(card) = store
        .cards
        .iter_mut()
        .find(|c| hashes_eq(&c.code_hash, &want))
    else {
        return Ok(None);
    };
    card.status = GiftCardStatus::Redeemed;
    card.redeem_txid = Some(txid.to_string());
    card.payout_zatoshis = Some(payout);
    card.backing_nullifier = None;
    let out = card.to_public();
    store.save()?;
    Ok(Some(out))
}

fn normalize_label(label: Option<String>) -> NozyResult<Option<String>> {
    let Some(label) = label else {
        return Ok(None);
    };
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().count() > 80 {
        return Err(NozyError::InvalidInput(
            "Gift card label must be 80 characters or fewer".into(),
        ));
    }
    Ok(Some(trimmed.to_string()))
}

fn encode_crockford(data: &[u8]) -> String {
    let mut bits: u32 = 0;
    let mut nbits: u32 = 0;
    let mut out = String::new();
    for &byte in data {
        bits = (bits << 8) | u32::from(byte);
        nbits += 8;
        while nbits >= 5 {
            nbits -= 5;
            let idx = ((bits >> nbits) & 31) as usize;
            out.push(CROCKFORD[idx] as char);
        }
    }
    if nbits > 0 {
        let idx = ((bits << (5 - nbits)) & 31) as usize;
        out.push(CROCKFORD[idx] as char);
    }
    out
}

fn gift_network() -> zcash_protocol::consensus::NetworkType {
    if crate::config::load_config().network == "testnet" {
        zcash_protocol::consensus::NetworkType::Test
    } else {
        zcash_protocol::consensus::NetworkType::Main
    }
}

fn ephemeral_spending_key() -> NozyResult<(SpendingKey, [u8; 32])> {
    for _ in 0..64 {
        let mut bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        let parsed = SpendingKey::from_bytes(bytes);
        if bool::from(parsed.is_some()) {
            return Ok((parsed.unwrap(), bytes));
        }
        bytes.zeroize();
    }
    Err(NozyError::KeyDerivation(
        "Could not generate a gift-card spending key".into(),
    ))
}

fn gift_unified_address(
    spending_key: &SpendingKey,
    network: zcash_protocol::consensus::NetworkType,
) -> NozyResult<String> {
    use zcash_address::unified::{Address as UnifiedAddress, Encoding, Receiver};
    let fvk = FullViewingKey::from(spending_key);
    let orchard_address = fvk.address_at(DiversifierIndex::from(0u32), Scope::External);
    let ua = UnifiedAddress::try_from_items(vec![Receiver::Orchard(
        orchard_address.to_raw_address_bytes(),
    )])
    .map_err(|e| {
        NozyError::InvalidOperation(format!("Failed to build gift-card address: {e:?}"))
    })?;
    Ok(ua.encode(&network))
}

/// Claim code payload: version, spending key, scan-from height, checksum.
fn encode_claim_code(spending_key: &[u8; 32], height: u32) -> String {
    let mut body = Vec::with_capacity(37);
    body.push(1u8);
    body.extend_from_slice(spending_key);
    body.extend_from_slice(&height.to_be_bytes());
    let mut hasher = Sha256::new();
    hasher.update(&body);
    let digest = hasher.finalize();
    body.extend_from_slice(&digest[..4]);
    format_claim_code(&encode_crockford(&body))
}

fn decode_claim_code(code: &str) -> NozyResult<([u8; 32], u32)> {
    let body = decode_crockford(&normalize_claim_code(code))
        .map_err(|_| NozyError::InvalidInput("Gift card code was not recognized.".into()))?;
    if body.len() != 41 || body[0] != 1 {
        return Err(NozyError::InvalidInput(
            "Gift card code was not recognized.".into(),
        ));
    }
    let mut hasher = Sha256::new();
    hasher.update(&body[..37]);
    let digest = hasher.finalize();
    if !hashes_eq(&hex::encode(&digest[..4]), &hex::encode(&body[37..41])) {
        return Err(NozyError::InvalidInput(
            "Gift card code was not recognized.".into(),
        ));
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&body[1..33]);
    let mut height_bytes = [0u8; 4];
    height_bytes.copy_from_slice(&body[33..37]);
    Ok((key, u32::from_be_bytes(height_bytes)))
}

fn decode_crockford(text: &str) -> Result<Vec<u8>, ()> {
    let mut bits: u32 = 0;
    let mut nbits: u32 = 0;
    let mut out = Vec::new();
    for ch in text.chars() {
        let idx = crockford_value(ch).ok_or(())?;
        bits = (bits << 5) | u32::from(idx);
        nbits += 5;
        if nbits >= 8 {
            nbits -= 8;
            out.push((bits >> nbits) as u8);
            bits &= (1u32 << nbits).saturating_sub(1);
        }
    }
    Ok(out)
}

fn crockford_value(ch: char) -> Option<u8> {
    let ch = match ch {
        'I' | 'L' => '1',
        'O' => '0',
        other => other,
    };
    CROCKFORD
        .iter()
        .position(|c| *c as char == ch)
        .map(|i| i as u8)
}

fn format_claim_code(body: &str) -> String {
    let chars: Vec<char> = body.chars().collect();
    let mut parts = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let end = (i + 4).min(chars.len());
        parts.push(chars[i..end].iter().collect::<String>());
        i = end;
    }
    format!("NZGC-{}", parts.join("-"))
}

fn normalize_claim_code(code: &str) -> String {
    let mut s: String = code.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    s.make_ascii_uppercase();
    s.strip_prefix("NZGC").unwrap_or(&s).to_string()
}

fn hash_claim_code(code: &str) -> String {
    let norm = normalize_claim_code(code);
    let mut hasher = Sha256::new();
    hasher.update(norm.as_bytes());
    hex::encode(hasher.finalize())
}

fn hashes_eq(a: &str, b: &str) -> bool {
    let ab = a.as_bytes();
    let bb = b.as_bytes();
    if ab.len() != bb.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in ab.iter().zip(bb.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn orchard_raw_from_ua(payment_address: &str) -> Option<[u8; 43]> {
    use zcash_address::unified::{Address as UnifiedAddress, Container, Encoding, Receiver};
    let (_net, ua) = UnifiedAddress::decode(payment_address.trim()).ok()?;
    for receiver in ua.items() {
        if let Receiver::Orchard(raw) = receiver {
            return Some(raw);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::with_wallet_data_dir;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(label: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("nozy_gift_{label}_{nanos}"))
    }

    #[test]
    fn claim_code_roundtrip_hides_the_key_on_disk() {
        let (sk, raw) = ephemeral_spending_key().unwrap();
        let code = encode_claim_code(&raw, 3_000_000);
        let (decoded, height) = decode_claim_code(&code).unwrap();
        assert_eq!(decoded, raw);
        assert_eq!(height, 3_000_000);
        assert!(code.starts_with("NZGC-"));
        let _ = sk;

        let mut broken: Vec<char> = code.chars().collect();
        // The last Crockford symbol carries unused low bits, so changing only
        // those bits still decodes. Corrupt the first payload symbol instead.
        let flip = "NZGC-".chars().count();
        broken[flip] = if broken[flip] == 'A' { 'B' } else { 'A' };
        let err = decode_claim_code(&broken.into_iter().collect::<String>()).unwrap_err();
        assert!(err.to_string().contains("not recognized"));

        let dir = temp_dir("disk");
        let _ = fs::create_dir_all(&dir);
        with_wallet_data_dir(&dir, || {
            let address =
                gift_unified_address(&sk, zcash_protocol::consensus::NetworkType::Main).unwrap();
            record_bought_card(BoughtGift {
                payment_address: address,
                amount_zatoshis: 5_000_000,
                amount_zec: 0.05,
                label: Some("For Sam".into()),
                claim_code: code.clone(),
                funding_txid: "abc".into(),
            })
            .unwrap();
            let raw_file = fs::read_to_string(dir.join("gift_cards.json")).unwrap();
            assert!(!raw_file.contains(&code));
            assert!(!raw_file.contains(&hex::encode(raw)));
        });
        let _ = fs::remove_dir_all(&dir);
    }
}
