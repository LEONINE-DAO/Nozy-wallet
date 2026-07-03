//! Keystone hardware wallet integration (air-gapped PCZT signing via BC-UR QR).
//!
//! NozyWallet supports Keystone as its sole hardware wallet. The hot wallet builds
//! a proved PCZT, redacts local metadata, and encodes it as `zcash-pczt` UR frames
//! for the device. Keystone signs and returns a signed PCZT; Nozy extracts and broadcasts.

use crate::error::{NozyError, NozyResult};
use crate::fee_policy::PilotSendOptions;
use crate::hd_wallet::HDWallet;
use crate::notes::SpendableNote;
use crate::orchard_tx::{select_single_spend_note, OrchardWitnessProvider};
use crate::paths::get_wallet_data_dir;
use crate::zebra_integration::ZebraClient;
use orchard::keys::{FullViewingKey, Scope, SpendAuthorizingKey, SpendingKey};
use pczt::roles::{
    creator::Creator, io_finalizer::IoFinalizer, prover::Prover, signer::Signer,
    tx_extractor::TransactionExtractor,
};
use pczt::Pczt;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use zcash_address::unified::{Container, Encoding};
use zcash_keys::keys::UnifiedFullViewingKey;
use zcash_primitives::transaction::builder::{BuildConfig, Builder};
use zcash_primitives::transaction::fees::{transparent::InputSize, FeeRule};
use zcash_protocol::consensus::{BlockHeight, MainNetwork, NetworkType, Parameters, TestNetwork};
use zcash_protocol::memo::MemoBytes;
use zcash_protocol::value::Zatoshis;

pub const UR_TYPE_ZCASH_PCZT: &str = "zcash-pczt";
pub const DEFAULT_UR_FRAGMENT_SIZE: usize = 200;

static ORCHARD_PROVING_KEY: OnceLock<orchard::circuit::ProvingKey> = OnceLock::new();

fn orchard_proving_key() -> &'static orchard::circuit::ProvingKey {
    ORCHARD_PROVING_KEY.get_or_init(orchard::circuit::ProvingKey::build)
}

/// Keystone pairing / custody settings persisted in `config.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystoneWalletConfig {
    /// When true, sends use Keystone PCZT signing (no local spend authorization).
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub device_label: Option<String>,
    /// Orchard UFVK (ZIP-316) for watch-only pairing with Keystone.
    #[serde(default)]
    pub ufvk: Option<String>,
}

impl Default for KeystoneWalletConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            device_label: None,
            ufvk: None,
        }
    }
}

