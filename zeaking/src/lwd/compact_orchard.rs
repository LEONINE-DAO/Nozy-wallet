//! Decode Orchard note commitments from compact block protobuf blobs.

use prost::Message;

use crate::error::{ZeakingError, ZeakingResult};
use crate::lwd::proto::CompactBlock;

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
}
