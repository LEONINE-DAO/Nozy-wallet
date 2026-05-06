use tonic::transport::Channel;

use super::proto::compact_tx_streamer_client::CompactTxStreamerClient;
use crate::error::{ZeakingError, ZeakingResult};

pub type LwdClient = CompactTxStreamerClient<Channel>;

/// Connect to a lightwalletd gRPC endpoint (e.g. `http://127.0.0.1:9067`).
pub async fn connect_lightwalletd(grpc_base: &str) -> ZeakingResult<LwdClient> {
    let uri = grpc_base.trim_end_matches('/');
    let uri = if uri.starts_with("http://") || uri.starts_with("https://") {
        uri.to_string()
    } else {
        format!("http://{uri}")
    };
    CompactTxStreamerClient::connect(uri)
        .await
        .map_err(|e| ZeakingError::Network(format!("lightwalletd connect: {e}")))
}
