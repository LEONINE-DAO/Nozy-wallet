use crate::error::{NozyError, NozyResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ZebraClient {
    client: Client,
    base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    pub height: u32,
    pub hash: String,
    pub time: u64,
    pub transactions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub txid: String,
    pub block_height: u32,
    pub orchard_actions: Vec<OrchardAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchardAction {
    pub commitment: Option<String>,
    pub nullifier: Option<String>,
    pub value: u64,
    pub recipient: String,
    pub encrypted_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub chain: String,
    pub blocks: u32,
    pub difficulty: f64,
    pub networkactive: bool,
}

impl ZebraClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn get_network_info(&self) -> NozyResult<NetworkInfo> {
        let response = self.make_rpc_call("getblockchaininfo", json!([])).await?;
        
        Ok(NetworkInfo {
            chain: response["chain"].as_str().unwrap_or("unknown").to_string(),
            blocks: response["blocks"].as_u64().unwrap_or(0) as u32,
            difficulty: response["difficulty"].as_f64().unwrap_or(0.0),
            networkactive: response["networkactive"].as_bool().unwrap_or(false),
        })
    }

    pub async fn get_block(&self, height: u32) -> NozyResult<BlockInfo> {
        let response = self.make_rpc_call("getblockhash", json!([height])).await?;
        let block_hash = response.as_str()
            .ok_or_else(|| NozyError::Network("Invalid block hash response".to_string()))?;

        let block_response = self.make_rpc_call("getblock", json!([block_hash, 2])).await?;
        
        let transactions: Vec<String> = block_response["tx"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|tx| tx.as_str().unwrap_or("").to_string())
            .collect();

        Ok(BlockInfo {
            height,
            hash: block_hash.to_string(),
            time: block_response["time"].as_u64().unwrap_or(0),
            transactions,
        })
    }

    pub async fn get_transaction(&self, txid: &str) -> NozyResult<TransactionInfo> {
        let response = self.make_rpc_call("getrawtransaction", json!([txid, true])).await?;
        
        let mut orchard_actions = Vec::new();
        
        // Parse Orchard actions from transaction
        if let Some(vjoinsplit) = response["vjoinsplit"].as_array() {
            for js in vjoinsplit {
                if let Some(orchard) = js["orchard"].as_object() {
                    if let Some(actions) = orchard["actions"].as_array() {
                        for action in actions {
                            let orchard_action = OrchardAction {
                                commitment: action["cmx"].as_str().map(|s| s.to_string()),
                                nullifier: action["nullifier"].as_str().map(|s| s.to_string()),
                                value: action["value"].as_u64().unwrap_or(0),
                                recipient: action["recipient"].as_str().unwrap_or("").to_string(),
                                encrypted_note: action["encrypted_note"].as_str().map(|s| s.to_string()),
                            };
                            orchard_actions.push(orchard_action);
                        }
                    }
                }
            }
        }

        Ok(TransactionInfo {
            txid: txid.to_string(),
            block_height: response["blockheight"].as_u64().unwrap_or(0) as u32,
            orchard_actions,
        })
    }

    pub async fn get_best_block_height(&self) -> NozyResult<u32> {
        let response = self.make_rpc_call("getblockcount", json!([])).await?;
        Ok(response.as_u64().unwrap_or(0) as u32)
    }

    pub async fn send_raw_transaction(&self, hex: &str) -> NozyResult<String> {
        let response = self.make_rpc_call("sendrawtransaction", json!([hex])).await?;
        
        let txid = response.as_str()
            .ok_or_else(|| NozyError::Network("Invalid transaction ID response".to_string()))?;
        
        Ok(txid.to_string())
    }

    pub async fn get_mempool_info(&self) -> NozyResult<HashMap<String, Value>> {
        let response = self.make_rpc_call("getmempoolinfo", json!([])).await?;
        
        let mut info = HashMap::new();
        if let Some(obj) = response.as_object() {
            for (key, value) in obj {
                info.insert(key.clone(), value.clone());
            }
        }
        
        Ok(info)
    }

    pub async fn get_peer_info(&self) -> NozyResult<Vec<HashMap<String, Value>>> {
        let response = self.make_rpc_call("getpeerinfo", json!([])).await?;
        
        let mut peers = Vec::new();
        if let Some(array) = response.as_array() {
            for peer in array {
                if let Some(obj) = peer.as_object() {
                    let mut peer_info = HashMap::new();
                    for (key, value) in obj {
                        peer_info.insert(key.clone(), value.clone());
                    }
                    peers.push(peer_info);
                }
            }
        }
        
        Ok(peers)
    }

    async fn make_rpc_call(&self, method: &str, params: Value) -> NozyResult<Value> {
        let request_body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });

        let response = self.client
            .post(&self.base_url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| NozyError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(NozyError::Network(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let response_json: Value = response
            .json()
            .await
            .map_err(|e| NozyError::Network(format!("Failed to parse response: {}", e)))?;

        if let Some(error) = response_json.get("error") {
            if !error.is_null() {
                return Err(NozyError::Network(format!(
                    "RPC error: {}",
                    error
                )));
            }
        }

        response_json.get("result")
            .ok_or_else(|| NozyError::Network("No result in response".to_string()))
            .map(|v| v.clone())
    }

    pub async fn test_connection(&self) -> NozyResult<()> {
        let _info = self.get_network_info().await?;
        println!("✅ Successfully connected to Zebra node");
        Ok(())
    }
}