impl KeystoneWalletConfig {
    pub fn is_active(&self) -> bool {
        self.enabled
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystonePreparedSend {
    pub recipient: String,
    pub amount_zatoshis: u64,
    pub fee_zatoshis: u64,
    pub summary: String,
    pub action_count: u32,
    pub pczt_hex: String,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct KeystonePcztBuild {
    pub pczt_bytes: Vec<u8>,
    pub summary: String,
    pub action_count: u32,
}

#[derive(Debug, Clone)]
pub struct KeystoneExtractedTx {
    pub raw_transaction: Vec<u8>,
    pub txid: String,
}

struct FixedFeeRule {
    fee: Zatoshis,
}

impl FeeRule for FixedFeeRule {
    type Error = core::convert::Infallible;

    fn fee_required<P: Parameters>(
        &self,
        _params: &P,
        _target_height: BlockHeight,
        _transparent_input_sizes: impl IntoIterator<Item = InputSize>,
        _transparent_output_sizes: impl IntoIterator<Item = usize>,
        _sapling_input_count: usize,
        _sapling_output_count: usize,
        _orchard_action_count: usize,
    ) -> Result<Zatoshis, Self::Error> {
        Ok(self.fee)
    }
}

/// Strip local-wallet metadata before sending PCZT to Keystone (matches zafu/zcli redaction).
pub fn redact_pczt_for_signer(pczt: Pczt) -> Pczt {
    pczt::roles::redactor::Redactor::new(pczt)
        .redact_global_with(|mut g| {
            g.clear_proprietary();
        })
        .redact_orchard_with(|mut o| {
            o.redact_actions(|mut a| {
                a.clear_spend_witness();
                a.clear_spend_zip32_derivation();
                a.clear_spend_dummy_sk();
                a.clear_spend_proprietary();
                a.clear_output_zip32_derivation();
                a.clear_output_user_address();
                a.clear_output_proprietary();
            });
        })
        .redact_transparent_with(|mut t| {
            t.redact_outputs(|mut o| {
                o.clear_user_address();
                o.clear_proprietary();
            });
        })
        .finish()
}

fn orchard_fvk_for_send(
    _wallet: &HDWallet,
    keystone: &KeystoneWalletConfig,
    spendable: &SpendableNote,
) -> NozyResult<FullViewingKey> {
    if let Some(ufvk_str) = keystone.ufvk.as_deref().filter(|s| !s.is_empty()) {
        let is_mainnet = ufvk_str.starts_with('u') && !ufvk_str.starts_with("utest");
        let fvk = if is_mainnet {
            UnifiedFullViewingKey::decode(&MainNetwork, ufvk_str)
        } else {
            UnifiedFullViewingKey::decode(&TestNetwork, ufvk_str)
        }
        .map_err(|e| NozyError::InvalidOperation(format!("Invalid Keystone UFVK: {e}")))?;
        return fvk.orchard().cloned().ok_or_else(|| {
            NozyError::InvalidOperation("Keystone UFVK has no Orchard component".to_string())
        });
    }
    Ok(FullViewingKey::from(&spendable.spending_key))
}

/// Export the wallet's Orchard UFVK (ZIP-316) for Keystone pairing.
pub fn export_ufvk_from_wallet(wallet: &HDWallet, network: NetworkType) -> NozyResult<String> {
    use zcash_keys::keys::UnifiedSpendingKey;
    use zip32::AccountId;

    let seed_bytes = wallet.get_mnemonic_object().to_seed("").to_vec();
    let account = AccountId::try_from(0)
        .map_err(|e| NozyError::KeyDerivation(format!("Invalid account ID: {e:?}")))?;

    match network {
        NetworkType::Main => {
            let usk =
                UnifiedSpendingKey::from_seed(&MainNetwork, &seed_bytes, account).map_err(|e| {
                    NozyError::KeyDerivation(format!(
                        "Failed to derive unified spending key: {e:?}"
                    ))
                })?;
            Ok(usk.to_unified_full_viewing_key().encode(&MainNetwork))
        }
        NetworkType::Test | NetworkType::Regtest => {
            let usk =
                UnifiedSpendingKey::from_seed(&TestNetwork, &seed_bytes, account).map_err(|e| {
                    NozyError::KeyDerivation(format!(
                        "Failed to derive unified spending key: {e:?}"
                    ))
                })?;
            Ok(usk.to_unified_full_viewing_key().encode(&TestNetwork))
        }
    }
}

/// Validate and store a Keystone-imported UFVK.
pub fn validate_ufvk(ufvk: &str, network: NetworkType) -> NozyResult<()> {
    let ufvk = match network {
        NetworkType::Main => UnifiedFullViewingKey::decode(&MainNetwork, ufvk),
        NetworkType::Test | NetworkType::Regtest => {
            UnifiedFullViewingKey::decode(&TestNetwork, ufvk)
        }
    }
    .map_err(|e| NozyError::InvalidOperation(format!("Invalid UFVK: {e}")))?;
    if ufvk.orchard().is_none() {
        return Err(NozyError::InvalidOperation(
            "UFVK must include an Orchard receiver for NozyWallet".to_string(),
        ));
    }
    Ok(())
}

/// Build a proved, IO-finalized PCZT ready for Keystone signing.
pub async fn build_keystone_send_pczt(
    zebra: &ZebraClient,
    witness_provider: &dyn OrchardWitnessProvider,
    wallet: &HDWallet,
    keystone: &KeystoneWalletConfig,
    spendable_notes: &[SpendableNote],
    recipient_address: &str,
    amount_zatoshis: u64,
    fee_zatoshis: u64,
    memo: Option<&[u8]>,
    pilot: PilotSendOptions,
    network: NetworkType,
) -> NozyResult<KeystonePcztBuild> {
    let spendable = select_single_spend_note(spendable_notes, amount_zatoshis, fee_zatoshis)?;
    let fvk = orchard_fvk_for_send(wallet, keystone, spendable)?;

    let (_, decoded) = zcash_address::unified::Address::decode(recipient_address)
        .map_err(|e| NozyError::InvalidOperation(format!("Invalid recipient address: {e}")))?;

    let mut orchard_receiver = None;
    for item in decoded.items() {
        if let zcash_address::unified::Receiver::Orchard(data) = item {
            orchard_receiver = Some(data);
            break;
        }
    }
    let orchard_raw = orchard_receiver.ok_or_else(|| {
        NozyError::AddressParsing(
            "Recipient must include an Orchard receiver (Orchard-only wallet).".to_string(),
        )
    })?;
    let recipient = orchard::Address::from_raw_address_bytes(&orchard_raw)
        .into_option()
        .ok_or_else(|| NozyError::AddressParsing("Invalid Orchard receiver bytes".to_string()))?;

    let tip_height = zebra.get_best_block_height().await?;
    let target_height = BlockHeight::from_u32(tip_height.saturating_add(1));
    let (anchor, merkle_path) = witness_provider
        .prepare_spend_anchor_and_path(zebra, spendable, tip_height)
        .await?;

    let total_input = spendable.orchard_note.value;
    let change = total_input
        .saturating_sub(amount_zatoshis)
        .saturating_sub(fee_zatoshis);

    let memo_bytes = memo.unwrap_or(&[]);
    let mut memo_arr = [0u8; 512];
    let len = memo_bytes.len().min(512);
    memo_arr[..len].copy_from_slice(&memo_bytes[..len]);
    let recipient_memo = MemoBytes::from_bytes(&memo_arr)
        .map_err(|e| NozyError::InvalidOperation(format!("Memo encoding failed: {e:?}")))?;

    let amount_zat = Zatoshis::from_u64(amount_zatoshis)
        .map_err(|_| NozyError::InvalidOperation("Invalid send amount".to_string()))?;
    let fee_zat = Zatoshis::from_u64(fee_zatoshis)
        .map_err(|_| NozyError::InvalidOperation("Invalid fee amount".to_string()))?;
    let fee_rule = FixedFeeRule { fee: fee_zat };

    let change_addr = fvk.to_ivk(Scope::Internal).address_at(0u64);
    let build_config = BuildConfig::Standard {
        sapling_anchor: None,
        orchard_anchor: Some(anchor),
    };

    let pczt = match network {
        NetworkType::Main => {
            let mut builder = Builder::new(MainNetwork, target_height, build_config);
            builder
                .add_orchard_spend::<NozyError>(
                    fvk.clone(),
                    spendable.orchard_note.note.clone(),
                    merkle_path,
                )
                .map_err(|e| NozyError::InvalidOperation(format!("add_orchard_spend: {e:?}")))?;
            builder
                .add_orchard_output::<NozyError>(
                    None,
                    recipient,
                    amount_zat,
                    recipient_memo.clone(),
                )
                .map_err(|e| NozyError::InvalidOperation(format!("add_orchard_output: {e:?}")))?;
            if change > 0 {
                let change_zat = Zatoshis::from_u64(change).map_err(|_| {
                    NozyError::InvalidOperation("Invalid change amount".to_string())
                })?;
                builder
                    .add_orchard_output::<NozyError>(
                        None,
                        change_addr,
                        change_zat,
                        MemoBytes::empty(),
                    )
                    .map_err(|e| {
                        NozyError::InvalidOperation(format!("add_orchard_output (change): {e:?}"))
                    })?;
            }
            let parts = builder
                .build_for_pczt(OsRng, &fee_rule)
                .map_err(|e| NozyError::InvalidOperation(format!("build_for_pczt: {e:?}")))?
                .pczt_parts;
            Creator::build_from_parts(parts).ok_or_else(|| {
                NozyError::InvalidOperation("PCZT creator: incompatible tx version".to_string())
            })?
        }
        NetworkType::Test | NetworkType::Regtest => {
            let mut builder = Builder::new(TestNetwork, target_height, build_config);
            builder
                .add_orchard_spend::<NozyError>(
                    fvk.clone(),
                    spendable.orchard_note.note.clone(),
                    merkle_path,
                )
                .map_err(|e| NozyError::InvalidOperation(format!("add_orchard_spend: {e:?}")))?;
            builder
                .add_orchard_output::<NozyError>(None, recipient, amount_zat, recipient_memo)
                .map_err(|e| NozyError::InvalidOperation(format!("add_orchard_output: {e:?}")))?;
            if change > 0 {
                let change_zat = Zatoshis::from_u64(change).map_err(|_| {
                    NozyError::InvalidOperation("Invalid change amount".to_string())
                })?;
                builder
                    .add_orchard_output::<NozyError>(
                        None,
                        change_addr,
                        change_zat,
                        MemoBytes::empty(),
                    )
                    .map_err(|e| {
                        NozyError::InvalidOperation(format!("add_orchard_output (change): {e:?}"))
                    })?;
            }
            let parts = builder
                .build_for_pczt(OsRng, &fee_rule)
                .map_err(|e| NozyError::InvalidOperation(format!("build_for_pczt: {e:?}")))?
                .pczt_parts;
            Creator::build_from_parts(parts).ok_or_else(|| {
                NozyError::InvalidOperation("PCZT creator: incompatible tx version".to_string())
            })?
        }
    };

    let pczt = Prover::new(pczt)
        .create_orchard_proof(orchard_proving_key())
        .map_err(|e| NozyError::InvalidOperation(format!("create_orchard_proof: {e:?}")))?
        .finish();

    let pczt = IoFinalizer::new(pczt)
        .finalize_io()
        .map_err(|e| NozyError::InvalidOperation(format!("io_finalize: {e:?}")))?;

    let pczt = redact_pczt_for_signer(pczt);
    let action_count = pczt.orchard().actions().len() as u32;
    let pczt_bytes = pczt.serialize();

    let recipient_short = if recipient_address.len() > 24 {
        format!("{}…", &recipient_address[..24])
    } else {
        recipient_address.to_string()
    };
    let summary = format!(
        "Send {:.8} ZEC to {}\nFee: {:.8} ZEC\nSpending 1 Orchard note",
        amount_zatoshis as f64 / 100_000_000.0,
        recipient_short,
        fee_zatoshis as f64 / 100_000_000.0,
    );

    let _ = pilot; // expiry encoded in target_height via chain tip

    Ok(KeystonePcztBuild {
        pczt_bytes,
        summary,
        action_count,
    })
}

/// Extract a broadcast-ready v5 transaction from a Keystone-signed PCZT.
pub fn extract_signed_tx_from_pczt_bytes(pczt_bytes: &[u8]) -> NozyResult<KeystoneExtractedTx> {
    let pczt = Pczt::parse(pczt_bytes)
        .map_err(|e| NozyError::InvalidOperation(format!("PCZT parse failed: {e:?}")))?;

    static ORCHARD_VK: OnceLock<orchard::circuit::VerifyingKey> = OnceLock::new();
    let vk = ORCHARD_VK.get_or_init(orchard::circuit::VerifyingKey::build);

    let tx = TransactionExtractor::new(pczt)
        .with_orchard(vk)
        .extract()
        .map_err(|e| NozyError::InvalidOperation(format!("TX extract failed: {e:?}")))?;

    let txid = tx.txid().to_string();
    let mut raw_transaction = Vec::new();
    tx.write(&mut raw_transaction)
        .map_err(|e| NozyError::InvalidOperation(format!("TX serialize failed: {e}")))?;

    Ok(KeystoneExtractedTx {
        raw_transaction,
        txid,
    })
}

/// Derive the default-account Orchard spending key (ZIP-32 coin type 133).
pub fn orchard_spending_key_from_wallet(wallet: &HDWallet) -> NozyResult<SpendingKey> {
    use zip32::AccountId;

    let seed_bytes = wallet.get_mnemonic_object().to_seed("");
    let account = AccountId::try_from(0)
        .map_err(|e| NozyError::KeyDerivation(format!("Invalid account ID: {e:?}")))?;
    SpendingKey::from_zip32_seed(&seed_bytes, 133, account).map_err(|e| {
        NozyError::KeyDerivation(format!("Failed to derive Orchard spending key: {e:?}"))
    })
}

/// Add Orchard spend authorization signatures to a proved, IO-finalized PCZT.
pub fn sign_pczt_orchard_spends(
    pczt_bytes: &[u8],
    spending_key: &SpendingKey,
) -> NozyResult<Vec<u8>> {
    let pczt = Pczt::parse(pczt_bytes)
        .map_err(|e| NozyError::InvalidOperation(format!("PCZT parse failed: {e:?}")))?;

    let action_count = pczt.orchard().actions().len();
    if action_count == 0 {
        return Err(NozyError::InvalidOperation(
            "PCZT has no Orchard actions to sign".to_string(),
        ));
    }

    let ask = SpendAuthorizingKey::from(spending_key);
    let mut signer = Signer::new(pczt)
        .map_err(|e| NozyError::InvalidOperation(format!("PCZT signer init failed: {e:?}")))?;

    let mut signed_any = false;
    for index in 0..action_count {
        if signer.sign_orchard(index, &ask).is_ok() {
            signed_any = true;
        }
    }

    if !signed_any {
        return Err(NozyError::InvalidOperation(
            "No Orchard spend signatures were applied (check PCZT and spending key)".to_string(),
        ));
    }

    Ok(signer.finish().serialize())
}

/// Build a portable co-sign request file payload from a PCZT build result.
pub fn prepared_send_from_build(
    recipient: &str,
    amount_zatoshis: u64,
    fee_zatoshis: u64,
    build: &KeystonePcztBuild,
) -> KeystonePreparedSend {
    KeystonePreparedSend {
        recipient: recipient.to_string(),
        amount_zatoshis,
        fee_zatoshis,
        summary: build.summary.clone(),
        action_count: build.action_count,
        pczt_hex: hex::encode(&build.pczt_bytes),
        created_at: chrono::Utc::now().to_rfc3339(),
    }
}

/// CBOR map `{1: bytes}` envelope expected by Keystone `zcash-pczt` UR type.
pub fn wrap_pczt_cbor(pczt_bytes: &[u8]) -> Vec<u8> {
    let mut cbor = Vec::with_capacity(pczt_bytes.len() + 8);
    cbor.push(0xa1); // map(1)
    cbor.push(0x01); // key 1
    encode_cbor_bytes(pczt_bytes, &mut cbor);
    cbor
}

pub fn unwrap_pczt_cbor(cbor: &[u8]) -> NozyResult<Vec<u8>> {
    if cbor.len() < 3 || cbor[0] != 0xa1 {
        return Err(NozyError::InvalidOperation(
            "Invalid PCZT CBOR envelope (expected map)".to_string(),
        ));
    }
    if cbor[1] != 0x01 {
        return Err(NozyError::InvalidOperation(
            "Invalid PCZT CBOR envelope (expected key 1)".to_string(),
        ));
    }
    decode_cbor_bytes(&cbor[2..])
}

fn encode_cbor_bytes(data: &[u8], out: &mut Vec<u8>) {
    let len = data.len();
    if len <= 23 {
        out.push(0x40 + len as u8);
    } else if len <= 255 {
        out.push(0x58);
        out.push(len as u8);
    } else if len <= 65535 {
        out.push(0x59);
        out.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        out.push(0x5a);
        out.extend_from_slice(&(len as u32).to_be_bytes());
    }
    out.extend_from_slice(data);
}

fn decode_cbor_bytes(cbor: &[u8]) -> NozyResult<Vec<u8>> {
    if cbor.is_empty() {
        return Err(NozyError::InvalidOperation(
            "Truncated CBOR bytes field".to_string(),
        ));
    }
    let (len, header_len) = match cbor[0] {
        b if (0x40..=0x57).contains(&b) => ((b - 0x40) as usize, 1),
        0x58 if cbor.len() >= 2 => (cbor[1] as usize, 2),
        0x59 if cbor.len() >= 3 => (u16::from_be_bytes([cbor[1], cbor[2]]) as usize, 3),
        0x5a if cbor.len() >= 5 => (
            u32::from_be_bytes([cbor[1], cbor[2], cbor[3], cbor[4]]) as usize,
            5,
        ),
        _ => {
            return Err(NozyError::InvalidOperation(
                "Unsupported CBOR bytes prefix".to_string(),
            ))
        }
    };
    let start: usize = header_len;
    let end = start.saturating_add(len);
    if end > cbor.len() {
        return Err(NozyError::InvalidOperation(
            "Truncated CBOR bytes payload".to_string(),
        ));
    }
    Ok(cbor[start..end].to_vec())
}

/// Encode PCZT bytes as BC-UR frames (`zcash-pczt`).
pub fn encode_pczt_ur_frames(pczt_bytes: &[u8], fragment_size: usize) -> NozyResult<Vec<String>> {
    let cbor = wrap_pczt_cbor(pczt_bytes);
    if fragment_size == 0 || cbor.len() <= fragment_size {
        return Ok(vec![ur::ur::encode(
            &cbor,
            &ur::ur::Type::Custom(UR_TYPE_ZCASH_PCZT),
        )]);
    }
    let mut encoder = ur::ur::Encoder::new(&cbor, fragment_size, UR_TYPE_ZCASH_PCZT)
        .map_err(|e| NozyError::InvalidOperation(format!("UR encoder: {e:?}")))?;
    let count = encoder.fragment_count();
    let mut parts = Vec::with_capacity(count.saturating_mul(2));
    for _ in 0..count.saturating_mul(2) {
        parts.push(
            encoder
                .next_part()
                .map_err(|e| NozyError::InvalidOperation(format!("UR part: {e:?}")))?,
        );
    }
    Ok(parts)
}

/// Decode BC-UR frames back into raw PCZT bytes.
pub fn decode_pczt_ur_frames(parts: &[String]) -> NozyResult<Vec<u8>> {
    if parts.is_empty() {
        return Err(NozyError::InvalidOperation(
            "No UR parts provided".to_string(),
        ));
    }
    let cbor = if parts.len() == 1 {
        ur::ur::decode(&parts[0])
            .map_err(|e| NozyError::InvalidOperation(format!("UR decode: {e:?}")))?
            .1
    } else {
        let mut decoder = ur::ur::Decoder::default();
        for part in parts {
            decoder
                .receive(part)
                .map_err(|e| NozyError::InvalidOperation(format!("UR decode: {e:?}")))?;
        }
        decoder
            .message()
            .map_err(|e| NozyError::InvalidOperation(format!("UR message incomplete: {e:?}")))?
            .ok_or_else(|| {
                NozyError::InvalidOperation("UR decoder returned no message".to_string())
            })?
    };
    unwrap_pczt_cbor(&cbor)
}

pub fn pending_send_path() -> PathBuf {
    get_wallet_data_dir().join("keystone_pending_send.json")
}

pub fn save_pending_send(prepared: &KeystonePreparedSend) -> NozyResult<()> {
    let path = pending_send_path();
    let json = serde_json::to_string_pretty(prepared)
        .map_err(|e| NozyError::Storage(format!("Serialize pending send: {e}")))?;
    fs::write(&path, json).map_err(|e| NozyError::Storage(format!("Write pending send: {e}")))?;
    Ok(())
}

pub fn load_pending_send() -> NozyResult<Option<KeystonePreparedSend>> {
    let path = pending_send_path();
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| NozyError::Storage(format!("Read pending send: {e}")))?;
    let prepared = serde_json::from_str(&content)
        .map_err(|e| NozyError::Storage(format!("Parse pending send: {e}")))?;
    Ok(Some(prepared))
}

pub fn clear_pending_send() -> NozyResult<()> {
    let path = pending_send_path();
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| NozyError::Storage(format!("Remove pending send: {e}")))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cbor_pczt_roundtrip() {
        let payload = b"pczt-test-bytes-12345";
        let cbor = wrap_pczt_cbor(payload);
        let decoded = unwrap_pczt_cbor(&cbor).unwrap();
        assert_eq!(decoded, payload);
    }

    #[test]
    fn ur_single_frame_roundtrip() {
        let pczt = b"sample-pczt";
        let frames = encode_pczt_ur_frames(pczt, 0).unwrap();
        assert_eq!(frames.len(), 1);
        let decoded = decode_pczt_ur_frames(&frames).unwrap();
        assert_eq!(decoded, pczt);
    }
}
