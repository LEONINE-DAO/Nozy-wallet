//! Sign Valar delegation PCZT sighash with the wallet Orchard SpendAuth key.
//!
//! Input/output JSON bridges `tools/nozy-vote` (which must not hold the seed)
//! and the main `nozy` binary. Tracking: issue #273.

use crate::error::{NozyError, NozyResult};
use crate::hd_wallet::HDWallet;
use group::ff::PrimeField;
use orchard::keys::{SpendAuthorizingKey, SpendingKey};
use pasta_curves::pallas;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::path::Path;
use zip32::fingerprint::SeedFingerprint;
use zip32::AccountId;

pub const REQUEST_FORMAT: &str = "nozy-vote-delegation-sign-v1";
pub const SIG_FORMAT: &str = "nozy-vote-delegation-sig-v1";

#[derive(Debug, Deserialize)]
pub struct DelegationSignRequestFile {
    pub format: String,
    pub account_index: u32,
    pub network: String,
    pub seed_fingerprint_hex: String,
    pub sighash_hex: String,
    pub alpha_hex: String,
    pub round_id: String,
    pub bundle_index: u32,
}

#[derive(Debug, Serialize)]
pub struct DelegationSigFile {
    pub format: String,
    pub round_id: String,
    pub bundle_index: u32,
    pub sighash_hex: String,
    pub spend_auth_sig_hex: String,
}

/// Sign a `nozy-vote` delegation signing request (JSON bytes; no disk I/O).
pub fn sign_delegation_request_json(
    wallet: &HDWallet,
    request_json: &[u8],
) -> NozyResult<DelegationSigFile> {
    let req: DelegationSignRequestFile = serde_json::from_slice(request_json)
        .map_err(|e| NozyError::InvalidOperation(format!("decode signing request: {e}")))?;
    if req.format != REQUEST_FORMAT {
        return Err(NozyError::InvalidOperation(format!(
            "unsupported signing request format {:?} (want {REQUEST_FORMAT})",
            req.format
        )));
    }

    let seed = wallet.get_mnemonic_object().to_seed("").to_vec();
    let seed_fp = SeedFingerprint::from_seed(&seed)
        .ok_or_else(|| NozyError::KeyDerivation("seed fingerprint: invalid seed length".into()))?;
    let expected_fp = hex::decode(req.seed_fingerprint_hex.trim())
        .map_err(|e| NozyError::InvalidOperation(format!("decode seed_fingerprint_hex: {e}")))?;
    if expected_fp.as_slice() != seed_fp.to_bytes() {
        return Err(NozyError::InvalidOperation(
            "wallet seed fingerprint does not match voting signing request \
             (wrong wallet / profile?)"
                .into(),
        ));
    }

    let sighash: [u8; 32] = hex::decode(req.sighash_hex.trim())
        .map_err(|e| NozyError::InvalidOperation(format!("decode sighash_hex: {e}")))?
        .try_into()
        .map_err(|_| NozyError::InvalidOperation("sighash must be 32 bytes".into()))?;
    let alpha: [u8; 32] = hex::decode(req.alpha_hex.trim())
        .map_err(|e| NozyError::InvalidOperation(format!("decode alpha_hex: {e}")))?
        .try_into()
        .map_err(|_| NozyError::InvalidOperation("alpha must be 32 bytes".into()))?;

    let account = AccountId::try_from(req.account_index).map_err(|e| {
        NozyError::KeyDerivation(format!("account id {}: {e:?}", req.account_index))
    })?;
    let sk = SpendingKey::from_zip32_seed(&seed, 133, account)
        .map_err(|e| NozyError::KeyDerivation(format!("spending key: {e:?}")))?;
    let ask = SpendAuthorizingKey::from(&sk);
    let alpha_scalar = Option::<pallas::Scalar>::from(pallas::Scalar::from_repr(alpha))
        .ok_or_else(|| NozyError::InvalidOperation("alpha is not a valid Pallas scalar".into()))?;
    let rsk = ask.randomize(&alpha_scalar);
    let sig = rsk.sign(&mut OsRng, &sighash);
    let sig_bytes: [u8; 64] = (&sig).into();

    Ok(DelegationSigFile {
        format: SIG_FORMAT.into(),
        round_id: req.round_id,
        bundle_index: req.bundle_index,
        sighash_hex: hex::encode(sighash),
        spend_auth_sig_hex: hex::encode(sig_bytes),
    })
}

/// Sign a `nozy-vote` delegation signing request with the unlocked wallet seed.
pub fn sign_delegation_request(
    wallet: &HDWallet,
    request_path: &Path,
    out_path: &Path,
) -> NozyResult<DelegationSigFile> {
    let bytes = std::fs::read(request_path).map_err(|e| {
        NozyError::InvalidOperation(format!("read {}: {e}", request_path.display()))
    })?;
    let out = sign_delegation_request_json(wallet, &bytes)?;
    let encoded = serde_json::to_vec_pretty(&out)
        .map_err(|e| NozyError::InvalidOperation(format!("serialize signature file: {e}")))?;
    std::fs::write(out_path, encoded)
        .map_err(|e| NozyError::InvalidOperation(format!("write {}: {e}", out_path.display())))?;
    Ok(out)
}
