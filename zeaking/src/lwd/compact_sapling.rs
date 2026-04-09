//! Decode Sapling note commitments (`cmu`) from compact block protobuf blobs (lightwalletd).

use prost::Message;

use crate::error::ZeakingResult;
use crate::lwd::proto::CompactBlock;

pub fn sapling_cmu_bytes_from_compact_block(data: &[u8]) -> ZeakingResult<Vec<[u8; 32]>> {
    let block: CompactBlock = CompactBlock::decode(data).map_err(|e| {
        crate::ZeakingError::InvalidOperation(format!("compact block decode failed: {}", e))
    })?;

    let mut out = Vec::new();
    for tx in block.vtx {
        for o in tx.outputs {
            if o.cmu.len() == 32 {
                let mut cmu = [0u8; 32];
                cmu.copy_from_slice(&o.cmu);
                out.push(cmu);
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lwd::proto::{CompactBlock, CompactSaplingOutput, CompactTx};

    #[test]
    fn sapling_cmu_order_single_output() {
        let mut out = CompactSaplingOutput::default();
        out.cmu = vec![9u8; 32];
        let mut tx = CompactTx::default();
        tx.outputs.push(out);
        let mut cb = CompactBlock::default();
        cb.vtx.push(tx);
        let mut buf = Vec::new();
        cb.encode(&mut buf).expect("encode");
        let cmus = sapling_cmu_bytes_from_compact_block(&buf).unwrap();
        assert_eq!(cmus.len(), 1);
        assert_eq!(cmus[0], [9u8; 32]);
    }
}
