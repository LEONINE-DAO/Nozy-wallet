//! Recover ZIP-302 memos from full shielded outputs.
//!
//! Lightwalletd compact blocks (and this wallet's compact scan) only include the
//! first 52 bytes of `encCiphertext`. The memo lives in the remaining ciphertext,
//! so history used to persist `memo: []` and the API returned `"memo": null`.

use crate::error::{NozyError, NozyResult};
use crate::hd_wallet::HDWallet;
use crate::key_management::{zeroize_bytes, SecureSeed};
#[cfg(feature = "native")]
use crate::notes::{load_wallet_notes, save_wallet_notes, SerializableOrchardNote};
use crate::shielded_pool::ShieldedPool;
#[cfg(feature = "native")]
use crate::zebra_integration::ZebraClient;
use orchard::keys::{FullViewingKey, IncomingViewingKey, PreparedIncomingViewingKey, SpendingKey};
use orchard::note::{ExtractedNoteCommitment, Nullifier};
use orchard::note_encryption::{CompactAction, IronwoodDomain, OrchardDomain};
use orchard::{Address, Note};
use zcash_note_encryption::{
    try_note_decryption, Domain, EphemeralKeyBytes, ShieldedOutput, ENC_CIPHERTEXT_SIZE,
};
use zcash_primitives::transaction::Transaction;
use zcash_protocol::consensus::BranchId;
use zcash_protocol::memo::Memo;
use zip32::AccountId;

#[cfg(feature = "native")]
const MAX_BACKFILL_TXS: usize = 32;

struct FullEncOutput {
    cmx: [u8; 32],
    epk: [u8; 32],
    enc: [u8; ENC_CIPHERTEXT_SIZE],
}

macro_rules! impl_full_enc_output {
    ($domain:ty) => {
        impl ShieldedOutput<$domain, ENC_CIPHERTEXT_SIZE> for FullEncOutput {
            fn ephemeral_key(&self) -> EphemeralKeyBytes {
                EphemeralKeyBytes(self.epk)
            }

            fn cmstar_bytes(&self) -> [u8; 32] {
                self.cmx
            }

            fn enc_ciphertext(&self) -> &[u8; ENC_CIPHERTEXT_SIZE] {
                &self.enc
            }
        }
    };
}

impl_full_enc_output!(OrchardDomain);
impl_full_enc_output!(IronwoodDomain);

struct ScanIvk {
    fvk: FullViewingKey,
    external_ivk: IncomingViewingKey,
    internal_ivk: IncomingViewingKey,
}

/// Bytes stored on a note: UTF-8 text, a single `0xF6` for a resolved empty memo,
/// or the raw 512-byte ZIP-302 encoding for non-text memos.
pub fn memo_plaintext_to_note_bytes(raw: &[u8]) -> Vec<u8> {
    if raw.is_empty() {
        return vec![0xF6];
    }
    if raw.len() == 512 {
        return match Memo::from_bytes(raw) {
            Ok(Memo::Empty) => vec![0xF6],
            Ok(Memo::Text(text)) => {
                let s = String::from(text);
                if s.is_empty() {
                    vec![0xF6]
                } else {
                    s.into_bytes()
                }
            }
            Ok(other) => other.encode().as_array().to_vec(),
            Err(_) => raw.to_vec(),
        };
    }
    match String::from_utf8(raw.to_vec()) {
        Ok(s) => {
            let trimmed = s.trim_end_matches('\0');
            if trimmed.is_empty() {
                vec![0xF6]
            } else {
                trimmed.as_bytes().to_vec()
            }
        }
        Err(_) => raw.to_vec(),
    }
}

/// Text shown by `/api/transaction/history`. Empty and unresolved memos are `None`.
pub fn memo_note_bytes_to_history_string(stored: &[u8]) -> Option<String> {
    if stored.is_empty() || stored == [0xF6] {
        return None;
    }
    if stored.len() == 512 {
        return match Memo::from_bytes(stored).ok()? {
            Memo::Empty => None,
            Memo::Text(text) => nonempty_memo_text(String::from(text)),
            Memo::Arbitrary(bytes) => {
                let end = bytes
                    .iter()
                    .rposition(|b| *b != 0)
                    .map(|i| i + 1)
                    .unwrap_or(0);
                let slice = &bytes[..end];
                if slice.is_empty() {
                    None
                } else if let Ok(text) = std::str::from_utf8(slice) {
                    nonempty_memo_text(text.to_string())
                } else {
                    Some(hex::encode(slice))
                }
            }
            Memo::Future(_) => None,
        };
    }
    let text = String::from_utf8(stored.to_vec()).ok()?;
    nonempty_memo_text(text.trim_end_matches('\0').to_string())
}

