//! Decode Orchard note commitments from compact block protobuf blobs.

use prost::Message;

use crate::error::{ZeakingError, ZeakingResult};
use crate::lwd::proto::CompactBlock;

/// One compact Orchard or Ironwood action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchardCompactActionBytes {
    pub nullifier: [u8; 32],
    pub cmx: [u8; 32],
    pub ephemeral_key: [u8; 32],
    pub ciphertext: Vec<u8>,
}

/// Compact Orchard/Ironwood material from one transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchardCompactTxSlice {
    pub txid_bytes: Vec<u8>,
    pub orchard_actions: Vec<OrchardCompactActionBytes>,
    pub ironwood_actions: Vec<OrchardCompactActionBytes>,
}

/// Parsed Orchard fields for one compact block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchardCompactBlockSlice {
    pub height: u64,
    pub txs: Vec<OrchardCompactTxSlice>,
}

fn action_bytes(
    nf: &[u8],
    cmx: &[u8],
    epk: &[u8],
    ciphertext: Vec<u8>,
) -> Option<OrchardCompactActionBytes> {
    if nf.len() != 32 || cmx.len() != 32 || epk.len() != 32 {
        return None;
    }
    let mut nullifier = [0u8; 32];
    nullifier.copy_from_slice(nf);
    let mut cmx_arr = [0u8; 32];
    cmx_arr.copy_from_slice(cmx);
    let mut ephemeral_key = [0u8; 32];
    ephemeral_key.copy_from_slice(epk);
    Some(OrchardCompactActionBytes {
        nullifier,
        cmx: cmx_arr,
        ephemeral_key,
        ciphertext,
    })
}

/// Decode Orchard and Ironwood compact actions from a compact block protobuf.
pub fn orchard_slice_from_compact_block(data: &[u8]) -> ZeakingResult<OrchardCompactBlockSlice> {
    let block: CompactBlock = CompactBlock::decode(data)
        .map_err(|e| ZeakingError::InvalidOperation(format!("compact block decode failed: {e}")))?;

    let mut txs = Vec::with_capacity(block.vtx.len());
    for tx in block.vtx {
        let mut orchard_actions = Vec::new();
        for action in tx.actions {
            if let Some(parsed) = action_bytes(
                &action.nullifier,
                &action.cmx,
                &action.ephemeral_key,
                action.ciphertext,
            ) {
                orchard_actions.push(parsed);
            }
        }
        let mut ironwood_actions = Vec::new();
        for action in tx.ironwood_actions {
            if let Some(parsed) = action_bytes(
                &action.nullifier,
                &action.cmx,
                &action.ephemeral_key,
                action.ciphertext,
            ) {
                ironwood_actions.push(parsed);
            }
        }
        txs.push(OrchardCompactTxSlice {
            txid_bytes: tx.hash,
            orchard_actions,
            ironwood_actions,
        });
    }

    Ok(OrchardCompactBlockSlice {
        height: block.height,
        txs,
    })
}

/// Orchard `cmx` values in consensus order (block vtx order, actions within each tx).
pub fn orchard_cmx_bytes_from_compact_block(data: &[u8]) -> ZeakingResult<Vec<[u8; 32]>> {
    let block: CompactBlock = CompactBlock::decode(data)
        .map_err(|e| ZeakingError::InvalidOperation(format!("compact block decode failed: {e}")))?;

    let mut out = Vec::new();
    for tx in block.vtx {
        for action in tx.actions {
            if action.cmx.len() == 32 {
                let mut cmx = [0u8; 32];
                cmx.copy_from_slice(&action.cmx);
                out.push(cmx);
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lwd::proto::{CompactBlock, CompactOrchardAction, CompactTx};

    #[test]
    fn orchard_cmx_order_matches_vtx_and_actions() {
        let cmx_a: Vec<u8> = (0u8..32).collect();
        let cmx_b: Vec<u8> = (100u8..132).collect();

        let mut tx1 = CompactTx::default();
        tx1.actions.push(CompactOrchardAction {
            nullifier: vec![0u8; 32],
            cmx: cmx_a.clone(),
            ephemeral_key: vec![0u8; 32],
            ciphertext: vec![],
        });
        let mut tx2 = CompactTx::default();
        tx2.actions.push(CompactOrchardAction {
            nullifier: vec![0u8; 32],
            cmx: cmx_b.clone(),
            ephemeral_key: vec![0u8; 32],
            ciphertext: vec![],
        });

        let block = CompactBlock {
            height: 7,
            vtx: vec![tx1, tx2],
            ..Default::default()
        };

        let mut buf = Vec::new();
        block.encode(&mut buf).unwrap();
        let out = orchard_cmx_bytes_from_compact_block(&buf).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(&out[0][..], &cmx_a[..]);
        assert_eq!(&out[1][..], &cmx_b[..]);
    }

    #[test]
    fn orchard_slice_includes_txid_and_ironwood_actions() {
        let nf: Vec<u8> = (1u8..33).collect();
        let cmx: Vec<u8> = (2u8..34).collect();
        let epk: Vec<u8> = (3u8..35).collect();
        let mut tx = CompactTx {
            hash: vec![9u8; 32],
            ..Default::default()
        };
        tx.actions.push(CompactOrchardAction {
            nullifier: nf.clone(),
            cmx: cmx.clone(),
            ephemeral_key: epk.clone(),
            ciphertext: vec![7u8; 52],
        });
        tx.ironwood_actions.push(CompactOrchardAction {
            nullifier: nf.clone(),
            cmx: cmx.clone(),
            ephemeral_key: epk.clone(),
            ciphertext: vec![8u8; 52],
        });
        let block = CompactBlock {
            height: 3484960,
            vtx: vec![tx],
            ..Default::default()
        };
        let mut buf = Vec::new();
        block.encode(&mut buf).unwrap();
        let slice = orchard_slice_from_compact_block(&buf).unwrap();
        assert_eq!(slice.height, 3484960);
        assert_eq!(slice.txs.len(), 1);
        assert_eq!(slice.txs[0].txid_bytes, vec![9u8; 32]);
        assert_eq!(slice.txs[0].orchard_actions.len(), 1);
        assert_eq!(slice.txs[0].ironwood_actions.len(), 1);
        assert_eq!(slice.txs[0].orchard_actions[0].ciphertext.len(), 52);
    }
}
