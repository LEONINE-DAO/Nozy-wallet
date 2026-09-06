//! Semantic compact-block parity vs a reference CompactTxStreamer (stock lightwalletd).
//!
//! Byte-identical protobuf is **not** required: we omit full headers, set fee 0,
//! skip transparent-only `CompactTx` rows that stock lightwalletd still emits empty,
//! and may fill `ChainMetadata` from a different seed path. This compares height, hashes,
//! shielded compact payloads, and tree sizes when both sides publish them.

use crate::proto::{CompactBlock, CompactTx};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParityIssue {
    pub field: String,
    pub ours: String,
    pub reference: String,
}

#[derive(Debug, Clone)]
pub struct HeightParity {
    pub height: u64,
    pub issues: Vec<ParityIssue>,
}

impl HeightParity {
    pub fn ok(&self) -> bool {
        self.issues.is_empty()
    }
}

fn hex_bytes(b: &[u8]) -> String {
    hex::encode(b)
}

fn bytes_eq_or_rev(a: &[u8], b: &[u8]) -> bool {
    a == b || {
        let mut rev = a.to_vec();
        rev.reverse();
        rev == b
    }
}

fn issue(field: &str, ours: impl Into<String>, reference: impl Into<String>) -> ParityIssue {
    ParityIssue {
        field: field.to_string(),
        ours: ours.into(),
        reference: reference.into(),
    }
}

fn has_shielded(tx: &CompactTx) -> bool {
    !tx.spends.is_empty()
        || !tx.outputs.is_empty()
        || !tx.actions.is_empty()
        || !tx.ironwood_actions.is_empty()
}

fn compare_tx(prefix: &str, ours: &CompactTx, reference: &CompactTx, out: &mut Vec<ParityIssue>) {
    if ours.spends.len() != reference.spends.len() {
        out.push(issue(
            &format!("{prefix}.spends.len"),
            ours.spends.len().to_string(),
            reference.spends.len().to_string(),
        ));
    } else {
        for (i, (a, b)) in ours.spends.iter().zip(reference.spends.iter()).enumerate() {
            if a.nf != b.nf {
                out.push(issue(
                    &format!("{prefix}.spends[{i}].nf"),
                    hex_bytes(&a.nf),
                    hex_bytes(&b.nf),
                ));
            }
        }
    }
    if ours.outputs.len() != reference.outputs.len() {
        out.push(issue(
            &format!("{prefix}.outputs.len"),
            ours.outputs.len().to_string(),
            reference.outputs.len().to_string(),
        ));
    } else {
        for (i, (a, b)) in ours
            .outputs
            .iter()
            .zip(reference.outputs.iter())
            .enumerate()
        {
            if a.cmu != b.cmu {
                out.push(issue(
                    &format!("{prefix}.outputs[{i}].cmu"),
                    hex_bytes(&a.cmu),
                    hex_bytes(&b.cmu),
                ));
            }
            if a.ephemeral_key != b.ephemeral_key {
                out.push(issue(
                    &format!("{prefix}.outputs[{i}].ephemeral_key"),
                    hex_bytes(&a.ephemeral_key),
                    hex_bytes(&b.ephemeral_key),
                ));
            }
            let n = a.ciphertext.len().min(b.ciphertext.len()).min(52);
            if a.ciphertext.get(..n) != b.ciphertext.get(..n) {
                out.push(issue(
                    &format!("{prefix}.outputs[{i}].ciphertext"),
                    hex_bytes(&a.ciphertext),
                    hex_bytes(&b.ciphertext),
                ));
            }
        }
    }
    compare_actions(
        &format!("{prefix}.actions"),
        &ours.actions,
        &reference.actions,
        out,
    );
    compare_actions(
        &format!("{prefix}.ironwood_actions"),
        &ours.ironwood_actions,
        &reference.ironwood_actions,
        out,
    );
}

fn compare_actions(
    prefix: &str,
    ours: &[crate::proto::CompactOrchardAction],
    reference: &[crate::proto::CompactOrchardAction],
    out: &mut Vec<ParityIssue>,
) {
    if ours.len() != reference.len() {
        out.push(issue(
            &format!("{prefix}.len"),
            ours.len().to_string(),
            reference.len().to_string(),
        ));
        return;
    }
    for (i, (a, b)) in ours.iter().zip(reference.iter()).enumerate() {
        if a.nullifier != b.nullifier {
            out.push(issue(
                &format!("{prefix}[{i}].nullifier"),
                hex_bytes(&a.nullifier),
                hex_bytes(&b.nullifier),
            ));
        }
        if a.cmx != b.cmx {
            out.push(issue(
                &format!("{prefix}[{i}].cmx"),
                hex_bytes(&a.cmx),
                hex_bytes(&b.cmx),
            ));
        }
        if a.ephemeral_key != b.ephemeral_key {
            out.push(issue(
                &format!("{prefix}[{i}].ephemeral_key"),
                hex_bytes(&a.ephemeral_key),
                hex_bytes(&b.ephemeral_key),
            ));
        }
        let n = a.ciphertext.len().min(b.ciphertext.len()).min(52);
        if a.ciphertext.get(..n) != b.ciphertext.get(..n) {
            out.push(issue(
                &format!("{prefix}[{i}].ciphertext"),
                hex_bytes(&a.ciphertext),
                hex_bytes(&b.ciphertext),
            ));
        }
    }
}

