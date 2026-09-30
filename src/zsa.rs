//! Zcash Shielded Assets on the dedicated public ZSA testnet.
//!
//! This is **not** Zcash mainnet or regular testnet. NU 6.2 is active; Ironwood
//! is not. Unified addresses use the regtest HRP `uregtest1`.
//!
//! Status / addresses talk to lightwalletd with the existing Zeaking client and
//! encode UAs with the current key stack. Issue / transfer / asset scan need an
//! isolated OrchardZSA crate (ZIP 226/227) so we do not replace the Ironwood
//! orchard pin used by mainnet Nozy.

use crate::error::{NozyError, NozyResult};
use crate::hd_wallet::HDWallet;
use serde::Serialize;
use zcash_protocol::consensus::NetworkType;

/// Public ZSA testnet compact-block / gRPC endpoint.
pub const DEFAULT_LWD_URL: &str = "https://zsa.methyl.cc";
/// Test tZEC faucet (no real value).
pub const FAUCET_URL: &str = "https://faucet.zsa.methyl.cc";
pub const NETWORK_NAME: &str = "zsa-testnet";
pub const ADDRESS_PREFIX: &str = "uregtest1";

const LWD_CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(20);

/// Override with `NOZY_ZSA_LWD`. Does not inherit `LIGHTWALLETD_GRPC` (that is
/// the operator's mainnet/testnet LWD and must not mix chains).
pub fn resolve_lwd_url() -> String {
    std::env::var("NOZY_ZSA_LWD").unwrap_or_else(|_| DEFAULT_LWD_URL.to_string())
}

fn orchard_zsa_tx_ready() -> NozyResult<()> {
    Err(NozyError::InvalidOperation(
        "OrchardZSA issue/transfer is not wired in this binary yet. Status/addresses work against \
         https://zsa.methyl.cc. Build the isolated crate: cd nozy-zsa && cargo run --release -- status \
         (QEDIT orchard pins — keeps Ironwood orchard 0.15 untouched)."
            .to_string(),
    ))
}

#[derive(Debug, Clone, Serialize)]
pub struct ZsaStatusSnapshot {
    pub network: &'static str,
    pub lwd_url: String,
    pub faucet_url: &'static str,
    pub address_prefix: &'static str,
    pub connected: bool,
    pub chain_name: Option<String>,
    pub block_height: Option<u64>,
    pub estimated_height: Option<u64>,
    pub lwd_version: Option<String>,
    pub backend: Option<String>,
    pub tip_orchard_actions: Option<usize>,
    pub error: Option<String>,
}

impl ZsaStatusSnapshot {
    pub fn print(&self) {
        println!("ZSA testnet status");
        println!("   Network:        {}", self.network);
        println!("   lightwalletd:   {}", self.lwd_url);
        println!("   Address prefix: {}", self.address_prefix);
        println!("   Faucet:         {}", self.faucet_url);
        if self.connected {
            println!(
                "   Chain:          {}",
                self.chain_name.as_deref().unwrap_or("(unknown)")
            );
            match self.block_height {
                Some(h) => println!("   Block height:   {h}"),
                None => println!("   Block height:   (unknown)"),
            }
            if let Some(v) = &self.lwd_version {
                println!("   LWD version:    {v}");
            }
            if let Some(b) = &self.backend {
                if !b.is_empty() {
                    println!("   Backend:        {b}");
                }
            }
            if let Some(n) = self.tip_orchard_actions {
                println!("   Tip Orchard actions (compact): {n}");
            }
            println!();
            println!("   Next:");
            println!("     nozy zsa addresses");
            println!("     faucet: {}", self.faucet_url);
            println!("     nozy zsa balance");
            println!("     nozy zsa issue / transfer  (OrchardZSA crate — next slice)");
        } else {
            println!(
                "   Connected:      no ({})",
                self.error.as_deref().unwrap_or("unknown error")
            );
        }
    }

    pub fn to_json_string(&self) -> NozyResult<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| NozyError::InvalidOperation(format!("JSON encode: {e}")))
    }
}

/// Probe the public ZSA lightwalletd (`GetLightdInfo` + one compact tip block).
pub async fn probe_status(lwd_url: &str) -> ZsaStatusSnapshot {
    let mut snap = ZsaStatusSnapshot {
        network: NETWORK_NAME,
        lwd_url: lwd_url.to_string(),
        faucet_url: FAUCET_URL,
        address_prefix: ADDRESS_PREFIX,
        connected: false,
        chain_name: None,
        block_height: None,
        estimated_height: None,
        lwd_version: None,
        backend: None,
        tip_orchard_actions: None,
        error: None,
    };

    let connect = tokio::time::timeout(
        LWD_CONNECT_TIMEOUT,
        zeaking::lwd::connect_lightwalletd(lwd_url),
    )
    .await;

    let mut client = match connect {
        Ok(Ok(c)) => c,
        Ok(Err(e)) => {
            snap.error = Some(e.to_string());
            return snap;
        }
        Err(_) => {
            snap.error = Some(format!(
                "connect timed out after {}s ({lwd_url})",
                LWD_CONNECT_TIMEOUT.as_secs()
            ));
            return snap;
        }
    };

    use zeaking::lwd::proto::{BlockId, BlockRange, Empty};

    let info = match client.get_lightd_info(Empty {}).await {
        Ok(r) => r.into_inner(),
        Err(e) => {
            snap.error = Some(format!("GetLightdInfo: {e}"));
            return snap;
        }
    };

    snap.connected = true;
    snap.chain_name = Some(info.chain_name);
    snap.block_height = Some(info.block_height);
    snap.estimated_height = Some(info.estimated_height);
    snap.lwd_version = Some(info.version);
    if !info.zcashd_subversion.is_empty() {
        snap.backend = Some(info.zcashd_subversion);
    }

    if info.block_height > 0 {
        let range = BlockRange {
            start: Some(BlockId {
                height: info.block_height,
                hash: vec![],
            }),
            end: Some(BlockId {
                height: info.block_height,
                hash: vec![],
            }),
        };
        match client.get_block_range(range).await {
            Ok(stream) => {
                let mut stream = stream.into_inner();
                let mut actions = 0usize;
                while let Ok(Some(block)) = stream.message().await {
                    for tx in &block.vtx {
                        actions += tx.actions.len();
                    }
                }
                snap.tip_orchard_actions = Some(actions);
            }
            Err(e) => {
                snap.error = Some(format!("GetBlockRange at tip: {e}"));
            }
        }
    }

    snap
}