fn nonempty_memo_text(text: String) -> Option<String> {
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// Trial-decrypt a full 580-byte `encCiphertext` and return the note plus stored memo bytes.
pub fn try_decrypt_full_action(
    pool: ShieldedPool,
    ivk: &IncomingViewingKey,
    nullifier: &[u8; 32],
    cmx: &[u8; 32],
    ephemeral_key: &[u8; 32],
    enc_ciphertext: &[u8],
) -> Option<(Note, Address, Vec<u8>)> {
    if enc_ciphertext.len() < ENC_CIPHERTEXT_SIZE {
        return None;
    }
    let mut enc = [0u8; ENC_CIPHERTEXT_SIZE];
    enc.copy_from_slice(&enc_ciphertext[..ENC_CIPHERTEXT_SIZE]);
    if enc[52..].iter().all(|b| *b == 0) {
        return None;
    }

    let nullifier = Nullifier::from_bytes(nullifier).into_option()?;
    let cmx_point = ExtractedNoteCommitment::from_bytes(cmx).into_option()?;
    let mut compact_ct = [0u8; 52];
    compact_ct.copy_from_slice(&enc[..52]);
    let compact = CompactAction::from_parts(
        nullifier,
        cmx_point,
        EphemeralKeyBytes::from(*ephemeral_key),
        compact_ct,
    );
    let output = FullEncOutput {
        cmx: *cmx,
        epk: *ephemeral_key,
        enc,
    };
    let prepared = PreparedIncomingViewingKey::new(ivk);

    let decrypted = match pool {
        ShieldedPool::Orchard => {
            let domain = OrchardDomain::for_compact_action(&compact);
            trial_decrypt(&domain, &prepared, &output)
        }
        ShieldedPool::Ironwood => {
            let domain = IronwoodDomain::for_compact_action(&compact);
            trial_decrypt(&domain, &prepared, &output)
        }
    }?;
    Some((
        decrypted.0,
        decrypted.1,
        memo_plaintext_to_note_bytes(&decrypted.2),
    ))
}

fn trial_decrypt<D>(
    domain: &D,
    ivk: &D::IncomingViewingKey,
    output: &FullEncOutput,
) -> Option<(Note, Address, [u8; 512])>
where
    D: Domain<Note = Note, Recipient = Address, Memo = [u8; 512]>,
    FullEncOutput: ShieldedOutput<D, ENC_CIPHERTEXT_SIZE>,
{
    try_note_decryption(domain, ivk, output)
}

fn scan_ivks(wallet: &HDWallet) -> NozyResult<Vec<ScanIvk>> {
    let mut seed_bytes = wallet.get_mnemonic_object().to_seed("").to_vec();
    let secure_seed = SecureSeed::new(seed_bytes.clone());
    let mut ivks = Vec::new();
    // Same cap as the mobile and extension account list (indices 0 through 20).
    for account in 0u32..=20 {
        let account_id = AccountId::try_from(account)
            .map_err(|e| NozyError::KeyDerivation(format!("Invalid account ID: {e:?}")))?;
        let sk =
            SpendingKey::from_zip32_seed(secure_seed.as_bytes(), 133, account_id).map_err(|e| {
                NozyError::KeyDerivation(format!("Failed to derive Orchard spending key: {e:?}"))
            })?;
        let fvk = FullViewingKey::from(&sk);
        let external_ivk = fvk.to_ivk(orchard::keys::Scope::External);
        let internal_ivk = fvk.to_ivk(orchard::keys::Scope::Internal);
        ivks.push(ScanIvk {
            fvk,
            external_ivk,
            internal_ivk,
        });
    }
    zeroize_bytes(&mut seed_bytes);
    Ok(ivks)
}

/// One shielded output this wallet can decrypt from a raw transaction.
#[derive(Clone, Debug)]
pub struct DecodedOutputMemo {
    pub cmx: [u8; 32],
    pub canonical_nullifier: [u8; 32],
    pub rho: [u8; 32],
    pub value: u64,
    /// Stored note bytes. A single `0xF6` is a resolved empty memo.
    pub memo_stored: Vec<u8>,
    /// Display text. An empty string is a decrypted memo with no text.
    pub memo_text: String,
}

/// Trial-decrypt every Orchard and Ironwood output in `raw`.
///
/// Returns an empty list when the bytes are not a transaction. Callers that only
/// have compact ciphertext use this after `getrawtransaction`.
pub fn memos_from_raw_transaction(wallet: &HDWallet, raw: &[u8]) -> Vec<DecodedOutputMemo> {
    let Ok(tx) = Transaction::read(raw, BranchId::Nu6) else {
        return Vec::new();
    };
    let Ok(ivks) = scan_ivks(wallet) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    if let Some(bundle) = tx.orchard_bundle() {
        for action in bundle.actions().iter() {
            let enc = action.encrypted_note();
            push_decoded_output(
                ShieldedPool::Orchard,
                &ivks,
                (*action.nullifier()).to_bytes(),
                (*action.cmx()).to_bytes(),
                enc.epk_bytes,
                enc.enc_ciphertext,
                &mut out,
            );
        }
    }
    if let Some(bundle) = tx.ironwood_bundle() {
        for action in bundle.actions().iter() {
            let enc = action.encrypted_note();
            push_decoded_output(
                ShieldedPool::Ironwood,
                &ivks,
                (*action.nullifier()).to_bytes(),
                (*action.cmx()).to_bytes(),
                enc.epk_bytes,
                enc.enc_ciphertext,
                &mut out,
            );
        }
    }
    out
}

fn push_decoded_output(
    pool: ShieldedPool,
    ivks: &[ScanIvk],
    nullifier: [u8; 32],
    cmx: [u8; 32],
    epk: [u8; 32],
    enc: [u8; ENC_CIPHERTEXT_SIZE],
    out: &mut Vec<DecodedOutputMemo>,
) {
    if out.iter().any(|row| row.cmx == cmx) {
        return;
    }
    for keys in ivks {
        for ivk in [&keys.external_ivk, &keys.internal_ivk] {
            let Some((note, _address, memo)) =
                try_decrypt_full_action(pool, ivk, &nullifier, &cmx, &epk, &enc)
            else {
                continue;
            };
            out.push(DecodedOutputMemo {
                cmx,
                canonical_nullifier: note.nullifier(&keys.fvk).to_bytes(),
                rho: note.rho().to_bytes(),
                value: note.value().inner(),
                memo_text: memo_note_bytes_to_history_string(&memo).unwrap_or_default(),
                memo_stored: memo,
            });
            return;
        }
    }
}

#[cfg(feature = "native")]
fn txid_candidates(txid: &str) -> Vec<String> {
    let mut out = vec![txid.to_string()];
    if let Ok(mut raw) = hex::decode(txid) {
        if raw.len() == 32 {
            raw.reverse();
            let flipped = hex::encode(raw);
            if flipped != txid {
                out.push(flipped);
            }
        }
    }
    out
}

#[cfg(feature = "native")]
async fn load_raw_tx(zebra: &ZebraClient, txid: &str) -> NozyResult<Option<Vec<u8>>> {
    let mut saw_network = None;
    for candidate in txid_candidates(txid) {
        match zebra.get_raw_transaction(&candidate).await {
            Ok(hex_tx) => {
                let cleaned = hex_tx.trim().trim_start_matches("0x");
                if let Ok(bytes) = hex::decode(cleaned) {
                    if !bytes.is_empty() {
                        return Ok(Some(bytes));
                    }
                }
            }
            Err(NozyError::NetworkError(msg)) => saw_network = Some(msg),
            Err(_) => {}
        }
    }
    if let Some(msg) = saw_network {
        return Err(NozyError::NetworkError(msg));
    }
    Ok(None)
}

#[cfg(feature = "native")]
fn apply_output_memo(
    pool: ShieldedPool,
    ivks: &[ScanIvk],
    txid: &str,
    nullifier: [u8; 32],
    cmx: [u8; 32],
    epk: [u8; 32],
    enc: [u8; ENC_CIPHERTEXT_SIZE],
    notes: &mut [SerializableOrchardNote],
) -> usize {
    let mut updated = 0;
    for keys in ivks {
        for ivk in [&keys.external_ivk, &keys.internal_ivk] {
            let Some((note, _address, memo)) =
                try_decrypt_full_action(pool, ivk, &nullifier, &cmx, &epk, &enc)
            else {
                continue;
            };
            let canonical = note.nullifier(&keys.fvk).to_bytes();
            let rho = note.rho().to_bytes();
            let value = note.value().inner();
            for row in notes.iter_mut() {
                if row.txid != txid || !row.memo.is_empty() {
                    continue;
                }
                let nf_ok = row.nullifier_bytes.as_slice() == canonical.as_slice();
                let rho_ok = row.rho_bytes.as_deref() == Some(rho.as_slice()) && row.value == value;
                if nf_ok || rho_ok {
                    row.memo = memo.clone();
                    updated += 1;
                }
            }
        }
    }
    updated
}

/// Fill empty memos on `notes` by fetching each transaction and decrypting outputs.
///
/// Stops early if the node is unreachable. At most [`MAX_BACKFILL_TXS`] transactions
/// are fetched per call so history stays bounded.
#[cfg(feature = "native")]
pub async fn backfill_unresolved_memos(
    wallet: &HDWallet,
    zebra: &ZebraClient,
    notes: &mut [SerializableOrchardNote],
) -> NozyResult<usize> {
    let mut txids = Vec::new();
    for note in notes.iter() {
        if note.memo.is_empty() && !txids.iter().any(|id: &String| id == &note.txid) {
            txids.push(note.txid.clone());
        }
    }
    if txids.is_empty() {
        return Ok(0);
    }
    txids.truncate(MAX_BACKFILL_TXS);
    let ivks = scan_ivks(wallet)?;
    let mut updated = 0usize;
    for txid in txids {
        let raw = match load_raw_tx(zebra, &txid).await {
            Ok(Some(bytes)) => bytes,
            Ok(None) => continue,
            Err(e) => {
                tracing::debug!(error = %e, "stopped memo backfill; node unavailable");
                break;
            }
        };
        let tx = match Transaction::read(raw.as_slice(), BranchId::Nu6) {
            Ok(tx) => tx,
            Err(e) => {
                tracing::debug!(%txid, error = %e, "could not parse transaction for memo");
                continue;
            }
        };
        if let Some(bundle) = tx.orchard_bundle() {
            for action in bundle.actions().iter() {
                let enc = action.encrypted_note();
                updated += apply_output_memo(
                    ShieldedPool::Orchard,
                    &ivks,
                    &txid,
                    (*action.nullifier()).to_bytes(),
                    (*action.cmx()).to_bytes(),
                    enc.epk_bytes,
                    enc.enc_ciphertext,
                    notes,
                );
            }
        }
        if let Some(bundle) = tx.ironwood_bundle() {
            for action in bundle.actions().iter() {
                let enc = action.encrypted_note();
                updated += apply_output_memo(
                    ShieldedPool::Ironwood,
                    &ivks,
                    &txid,
                    (*action.nullifier()).to_bytes(),
                    (*action.cmx()).to_bytes(),
                    enc.epk_bytes,
                    enc.enc_ciphertext,
                    notes,
                );
            }
        }
    }
    Ok(updated)
}

/// Write decrypted memo bytes onto notes from this wallet that belong to `txid`.
#[cfg(feature = "native")]
pub fn apply_raw_transaction_memos(
    wallet: &HDWallet,
    txid: &str,
    raw: &[u8],
    notes: &mut [SerializableOrchardNote],
) -> usize {
    let decoded = memos_from_raw_transaction(wallet, raw);
    let mut updated = 0usize;
    for row in notes.iter_mut() {
        if row.txid != txid || !row.memo.is_empty() {
            continue;
        }
        let Some(hit) = decoded.iter().find(|item| {
            row.nullifier_bytes.as_slice() == item.canonical_nullifier.as_slice()
                || (row.rho_bytes.as_deref() == Some(item.rho.as_slice())
                    && row.value == item.value)
        }) else {
            continue;
        };
        row.memo = hit.memo_stored.clone();
        updated += 1;
    }
    updated
}

/// Fill empty memos by fetching full transactions from lightwalletd `GetTransaction`.
#[cfg(feature = "native")]
pub async fn backfill_unresolved_memos_from_lwd(
    wallet: &HDWallet,
    lwd_url: &str,
    notes: &mut [SerializableOrchardNote],
) -> NozyResult<usize> {
    let mut txids = Vec::new();
    for note in notes.iter() {
        if note.memo.is_empty() && !txids.iter().any(|id: &String| id == &note.txid) {
            txids.push(note.txid.clone());
        }
    }
    if txids.is_empty() {
        return Ok(0);
    }
    txids.truncate(MAX_BACKFILL_TXS);
    let mut client = zeaking::lwd::connect_lightwalletd(lwd_url.trim())
        .await
        .map_err(|e| NozyError::NetworkError(e.to_string()))?;
    let mut updated = 0usize;
    for txid in txids {
        let raw = match zeaking::lwd::get_transaction(&mut client, &txid).await {
            Ok(Some(bytes)) => bytes,
            Ok(None) => continue,
            Err(e) => {
                tracing::debug!(error = %e, "stopped memo backfill; lightwalletd unavailable");
                break;
            }
        };
        updated += apply_raw_transaction_memos(wallet, &txid, &raw, notes);
    }
    Ok(updated)
}

/// Load `notes.json`, recover missing memos from lightwalletd, and save when anything changed.
#[cfg(feature = "native")]
pub async fn backfill_and_save_from_lwd(wallet: &HDWallet, lwd_url: &str) -> NozyResult<usize> {
    let mut notes = load_wallet_notes()?;
    if !notes.iter().any(|n| n.memo.is_empty()) {
        return Ok(0);
    }
    let updated = backfill_unresolved_memos_from_lwd(wallet, lwd_url, &mut notes).await?;
    if updated > 0 {
        save_wallet_notes(&notes)?;
        tracing::info!(
            memos = updated,
            "filled shielded memos from lightwalletd transactions"
        );
    }
    Ok(updated)
}

/// Load `notes.json`, recover missing memos, and save when anything changed.
#[cfg(feature = "native")]
pub async fn backfill_and_save(wallet: &HDWallet, zebra: &ZebraClient) -> NozyResult<usize> {
    let mut notes = load_wallet_notes()?;
    if !notes.iter().any(|n| n.memo.is_empty()) {
        return Ok(0);
    }
    let updated = backfill_unresolved_memos(wallet, zebra, &mut notes).await?;
    if updated > 0 {
        save_wallet_notes(&notes)?;
        tracing::info!(
            memos = updated,
            "filled shielded memos from full transactions"
        );
    }
    Ok(updated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zip302_text_memo_round_trips_to_history_string() {
        let mut raw = [0u8; 512];
        let text = b"hello zodl";
        raw[..text.len()].copy_from_slice(text);
        let stored = memo_plaintext_to_note_bytes(&raw);
        assert_eq!(
            memo_note_bytes_to_history_string(&stored).as_deref(),
            Some("hello zodl")
        );
    }

    #[test]
    fn zip302_empty_memo_stays_null() {
        let mut raw = [0u8; 512];
        raw[0] = 0xF6;
        let stored = memo_plaintext_to_note_bytes(&raw);
        assert_eq!(stored, vec![0xF6]);
        assert!(memo_note_bytes_to_history_string(&stored).is_none());
    }

    #[test]
    fn plain_utf8_memo_trims_null_padding() {
        let stored = memo_plaintext_to_note_bytes(b"paid\0\0");
        assert_eq!(
            memo_note_bytes_to_history_string(&stored).as_deref(),
            Some("paid")
        );
    }

    #[test]
    fn raw_transaction_garbage_yields_no_memos() {
        let wallet = HDWallet::from_mnemonic(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        )
        .expect("mnemonic");
        assert!(memos_from_raw_transaction(&wallet, b"not a transaction").is_empty());
    }
}
