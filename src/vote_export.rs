//! Export Ironwood notes + Merkle witnesses for `tools/nozy-vote` (coinholder voting).
//!
//! Format: `nozy-vote-notes-v1` JSON consumed by `nozy-vote import-notes`.
//! Witnesses MUST be rooted at the vote round's snapshot height (`nc_root`), not tip.
//! Tracking: https://github.com/LEONINE-DAO/Nozy-wallet/issues/273

use crate::error::{NozyError, NozyResult};
use crate::hd_wallet::HDWallet;
use crate::ironwood_tree_codec::{
    ironwood_commitment_tree_from_final_state, ironwood_incremental_witness_from_bytes,
    IronwoodCommitmentTree,
};
use crate::ironwood_tx::fetch_ironwood_cmx_nodes_for_height;
use crate::ironwood_witness::IronwoodWitnessTracker;
use crate::keystone::export_ufvk_from_wallet;
use crate::notes::{load_wallet_notes, SerializableOrchardNote};
use crate::orchard_witness::merkle_path_from_witness;
use crate::shielded_pool::ShieldedPool;
use crate::zebra_integration::ZebraClient;
use futures::future::join_all;
use orchard::keys::{FullViewingKey, SpendingKey};
use orchard::note::ExtractedNoteCommitment;
use orchard::tree::MerkleHashOrchard;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use zcash_protocol::consensus::NetworkType;
use zip32::fingerprint::SeedFingerprint;
use zip32::AccountId;

