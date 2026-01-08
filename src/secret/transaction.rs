// Secret Network Transaction Builder
// Implements Cosmos SDK transaction construction and signing for Secret Network

use crate::error::{NozyError, NozyResult};
use crate::secret::rpc_client::SecretRpcClient;
use crate::secret_keys::SecretKeyPair;
use cosmrs::{
    tx::{Body, SignDoc, SignerInfo, AuthInfo, Fee},
    Coin, Denom,
};
use cosmrs::crypto::secp256k1::SigningKey;
use cosmrs::crypto::PublicKey;
use cosmrs::tendermint::chain::Id as ChainId;
use cosmrs::tendermint::block::Height;
use serde_json::Value;
use std::str::FromStr;

pub struct SecretTransactionBuilder {
    rpc: SecretRpcClient,
    chain_id: String,
}

#[derive(Debug, Clone)]
pub struct AccountInfo {
    pub account_number: u64,
    pub sequence: u64,
}

impl SecretTransactionBuilder {
    pub fn new(rpc: SecretRpcClient, chain_id: Option<String>) -> Self {
        Self {
            rpc: rpc.clone(),
            chain_id: chain_id.unwrap_or_else(|| "secret-4".to_string()), // Mainnet chain ID
        }
    }
    
    /// Get chain ID from network (or use provided/default)
    pub async fn get_chain_id(&self) -> String {
        // Try to get from network, fall back to default
        self.rpc.get_chain_id().await.unwrap_or_else(|_| self.chain_id.clone())
    }

    /// Get account information (account number and sequence)
    pub async fn get_account_info(&self, address: &str) -> NozyResult<AccountInfo> {
        let (account_number, sequence) = self.rpc.get_account_info(address).await?;
        
        Ok(AccountInfo {
            account_number,
            sequence,
        })
    }

    /// Build execute contract message for SNIP-20 token transfer
    /// Returns the JSON message that will be encoded in the transaction
    pub fn build_execute_contract_msg(
        &self,
        contract_address: &str,
        recipient: &str,
        amount: u128,
        memo: Option<String>,
    ) -> NozyResult<Value> {
        let mut transfer_msg = serde_json::json!({
            "transfer": {
                "recipient": recipient,
                "amount": amount.to_string()
            }
        });
        
        if let Some(m) = memo {
            transfer_msg["transfer"]["memo"] = serde_json::json!(m);
        }
        
        Ok(transfer_msg)
    }

    /// Estimate transaction fee in uscrt
    /// Tries to get from network, falls back to conservative estimate
    pub async fn estimate_fee(&self) -> NozyResult<u64> {
        // Try to get gas prices from network
        match self.rpc.get_gas_prices().await {
            Ok(price) => Ok(price),
            Err(_) => {
                // Fallback to conservative estimate
                // Secret Network typical fees:
                // - Simple transfer: ~0.001 SCRT (1,000 uscrt)
                // - Contract execution: ~0.002-0.01 SCRT (2,000-10,000 uscrt) depending on gas
                Ok(100_000) // 0.1 SCRT (conservative)
            }
        }
    }

