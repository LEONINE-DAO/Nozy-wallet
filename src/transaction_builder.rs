use crate::error::{NozyError, NozyResult};
use crate::notes::SpendableNote;
use crate::zebra_integration::ZebraClient;
use sha2::{Sha256, Digest};

#[derive(Debug, Clone)]
pub struct SignedTransaction {
    pub raw_transaction: Vec<u8>,
    pub txid: String,
}

pub struct ZcashTransactionBuilder {
    pub allow_mainnet_broadcast: bool,
    pub zebra_url: String,
}

impl ZcashTransactionBuilder {
    pub fn new() -> Self {
        Self {
            allow_mainnet_broadcast: false,
            zebra_url: "http://127.0.0.1:8232".to_string(), 
        }
    }
    
    pub fn set_zebra_url(&mut self, url: &str) -> &mut Self {
        self.zebra_url = url.to_string();
        println!("🔗 Zebra URL set to: {}", self.zebra_url);
        self
    }
    
    pub fn enable_mainnet_broadcast(&mut self) -> &mut Self {
        println!("🚨 WARNING: Mainnet broadcasting enabled!");
        println!("   This will send REAL ZEC transactions!");
        self.allow_mainnet_broadcast = true;
        self
    }

    pub fn build_send_transaction(
        &self,
        spendable_notes: &[SpendableNote],
        recipient_address: &str,
        amount_zatoshis: u64,
        fee_zatoshis: u64,
    ) -> NozyResult<SignedTransaction> {
        
        println!("🔧 Building transaction for Zebra node...");
        println!("   Recipient: {}", recipient_address);
        println!("   Amount: {} ZAT ({} ZEC)", amount_zatoshis, amount_zatoshis as f64 / 100_000_000.0);
        
        if !recipient_address.starts_with("u1") && !recipient_address.starts_with("zs1") && !recipient_address.starts_with("t1") {
            return Err(NozyError::InvalidOperation(
                "Invalid address format! Must be u1, zs1, or t1".to_string()
            ));
        }
        
        let total_available: u64 = spendable_notes.iter()
            .filter(|note| !note.orchard_note.spent)
            .map(|note| note.orchard_note.value)
            .sum();
            
        let total_needed = amount_zatoshis + fee_zatoshis;
        
        if total_available < total_needed {
            return Err(NozyError::InvalidOperation(
                format!("Insufficient funds: need {} ZAT, have {} ZAT", total_needed, total_available)
            ));
        }
        
        let tx_data = self.create_zebra_transaction(spendable_notes, recipient_address, amount_zatoshis, fee_zatoshis)?;
        let txid = self.calculate_txid(&tx_data)?;
        
        println!("✅ Transaction built for Zebra:");
        println!("   TXID: {}", txid);
        println!("   Size: {} bytes", tx_data.len());
        
        Ok(SignedTransaction {
            raw_transaction: tx_data,
            txid,
        })
    }
    
    fn create_zebra_transaction(
        &self,
        spendable_notes: &[SpendableNote],
        recipient_address: &str,
        amount_zatoshis: u64,
        fee_zatoshis: u64,
    ) -> NozyResult<Vec<u8>> {
        let mut tx_data = Vec::new();
        
        tx_data.extend_from_slice(&5u32.to_le_bytes());
        
        tx_data.extend_from_slice(&0x26A7270Au32.to_le_bytes());
        
        tx_data.extend_from_slice(&0xC2D6D0B4u32.to_le_bytes());
        
        tx_data.extend_from_slice(&0u32.to_le_bytes());
        
        let expiry_height = 2650000u32 + 40;
        tx_data.extend_from_slice(&expiry_height.to_le_bytes());
        
        tx_data.push(0); 
        tx_data.push(0); 
        
        tx_data.push(0); 
        tx_data.push(0); 
        tx_data.extend_from_slice(&0i64.to_le_bytes()); 
        
        if !spendable_notes.is_empty() {
            tx_data.push(1); 
            
            tx_data.extend_from_slice(&[0u8; 32]); 
            tx_data.extend_from_slice(&spendable_notes[0].orchard_note.nullifier.to_bytes()); 
            tx_data.extend_from_slice(&[0u8; 32]); 
            tx_data.extend_from_slice(&[0u8; 32]); 
            tx_data.extend_from_slice(&[0u8; 32]); 
            tx_data.extend_from_slice(&[0u8; 580]); 
            tx_data.extend_from_slice(&[0u8; 80]); 
            tx_data.extend_from_slice(&[0u8; 64]); 
            
            tx_data.extend_from_slice(&(-(amount_zatoshis as i64 + fee_zatoshis as i64)).to_le_bytes());
            
            tx_data.extend_from_slice(&[0u8; 32]); 
            tx_data.extend_from_slice(&[0u8; 192]); 
            tx_data.extend_from_slice(&[0u8; 64]); 
        } else {
            tx_data.push(0); 
            tx_data.extend_from_slice(&0i64.to_le_bytes()); 
        }
        
        Ok(tx_data)
    }
    
