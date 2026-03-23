//! Replay Orchard commitments from [`zeaking::lwd::LwdCompactStore`] and cross-check roots with Zebra `z_gettreestate`.

use crate::error::{NozyError, NozyResult};
use crate::orchard_tree_codec::orchard_commitment_tree_from_final_state;
use crate::orchard_witness::merkle_hash_from_cmx_bytes;
use crate::zebra_integration::ZebraClient;
use incrementalmerkletree::frontier::CommitmentTree;
use orchard::tree::MerkleHashOrchard;
use zeaking::lwd::{orchard_cmx_bytes_from_compact_block, LwdCompactStore};

type OrchardTree = CommitmentTree<MerkleHashOrchard, 32>;

/// Replay compact blocks `(checkpoint_height+1)..=end_height` onto the tree from `z_gettreestate(checkpoint_height)`.
pub async fn replay_orchard_tree_from_compact_store(
    store: &LwdCompactStore,
    zebra: &ZebraClient,
    checkpoint_height: u32,
    end_height: u32,
) -> NozyResult<OrchardTree> {
    if end_height < checkpoint_height {
        return Err(NozyError::InvalidOperation(
            "replay_orchard_tree: end_height < checkpoint_height".to_string(),
        ));
    }

    let checkpoint = zebra
        .get_orchard_treestate_parsed(checkpoint_height)
        .await?;
    let Some(state) = checkpoint.final_state else {
        return Err(NozyError::InvalidOperation(format!(
            "z_gettreestate({}) has no Orchard finalState (cannot checkpoint)",
            checkpoint_height
        )));
    };
    let mut tree = orchard_commitment_tree_from_final_state(&state)?;

    for h in (checkpoint_height + 1)..=end_height {
        let Some(blob) = store.get_compact_block(u64::from(h)).map_err(|e| {
            NozyError::InvalidOperation(format!("compact store read: {}", e))
        })? else {
            return Err(NozyError::InvalidOperation(format!(
                "Missing compact block at height {} in local store (sync compact range first)",
                h
            )));
        };
        let cmxs = orchard_cmx_bytes_from_compact_block(&blob).map_err(|e| {
            NozyError::InvalidOperation(format!("compact orchard decode: {}", e))
        })?;
        for cmx in cmxs {
            let node = merkle_hash_from_cmx_bytes(&cmx)?;
            tree.append(node).map_err(|_| {
                NozyError::InvalidOperation("Orchard commitment tree full during replay".to_string())
            })?;
        }
    }

    Ok(tree)
}

/// After [`replay_orchard_tree_from_compact_store`], assert the computed root matches `z_gettreestate(end_height)`.
pub async fn verify_orchard_tree_root_vs_rpc(
    store: &LwdCompactStore,
    zebra: &ZebraClient,
    checkpoint_height: u32,
    end_height: u32,
) -> NozyResult<()> {
    let tree =
        replay_orchard_tree_from_compact_store(store, zebra, checkpoint_height, end_height).await?;
    let local_root = tree.root().to_bytes();
    let remote = zebra.get_orchard_tree_state(end_height).await?;
    if local_root != remote.anchor {
        return Err(NozyError::InvalidOperation(format!(
            "Orchard root mismatch at height {}: local {} remote {}",
            end_height,
            hex::encode(local_root),
            hex::encode(remote.anchor)
        )));
    }
    Ok(())
}
