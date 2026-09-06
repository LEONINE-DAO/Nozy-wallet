//! CompactTxStreamer gRPC server (Phase 1 compact + Phase 2 treestate/submit).

use std::pin::Pin;
use std::sync::Arc;

use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use crate::compact::decode_compact_block;
use crate::proto::compact_tx_streamer_server::{CompactTxStreamer, CompactTxStreamerServer};
use crate::proto::{
    self, BlockId, BlockRange, ChainSpec, Empty, GetAddressUtxosArg, GetAddressUtxosReplyList,
    GetSubtreeRootsArg, LightdInfo, PingResponse, RawTransaction, SendResponse, TreeState,
};
use crate::rpc::{NodeInfo, RpcClient};
use crate::store::IndexerStore;
use crate::treestate::{fetch_subtree_roots, fetch_tree_state};

#[derive(Clone)]
pub struct ServeState {
    pub store: Arc<IndexerStore>,
    pub rpc: Arc<RpcClient>,
    pub node: Arc<RwLock<NodeInfo>>,
    pub node_kind: &'static str,
    pub vendor: String,
    pub version: String,
}

impl ServeState {
    pub fn into_service(self) -> CompactTxStreamerServer<Self> {
        CompactTxStreamerServer::new(self)
    }
}

fn unimplemented(method: &str) -> Status {
    Status::unimplemented(format!(
        "Nozy Sync Engine does not implement {method} (see issue #274)"
    ))
}