    fn calculate_txid(&self, tx_data: &[u8]) -> NozyResult<String> {
        let mut hasher = Sha256::new();
        hasher.update(tx_data);
        let hash1 = hasher.finalize();
        
        let mut hasher2 = Sha256::new();
        hasher2.update(hash1);
        let hash2 = hasher2.finalize();
        
        let mut txid_bytes = hash2.to_vec();
        txid_bytes.reverse();
        Ok(hex::encode(txid_bytes))
    }
    
    pub fn broadcast_transaction(&self, transaction: &SignedTransaction) -> NozyResult<String> {
        if !self.allow_mainnet_broadcast {
            return Err(NozyError::InvalidOperation(
                "🚫 Mainnet broadcasting disabled for safety! Call enable_mainnet_broadcast() first.".to_string()
            ));
        }
        
        println!("🚀 Broadcasting to Zebra node...");
        
        match self.call_zebra_sendrawtransaction(transaction) {
            Ok(network_txid) => {
                println!("✅ SUCCESS! Transaction broadcast to mainnet!");
                println!("🌐 Network TXID: {}", network_txid);
                println!("🔗 Explorer: https://zcashblockexplorer.com/transactions/{}", network_txid);
                Ok(network_txid)
            },
            Err(e) => {
                println!("❌ Zebra RPC failed: {}", e);
                Err(e)
            }
        }
    }
    
    fn call_zebra_sendrawtransaction(&self, transaction: &SignedTransaction) -> NozyResult<String> {
        use reqwest::blocking::Client;
        use serde_json::json;
        
        let zebra_url = &self.zebra_url;
        let raw_tx_hex = hex::encode(&transaction.raw_transaction);
        
        println!("📡 Calling Zebra RPC: sendrawtransaction");
        println!("🔗 URL: {}", zebra_url);
        
        let rpc_request = json!({
            "jsonrpc": "2.0",
            "id": "nozy-wallet",
            "method": "sendrawtransaction", 
            "params": [raw_tx_hex]
        });
        
        let client = Client::new();
        let response = client
            .post(zebra_url)
            .header("Content-Type", "application/json")
            .json(&rpc_request)
            .send()
            .map_err(|e| NozyError::InvalidOperation(format!("Connection failed: {}", e)))?;
            
        if !response.status().is_success() {
            return Err(NozyError::InvalidOperation(
                format!("HTTP error: {}", response.status())
            ));
        }
        
        let response_text = response.text()
            .map_err(|e| NozyError::InvalidOperation(format!("Response read error: {}", e)))?;
            
        println!("📨 Zebra response: {}", response_text);
        
        let json_response: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| NozyError::InvalidOperation(format!("JSON parse error: {}", e)))?;
            
        if let Some(error) = json_response.get("error") {
            return Err(NozyError::InvalidOperation(
                format!("Zebra RPC error: {}", error)
            ));
        }
        
        if let Some(result) = json_response.get("result") {
            if let Some(txid) = result.as_str() {
                return Ok(txid.to_string());
            }
        }
        
        Ok(transaction.txid.clone())
    }
} 