/// Compare Nozy compact encode/serve output to a reference lightwalletd block.
pub fn compare_compact(ours: &CompactBlock, reference: &CompactBlock) -> HeightParity {
    let mut issues = Vec::new();
    let height = ours.height;
    if ours.height != reference.height {
        issues.push(issue(
            "height",
            ours.height.to_string(),
            reference.height.to_string(),
        ));
    }
    if !bytes_eq_or_rev(&ours.hash, &reference.hash) {
        issues.push(issue(
            "hash",
            hex_bytes(&ours.hash),
            hex_bytes(&reference.hash),
        ));
    }
    if !bytes_eq_or_rev(&ours.prev_hash, &reference.prev_hash) {
        issues.push(issue(
            "prev_hash",
            hex_bytes(&ours.prev_hash),
            hex_bytes(&reference.prev_hash),
        ));
    }
    if ours.time != reference.time {
        issues.push(issue(
            "time",
            ours.time.to_string(),
            reference.time.to_string(),
        ));
    }

    let ours_n = ours.vtx.iter().filter(|tx| has_shielded(tx)).count();
    let ref_n = reference.vtx.iter().filter(|tx| has_shielded(tx)).count();
    if ours_n != ref_n {
        issues.push(issue(
            "vtx.shielded.len",
            ours_n.to_string(),
            ref_n.to_string(),
        ));
    }
    let mut used = vec![false; reference.vtx.len()];
    for (i, otx) in ours.vtx.iter().enumerate() {
        let found = reference
            .vtx
            .iter()
            .enumerate()
            .find(|(j, rtx)| !used[*j] && bytes_eq_or_rev(&otx.hash, &rtx.hash));
        match found {
            Some((j, rtx)) => {
                used[j] = true;
                compare_tx(&format!("vtx[{i}]"), otx, rtx, &mut issues);
            }
            None => issues.push(issue(
                &format!("vtx[{i}].hash"),
                hex_bytes(&otx.hash),
                "unmatched",
            )),
        }
    }
    for (j, rtx) in reference.vtx.iter().enumerate() {
        if !used[j] && has_shielded(rtx) {
            issues.push(issue(
                &format!("reference.vtx[{j}].hash"),
                "missing",
                hex_bytes(&rtx.hash),
            ));
        }
    }

    if let (Some(a), Some(b)) = (&ours.chain_metadata, &reference.chain_metadata) {
        if a.sapling_commitment_tree_size != 0
            && b.sapling_commitment_tree_size != 0
            && a.sapling_commitment_tree_size != b.sapling_commitment_tree_size
        {
            issues.push(issue(
                "chain_metadata.sapling",
                a.sapling_commitment_tree_size.to_string(),
                b.sapling_commitment_tree_size.to_string(),
            ));
        }
        if a.orchard_commitment_tree_size != 0
            && b.orchard_commitment_tree_size != 0
            && a.orchard_commitment_tree_size != b.orchard_commitment_tree_size
        {
            issues.push(issue(
                "chain_metadata.orchard",
                a.orchard_commitment_tree_size.to_string(),
                b.orchard_commitment_tree_size.to_string(),
            ));
        }
        if a.ironwood_commitment_tree_size != 0
            && b.ironwood_commitment_tree_size != 0
            && a.ironwood_commitment_tree_size != b.ironwood_commitment_tree_size
        {
            issues.push(issue(
                "chain_metadata.ironwood",
                a.ironwood_commitment_tree_size.to_string(),
                b.ironwood_commitment_tree_size.to_string(),
            ));
        }
    }

    HeightParity { height, issues }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::{ChainMetadata, CompactOrchardAction, CompactTx};

    fn block(height: u64, hash: u8, cmx: u8) -> CompactBlock {
        CompactBlock {
            proto_version: 4,
            height,
            hash: vec![hash; 32],
            prev_hash: vec![0; 32],
            time: 1,
            header: vec![],
            vtx: vec![CompactTx {
                index: 0,
                hash: vec![9; 32],
                fee: 0,
                spends: vec![],
                outputs: vec![],
                actions: vec![CompactOrchardAction {
                    nullifier: vec![1; 32],
                    cmx: vec![cmx; 32],
                    ephemeral_key: vec![2; 32],
                    ciphertext: vec![3; 52],
                }],
                ironwood_actions: vec![],
            }],
            chain_metadata: Some(ChainMetadata {
                sapling_commitment_tree_size: 10,
                orchard_commitment_tree_size: 20,
                ironwood_commitment_tree_size: 0,
            }),
        }
    }

    #[test]
    fn matching_blocks_ok() {
        let a = block(100, 1, 7);
        let b = block(100, 1, 7);
        assert!(compare_compact(&a, &b).ok());
    }

    #[test]
    fn reversed_block_hash_ok() {
        let a = block(100, 1, 7);
        let mut b = block(100, 1, 7);
        b.hash.reverse();
        b.prev_hash.reverse();
        assert!(compare_compact(&a, &b).ok());
    }

    #[test]
    fn cmx_mismatch_fails() {
        let a = block(100, 1, 7);
        let b = block(100, 1, 8);
        let r = compare_compact(&a, &b);
        assert!(!r.ok());
        assert!(r.issues.iter().any(|i| i.field.contains("cmx")));
    }

    #[test]
    fn zero_tree_size_not_compared() {
        let mut a = block(100, 1, 7);
        let mut b = block(100, 1, 7);
        a.chain_metadata
            .as_mut()
            .unwrap()
            .sapling_commitment_tree_size = 0;
        b.chain_metadata
            .as_mut()
            .unwrap()
            .sapling_commitment_tree_size = 99;
        assert!(compare_compact(&a, &b).ok());
    }

    #[test]
    fn extra_empty_reference_txs_ok() {
        let a = block(100, 1, 7);
        let mut b = block(100, 1, 7);
        b.vtx.push(CompactTx {
            index: 99,
            hash: vec![3; 32],
            fee: 0,
            spends: vec![],
            outputs: vec![],
            actions: vec![],
            ironwood_actions: vec![],
        });
        assert!(compare_compact(&a, &b).ok());
    }
}
