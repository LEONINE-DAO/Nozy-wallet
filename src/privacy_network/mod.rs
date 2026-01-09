// Privacy Network Module
// Provides Tor/I2P proxy support for all network connections

pub mod i2p;
pub mod proxy;
pub mod tor;

pub use i2p::I2PProxy;
pub use proxy::{PrivacyNetwork, PrivacyProxy, ProxyConfig};
pub use tor::TorProxy;
