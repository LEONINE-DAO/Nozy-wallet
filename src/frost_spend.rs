//! Orchard spend authorization with rerandomized RedPallas FROST (ZIP 312).
//!
//! The agent coordinator stores the group public key and combines signature
//! shares. It has no method that accepts a long-term spend share. Each device
//! keeps its own [`KeyPackage`] from distributed key generation. A threshold
//! of 1 is rejected.
//!
//! The randomizer `alpha` and the sighash come from the PCZT the device
//! parses itself. The coordinator does not choose them. A shielded broadcast
//! leaves this process only after the signature verifies and a person has
//! confirmed. This module does not build the Orchard bundle and does not open
//! a network connection.
//!
//! Member backups use the version-1 `zkfrost` string (draft ZIP, zcash/zips#1432).
//! Restoring one against a unified full viewing key waits on ZIP 2005
//! `use_qsk` in the wallet's Orchard crate. The group descriptor records that
//! derivation so a later envelope does not have to guess it.

use std::collections::{BTreeMap, HashMap};
use std::fmt;

use bech32_11::{Bech32m, Hrp};
#[allow(deprecated)]
use frost_rerandomized::sign;
use rand::{CryptoRng, RngCore};
use reddsa::frost::redpallas::{self as frost};
use zeroize::Zeroize;

use frost::keys::dkg::{part1, part2, part3, round1, round2};
use frost::keys::{EvenY, KeyPackage, PublicKeyPackage};
use frost::rerandomized::{RandomizedParams, Randomizer};
use frost::{Identifier, SigningPackage};

use crate::{NozyError, NozyResult};

/// Smallest threshold this wallet will open. One share must not be able to spend.
pub const MIN_THRESHOLD: u16 = 2;

/// Derivation the group descriptor records. ZIP 2005 with `use_qsk`.
pub const DERIVATION_ZIP2005_USE_QSK: &str = "zip-2005-use-qsk";

const ZKFROST_HRP: &str = "zkfrost";
const ZKFROST_VERSION: u8 = 1;
const ZKFROST_LEN: usize = 68;

/// Spend policy for one Orchard FROST wallet. The agent may see this. It is not a key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpendPolicy {
    pub min_signers: u16,
    pub max_signers: u16,
}

impl SpendPolicy {
    pub fn open(min_signers: u16, max_signers: u16) -> NozyResult<Self> {
        if min_signers < MIN_THRESHOLD {
            return Err(NozyError::InvalidOperation(
                "FROST threshold must be at least 2. One share cannot spend alone.".into(),
            ));
        }
        if max_signers < min_signers {
            return Err(NozyError::InvalidOperation(
                "FROST signer count is smaller than the threshold.".into(),
            ));
        }
        Ok(Self {
            min_signers,
            max_signers,
        })
    }
}

/// What the device read from the PCZT. The coordinator does not fill this in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PcztSpendAuth {
    pub sighash: [u8; 32],
    pub alpha: [u8; 32],
}

/// Public description of the group. No share and no `sk`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GroupDescriptor {
    pub derivation: &'static str,
    pub min_signers: u16,
    pub max_signers: u16,
    pub ak: [u8; 32],
}

/// Coordinates a FROST session. It keeps the public key. It does not keep a spend share
/// and it does not choose the PCZT randomizer.
pub struct AgentCoordinator {
    policy: SpendPolicy,
    group_public: Option<PublicKeyPackage>,
    ak: Option<[u8; 32]>,
}

impl AgentCoordinator {
    pub fn new(policy: SpendPolicy) -> Self {
        Self {
            policy,
            group_public: None,
            ak: None,
        }
    }

    pub fn policy(&self) -> SpendPolicy {
        self.policy
    }

    /// Always false. A mode that hands the agent a share is a separate review.
    pub fn holds_spend_share(&self) -> bool {
        false
    }