/// NU7 mainnet snapshot height (forum #56912 / Valar ACTIVE round).
pub const NU7_SNAPSHOT_HEIGHT_MAINNET: u32 = 3_459_350;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteNoteExportFile {
    pub format: String,
    pub network: String,
    pub ufvk: String,
    pub orchard_fvk_hex: String,
    pub seed_fingerprint_hex: String,
    pub account_index: u32,
    /// Height the Merkle witnesses / roots are anchored to.
    #[serde(default)]
    pub snapshot_height: Option<u32>,
    pub notes: Vec<VoteNoteExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteNoteExport {
    pub commitment_hex: String,
    pub nullifier_hex: String,
    pub value: u64,
    pub position: u64,
    pub diversifier_hex: String,
    pub rho_hex: String,
    pub rseed_hex: String,
    pub scope: u32,
    pub root_hex: String,
    pub auth_path_hex: Vec<String>,
    pub txid: String,
    pub block_height: u32,
}

/// Build Ironwood vote-note export using **current tip** witnesses (legacy / debug only).
/// Prefer [`build_ironwood_vote_notes_at_snapshot`] for Valar import.
pub fn build_ironwood_vote_notes(
    wallet: &HDWallet,
    network: NetworkType,
) -> NozyResult<VoteNoteExportFile> {
    let (ufvk, fvk, seed_fp) = export_identity(wallet, network)?;
    let notes = load_wallet_notes()?;
    let mut exported = Vec::new();
    for n in notes
        .iter()
        .filter(|n| !n.spent && n.pool == ShieldedPool::Ironwood)
    {
        exported.push(export_one_from_cached_witness(n)?);
    }

    if exported.is_empty() {
        return Err(NozyError::InvalidOperation(
            "No unspent Ironwood notes to export. Migrate Orchard → Ironwood before the NU7 snapshot."
                .into(),
        ));
    }

    Ok(VoteNoteExportFile {
        format: "nozy-vote-notes-v1".into(),
        network: network_label(network),
        ufvk,
        orchard_fvk_hex: hex::encode(fvk.to_bytes()),
        seed_fingerprint_hex: hex::encode(seed_fp.to_bytes()),
        account_index: 0,
        snapshot_height: None,
        notes: exported,
    })
}

/// Rebuild witnesses to `snapshot_height` so roots match the round `nc_root`.
pub async fn build_ironwood_vote_notes_at_snapshot(
    wallet: &HDWallet,
    network: NetworkType,
    zebra: &ZebraClient,
    snapshot_height: u32,
) -> NozyResult<VoteNoteExportFile> {
    let (ufvk, fvk, seed_fp) = export_identity(wallet, network)?;
    let notes = load_wallet_notes()?;
    let eligible: Vec<&SerializableOrchardNote> = notes
        .iter()
        .filter(|n| !n.spent && n.pool == ShieldedPool::Ironwood && n.block_height <= snapshot_height)
        .collect();

    if eligible.is_empty() {
        return Err(NozyError::InvalidOperation(format!(
            "No unspent Ironwood notes at or before snapshot height {snapshot_height}. \
             Notes created after the snapshot are not eligible."
        )));
    }

    let witnesses_by_nf =
        rebuild_ironwood_witnesses_to_snapshot(zebra, &eligible, snapshot_height).await?;

    let mut exported = Vec::new();
    for n in &eligible {
        let wit_bytes = witnesses_by_nf.get(n.nullifier_bytes.as_slice()).ok_or_else(|| {
            NozyError::InvalidOperation(format!(
                "failed to rebuild Ironwood witness for note in tx {} at snapshot {snapshot_height}",
                n.txid
            ))
        })?;
        exported.push(export_one_from_witness_bytes(n, wit_bytes)?);
    }

    Ok(VoteNoteExportFile {
        format: "nozy-vote-notes-v1".into(),
        network: network_label(network),
        ufvk,
        orchard_fvk_hex: hex::encode(fvk.to_bytes()),
        seed_fingerprint_hex: hex::encode(seed_fp.to_bytes()),
        account_index: 0,
        snapshot_height: Some(snapshot_height),
        notes: exported,
    })
}

/// Export tip witnesses (legacy). Prefer [`export_ironwood_vote_notes_at_snapshot`].
pub fn export_ironwood_vote_notes(
    wallet: &HDWallet,
    network: NetworkType,
    out_path: &Path,
) -> NozyResult<VoteNoteExportFile> {
    let file = build_ironwood_vote_notes(wallet, network)?;
    write_export(out_path, &file)?;
    Ok(file)
}

/// Export notes with Merkle witnesses anchored at the vote snapshot height.
pub async fn export_ironwood_vote_notes_at_snapshot(
    wallet: &HDWallet,
    network: NetworkType,
    zebra: &ZebraClient,
    snapshot_height: u32,
    out_path: &Path,
) -> NozyResult<VoteNoteExportFile> {
    let file =
        build_ironwood_vote_notes_at_snapshot(wallet, network, zebra, snapshot_height).await?;
    write_export(out_path, &file)?;
    Ok(file)
}

fn write_export(out_path: &Path, file: &VoteNoteExportFile) -> NozyResult<()> {
    let bytes = serde_json::to_vec_pretty(file)
        .map_err(|e| NozyError::InvalidOperation(format!("serialize vote note export: {e}")))?;
    std::fs::write(out_path, bytes)
        .map_err(|e| NozyError::InvalidOperation(format!("write {}: {e}", out_path.display())))?;
    Ok(())
}

fn network_label(network: NetworkType) -> String {
    match network {
        NetworkType::Main => "mainnet".into(),
        NetworkType::Test => "testnet".into(),
        NetworkType::Regtest => "regtest".into(),
    }
}

fn export_identity(
    wallet: &HDWallet,
    network: NetworkType,
) -> NozyResult<(String, FullViewingKey, SeedFingerprint)> {
    let ufvk = export_ufvk_from_wallet(wallet, network)?;
    let seed = wallet.get_mnemonic_object().to_seed("").to_vec();
    let seed_fp = SeedFingerprint::from_seed(&seed)
        .ok_or_else(|| NozyError::KeyDerivation("seed fingerprint: invalid seed length".into()))?;
    let account = AccountId::try_from(0)
        .map_err(|e| NozyError::KeyDerivation(format!("account id: {e:?}")))?;
    let sk = SpendingKey::from_zip32_seed(&seed, 133, account)
        .map_err(|e| NozyError::KeyDerivation(format!("spending key: {e:?}")))?;
    let fvk = FullViewingKey::from(&sk);
    Ok((ufvk, fvk, seed_fp))
}

/// Replay Ironwood cmxs from the earliest note height through snapshot, building witnesses.
async fn rebuild_ironwood_witnesses_to_snapshot(
    zebra: &ZebraClient,
    notes: &[&SerializableOrchardNote],
    snapshot_height: u32,
) -> NozyResult<HashMap<Vec<u8>, Vec<u8>>> {
    let start = notes
        .iter()
        .map(|n| n.block_height)
        .min()
        .ok_or_else(|| NozyError::InvalidOperation("no notes for witness rebuild".into()))?;

    let mut cmx_to_nf: HashMap<[u8; 32], [u8; 32]> = HashMap::new();
    for n in notes {
        let orchard = n.to_orchard_note().ok_or_else(|| {
            NozyError::InvalidOperation(format!(
                "note in tx {} missing rho/rseed — run sync before vote export",
                n.txid
            ))
        })?;
        let cmx: ExtractedNoteCommitment = orchard.commitment().into();
        let mut nf = [0u8; 32];
        nf.copy_from_slice(n.nullifier_bytes.as_slice());
        cmx_to_nf.insert(cmx.to_bytes(), nf);
    }

    let initial_tree: IronwoodCommitmentTree = if start <= 1 {
        IronwoodCommitmentTree::empty()
    } else {
        let cp = start.saturating_sub(1);
        let parsed = zebra.get_ironwood_treestate_parsed(cp).await?;
        if let Some(fs) = parsed.final_state {
            ironwood_commitment_tree_from_final_state(&fs)?
        } else {
            IronwoodCommitmentTree::empty()
        }
    };

    let mut tracker = IronwoodWitnessTracker::new(initial_tree);
    let parallel = 20usize;
    let mut height = start;
    while height <= snapshot_height {
        let batch_end = (height + parallel as u32 - 1).min(snapshot_height);
        let fetch_futures: Vec<_> = (height..=batch_end)
            .map(|h| fetch_ironwood_cmx_nodes_for_height(zebra, h))
            .collect();
        let batch = join_all(fetch_futures).await;
        for nodes in batch {
            for node in nodes? {
                let cmx_bytes = merkle_hash_to_bytes(&node);
                tracker.append_cmx(node)?;
                if let Some(nf) = cmx_to_nf.get(&cmx_bytes) {
                    tracker.register_discovered_note(*nf)?;
                }
            }
        }
        if height == start || batch_end == snapshot_height || height % 500 == 0 {
            tracing::info!(
                height,
                batch_end,
                snapshot_height,
                "vote export: rebuilding Ironwood witnesses toward snapshot"
            );
        }
        height = batch_end.saturating_add(1);
    }

    let tip_ts = zebra.get_ironwood_tree_state(snapshot_height).await?;
    let mut out = HashMap::new();
    for n in notes {
        let mut nf = [0u8; 32];
        nf.copy_from_slice(n.nullifier_bytes.as_slice());
        let Some(bytes) = tracker.serialized_witness_for_nullifier(&nf)? else {
            return Err(NozyError::InvalidOperation(format!(
                "note in tx {} (height {}) was not found while replaying Ironwood tree to snapshot {snapshot_height}",
                n.txid, n.block_height
            )));
        };
        let witness = ironwood_incremental_witness_from_bytes(&bytes)?;
        if !crate::orchard_witness::witness_root_matches_anchor(&witness, &tip_ts.anchor) {
            return Err(NozyError::InvalidOperation(format!(
                "rebuilt witness for tx {} does not match Ironwood treestate at snapshot {snapshot_height}",
                n.txid
            )));
        }
        out.insert(n.nullifier_bytes.clone(), bytes);
    }
    Ok(out)
}

fn merkle_hash_to_bytes(node: &MerkleHashOrchard) -> [u8; 32] {
    node.to_bytes()
}

fn export_one_from_cached_witness(n: &SerializableOrchardNote) -> NozyResult<VoteNoteExport> {
    let wit_hex = n.ironwood_incremental_witness_hex.as_ref().ok_or_else(|| {
        NozyError::InvalidOperation(format!(
            "Ironwood note in tx {} has no witness — resync to rebuild witnesses",
            n.txid
        ))
    })?;
    let wit_bytes = hex::decode(wit_hex)
        .map_err(|e| NozyError::InvalidOperation(format!("decode Ironwood witness hex: {e}")))?;
    export_one_from_witness_bytes(n, &wit_bytes)
}

fn export_one_from_witness_bytes(
    n: &SerializableOrchardNote,
    wit_bytes: &[u8],
) -> NozyResult<VoteNoteExport> {
    let note = n.to_orchard_note().ok_or_else(|| {
        NozyError::InvalidOperation(format!(
            "note in tx {} missing rho/rseed — run sync before vote export",
            n.txid
        ))
    })?;
    let witness = ironwood_incremental_witness_from_bytes(wit_bytes)?;
    let (anchor, merkle_path) = merkle_path_from_witness(&witness)?;

    let cmx: ExtractedNoteCommitment = note.commitment().into();
    let position = u64::from(u32::from(merkle_path.position()));
    let auth_path_hex: Vec<String> = merkle_path
        .auth_path()
        .iter()
        .map(|h| hex::encode(h.to_bytes()))
        .collect();
    if auth_path_hex.len() != 32 {
        return Err(NozyError::InvalidOperation(format!(
            "expected 32 auth path elems, got {}",
            auth_path_hex.len()
        )));
    }

    let diversifier = *note.recipient().diversifier().as_array();
    Ok(VoteNoteExport {
        commitment_hex: hex::encode(cmx.to_bytes()),
        nullifier_hex: hex::encode(n.nullifier_bytes.as_slice()),
        value: n.value,
        position,
        diversifier_hex: hex::encode(diversifier),
        rho_hex: hex::encode(note.rho().to_bytes()),
        rseed_hex: hex::encode(note.rseed().as_bytes()),
        scope: 0, // external
        root_hex: hex::encode(anchor.to_bytes()),
        auth_path_hex,
        txid: n.txid.clone(),
        block_height: n.block_height,
    })
}