pub fn print_addresses(wallet: &HDWallet, accounts: u32) -> NozyResult<()> {
    let n = accounts.max(1);
    println!("ZSA testnet addresses ({ADDRESS_PREFIX}…, {NETWORK_NAME})");
    println!("   LWD:    {}", resolve_lwd_url());
    println!("   Faucet: {FAUCET_URL}");
    println!();
    for account in 0..n {
        let addr = wallet.generate_orchard_address(account, 0, NetworkType::Regtest)?;
        if !addr.starts_with(ADDRESS_PREFIX) {
            return Err(NozyError::InvalidOperation(format!(
                "expected {ADDRESS_PREFIX} prefix, got {addr}"
            )));
        }
        println!("   account {account}: {addr}");
    }
    println!();
    println!("   Fund with test tZEC at {FAUCET_URL}");
    println!("   Custom-asset scan/issue still needs the OrchardZSA crate.");
    Ok(())
}

pub async fn print_balance(json: bool) -> NozyResult<()> {
    let url = resolve_lwd_url();
    let snap = probe_status(&url).await;
    if json {
        let mut v = serde_json::to_value(&snap)
            .map_err(|e| NozyError::InvalidOperation(format!("JSON encode: {e}")))?;
        if let Some(obj) = v.as_object_mut() {
            obj.insert(
                "assets".to_string(),
                serde_json::json!([{
                    "asset": "tZEC",
                    "balance": 0,
                    "note": "ZSA compact-note decrypt is not wired (vanilla orchard cannot scan OrchardZSA notes)."
                }]),
            );
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&v)
                .map_err(|e| NozyError::InvalidOperation(format!("JSON encode: {e}")))?
        );
        return Ok(());
    }

    println!("ZSA testnet balance");
    println!("   lightwalletd: {}", snap.lwd_url);
    if snap.connected {
        println!(
            "   Chain tip:    {}",
            snap.block_height
                .map(|h| h.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        );
    } else {
        println!(
            "   Chain:        unreachable ({})",
            snap.error.as_deref().unwrap_or("unknown")
        );
    }
    println!("   tZEC:         (not scanned — OrchardZSA note decrypt not wired)");
    println!("   Custom assets: none scanned");
    println!();
    println!("   Addresses: nozy zsa addresses");
    println!("   Faucet:    {FAUCET_URL}");
    Ok(())
}

pub fn issue_asset(_name: &str, amount: u64) -> NozyResult<()> {
    if amount == 0 {
        return Err(NozyError::InvalidOperation(
            "amount must be > 0 asset units".to_string(),
        ));
    }
    orchard_zsa_tx_ready()
}

pub fn transfer_asset(recipient: &str, amount: u64) -> NozyResult<()> {
    if amount == 0 {
        return Err(NozyError::InvalidOperation(
            "amount must be > 0".to_string(),
        ));
    }
    let r = recipient.trim();
    if !r.starts_with(ADDRESS_PREFIX) {
        return Err(NozyError::AddressParsing(format!(
            "ZSA testnet recipients must start with {ADDRESS_PREFIX} (got {recipient})"
        )));
    }
    orchard_zsa_tx_ready()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_lwd_is_public_zsa_testnet() {
        assert_eq!(DEFAULT_LWD_URL, "https://zsa.methyl.cc");
        assert!(FAUCET_URL.contains("faucet.zsa.methyl.cc"));
        assert_eq!(ADDRESS_PREFIX, "uregtest1");
    }

    #[test]
    fn resolve_lwd_url_ignores_missing_override() {
        let prev = std::env::var("NOZY_ZSA_LWD").ok();
        std::env::remove_var("NOZY_ZSA_LWD");
        assert_eq!(resolve_lwd_url(), DEFAULT_LWD_URL);
        if let Some(v) = prev {
            std::env::set_var("NOZY_ZSA_LWD", v);
        }
    }

    #[test]
    fn transfer_rejects_mainnet_ua() {
        let err = transfer_asset("u1thisisnotazsatestnetaddress", 1).unwrap_err();
        assert!(err.to_string().contains(ADDRESS_PREFIX));
    }

    #[test]
    fn issue_rejects_zero_amount() {
        let err = issue_asset("demo", 0).unwrap_err();
        assert!(err.to_string().contains("amount"));
    }

    #[test]
    fn regtest_unified_address_uses_uregtest_prefix() {
        let wallet = HDWallet::new().expect("wallet");
        let addr = wallet
            .generate_orchard_address(0, 0, NetworkType::Regtest)
            .expect("ua");
        assert!(
            addr.starts_with(ADDRESS_PREFIX),
            "expected {ADDRESS_PREFIX} prefix, got {addr}"
        );
    }
}
