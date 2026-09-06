//! Raw Zcash block → lightwalletd `CompactBlock` (Sapling / Orchard / Ironwood).
//!
//! Approach mirrors ecosystem lightwalletd-rs: parse header + txs with librustzcash,
//! keep only shielded fields wallets need for trial decrypt / witnesses.

use std::io::{self, Cursor, Read};

use prost::Message;
use sha2::{Digest, Sha256};
use zcash_primitives::transaction::Transaction;
use zcash_protocol::consensus::BranchId;

use crate::error::{IndexerError, IndexerResult};
use crate::proto::{
    ChainMetadata, CompactBlock, CompactOrchardAction, CompactSaplingOutput, CompactSaplingSpend,
    CompactTx,
};

const HEADER_PREFIX_LEN: usize = 140;
const COMPACT_CIPHERTEXT_LEN: usize = 52;
const MIN_TX_BYTES: usize = 4;
const OP_0: u8 = 0x00;
const OP_1: u8 = 0x51;
const OP_16: u8 = 0x60;
const GENESIS_TARGET_DIFFICULTY: u64 = 520_617_983;

/// Parse raw block bytes into a protobuf `CompactBlock`.
///
/// Pass tree sizes after this block (lightwalletd `ChainMetadata`). Ingest fills
/// them via [`crate::tree_sizes::tree_sizes_after_block`]; tests may pass zeros.
pub fn raw_block_to_compact(
    raw: &[u8],
    sapling_tree_size: u32,
    orchard_tree_size: u32,
    ironwood_tree_size: u32,
) -> IndexerResult<CompactBlock> {
    if raw.len() < HEADER_PREFIX_LEN {
        return Err(IndexerError::Compact("block data is truncated".into()));
    }

    let prev_hash = raw[4..36].to_vec();
    let time = u32::from_le_bytes([raw[100], raw[101], raw[102], raw[103]]);

    let mut header_cursor = Cursor::new(&raw[HEADER_PREFIX_LEN..]);
    let solution_len = read_compact_size(&mut header_cursor)
        .map_err(|e| IndexerError::Compact(format!("equihash solution length: {e}")))?
        as usize;
    let header_end = HEADER_PREFIX_LEN + header_cursor.position() as usize + solution_len;
    if raw.len() < header_end {
        return Err(IndexerError::Compact("block header truncated".into()));
    }
    let hash = sha256d(&raw[..header_end]);

    let mut tx_cursor = Cursor::new(&raw[header_end..]);
    let tx_count = read_compact_size(&mut tx_cursor)
        .map_err(|e| IndexerError::Compact(format!("tx count: {e}")))? as usize;
    let capacity = tx_count.min(raw[header_end..].len() / MIN_TX_BYTES.max(1));
    let mut vtx = Vec::with_capacity(capacity);
    let mut height = None;

    for index in 0..tx_count {
        let tx = Transaction::read(&mut tx_cursor, BranchId::Nu5)
            .map_err(|e| IndexerError::Compact(format!("tx {index}: {e}")))?;
        if index == 0 {
            height = Some(coinbase_height(&tx)?);
        }
        if let Some(compact_tx) = to_compact_tx(index as u64, &tx) {
            vtx.push(compact_tx);
        }
    }

    if header_end + tx_cursor.position() as usize != raw.len() {
        return Err(IndexerError::Compact(
            "block has trailing data after last transaction".into(),
        ));
    }

    Ok(CompactBlock {
        proto_version: 4,
        height: height.ok_or_else(|| IndexerError::Compact("no coinbase height".into()))?,
        hash,
        prev_hash,
        time,
        header: Vec::new(),
        vtx,
        chain_metadata: Some(ChainMetadata {
            sapling_commitment_tree_size: sapling_tree_size,
            orchard_commitment_tree_size: orchard_tree_size,
            ironwood_commitment_tree_size: ironwood_tree_size,
        }),
    })
}

pub fn encode_compact_block(block: &CompactBlock) -> IndexerResult<Vec<u8>> {
    let mut buf = Vec::new();
    block
        .encode(&mut buf)
        .map_err(|e| IndexerError::Compact(format!("protobuf encode: {e}")))?;
    Ok(buf)
}

pub fn decode_compact_block(data: &[u8]) -> IndexerResult<CompactBlock> {
    CompactBlock::decode(data).map_err(|e| IndexerError::Compact(format!("protobuf decode: {e}")))
}

fn sha256d(data: &[u8]) -> Vec<u8> {
    Sha256::digest(Sha256::digest(data)).to_vec()
}

fn read_compact_size<R: Read>(r: &mut R) -> io::Result<u64> {
    let mut buf = [0u8; 1];
    r.read_exact(&mut buf)?;
    match buf[0] {
        n @ 0..=252 => Ok(u64::from(n)),
        253 => {
            let mut b = [0u8; 2];
            r.read_exact(&mut b)?;
            Ok(u64::from(u16::from_le_bytes(b)))
        }
        254 => {
            let mut b = [0u8; 4];
            r.read_exact(&mut b)?;
            Ok(u64::from(u32::from_le_bytes(b)))
        }
        255 => {
            let mut b = [0u8; 8];
            r.read_exact(&mut b)?;
            Ok(u64::from_le_bytes(b))
        }
    }
}

