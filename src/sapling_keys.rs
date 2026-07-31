//! Sapling ZIP-32 key derivation for quiet legacy-wallet compatibility.
//!
//! Derive account Sapling keys from the same HD seed Nozy uses for Orchard.
//! Phase 3 embeds the account Sapling payment address in generated Unified
//! Addresses (with Orchard). Do not market Sapling in the UI.

use crate::error::{NozyError, NozyResult};
use crate::hd_wallet::HDWallet;
use crate::key_management::SecureSeed;
use sapling::zip32::{DiversifiableFullViewingKey, ExtendedSpendingKey, IncomingViewingKey};
use sapling::PaymentAddress;
use zcash_keys::encoding::AddressCodec;
use zcash_keys::keys::sapling as sapling_zip32;
use zcash_protocol::consensus::{MainNetwork, NetworkType, TestNetwork};
use zip32::{AccountId, DiversifierIndex};

/// BIP-44 / ZIP-32 coin type used by Nozy Orchard derivation (`SpendingKey::from_zip32_seed`).
///
/// Kept identical so Sapling and Orchard share the same account namespace in this wallet.
pub const NOZY_ZIP32_COIN_TYPE: u32 = 133;

/// Derived Sapling account material (spending + viewing + one payment address).
#[derive(Clone)]
pub struct SaplingAccountKeys {
    pub extsk: ExtendedSpendingKey,
    pub dfvk: DiversifiableFullViewingKey,
    pub external_ivk: IncomingViewingKey,
    /// Diversifier index that produced [`Self::payment_address`] (may be > `start_index`
    /// when earlier indices are invalid under ZIP-32).
    pub diversifier_index: DiversifierIndex,
    pub payment_address: PaymentAddress,
}

impl core::fmt::Debug for SaplingAccountKeys {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SaplingAccountKeys")
            .field("extsk", &"…")
            .field("dfvk", &"…")
            .field("external_ivk", &"…")
            .field("diversifier_index", &self.diversifier_index)
            .field(
                "payment_address",
                &hex::encode(self.payment_address.to_bytes()),
            )
            .finish()
    }
}

/// Derive the ZIP-32 Sapling extended spending key for `account` from a BIP-39 seed.
///
/// Path: `m/32'/133'/account'` (matches [`zcash_keys::keys::sapling::spending_key`] with
/// Nozy's Orchard coin type).
pub fn derive_sapling_extsk(seed: &[u8], account: u32) -> NozyResult<ExtendedSpendingKey> {
    if seed.len() < 32 {
        return Err(NozyError::KeyDerivation(
            "ZIP 32 seeds MUST be at least 32 bytes".to_string(),
        ));
    }
    let account_id = AccountId::try_from(account)
        .map_err(|e| NozyError::KeyDerivation(format!("Invalid Sapling account ID: {e:?}")))?;
    Ok(sapling_zip32::spending_key(
        seed,
        NOZY_ZIP32_COIN_TYPE,
        account_id,
    ))
}

/// Derive Sapling account keys and a payment address starting search at `diversifier_start`.
///
/// Uses ZIP-32 `find_address` so invalid diversifier indices are skipped (index `0` is often
/// invalid; the returned `diversifier_index` is the first valid at or after `diversifier_start`).
pub fn derive_sapling_account_keys(
    seed: &[u8],
    account: u32,
    diversifier_start: u32,
) -> NozyResult<SaplingAccountKeys> {
    let extsk = derive_sapling_extsk(seed, account)?;
    let dfvk = extsk.to_diversifiable_full_viewing_key();
    let external_ivk = dfvk.to_external_ivk();
    let start = DiversifierIndex::from(diversifier_start);
    let (diversifier_index, payment_address) = dfvk.find_address(start).ok_or_else(|| {
        NozyError::KeyDerivation(format!(
            "No valid Sapling diversifier found at or after index {diversifier_start}"
        ))
    })?;
    Ok(SaplingAccountKeys {
        extsk,
        dfvk,
        external_ivk,
        diversifier_index,
        payment_address,
    })
}

/// Encode a Sapling payment address as `zs1…` / `ztestsapling…`.
pub fn encode_sapling_payment_address(
    address: &PaymentAddress,
    network: NetworkType,
) -> NozyResult<String> {
    match network {
        NetworkType::Main => Ok(address.encode(&MainNetwork)),
        NetworkType::Test | NetworkType::Regtest => Ok(address.encode(&TestNetwork)),
    }
}