    pub fn set_group_public(&mut self, package: PublicKeyPackage) -> NozyResult<()> {
        let ak = group_ak(&package)?;
        self.ak = Some(ak);
        self.group_public = Some(package);
        Ok(())
    }

    pub fn group_public(&self) -> Option<&PublicKeyPackage> {
        self.group_public.as_ref()
    }

    pub fn descriptor(&self) -> Option<GroupDescriptor> {
        Some(GroupDescriptor {
            derivation: DERIVATION_ZIP2005_USE_QSK,
            min_signers: self.policy.min_signers,
            max_signers: self.policy.max_signers,
            ak: self.ak?,
        })
    }

    pub fn signing_package(
        &self,
        commitments: BTreeMap<Identifier, frost::round1::SigningCommitments>,
        spend: &PcztSpendAuth,
    ) -> NozyResult<SigningPackage> {
        let public = self.group_public.as_ref().ok_or_else(|| {
            NozyError::InvalidOperation("FROST coordinator has no group public key yet.".into())
        })?;
        if commitments.len() < self.policy.min_signers as usize {
            return Err(NozyError::InvalidOperation(
                "FROST signing needs at least the threshold number of commitments.".into(),
            ));
        }
        for identifier in commitments.keys() {
            if !public.verifying_shares().contains_key(identifier) {
                return Err(NozyError::InvalidOperation(
                    "FROST commitment is from an unknown signer.".into(),
                ));
            }
        }
        Ok(SigningPackage::new(commitments, &spend.sighash))
    }

    pub fn aggregate(
        &self,
        signing_package: &SigningPackage,
        shares: &HashMap<Identifier, frost::round2::SignatureShare>,
        spend: &PcztSpendAuth,
    ) -> NozyResult<[u8; 64]> {
        if signing_package.message() != spend.sighash.as_slice() {
            return Err(NozyError::InvalidOperation(
                "FROST signing package is for a different sighash than the PCZT.".into(),
            ));
        }
        if shares.len() < self.policy.min_signers as usize {
            return Err(NozyError::InvalidOperation(
                "FROST aggregate needs at least the threshold number of shares.".into(),
            ));
        }
        let public = self.group_public.as_ref().ok_or_else(|| {
            NozyError::InvalidOperation("FROST coordinator has no group public key yet.".into())
        })?;
        let share_map: BTreeMap<_, _> = shares.iter().map(|(id, share)| (*id, *share)).collect();
        let randomized = randomized_params(public, &spend.alpha)?;
        let signature =
            frost::rerandomized::aggregate(signing_package, &share_map, public, &randomized)
                .map_err(cryptographic)?;
        let bytes = signature_bytes(&signature)?;
        ShieldedSpendAuth::from_pczt(spend.sighash, bytes, &randomized)?.verify()?;
        Ok(bytes)
    }
}

pub fn identifier(index: u16) -> NozyResult<Identifier> {
    if index == 0 {
        return Err(NozyError::InvalidInput(
            "FROST participant identifiers start at 1.".into(),
        ));
    }
    Identifier::try_from(index)
        .map_err(|_| NozyError::InvalidInput("FROST participant identifier was rejected.".into()))
}

pub fn dkg_part1<R: RngCore + CryptoRng>(
    policy: SpendPolicy,
    id: Identifier,
    rng: &mut R,
) -> NozyResult<(round1::SecretPackage, round1::Package)> {
    part1(id, policy.max_signers, policy.min_signers, rng).map_err(cryptographic)
}

pub fn dkg_part2(
    secret: round1::SecretPackage,
    received: &BTreeMap<Identifier, round1::Package>,
) -> NozyResult<(round2::SecretPackage, BTreeMap<Identifier, round2::Package>)> {
    part2(secret, received).map_err(cryptographic)
}

pub fn dkg_part3(
    secret: &round2::SecretPackage,
    round1_packages: &BTreeMap<Identifier, round1::Package>,
    round2_packages: &BTreeMap<Identifier, round2::Package>,
) -> NozyResult<(KeyPackage, PublicKeyPackage)> {
    part3(secret, round1_packages, round2_packages).map_err(cryptographic)
}

