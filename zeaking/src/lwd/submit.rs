//! lightwalletd submit + treestate (Zodl-style: no public JSON-RPC).

use super::client::LwdClient;
use super::proto::{BlockId, Empty, RawTransaction};
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
}

fn map_tree_state(ts: super::proto::TreeState) -> LwdTreeState {
    LwdTreeState {
        network: ts.network,
        height: ts.height,
        hash: ts.hash,
        time: ts.time,
        sapling_tree: ts.sapling_tree,
        orchard_tree: ts.orchard_tree,
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
        });
        assert_eq!(ts.height, 1);
        assert_eq!(ts.orchard_tree, "00");
    }
}
