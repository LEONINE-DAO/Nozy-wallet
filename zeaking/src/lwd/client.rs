use tonic::transport::Channel;

use super::proto::compact_tx_streamer_client::CompactTxStreamerClient;
use crate::error::{ZeakingError, ZeakingResult};

pub type LwdClient = CompactTxStreamerClient<Channel>;

/// Connect to a lightwalletd gRPC endpoint (e.g. `http://127.0.0.1:9067`).
///
/// Follow-up (issue #146): add `connect_lightwalletd_with_connector` for
/// tonic `Endpoint::connect_with_connector` so compact sync can ride Nym
/// smol-dvpn / smolmix transports. Spike lives in `tools/nym-dvpn-lwd-spike`.
pub async fn connect_lightwalletd(grpc_base: &str) -> ZeakingResult<LwdClient> {
    let uri = grpc_base.trim_end_matches('/');
    let uri = if uri.starts_with("http://") || uri.starts_with("https://") {
        uri.to_string()
    } else {
        format!("http://{uri}")
    };
    let uri_for_err = uri.clone();
    CompactTxStreamerClient::connect(uri)
        .await
        .map_err(move |e| ZeakingError::Grpc(format!("connect to {uri_for_err}: {e}")))
}