/// Orchard `ak` has an even y coordinate. Every participant applies the same flip.
pub fn apply_even_y(
    key: KeyPackage,
    public: PublicKeyPackage,
) -> NozyResult<(KeyPackage, PublicKeyPackage)> {
    let is_even = public.has_even_y();
    Ok((
        key.into_even_y(Some(is_even)),
        public.into_even_y(Some(is_even)),
    ))
}

pub fn group_ak(public: &PublicKeyPackage) -> NozyResult<[u8; 32]> {
    let serialized = public.verifying_key().serialize().map_err(cryptographic)?;
    let bytes: [u8; 32] = serialized
        .as_slice()
        .try_into()
        .map_err(|_| NozyError::Cryptographic("FROST group key was not 32 bytes.".into()))?;
    if bytes[31] & 0x80 != 0 {
        return Err(NozyError::Cryptographic(
            "FROST group key has an odd y coordinate, so it is not an Orchard ak.".into(),
        ));
    }
    Ok(bytes)
}

pub fn commit<R: RngCore + CryptoRng>(
    key: &KeyPackage,
    rng: &mut R,
) -> (
    frost::round1::SigningNonces,
    frost::round1::SigningCommitments,
) {
    frost::round1::commit(key.signing_share(), rng)
}

pub fn sign_share(
    signing_package: &SigningPackage,
    nonces: &frost::round1::SigningNonces,
    key: &KeyPackage,
    spend: &PcztSpendAuth,
) -> NozyResult<frost::round2::SignatureShare> {
    if signing_package.message() != spend.sighash.as_slice() {
        return Err(NozyError::InvalidOperation(
            "FROST signer refuses a sighash that is not the one from the PCZT.".into(),
        ));
    }
    let randomizer = Randomizer::deserialize(&spend.alpha).map_err(cryptographic)?;
    #[allow(deprecated)]
    sign(signing_package, nonces, key, randomizer).map_err(cryptographic)
}

fn cryptographic(err: impl fmt::Display) -> NozyError {
    NozyError::Cryptographic(err.to_string())
}

fn randomized_params(public: &PublicKeyPackage, alpha: &[u8; 32]) -> NozyResult<RandomizedParams> {
    let randomizer = Randomizer::deserialize(alpha).map_err(cryptographic)?;
    Ok(RandomizedParams::from_randomizer(
        public.verifying_key(),
        randomizer,
    ))
}

fn signature_bytes(signature: &frost::Signature) -> NozyResult<[u8; 64]> {
    let serialized = signature.serialize().map_err(cryptographic)?;
    let slice: &[u8] = serialized.as_slice();
    let mut bytes = [0u8; 64];
    if slice.len() != bytes.len() {
        return Err(NozyError::Cryptographic(
            "FROST signature was not 64 bytes.".into(),
        ));
    }
    bytes.copy_from_slice(slice);
    Ok(bytes)
}

/// A ZIP-244 sighash and the FROST spend-auth signature over it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShieldedSpendAuth {
    pub sighash: [u8; 32],
    pub signature: [u8; 64],
    pub verifying_key: [u8; 32],
}

impl ShieldedSpendAuth {
    pub fn from_pczt(
        sighash: [u8; 32],
        signature: [u8; 64],
        randomized: &RandomizedParams,
    ) -> NozyResult<Self> {
        let serialized_key = randomized
            .randomized_verifying_key()
            .serialize()
            .map_err(cryptographic)?;
        let key_slice: &[u8] = serialized_key.as_slice();
        if key_slice.len() != 32 {
            return Err(NozyError::Cryptographic(
                "FROST spend-auth key was not 32 bytes.".into(),
            ));
        }
        let mut verifying_key = [0u8; 32];
        verifying_key.copy_from_slice(key_slice);
        let auth = Self {
            sighash,
            signature,
            verifying_key,
        };
        auth.verify()?;
        Ok(auth)
    }

