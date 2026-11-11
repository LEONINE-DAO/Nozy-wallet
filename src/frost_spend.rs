//! Orchard spend authorization with rerandomized RedPallas FROST (ZIP-312).
//!
//! The agent coordinator stores the group public key and combines signature
//! shares. It has no method that accepts a long-term spend share. Each device
//! keeps its own [`KeyPackage`] from distributed key generation. A threshold
//! of 1 is rejected. A shielded broadcast leaves this process only after the
//! signature verifies and a person has confirmed. This module does not build
//! the Orchard bundle and does not open a network connection itself.

use std::collections::{BTreeMap, HashMap};

use frost_rerandomized::frost_core::{
    Ciphersuite, Group,
    frost::{
        self, Identifier, SigningPackage,
        keys::{
            KeyPackage, PublicKeyPackage,
            dkg::{part1, part2, part3, round1, round2},
        },
    },
};
use frost_rerandomized::{RandomizedParams, aggregate, sign};
use rand::{CryptoRng, RngCore};
use reddsa::frost::redpallas::PallasBlake2b512;

use crate::{NozyError, NozyResult};

type Suite = PallasBlake2b512;

/// Smallest threshold this wallet will open. One share must not be able to spend.
pub const MIN_THRESHOLD: u16 = 2;

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

/// Coordinates a FROST session. It keeps the public key and, during one spend,
/// the one-time randomizer used to build the Orchard proof. It does not keep
/// a spend share.
pub struct AgentCoordinator {
    policy: SpendPolicy,
    group_public: Option<PublicKeyPackage<Suite>>,
}

impl AgentCoordinator {
    pub fn new(policy: SpendPolicy) -> Self {
        Self {
            policy,
            group_public: None,
        }
    }

    pub fn policy(&self) -> SpendPolicy {
        self.policy
    }

    /// Always false. A mode that hands the agent a share is a separate review.
    pub fn holds_spend_share(&self) -> bool {
        false
    }

    pub fn set_group_public(&mut self, package: PublicKeyPackage<Suite>) {
        self.group_public = Some(package);
    }

    pub fn group_public(&self) -> Option<&PublicKeyPackage<Suite>> {
        self.group_public.as_ref()
    }

    /// One-time randomizer for this spend. The coordinator needs it for the
    /// proof and must not send the scalar to the other signers.
    pub fn randomize<R: RngCore + CryptoRng>(
        &self,
        rng: &mut R,
    ) -> NozyResult<RandomizedParams<Suite>> {
        let public = self.group_public.as_ref().ok_or_else(|| {
            NozyError::InvalidOperation("FROST coordinator has no group public key yet.".into())
        })?;
        Ok(RandomizedParams::new(public, rng))
    }

    pub fn signing_package(
        &self,
        commitments: BTreeMap<Identifier<Suite>, frost::round1::SigningCommitments<Suite>>,
        message: &[u8],
    ) -> NozyResult<SigningPackage<Suite>> {
        let public = self.group_public.as_ref().ok_or_else(|| {
            NozyError::InvalidOperation("FROST coordinator has no group public key yet.".into())
        })?;
        if commitments.len() < self.policy.min_signers as usize {
            return Err(NozyError::InvalidOperation(
                "FROST signing needs at least the threshold number of commitments.".into(),
            ));
        }
        for identifier in commitments.keys() {
            if !public.signer_pubkeys().contains_key(identifier) {
                return Err(NozyError::InvalidOperation(
                    "FROST commitment is from an unknown signer.".into(),
                ));
            }
        }
        Ok(SigningPackage::new(commitments, message))
    }

    pub fn aggregate(
        &self,
        signing_package: &SigningPackage<Suite>,
        shares: &HashMap<Identifier<Suite>, frost::round2::SignatureShare<Suite>>,
        randomized: &RandomizedParams<Suite>,
    ) -> NozyResult<[u8; 64]> {
        if shares.len() < self.policy.min_signers as usize {
            return Err(NozyError::InvalidOperation(
                "FROST aggregate needs at least the threshold number of shares.".into(),
            ));
        }
        let public = self.group_public.as_ref().ok_or_else(|| {
            NozyError::InvalidOperation("FROST coordinator has no group public key yet.".into())
        })?;
        let signature =
            aggregate(signing_package, shares, public, randomized).map_err(cryptographic)?;
        let serialized = signature.serialize();
        let slice: &[u8] = serialized.as_ref();
        let mut bytes = [0u8; 64];
        if slice.len() != bytes.len() {
            return Err(NozyError::Cryptographic(
                "FROST signature was not 64 bytes.".into(),
            ));
        }
        bytes.copy_from_slice(slice);
        Ok(bytes)
    }
}

