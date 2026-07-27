use std::error::Error as StdError;

use http::Uri;
use hyper::rt::{Read as HyperRead, Write as HyperWrite};
use tonic::transport::{Channel, Endpoint};
use tower::Service;

use super::proto::compact_tx_streamer_client::CompactTxStreamerClient;
use crate::error::{ZeakingError, ZeakingResult};

pub type LwdClient = CompactTxStreamerClient<Channel>;

/// Normalize a lightwalletd base URL for tonic (`http://` / `https://` required).
pub fn normalize_lwd_uri(grpc_base: &str) -> String {
    let uri = grpc_base.trim().trim_end_matches('/');
    if uri.starts_with("http://") || uri.starts_with("https://") {
        uri.to_string()
    } else {
        format!("http://{uri}")
    }
}

/// Connect to a lightwalletd gRPC endpoint (e.g. `http://127.0.0.1:9067`).
///
/// Default path: direct tonic transport (local LWD / clearnet). For Nym
/// smoldvpn / other custom dialers see [`connect_lightwalletd_with_connector`]
/// (issue #146 / track C5).
pub async fn connect_lightwalletd(grpc_base: &str) -> ZeakingResult<LwdClient> {
    let uri = normalize_lwd_uri(grpc_base);
    let uri_for_err = uri.clone();
    CompactTxStreamerClient::connect(uri)
        .await
        .map_err(move |e| ZeakingError::Grpc(format!("connect to {uri_for_err}: {e}")))
}

/// Connect using a caller-supplied tonic connector (e.g. smoldvpn `tunnel.connector()`).
///
/// For `https://` LWD the connector must already perform TLS (ALPN `h2`), as in
/// `tools/nym-dvpn-lwd-spike` `TlsWrap::h2(tunnel.connector())`. For `http://`
/// pass the raw TCP connector (typically `TokioIo`-wrapped).
///
/// Keeps Nym / smoldvpn **out** of default `zeaking` deps — the wallet or spike
/// owns tunnel lifecycle and destination split (sync LWD ≠ mixnet submit).
pub async fn connect_lightwalletd_with_connector<C>(
    grpc_base: &str,
    connector: C,
) -> ZeakingResult<LwdClient>
where
    C: Service<Uri> + Send + 'static,
    C::Response: HyperRead + HyperWrite + Send + Unpin + 'static,
    C::Future: Send + 'static,
    C::Error: Into<Box<dyn StdError + Send + Sync>> + Send + Sync + 'static,
    Box<dyn StdError + Send + Sync>: From<C::Error>,
{
    let uri = normalize_lwd_uri(grpc_base);
    let uri_for_err = uri.clone();
    let endpoint = Endpoint::from_shared(uri)
        .map_err(|e| ZeakingError::Grpc(format!("invalid lightwalletd URI {uri_for_err}: {e}")))?;
    let channel = endpoint
        .connect_with_connector(connector)
        .await
        .map_err(move |e| ZeakingError::Grpc(format!("connect to {uri_for_err}: {e}")))?;
    Ok(CompactTxStreamerClient::new(channel))
}

#[cfg(test)]
mod tests {
    use super::normalize_lwd_uri;

    #[test]
    fn normalize_adds_http_when_missing() {
        assert_eq!(normalize_lwd_uri("127.0.0.1:9067"), "http://127.0.0.1:9067");
    }

    #[test]
    fn normalize_preserves_https_and_strips_trailing_slash() {
        assert_eq!(
            normalize_lwd_uri("https://zec.rocks:443/"),
            "https://zec.rocks:443"
        );
    }
}