    pub fn verify(&self) -> NozyResult<()> {
        let verifying = reddsa::VerificationKey::<reddsa::orchard::SpendAuth>::try_from(
            reddsa::VerificationKeyBytes::from(self.verifying_key),
        )
        .map_err(|_| NozyError::Cryptographic("FROST spend-auth key was rejected.".into()))?;
        verifying
            .verify(&self.sighash, &reddsa::Signature::from(self.signature))
            .map_err(|_| {
                NozyError::Cryptographic("FROST spend-auth signature was rejected.".into())
            })
    }
}

/// Built shielded transaction bytes plus the FROST authorization for its sighash.
pub struct ShieldedBroadcast {
    pub auth: ShieldedSpendAuth,
    pub raw_transaction: Vec<u8>,
    pub confirmed: bool,
}

/// Hand a confirmed, FROST-authorized v5 or v6 transaction to `broadcaster`.
pub fn broadcast_shielded_spend<B>(request: ShieldedBroadcast, broadcaster: B) -> NozyResult<String>
where
    B: FnOnce(&[u8]) -> NozyResult<String>,
{
    if !request.confirmed {
        return Err(NozyError::InvalidOperation(
            "A shielded FROST spend waits for the confirm card.".into(),
        ));
    }
    request.auth.verify()?;
    if !is_shielded_tx_header(&request.raw_transaction) {
        return Err(NozyError::InvalidOperation(
            "FROST broadcast expects a Zcash v5 or v6 transaction.".into(),
        ));
    }
    broadcaster(&request.raw_transaction)
}

fn is_shielded_tx_header(raw: &[u8]) -> bool {
    raw.len() >= 4 && (raw[..4] == [0x05, 0x00, 0x00, 0x80] || raw[..4] == [0x06, 0x00, 0x00, 0x80])
}

/// One device's version-1 `zkfrost` backup. The coordinator never builds this:
/// it contains that device's signing share and the group's `sk`.
pub struct MemberBackup {
    pub participant: u8,
    pub min_signers: u8,
    pub max_signers: u8,
    pub signing_share: [u8; 32],
    pub spending_key: [u8; 32],
}

impl fmt::Debug for MemberBackup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemberBackup")
            .field("participant", &self.participant)
            .field("min_signers", &self.min_signers)
            .field("max_signers", &self.max_signers)
            .field("signing_share", &"<redacted>")
            .field("spending_key", &"<redacted>")
            .finish()
    }
}

impl Drop for MemberBackup {
    fn drop(&mut self) {
        self.signing_share.zeroize();
        self.spending_key.zeroize();
    }
}

impl MemberBackup {
    pub fn from_device(
        key: &KeyPackage,
        max_signers: u16,
        spending_key: &[u8; 32],
    ) -> NozyResult<Self> {
        let participant = small_identifier(key.identifier())?;
        let max_signers = u8::try_from(max_signers).map_err(|_| {
            NozyError::InvalidInput("FROST version-1 backups hold a group of at most 255.".into())
        })?;
        let min_signers = u8::try_from(*key.min_signers()).map_err(|_| {
            NozyError::InvalidInput("FROST threshold does not fit a version-1 backup.".into())
        })?;
        if participant == 0
            || participant > max_signers
            || min_signers < 2
            || min_signers > max_signers
        {
            return Err(NozyError::InvalidInput(
                "FROST backup participant or threshold is outside the group.".into(),
            ));
        }
        let serialized = key.signing_share().serialize();
        let signing_share: [u8; 32] = serialized.as_slice().try_into().map_err(|_| {
            NozyError::Cryptographic("FROST signing share was not 32 bytes.".into())
        })?;
        Ok(Self {
            participant,
            min_signers,
            max_signers,
            signing_share,
            spending_key: *spending_key,
        })
    }