pub fn identifier(index: u16) -> NozyResult<Identifier<Suite>> {
    if index == 0 {
        return Err(NozyError::InvalidInput(
            "FROST participant identifiers start at 1.".into(),
        ));
    }
    index
        .try_into()
        .map_err(|_| NozyError::InvalidInput("FROST participant identifier was rejected.".into()))
}

pub fn dkg_part1<R: RngCore + CryptoRng>(
    policy: SpendPolicy,
    id: Identifier<Suite>,
    rng: &mut R,
) -> NozyResult<(round1::SecretPackage<Suite>, round1::Package<Suite>)> {
    part1(id, policy.max_signers, policy.min_signers, rng).map_err(cryptographic)
}

pub fn dkg_part2(
    secret: round1::SecretPackage<Suite>,
    received: &HashMap<Identifier<Suite>, round1::Package<Suite>>,
) -> NozyResult<(
    round2::SecretPackage<Suite>,
    HashMap<Identifier<Suite>, round2::Package<Suite>>,
)> {
    part2(secret, received).map_err(cryptographic)
}

pub fn dkg_part3(
    secret: &round2::SecretPackage<Suite>,
    round1_packages: &HashMap<Identifier<Suite>, round1::Package<Suite>>,
    round2_packages: &HashMap<Identifier<Suite>, round2::Package<Suite>>,
) -> NozyResult<(KeyPackage<Suite>, PublicKeyPackage<Suite>)> {
    part3(secret, round1_packages, round2_packages).map_err(cryptographic)
}

pub fn commit<R: RngCore + CryptoRng>(
    key: &KeyPackage<Suite>,
    rng: &mut R,
) -> (
    frost::round1::SigningNonces<Suite>,
    frost::round1::SigningCommitments<Suite>,
) {
    frost::round1::commit(key.secret_share(), rng)
}

pub fn sign_share(
    signing_package: &SigningPackage<Suite>,
    nonces: &frost::round1::SigningNonces<Suite>,
    key: &KeyPackage<Suite>,
    randomizer_point: &<<Suite as Ciphersuite>::Group as Group>::Element,
) -> NozyResult<frost::round2::SignatureShare<Suite>> {
    sign(signing_package, nonces, key, randomizer_point).map_err(cryptographic)
}

fn cryptographic(err: impl std::fmt::Display) -> NozyError {
    NozyError::Cryptographic(err.to_string())
}

/// A ZIP-244 Orchard sighash and the FROST spend-auth signature over it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShieldedSpendAuth {
    pub sighash: [u8; 32],
    pub signature: [u8; 64],
    pub verifying_key: [u8; 32],
}