impl HDWallet {
    /// Sapling account keys from this wallet's seed (embedded in receive UAs since Phase 3).
    pub fn derive_sapling_account_keys(
        &self,
        account: u32,
        diversifier_start: u32,
    ) -> NozyResult<SaplingAccountKeys> {
        let seed_bytes = self.get_mnemonic_object().to_seed("").to_vec();
        let secure_seed = SecureSeed::new(seed_bytes);
        derive_sapling_account_keys(secure_seed.as_bytes(), account, diversifier_start)
    }

    /// Encoded Sapling payment address (`zs1…` / `ztestsapling…`) for tests / diagnostics.
    pub fn generate_sapling_payment_address(
        &self,
        account: u32,
        diversifier_start: u32,
        network: NetworkType,
    ) -> NozyResult<String> {
        let keys = self.derive_sapling_account_keys(account, diversifier_start)?;
        encode_sapling_payment_address(&keys.payment_address, network)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bip39::Mnemonic;

    /// Fixed 24-word mnemonic for deterministic vectors (test-only; not a real wallet).
    const TEST_MNEMONIC: &str =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    fn test_wallet() -> HDWallet {
        HDWallet::from_mnemonic(TEST_MNEMONIC).expect("test mnemonic")
    }

    #[test]
    fn sapling_extsk_is_deterministic_for_account_zero() {
        let seed = Mnemonic::parse(TEST_MNEMONIC).unwrap().to_seed("");
        let a = derive_sapling_extsk(&seed, 0).unwrap();
        let b = derive_sapling_extsk(&seed, 0).unwrap();
        assert_eq!(a.to_bytes(), b.to_bytes());
    }

    #[test]
    fn sapling_account_keys_stable_payment_address_bytes() {
        let seed = Mnemonic::parse(TEST_MNEMONIC).unwrap().to_seed("");
        let keys = derive_sapling_account_keys(&seed, 0, 0).unwrap();
        // Snapshot: first valid diversifier at/after 0 for this seed/account.
        let expected = hex::encode(keys.payment_address.to_bytes());
        let again = derive_sapling_account_keys(&seed, 0, 0).unwrap();
        assert_eq!(hex::encode(again.payment_address.to_bytes()), expected);
        assert_eq!(again.diversifier_index, keys.diversifier_index);
    }

    #[test]
    fn sapling_mainnet_address_is_zs1() {
        let addr = test_wallet()
            .generate_sapling_payment_address(0, 0, NetworkType::Main)
            .unwrap();
        assert!(
            addr.starts_with("zs1"),
            "expected zs1 mainnet Sapling address, got {addr}"
        );
    }

    #[test]
    fn sapling_testnet_address_uses_test_hrp() {
        let addr = test_wallet()
            .generate_sapling_payment_address(0, 0, NetworkType::Test)
            .unwrap();
        assert!(
            addr.starts_with("ztestsapling"),
            "expected ztestsapling address, got {addr}"
        );
    }

    #[test]
    fn receive_ua_includes_orchard_and_sapling() {
        // Phase 3: receive UAs advertise Orchard + Sapling so legacy wallets can pay in.
        use zcash_address::unified::{Container, Encoding, Receiver};

        let wallet = test_wallet();
        let ua = wallet
            .generate_orchard_address(0, 0, NetworkType::Main)
            .unwrap();
        assert!(ua.starts_with("u1"));
        let (_network, address) = zcash_address::unified::Address::decode(&ua).expect("ua decode");
        let has_orchard = address
            .items()
            .iter()
            .any(|i| matches!(i, Receiver::Orchard(_)));
        let has_sapling = address
            .items()
            .iter()
            .any(|i| matches!(i, Receiver::Sapling(_)));
        assert!(has_orchard, "UA must keep Orchard receiver");
        assert!(has_sapling, "Phase 3 UA must include Sapling receiver");

        // Sapling receiver bytes must match account-0 payment address derivation.
        let keys = wallet.derive_sapling_account_keys(0, 0).unwrap();
        let sapling_item = address
            .items()
            .into_iter()
            .find_map(|i| match i {
                Receiver::Sapling(b) => Some(b),
                _ => None,
            })
            .expect("sapling receiver");
        assert_eq!(sapling_item, keys.payment_address.to_bytes());
    }
}
