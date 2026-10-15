//! lightwalletd submit + treestate (Zodl-style: no public JSON-RPC).

use super::client::LwdClient;
use super::proto::{BlockId, Empty, RawTransaction, TxFilter};
use super::sync::map_grpc_transport;
use crate::error::{ZeakingError, ZeakingResult};

/// Orchard/Sapling treestate from lightwalletd (`GetTreeState` / `GetLatestTreeState`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LwdTreeState {
    pub network: String,
    pub height: u64,
    pub hash: String,
    pub time: u32,
    pub sapling_tree: String,
    pub orchard_tree: String,
    pub ironwood_tree: String,
}

fn map_tree_state(ts: super::proto::TreeState) -> LwdTreeState {
    LwdTreeState {
        network: ts.network,
        height: ts.height,
        hash: ts.hash,
        time: ts.time,
        sapling_tree: ts.sapling_tree,
        orchard_tree: ts.orchard_tree,
        ironwood_tree: ts.ironwood_tree,
    }
}

/// `GetLatestTreeState` — tip treestate for witness/anchor checks without Zebrad RPC.
pub async fn get_latest_tree_state(client: &mut LwdClient) -> ZeakingResult<LwdTreeState> {
    let ts = client
        .get_latest_tree_state(Empty {})
        .await
        .map_err(|e| map_grpc_transport("GetLatestTreeState", e))?
        .into_inner();
    Ok(map_tree_state(ts))
}

/// `GetTreeState` at a height (hash optional).
pub async fn get_tree_state(
    client: &mut LwdClient,
    height: u64,
    hash: Option<Vec<u8>>,
) -> ZeakingResult<LwdTreeState> {
    let ts = client
        .get_tree_state(BlockId {
            height,
            hash: hash.unwrap_or_default(),
        })
        .await
        .map_err(|e| map_grpc_transport("GetTreeState", e))?
        .into_inner();
    Ok(map_tree_state(ts))
}

fn txid_hash_candidates(txid_hex: &str) -> ZeakingResult<Vec<Vec<u8>>> {
    let cleaned = txid_hex.trim().trim_start_matches("0x");
    let raw = hex::decode(cleaned)
        .map_err(|e| ZeakingError::InvalidOperation(format!("txid is not hex: {e}")))?;
    if raw.len() != 32 {
        return Err(ZeakingError::InvalidOperation(format!(
            "txid must be 32 bytes, got {}",
            raw.len()
        )));
    }
    let mut flipped = raw.clone();
    flipped.reverse();
    if flipped == raw {
        Ok(vec![raw])
    } else {
        // Compact blocks store the hash little-endian. Display txids are reversed.
        Ok(vec![raw, flipped])
    }
}

/// Full transaction bytes from lightwalletd `GetTransaction`.
///
/// Tries the given txid and its reversed 32-byte form. `Ok(None)` means the
/// server answered and does not have that transaction.
pub async fn get_transaction(
    client: &mut LwdClient,
    txid_hex: &str,
) -> ZeakingResult<Option<Vec<u8>>> {
    let candidates = txid_hash_candidates(txid_hex)?;
    let mut transport_error = None;
    for hash in candidates {
        match client
            .get_transaction(TxFilter {
                block: None,
                index: 0,
                hash,
            })
            .await
        {
            Ok(resp) => {
                let raw = resp.into_inner().data;
                if !raw.is_empty() {
                    return Ok(Some(raw));
                }
            }
            Err(status) => {
                let missing = matches!(
                    status.code(),
                    tonic::Code::NotFound | tonic::Code::InvalidArgument
                );
                if missing {
                    continue;
                }
                transport_error = Some(map_grpc_transport("GetTransaction", status));
                break;
            }
        }
    }
    if let Some(err) = transport_error {
        return Err(err);
    }
    Ok(None)
}

/// Broadcast a raw tx via lightwalletd `SendTransaction` (same door Zodl uses).
pub async fn send_transaction(client: &mut LwdClient, raw_tx: Vec<u8>) -> ZeakingResult<()> {
    let resp = client
        .send_transaction(RawTransaction {
            data: raw_tx,
            height: 0,
        })
        .await
        .map_err(|e| map_grpc_transport("SendTransaction", e))?
        .into_inner();
    if resp.error_code != 0 {
        return Err(ZeakingError::Grpc(format!(
            "SendTransaction: error_code={} {}",
            resp.error_code, resp.error_message
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::map_tree_state;
    use crate::lwd::proto::TreeState;

    #[test]
    fn maps_orchard_tree_field() {
        let ts = map_tree_state(TreeState {
            network: "main".into(),
            height: 1,
            hash: "ab".into(),
            time: 0,
            sapling_tree: String::new(),
            orchard_tree: "00".into(),
            ironwood_tree: String::new(),
        });
        assert_eq!(ts.height, 1);
        assert_eq!(ts.orchard_tree, "00");
    }
}