impl ShieldedSpendAuth {
    pub fn from_randomized(
        sighash: [u8; 32],
        signature: [u8; 64],
        randomized: &RandomizedParams<Suite>,
    ) -> NozyResult<Self> {
        let serialized_key = randomized.randomized_group_public_key().serialize();
        let key_slice: &[u8] = serialized_key.as_ref();
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

/// Built Orchard v5 transaction bytes plus the FROST authorization for its sighash.
pub struct ShieldedBroadcast {
    pub auth: ShieldedSpendAuth,
    pub raw_transaction: Vec<u8>,
    pub confirmed: bool,
}

/// Hand a confirmed, FROST-authorized v5 transaction to `broadcaster`.
///
/// `broadcaster` is the wallet's existing submit path. This function checks
/// the confirm card, the RedPallas signature, and the v5 header before that
/// call. It does not contact a node.
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
    if request.raw_transaction.len() < 4 || request.raw_transaction[..4] != [0x05, 0x00, 0x00, 0x80]
    {
        return Err(NozyError::InvalidOperation(
            "FROST broadcast expects a Zcash v5 transaction.".into(),
        ));
    }
    broadcaster(&request.raw_transaction)
}

/// Run distributed key generation for `policy` participants.
///
/// The returned map is the stand-in for separate devices. Callers that are
/// the agent must keep only one [`PublicKeyPackage`] and discard every
/// [`KeyPackage`].
fn distributed_keygen<R: RngCore + CryptoRng>(
    policy: SpendPolicy,
    rng: &mut R,
) -> NozyResult<(
    BTreeMap<Identifier<Suite>, KeyPackage<Suite>>,
    PublicKeyPackage<Suite>,
)> {
    let mut round1_secrets = BTreeMap::new();
    let mut round1_sent = HashMap::new();

    for index in 1..=policy.max_signers {
        let id = identifier(index)?;
        let (secret, package) = dkg_part1(policy, id, rng)?;
        round1_secrets.insert(id, secret);
        round1_sent.insert(id, package);
    }

    let mut round2_secrets = BTreeMap::new();
    let mut round2_inbox: HashMap<
        Identifier<Suite>,
        HashMap<Identifier<Suite>, round2::Package<Suite>>,
    > = HashMap::new();
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
    let mut group_public: Option<PublicKeyPackage<Suite>> = None;
    for index in 1..=policy.max_signers {
        let id = identifier(index)?;
        let round1_packages = round1_sent
            .iter()
            .filter(|(sender, _)| **sender != id)
            .map(|(sender, package)| (*sender, package.clone()))
            .collect();
        let empty = HashMap::new();
        let round2_packages = round2_inbox.get(&id).unwrap_or(&empty);
        let (key, public) = dkg_part3(&round2_secrets[&id], &round1_packages, round2_packages)?;
        if let Some(previous) = &group_public {
            let left = previous.group_public().serialize();
            let right = public.group_public().serialize();
            let left_bytes: &[u8] = left.as_ref();
            let right_bytes: &[u8] = right.as_ref();
            if left_bytes != right_bytes {
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
    use reddsa::orchard;

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
    }

    #[test]
    fn two_of_three_signs_an_orchard_spend_auth() {
        let mut rng = thread_rng();
        let policy = SpendPolicy::open(2, 3).expect("2-of-3");
        let (mut keys, group_public) = distributed_keygen(policy, &mut rng).expect("dkg");

        let mut agent = AgentCoordinator::new(policy);
        agent.set_group_public(group_public);
        assert!(!agent.holds_spend_share());

        let sighash = [7u8; 32];
        let randomized = agent.randomize(&mut rng).expect("randomizer");
        let randomizer_point = *randomized.randomizer_point();

        let mut nonces = HashMap::new();
        let mut commitments = BTreeMap::new();
        for index in 1..=policy.min_signers {
            let id = identifier(index).expect("id");
            let key = keys.remove(&id).expect("device share");
            let (nonce, commitment) = commit(&key, &mut rng);
            nonces.insert(id, (nonce, key));
            commitments.insert(id, commitment);
        }

        let signing_package = agent
            .signing_package(commitments, &sighash)
            .expect("package");
        let mut shares = HashMap::new();
        for (id, (nonce, key)) in &nonces {
            let share = sign_share(&signing_package, nonce, key, &randomizer_point).expect("share");
            shares.insert(*id, share);
        }

        let signature = agent
            .aggregate(&signing_package, &shares, &randomized)
            .expect("aggregate");

        let serialized_key = randomized.randomized_group_public_key().serialize();
        let key_slice: &[u8] = serialized_key.as_ref();
        let mut vk_bytes = [0u8; 32];
        vk_bytes.copy_from_slice(key_slice);
        let verifying = reddsa::VerificationKey::<orchard::SpendAuth>::try_from(
            reddsa::VerificationKeyBytes::from(vk_bytes),
        )
        .expect("spend-auth key");
        verifying
            .verify(&sighash, &reddsa::Signature::from(signature))
            .expect("Orchard spend auth accepts the FROST signature");
    }

    fn authorized_spend(sighash: [u8; 32]) -> (AgentCoordinator, ShieldedSpendAuth) {
        let mut rng = thread_rng();
        let policy = SpendPolicy::open(2, 3).expect("2-of-3");
        let (mut keys, group_public) = distributed_keygen(policy, &mut rng).expect("dkg");
        let mut agent = AgentCoordinator::new(policy);
        agent.set_group_public(group_public);
        let randomized = agent.randomize(&mut rng).expect("randomizer");
        let randomizer_point = *randomized.randomizer_point();
        let mut nonces = HashMap::new();
        let mut commitments = BTreeMap::new();
        for index in 1..=policy.min_signers {
            let id = identifier(index).expect("id");
            let key = keys.remove(&id).expect("device share");
            let (nonce, commitment) = commit(&key, &mut rng);
            nonces.insert(id, (nonce, key));
            commitments.insert(id, commitment);
        }
        let signing_package = agent
            .signing_package(commitments, &sighash)
            .expect("package");
        let mut shares = HashMap::new();
        for (id, (nonce, key)) in &nonces {
            let share = sign_share(&signing_package, nonce, key, &randomizer_point).expect("share");
            shares.insert(*id, share);
        }
        let signature = agent
            .aggregate(&signing_package, &shares, &randomized)
            .expect("aggregate");
        let auth = ShieldedSpendAuth::from_randomized(sighash, signature, &randomized)
            .expect("spend auth");
        (agent, auth)
    }

    #[test]
    fn a_shielded_spend_is_broadcast_only_after_frost_and_confirmation() {
        let sighash = [9u8; 32];
        let (agent, auth) = authorized_spend(sighash);
        assert!(!agent.holds_spend_share());
        let mut raw = vec![0x05, 0x00, 0x00, 0x80];
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
}
