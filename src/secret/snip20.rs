// SNIP-20 Token Support
// Implements SNIP-20 standard for Secret Network tokens (including Shade tokens)

use crate::error::{NozyError, NozyResult};
use crate::secret::rpc_client::SecretRpcClient;
use serde_json::{json, Value};

pub struct Snip20Token {
    contract_address: String,
    rpc: SecretRpcClient,
}

impl Snip20Token {
    /// Create new SNIP-20 token interface
    pub fn new(contract_address: String, rpc: SecretRpcClient) -> Self {
        Self {
            contract_address,
            rpc,
        }
    }
    
    /// Get token info (name, symbol, decimals)
    pub async fn get_token_info(&self) -> NozyResult<TokenInfo> {
        let query = json!({
            "token_info": {}
        });
        
        let response = self.rpc.query_contract(&self.contract_address, query).await?;
        
        // Parse response (Secret Network encrypts responses, but LCD API handles decryption)
        let data = response.get("data")
            .ok_or_else(|| NozyError::NetworkError("No data in response".to_string()))?;
        
        Ok(TokenInfo {
            name: data.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            symbol: data.get("symbol")
                .and_then(|v| v.as_str())
                .unwrap_or("UNK")
                .to_string(),
            decimals: data.get("decimals")
                .and_then(|v| v.as_u64())
                .map(|n| n as u8)
                .unwrap_or(6),
        })
    }
    
    /// Get balance for an address
    pub async fn get_balance(&self, address: &str) -> NozyResult<u128> {
        let query = json!({
            "balance": {
                "address": address
            }
        });
        
        let response = self.rpc.query_contract(&self.contract_address, query).await?;
        
        let data = response.get("data")
            .ok_or_else(|| NozyError::NetworkError("No data in response".to_string()))?;
        
        let amount = data.get("amount")
            .and_then(|v| v.as_str())
            .ok_or_else(|| NozyError::NetworkError("Invalid balance response".to_string()))?;
        
        amount.parse::<u128>()
            .map_err(|_| NozyError::NetworkError("Invalid balance format".to_string()))
    }
    
    /// Transfer tokens
    pub async fn transfer(
        &self,
        sender: &str,
        recipient: &str,
        amount: u128,
        memo: Option<String>,
    ) -> NozyResult<String> {
        let mut transfer_msg = json!({
            "transfer": {
                "recipient": recipient,
                "amount": amount.to_string()
            }
        });
        
        if let Some(m) = memo {
            transfer_msg["transfer"]["memo"] = json!(m);
        }
        
        let response = self.rpc.execute_contract(
            &self.contract_address,
            sender,
            transfer_msg,
            None,
        ).await?;
        
        // Extract transaction hash from response
        let tx_hash = response.get("txhash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| NozyError::NetworkError("No transaction hash in response".to_string()))?;
        
        Ok(tx_hash.to_string())
    }
    
    /// Get contract address
    pub fn contract_address(&self) -> &str {
        &self.contract_address
    }
}

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

// Common Shade Protocol token addresses (mainnet)
pub mod shade_tokens {
    pub const SHD: &str = "secret1qfql357amn448duf5gvp9gr48sxx9tsnhupu3d"; // Shade token
    pub const SILK: &str = "secret1fl449muk5yq8dlad7a22nje4p5d2pnsgymhjfd"; // Silk token
}