fn coinbase_height(tx: &Transaction) -> IndexerResult<u64> {
    let bundle = tx
        .transparent_bundle()
        .ok_or_else(|| IndexerError::Compact("coinbase missing transparent bundle".into()))?;
    let script = &bundle
        .vin
        .first()
        .ok_or_else(|| IndexerError::Compact("coinbase missing vin".into()))?
        .script_sig()
        .0
         .0;
    let first = *script
        .first()
        .ok_or_else(|| IndexerError::Compact("empty coinbase scriptSig".into()))?;
    let height = if first == OP_0 {
        0
    } else if (OP_1..=OP_16).contains(&first) {
        u64::from(first - (OP_1 - 1))
    } else {
        let n = first as usize;
        if n > 8 || script.len() < 1 + n {
            return Err(IndexerError::Compact("invalid BIP34 height push".into()));
        }
        let mut bytes = [0u8; 8];
        bytes[..n].copy_from_slice(&script[1..1 + n]);
        u64::from_le_bytes(bytes)
    };
    Ok(if height == GENESIS_TARGET_DIFFICULTY {
        0
    } else {
        height
    })
}

/// Only include txs that carry shielded compact material (matches classic lightwalletd).
fn to_compact_tx(index: u64, tx: &Transaction) -> Option<CompactTx> {
    let mut spends = Vec::new();
    let mut outputs = Vec::new();
    if let Some(sapling) = tx.sapling_bundle() {
        for spend in sapling.shielded_spends() {
            spends.push(CompactSaplingSpend {
                nf: spend.nullifier().0.to_vec(),
            });
        }
        for output in sapling.shielded_outputs() {
            let enc = output.enc_ciphertext();
            let ct = if enc.len() >= COMPACT_CIPHERTEXT_LEN {
                enc[..COMPACT_CIPHERTEXT_LEN].to_vec()
            } else {
                enc.to_vec()
            };
            outputs.push(CompactSaplingOutput {
                cmu: output.cmu().to_bytes().to_vec(),
                ephemeral_key: output.ephemeral_key().0.to_vec(),
                ciphertext: ct,
            });
        }
    }

    let mut actions = Vec::new();
    if let Some(orchard) = tx.orchard_bundle() {
        for action in orchard.actions().iter() {
            let note = action.encrypted_note();
            let ct = if note.enc_ciphertext.len() >= COMPACT_CIPHERTEXT_LEN {
                note.enc_ciphertext[..COMPACT_CIPHERTEXT_LEN].to_vec()
            } else {
                note.enc_ciphertext.to_vec()
            };
            actions.push(CompactOrchardAction {
                nullifier: action.nullifier().to_bytes().to_vec(),
                cmx: action.cmx().to_bytes().to_vec(),
                ephemeral_key: note.epk_bytes.to_vec(),
                ciphertext: ct,
            });
        }
    }

    let mut ironwood_actions = Vec::new();
    if let Some(ironwood) = tx.ironwood_bundle() {
        for action in ironwood.actions().iter() {
            let note = action.encrypted_note();
            let ct = if note.enc_ciphertext.len() >= COMPACT_CIPHERTEXT_LEN {
                note.enc_ciphertext[..COMPACT_CIPHERTEXT_LEN].to_vec()
            } else {
                note.enc_ciphertext.to_vec()
            };
            ironwood_actions.push(CompactOrchardAction {
                nullifier: action.nullifier().to_bytes().to_vec(),
                cmx: action.cmx().to_bytes().to_vec(),
                ephemeral_key: note.epk_bytes.to_vec(),
                ciphertext: ct,
            });
        }
    }

    if spends.is_empty() && outputs.is_empty() && actions.is_empty() && ironwood_actions.is_empty()
    {
        return None;
    }

    Some(CompactTx {
        index,
        hash: tx.txid().as_ref().to_vec(),
        fee: 0,
        spends,
        outputs,
        actions,
        ironwood_actions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_size_reads_small() {
        let mut c = Cursor::new([42u8]);
        assert_eq!(read_compact_size(&mut c).unwrap(), 42);
    }

    #[test]
    fn encode_roundtrip_empty_vtx() {
        let block = CompactBlock {
            proto_version: 4,
            height: 1,
            hash: vec![1; 32],
            prev_hash: vec![0; 32],
            time: 1,
            header: vec![],
            vtx: vec![],
            chain_metadata: Some(ChainMetadata::default()),
        };
        let bytes = encode_compact_block(&block).unwrap();
        let decoded = decode_compact_block(&bytes).unwrap();
        assert_eq!(decoded.height, 1);
        assert_eq!(decoded.hash, vec![1; 32]);
    }
}