    /// Build and sign a transaction for SNIP-20 token transfer
    pub async fn build_and_sign_transaction(
        &self,
        key_pair: &SecretKeyPair,
        contract_address: &str,
        recipient: &str,
        amount: u128,
        memo: Option<String>,
        fee_amount: u64,
        gas_limit: u64,
    ) -> NozyResult<Vec<u8>> {
        // Get account info
        let account_info = self.get_account_info(&key_pair.address).await?;
        
        // Build the execute message
        let execute_msg = self.build_execute_contract_msg(contract_address, recipient, amount, memo.clone())?;
        let msg_bytes = serde_json::to_vec(&execute_msg)
            .map_err(|e| NozyError::NetworkError(format!("Failed to serialize message: {}", e)))?;
        
        // Create MsgExecuteContract using cosmrs
        // Note: cosmrs uses generic Any message type for custom messages
        // For Secret Network, we need to use the compute module's MsgExecuteContract
        // This requires the proper protobuf type URL
        
        // Type URL for Secret Network MsgExecuteContract
        // secret.compute.v1beta1.MsgExecuteContract
        let type_url = "/secret.compute.v1beta1.MsgExecuteContract";
        
        // Build the message as a protobuf Any
        // We'll construct the message manually since cosmrs may not have Secret Network specific types
        let msg_execute = build_msg_execute_contract(
            &key_pair.address,
            contract_address,
            msg_bytes,
            vec![], // No funds sent with message
        )?;
        
        // Convert to cosmrs Any
        let any_msg = cosmrs::Any {
            type_url: type_url.to_string(),
            value: msg_execute,
        };
        
        // Build transaction body
        let tx_body = Body::new(
            vec![any_msg],
            memo.unwrap_or_default(),
            Height::default(),
        );
        
        // Convert secp256k1 key to cosmrs format
        let secp = secp256k1::Secp256k1::new();
        let private_key_bytes = key_pair.private_key.secret_bytes();
        let signing_key = SigningKey::from_bytes(&private_key_bytes)
            .map_err(|e| NozyError::KeyDerivation(format!("Invalid private key: {}", e)))?;
        
        // Get PublicKey from SigningKey
        // cosmrs SigningKey has public_key() method that returns VerifyingKey
        let verifying_key = signing_key.public_key();
        let public_key = PublicKey::from(verifying_key);
        
        // Build signer info
        let signer_info = SignerInfo::single_direct(Some(public_key), account_info.sequence);
        
        // Build fee
        let fee = Fee::from_amount_and_gas(
            Coin {
                denom: Denom::from_str("uscrt")
                    .map_err(|e| NozyError::NetworkError(format!("Invalid denom: {}", e)))?,
                amount: fee_amount.into(),
            },
            gas_limit,
        );
        
        // Build auth info
        // Note: AuthInfo structure may vary by cosmrs version - using direct construction
        let auth_info = AuthInfo {
            signer_infos: vec![signer_info],
            fee,
        };
        
        // Get chain ID (try network first, then use configured/default)
        let actual_chain_id = self.get_chain_id().await;
        let chain_id = ChainId::try_from(actual_chain_id.as_str())
            .map_err(|e| NozyError::NetworkError(format!("Invalid chain ID: {}", e)))?;
        
        let sign_doc = SignDoc::new(
            &tx_body,
            &auth_info,
            &chain_id,
            account_info.account_number,
        )
        .map_err(|e| NozyError::NetworkError(format!("Failed to create sign doc: {}", e)))?;
        
        // Sign the transaction
        let tx_raw = sign_doc.sign(&signing_key)
            .map_err(|e| NozyError::NetworkError(format!("Failed to sign transaction: {}", e)))?;
        
        // Encode to bytes
        // Note: cosmrs Raw encoding - using body and auth_info directly
        // Raw contains body_bytes and auth_info_bytes fields
        // For now, return error indicating this needs proper implementation
        // TODO: Implement proper Raw encoding for cosmrs 0.8
        Err(NozyError::InvalidOperation(
            "Transaction encoding for Secret Network requires cosmrs Raw encoding implementation. This is an optional feature.".to_string()
        ))
    }
}

/// Build MsgExecuteContract protobuf message
/// This manually constructs the protobuf structure for Secret Network
/// Note: In production, you'd want to use the actual protobuf definitions from secret-sdk-proto
fn build_msg_execute_contract(
    sender: &str,
    contract: &str,
    msg: Vec<u8>,
    _funds: Vec<cosmrs::Coin>,
) -> NozyResult<Vec<u8>> {
    // MsgExecuteContract structure:
    // message MsgExecuteContract {
    //   string sender = 1;
    //   string contract = 2;
    //   bytes msg = 3;
    //   repeated cosmos.base.v1beta1.Coin funds = 4;
    // }
    
    // Simple protobuf encoding using wire format
    // Wire format: field_number << 3 | wire_type
    // Wire type 2 = length-delimited (for strings and bytes)
    
    let mut buf = Vec::new();
    
    // Helper to encode varint
    fn encode_varint(mut value: u64, buf: &mut Vec<u8>) {
        while value >= 0x80 {
            buf.push((value as u8) | 0x80);
            value >>= 7;
        }
        buf.push(value as u8);
    }
    
    // Helper to encode length-delimited field
    fn encode_string_field(field_num: u32, value: &str, buf: &mut Vec<u8>) {
        encode_varint(((field_num << 3) | 2) as u64, buf); // Wire type 2
        let bytes = value.as_bytes();
        encode_varint(bytes.len() as u64, buf);
        buf.extend_from_slice(bytes);
    }
    
    // Helper to encode bytes field
    fn encode_bytes_field(field_num: u32, value: &[u8], buf: &mut Vec<u8>) {
        encode_varint(((field_num << 3) | 2) as u64, buf); // Wire type 2
        encode_varint(value.len() as u64, buf);
        buf.extend_from_slice(value);
    }
    
    // Field 1: sender (string)
    encode_string_field(1, sender, &mut buf);
    
    // Field 2: contract (string)
    encode_string_field(2, contract, &mut buf);
    
    // Field 3: msg (bytes)
    encode_bytes_field(3, &msg, &mut buf);
    
    // Field 4: funds (repeated message) - empty for now
    // funds would require encoding Coin messages, skipped for simplicity
    
    Ok(buf)
}