    pub fn encode(&self) -> NozyResult<String> {
        let mut payload = Vec::with_capacity(ZKFROST_LEN);
        payload.extend_from_slice(&[
            ZKFROST_VERSION,
            self.participant,
            self.min_signers,
            self.max_signers,
        ]);
        payload.extend_from_slice(&self.signing_share);
        payload.extend_from_slice(&self.spending_key);
        let hrp = Hrp::parse(ZKFROST_HRP).map_err(cryptographic)?;
        bech32_11::encode::<Bech32m>(hrp, &payload).map_err(cryptographic)
    }

    pub fn decode(text: &str) -> NozyResult<Self> {
        let checked = bech32_11::primitives::decode::CheckedHrpstring::new::<Bech32m>(text.trim())
            .map_err(|_| {
                NozyError::InvalidInput("FROST backup string is not valid Bech32m.".into())
            })?;
        if checked.hrp().to_lowercase() != ZKFROST_HRP {
            return Err(NozyError::InvalidInput(
                "FROST backup string does not use the zkfrost prefix.".into(),
            ));
        }
        checked.validate_segwit_padding().map_err(|_| {
            NozyError::InvalidInput("FROST backup string has non-zero padding bits.".into())
        })?;
        let payload: Vec<u8> = checked.byte_iter().collect();
        if payload.first() != Some(&ZKFROST_VERSION) {
            return Err(NozyError::InvalidInput(
                "FROST backup string version is not 1.".into(),
            ));
        }
        if payload.len() != ZKFROST_LEN {
            return Err(NozyError::InvalidInput(
                "FROST backup string is not 68 bytes.".into(),
            ));
        }
        let (participant, min_signers, max_signers) = (payload[1], payload[2], payload[3]);
        if max_signers == 0
            || participant == 0
            || participant > max_signers
            || min_signers < 2
            || min_signers > max_signers
        {
            return Err(NozyError::InvalidInput(
                "FROST backup participant or threshold is outside the group.".into(),
            ));
        }
        let mut signing_share = [0u8; 32];
        let mut spending_key = [0u8; 32];
        signing_share.copy_from_slice(&payload[4..36]);
        spending_key.copy_from_slice(&payload[36..68]);
        Ok(Self {
            participant,
            min_signers,
            max_signers,
            signing_share,
            spending_key,
        })
    }
}

fn small_identifier(id: &Identifier) -> NozyResult<u8> {
    let bytes = id.serialize();
    let raw: &[u8] = bytes.as_slice();
    if raw.len() < 2 || raw[0] == 0 || raw[1..].iter().any(|byte| *byte != 0) {
        return Err(NozyError::InvalidInput(
            "FROST version-1 backups hold identifiers 1 through 255.".into(),
        ));
    }
    Ok(raw[0])
}

