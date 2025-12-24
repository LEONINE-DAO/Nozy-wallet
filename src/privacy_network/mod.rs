// Privacy Network Module
// Provides Tor/I2P proxy support for all network connections

pub mod tor;
pub mod i2p;
pub mod proxy;

pub use proxy::{PrivacyProxy, PrivacyNetwork, ProxyConfig};
pub use tor::TorProxy;
pub use i2p::I2PProxy;
