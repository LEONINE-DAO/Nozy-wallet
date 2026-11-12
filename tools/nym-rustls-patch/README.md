# Nym rustls 0.23 patches

Nym 1.21 still depends on `tungstenite` 0.20 and `tendermint-rpc` 0.40.4.
Those crates enable `rustls` 0.21 (`tungstenite` / `tokio-tungstenite`) and
`reqwest` 0.11 (`tendermint-rpc`'s `http-client` feature). GitHub's rustls
advisories cover every release before 0.23.46, and there is no 0.21 backport.

These path patches keep the crate versions Nym requires and move the TLS
stack onto `rustls` 0.23.46:

- `tungstenite` and `tokio-tungstenite` 0.20.1, with the rustls connector updated for 0.23
- `tendermint-rpc` 0.40.4, with the optional HTTP client on `reqwest` 0.12

Drop the patches when a Nym release no longer pulls rustls older than 0.23.46.

`nym-smolmix` 1.21.6 is pinned exactly, along with the other 1.21 crates it
builds against. A caret requirement also matches the 1.22 crates, and that
mix does not compile. The dVPN spike stays on its existing `develop` git
revision and only replaces these three crates.