/// Run distributed key generation for `policy` participants and flip to an even `ak`.
fn distributed_keygen<R: RngCore + CryptoRng>(
    policy: SpendPolicy,
    rng: &mut R,
) -> NozyResult<(BTreeMap<Identifier, KeyPackage>, PublicKeyPackage)> {
    let mut round1_secrets = BTreeMap::new();
    let mut round1_sent = BTreeMap::new();
    for index in 1..=policy.max_signers {
        let id = identifier(index)?;
        let (secret, package) = dkg_part1(policy, id, rng)?;
        round1_secrets.insert(id, secret);
        round1_sent.insert(id, package);
    }

    let mut round2_secrets = BTreeMap::new();
    let mut round2_inbox: BTreeMap<Identifier, BTreeMap<Identifier, round2::Package>> =
        BTreeMap::new();
    for index in 1..=policy.max_signers {
        let id = identifier(index)?;
        let received = round1_sent
            .iter()
            .filter(|(sender, _)| **sender != id)
            .map(|(sender, package)| (*sender, package.clone()))
            .collect();
        let secret = round1_secrets
            .remove(&id)
            .ok_or_else(|| NozyError::Cryptographic("FROST round 1 secret was missing.".into()))?;
        let (secret, packages) = dkg_part2(secret, &received)?;
        round2_secrets.insert(id, secret);
        for (receiver, package) in packages {
            round2_inbox
                .entry(receiver)
                .or_default()
                .insert(id, package);
        }
    }

    let mut keys = BTreeMap::new();
    let mut group_public: Option<PublicKeyPackage> = None;
    for index in 1..=policy.max_signers {
        let id = identifier(index)?;
        let round1_packages = round1_sent
            .iter()
            .filter(|(sender, _)| **sender != id)
            .map(|(sender, package)| (*sender, package.clone()))
            .collect();
        let empty = BTreeMap::new();
        let round2_packages = round2_inbox.get(&id).unwrap_or(&empty);
        let (key, public) = dkg_part3(&round2_secrets[&id], &round1_packages, round2_packages)?;
        let (key, public) = apply_even_y(key, public)?;
        if let Some(previous) = &group_public {
            if previous.verifying_key() != public.verifying_key() {
                return Err(NozyError::Cryptographic(
                    "FROST participants derived different group keys.".into(),
                ));
            }
        }
        group_public = Some(public);
        keys.insert(id, key);
    }

    Ok((
        keys,
        group_public.ok_or_else(|| {
            NozyError::Cryptographic("FROST key generation produced no group key.".into())
        })?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    fn spend_of(sighash: [u8; 32]) -> PcztSpendAuth {
        PcztSpendAuth {
            sighash,
            alpha: [9u8; 32],
        }
    }

    #[test]
    fn a_single_share_cannot_open_a_session() {
        let err = SpendPolicy::open(1, 3).expect_err("1-of-n must be refused");
        assert!(err.to_string().contains("at least 2"));
    }

    #[test]
    fn the_agent_holds_no_spend_share() {
        let policy = SpendPolicy::open(2, 3).expect("2-of-3");
        let agent = AgentCoordinator::new(policy);
        assert!(!agent.holds_spend_share());
        assert!(agent.group_public().is_none());
        assert!(agent.descriptor().is_none());
    }

    #[test]
    fn two_of_three_signs_an_orchard_spend_auth() {
        let mut rng = thread_rng();
        let policy = SpendPolicy::open(2, 3).expect("2-of-3");
        let (mut keys, group_public) = distributed_keygen(policy, &mut rng).expect("dkg");
        let mut agent = AgentCoordinator::new(policy);
        agent.set_group_public(group_public).expect("even ak");
        assert!(!agent.holds_spend_share());
        let descriptor = agent.descriptor().expect("descriptor");
        assert_eq!(descriptor.derivation, DERIVATION_ZIP2005_USE_QSK);
        assert_eq!(descriptor.ak[31] & 0x80, 0);

        let spend = spend_of([7u8; 32]);
        let mut nonces = HashMap::new();
        let mut commitments = BTreeMap::new();
        for index in 1..=policy.min_signers {
            let id = identifier(index).expect("id");
            let key = keys.remove(&id).expect("device share");
            let (nonce, commitment) = commit(&key, &mut rng);
            nonces.insert(id, (nonce, key));
            commitments.insert(id, commitment);
        }
        let signing_package = agent.signing_package(commitments, &spend).expect("package");
        let mut shares = HashMap::new();
        for (id, (nonce, key)) in &nonces {
            let share = sign_share(&signing_package, nonce, key, &spend).expect("share");
            shares.insert(*id, share);
        }
        let signature = agent
            .aggregate(&signing_package, &shares, &spend)
            .expect("aggregate");
        assert_eq!(signature.len(), 64);
    }

    fn authorized_spend(sighash: [u8; 32]) -> (AgentCoordinator, ShieldedSpendAuth, PcztSpendAuth) {
        let mut rng = thread_rng();
        let policy = SpendPolicy::open(2, 3).expect("2-of-3");
        let (mut keys, group_public) = distributed_keygen(policy, &mut rng).expect("dkg");
        let mut agent = AgentCoordinator::new(policy);
        agent.set_group_public(group_public).expect("even ak");
        let spend = spend_of(sighash);
        let mut nonces = HashMap::new();
        let mut commitments = BTreeMap::new();
        for index in 1..=policy.min_signers {
            let id = identifier(index).expect("id");
            let key = keys.remove(&id).expect("device share");
            let (nonce, commitment) = commit(&key, &mut rng);
            nonces.insert(id, (nonce, key));
            commitments.insert(id, commitment);
        }
        let signing_package = agent.signing_package(commitments, &spend).expect("package");
        let mut shares = HashMap::new();
        for (id, (nonce, key)) in &nonces {
            let share = sign_share(&signing_package, nonce, key, &spend).expect("share");
            shares.insert(*id, share);
        }
        let signature = agent
            .aggregate(&signing_package, &shares, &spend)
            .expect("aggregate");
        let public = agent.group_public().expect("public");
        let randomized = randomized_params(public, &spend.alpha).expect("alpha");
        let auth =
            ShieldedSpendAuth::from_pczt(sighash, signature, &randomized).expect("spend auth");
        (agent, auth, spend)
    }

    #[test]
    fn a_shielded_spend_is_broadcast_only_after_frost_and_confirmation() {
        let sighash = [9u8; 32];
        let (agent, auth, _) = authorized_spend(sighash);
        assert!(!agent.holds_spend_share());
        let mut raw = vec![0x06, 0x00, 0x00, 0x80];
        raw.extend_from_slice(&sighash);

        let mut calls = 0;
        let waiting = broadcast_shielded_spend(
            ShieldedBroadcast {
                auth: auth.clone(),
                raw_transaction: raw.clone(),
                confirmed: false,
            },
            |_| {
                calls += 1;
                Ok("submitted".into())
            },
        );
        assert!(waiting.is_err());
        assert_eq!(calls, 0);

        let mut broken = auth.clone();
        broken.signature[0] ^= 0xff;
        let rejected = broadcast_shielded_spend(
            ShieldedBroadcast {
                auth: broken,
                raw_transaction: raw.clone(),
                confirmed: true,
            },
            |_| {
                calls += 1;
                Ok("submitted".into())
            },
        );
        assert!(rejected.is_err());
        assert_eq!(calls, 0);

        let submitted = broadcast_shielded_spend(
            ShieldedBroadcast {
                auth,
                raw_transaction: raw.clone(),
                confirmed: true,
            },
            |bytes| {
                calls += 1;
                assert_eq!(bytes, raw.as_slice());
                Ok("submitted".into())
            },
        )
        .expect("broadcaster");
        assert_eq!(submitted, "submitted");
        assert_eq!(calls, 1);
        assert!(!agent.holds_spend_share());
    }

    #[test]
    fn a_device_share_round_trips_as_a_zkfrost_string() {
        let mut rng = thread_rng();
        let policy = SpendPolicy::open(2, 3).expect("2-of-3");
        let (keys, group_public) = distributed_keygen(policy, &mut rng).expect("dkg");
        let mut agent = AgentCoordinator::new(policy);
        agent.set_group_public(group_public).expect("even ak");
        let key = keys.values().next().expect("device");
        let sk = [4u8; 32];
        let text = MemberBackup::from_device(key, policy.max_signers, &sk)
            .expect("backup")
            .encode()
            .expect("string");
        assert!(text.to_lowercase().starts_with("zkfrost1"));
        let decoded = MemberBackup::decode(&text).expect("decode");
        assert_eq!(decoded.participant, 1);
        assert_eq!(decoded.min_signers, 2);
        assert_eq!(decoded.max_signers, 3);
        assert_eq!(decoded.spending_key, sk);
        let shown = format!("{decoded:?}");
        assert!(shown.contains("<redacted>"));
        assert!(!shown.contains("040404"));
        assert!(MemberBackup::decode("zkfrost1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqxk3k2s").is_err());
    }
}
