//! Decode Sapling compact spends/outputs from lightwalletd compact block blobs.

use prost::Message;

use crate::error::{ZeakingError, ZeakingResult};
use crate::lwd::proto::CompactBlock;

/// One Sapling output as carried in `CompactTx.outputs`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaplingCompactOutputBytes {
    pub cmu: [u8; 32],
    pub ephemeral_key: [u8; 32],
    pub ciphertext: Vec<u8>,
}

/// Sapling material from one compact transaction (consensus order).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaplingCompactTxSlice {
    pub txid_bytes: Vec<u8>,
    pub spends_nf: Vec<[u8; 32]>,
    pub outputs: Vec<SaplingCompactOutputBytes>,
}

/// Parsed Sapling fields for one compact block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaplingCompactBlockSlice {
    pub height: u64,
    /// Tree size after this block when `chainMetadata` is present.
    pub sapling_commitment_tree_size: Option<u32>,
    pub txs: Vec<SaplingCompactTxSlice>,
}

/// Decode Sapling spends/outputs (and optional tree size) from a compact block protobuf.
pub fn sapling_slice_from_compact_block(data: &[u8]) -> ZeakingResult<SaplingCompactBlockSlice> {
    let block: CompactBlock = CompactBlock::decode(data)
        .map_err(|e| ZeakingError::InvalidOperation(format!("compact block decode failed: {e}")))?;

    let sapling_commitment_tree_size = block
        .chain_metadata
        .as_ref()
        .map(|m| m.sapling_commitment_tree_size);

    let mut txs = Vec::with_capacity(block.vtx.len());
    for tx in block.vtx {
        let mut spends_nf = Vec::new();
        for spend in tx.spends {
            if spend.nf.len() == 32 {
                let mut nf = [0u8; 32];
                nf.copy_from_slice(&spend.nf);
                spends_nf.push(nf);
            }
        }
        let mut outputs = Vec::new();
        for out in tx.outputs {
            if out.cmu.len() != 32 || out.ephemeral_key.len() != 32 {
                continue;
            }
            let mut cmu = [0u8; 32];
            cmu.copy_from_slice(&out.cmu);
            let mut ephemeral_key = [0u8; 32];
            ephemeral_key.copy_from_slice(&out.ephemeral_key);
            outputs.push(SaplingCompactOutputBytes {
                cmu,
                ephemeral_key,
                ciphertext: out.ciphertext,
            });
        }
        txs.push(SaplingCompactTxSlice {
            txid_bytes: tx.hash,
            spends_nf,
            outputs,
        });
    }

    Ok(SaplingCompactBlockSlice {
        height: block.height,
        sapling_commitment_tree_size,
        txs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lwd::proto::{
        ChainMetadata, CompactBlock, CompactSaplingOutput, CompactSaplingSpend, CompactTx,
    };

    #[test]
    fn sapling_outputs_and_spends_round_trip_order() {
        let cmu: Vec<u8> = (1u8..33).collect();
        let epk: Vec<u8> = (2u8..34).collect();
        let nf: Vec<u8> = (3u8..35).collect();
        let mut tx = CompactTx::default();
        tx.hash = vec![9u8; 32];
        tx.spends.push(CompactSaplingSpend { nf: nf.clone() });
        tx.outputs.push(CompactSaplingOutput {
            cmu: cmu.clone(),
            ephemeral_key: epk.clone(),
            ciphertext: vec![7u8; 52],
        });
        let block = CompactBlock {
            height: 42,
            vtx: vec![tx],
            chain_metadata: Some(ChainMetadata {
                sapling_commitment_tree_size: 100,
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut buf = Vec::new();
        block.encode(&mut buf).unwrap();
        let slice = sapling_slice_from_compact_block(&buf).unwrap();
        assert_eq!(slice.height, 42);
        assert_eq!(slice.sapling_commitment_tree_size, Some(100));
        assert_eq!(slice.txs.len(), 1);
        assert_eq!(slice.txs[0].spends_nf[0].as_slice(), nf.as_slice());
        assert_eq!(slice.txs[0].outputs[0].cmu.as_slice(), cmu.as_slice());
        assert_eq!(slice.txs[0].outputs[0].ciphertext.len(), 52);
    }
}