#[tonic::async_trait]
impl CompactTxStreamer for ServeState {
    async fn get_latest_block(
        &self,
        _request: Request<ChainSpec>,
    ) -> Result<Response<BlockId>, Status> {
        let height = self
            .store
            .max_height()
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::unavailable("Nozy Sync Engine has no compact blocks yet"))?;
        let hash = self
            .store
            .get_block_hash(height)
            .map_err(|e| Status::internal(e.to_string()))?
            .unwrap_or_default();
        Ok(Response::new(BlockId { height, hash }))
    }

    async fn get_block(
        &self,
        request: Request<BlockId>,
    ) -> Result<Response<proto::CompactBlock>, Status> {
        let id = request.into_inner();
        let height = if id.height > 0 {
            id.height
        } else if !id.hash.is_empty() {
            return Err(Status::invalid_argument(
                "get_block by hash not supported yet; use height",
            ));
        } else {
            return Err(Status::invalid_argument("height required"));
        };
        let data = self
            .store
            .get_compact_block(height)
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found(format!("compact block {height} not indexed")))?;
        let block = decode_compact_block(&data).map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(block))
    }

    type GetBlockRangeStream =
        Pin<Box<dyn futures::Stream<Item = Result<proto::CompactBlock, Status>> + Send + 'static>>;

    async fn get_block_range(
        &self,
        request: Request<BlockRange>,
    ) -> Result<Response<Self::GetBlockRangeStream>, Status> {
        let range = request.into_inner();
        let start = range.start.map(|b| b.height).unwrap_or(0);
        let end = range.end.map(|b| b.height).unwrap_or(0);
        if end < start {
            return Err(Status::invalid_argument(format!(
                "end {end} < start {start}"
            )));
        }
        let store = Arc::clone(&self.store);
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        tokio::spawn(async move {
            for height in start..=end {
                let result = match store.get_compact_block(height) {
                    Ok(Some(data)) => match decode_compact_block(&data) {
                        Ok(b) => Ok(b),
                        Err(e) => Err(Status::internal(e.to_string())),
                    },
                    Ok(None) => Err(Status::not_found(format!(
                        "compact block {height} not indexed"
                    ))),
                    Err(e) => Err(Status::internal(e.to_string())),
                };
                if tx.send(result).await.is_err() {
                    break;
                }
            }
        });
        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }

    async fn get_lightd_info(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<LightdInfo>, Status> {
        let node = self.node.read().await.clone();
        let indexed = self
            .store
            .max_height()
            .map_err(|e| Status::internal(e.to_string()))?
            .unwrap_or(0);
        Ok(Response::new(LightdInfo {
            version: self.version.clone(),
            vendor: self.vendor.clone(),
            taddr_support: false,
            chain_name: node.chain.clone(),
            sapling_activation_height: 0,
            consensus_branch_id: String::new(),
            block_height: indexed,
            git_commit: String::new(),
            branch: String::new(),
            build_date: String::new(),
            build_user: String::new(),
            estimated_height: node.blocks,
            zcashd_build: node.build.clone(),
            zcashd_subversion: format!("{} {}", self.node_kind, node.subversion),
            donation_address: String::new(),
        }))
    }

    async fn get_block_nullifiers(
        &self,
        _request: Request<BlockId>,
    ) -> Result<Response<proto::CompactBlock>, Status> {
        Err(unimplemented("GetBlockNullifiers"))
    }

    type GetBlockRangeNullifiersStream = Self::GetBlockRangeStream;

    async fn get_block_range_nullifiers(
        &self,
        _request: Request<BlockRange>,
    ) -> Result<Response<Self::GetBlockRangeNullifiersStream>, Status> {
        Err(unimplemented("GetBlockRangeNullifiers"))
    }

    async fn get_transaction(
        &self,
        _request: Request<proto::TxFilter>,
    ) -> Result<Response<RawTransaction>, Status> {
        Err(unimplemented("GetTransaction"))
    }

    async fn send_transaction(
        &self,
        request: Request<RawTransaction>,
    ) -> Result<Response<SendResponse>, Status> {
        let raw = request.into_inner();
        if raw.data.is_empty() {
            return Ok(Response::new(SendResponse {
                error_code: -1,
                error_message: "empty transaction".into(),
            }));
        }
        let hex_tx = hex::encode(&raw.data);
        match self.rpc.send_raw_transaction(&hex_tx).await {
            Ok(txid) => Ok(Response::new(SendResponse {
                error_code: 0,
                error_message: txid,
            })),
            Err(e) => Ok(Response::new(SendResponse {
                error_code: -1,
                error_message: e.to_string(),
            })),
        }
    }

    type GetTaddressTxidsStream =
        Pin<Box<dyn futures::Stream<Item = Result<RawTransaction, Status>> + Send + 'static>>;

    async fn get_taddress_txids(
        &self,
        _request: Request<proto::TransparentAddressBlockFilter>,
    ) -> Result<Response<Self::GetTaddressTxidsStream>, Status> {
        Err(unimplemented("GetTaddressTxids"))
    }

    type GetTaddressTransactionsStream = Self::GetTaddressTxidsStream;

    async fn get_taddress_transactions(
        &self,
        _request: Request<proto::TransparentAddressBlockFilter>,
    ) -> Result<Response<Self::GetTaddressTransactionsStream>, Status> {
        Err(unimplemented("GetTaddressTransactions"))
    }

    async fn get_taddress_balance(
        &self,
        _request: Request<proto::AddressList>,
    ) -> Result<Response<proto::Balance>, Status> {
        Err(unimplemented("GetTaddressBalance"))
    }

    async fn get_taddress_balance_stream(
        &self,
        _request: Request<tonic::Streaming<proto::Address>>,
    ) -> Result<Response<proto::Balance>, Status> {
        Err(unimplemented("GetTaddressBalanceStream"))
    }

    type GetMempoolTxStream =
        Pin<Box<dyn futures::Stream<Item = Result<proto::CompactTx, Status>> + Send + 'static>>;

    async fn get_mempool_tx(
        &self,
        _request: Request<proto::Exclude>,
    ) -> Result<Response<Self::GetMempoolTxStream>, Status> {
        Err(unimplemented("GetMempoolTx"))
    }

    type GetMempoolStreamStream = Self::GetTaddressTxidsStream;

    async fn get_mempool_stream(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::GetMempoolStreamStream>, Status> {
        Err(unimplemented("GetMempoolStream"))
    }

    async fn get_tree_state(
        &self,
        request: Request<BlockId>,
    ) -> Result<Response<TreeState>, Status> {
        let id = request.into_inner();
        let height = if id.height > 0 {
            id.height
        } else {
            self.store
                .max_height()
                .map_err(|e| Status::internal(e.to_string()))?
                .ok_or_else(|| Status::unavailable("no indexed tip for treestate"))?
        };
        let network = self.node.read().await.chain.clone();
        let ts = fetch_tree_state(&self.rpc, &network, height)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ts))
    }

    async fn get_latest_tree_state(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<TreeState>, Status> {
        let height = match self
            .store
            .max_height()
            .map_err(|e| Status::internal(e.to_string()))?
        {
            Some(h) => h,
            None => self
                .rpc
                .get_block_count()
                .await
                .map_err(|e| Status::unavailable(e.to_string()))?,
        };
        let network = self.node.read().await.chain.clone();
        let ts = fetch_tree_state(&self.rpc, &network, height)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ts))
    }

    type GetSubtreeRootsStream =
        Pin<Box<dyn futures::Stream<Item = Result<proto::SubtreeRoot, Status>> + Send + 'static>>;

    async fn get_subtree_roots(
        &self,
        request: Request<GetSubtreeRootsArg>,
    ) -> Result<Response<Self::GetSubtreeRootsStream>, Status> {
        let arg = request.into_inner();
        let roots = fetch_subtree_roots(
            &self.rpc,
            arg.shielded_protocol,
            arg.start_index,
            arg.max_entries,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        tokio::spawn(async move {
            for root in roots {
                if tx.send(Ok(root)).await.is_err() {
                    break;
                }
            }
        });
        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }

    async fn get_address_utxos(
        &self,
        _request: Request<GetAddressUtxosArg>,
    ) -> Result<Response<GetAddressUtxosReplyList>, Status> {
        Err(unimplemented("GetAddressUtxos"))
    }

    type GetAddressUtxosStreamStream = Pin<
        Box<
            dyn futures::Stream<Item = Result<proto::GetAddressUtxosReply, Status>>
                + Send
                + 'static,
        >,
    >;

    async fn get_address_utxos_stream(
        &self,
        _request: Request<GetAddressUtxosArg>,
    ) -> Result<Response<Self::GetAddressUtxosStreamStream>, Status> {
        Err(unimplemented("GetAddressUtxosStream"))
    }

    async fn ping(
        &self,
        _request: Request<proto::Duration>,
    ) -> Result<Response<PingResponse>, Status> {
        Ok(Response::new(PingResponse { entry: 0, exit: 0 }))
    }
}
