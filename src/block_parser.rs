use crate::error::{NozyError, NozyResult};
use crate::zebra_integration::ZebraClient;
use hex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct BlockParser {
    zebra_client: ZebraClient,
}

#[derive(Debug, Clone)]
pub struct ParsedTransaction {
    pub txid: String,
    pub height: u32,
    pub index: u32,
    pub raw_data: Vec<u8>,
    pub orchard_actions: Vec<crate::notes::OrchardActionData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchardAction {
    pub action_type: ActionType,
    pub nullifier: Option<String>,
    pub commitment: Option<String>,
    pub encrypted_note: Option<String>,
    pub cv: Option<String>,
    pub rk: Option<String>,
    pub cmx: Option<String>,
    pub ephemeral_key: Option<String>,
    pub encrypted_ciphertext: Option<String>,
    pub out_ciphertext: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionType {
    Spend,
    Output,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaplingSpend {
    pub cv: String,
    pub anchor: String,
    pub nullifier: String,
    pub rk: String,
    pub zkproof: String,
    pub spend_auth_sig: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaplingOutput {
    pub cv: String,
    pub cmu: String,
    pub ephemeral_key: String,
    pub enc_ciphertext: String,
    pub out_ciphertext: String,
    pub zkproof: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransparentInput {
    pub prevout_hash: String,
    pub prevout_index: u32,
    pub script_sig: String,
    pub sequence: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransparentOutput {
    pub value: u64,
    pub script_pubkey: String,
}

impl BlockParser {
    pub fn new(zebra_client: ZebraClient) -> Self {
        Self { zebra_client }
    }

    pub async fn parse_block(&self, block_height: u32) -> NozyResult<Vec<ParsedTransaction>> {
        let block_data = self.zebra_client.get_block(block_height).await?;

        let json_string = serde_json::to_string(&block_data).map_err(|e| {
            NozyError::InvalidOperation(format!("Failed to serialize block data: {}", e))
        })?;

        self.parse_block_data(&json_string)
    }

    pub fn parse_block_data(&self, block_data: &str) -> NozyResult<Vec<ParsedTransaction>> {
        let json_data: Value = serde_json::from_str(block_data).map_err(|e| {
            NozyError::InvalidOperation(format!("Failed to parse block JSON: {}", e))
        })?;

        let transactions = json_data["result"]["transactions"]
            .as_array()
            .ok_or_else(|| {
                NozyError::InvalidOperation("No transactions found in block".to_string())
            })?;

        let mut parsed_transactions = Vec::new();

        for (tx_index, tx_data) in transactions.iter().enumerate() {
            if let Ok(parsed_tx) = self.parse_transaction_from_value(tx_data, tx_index as u32) {
                parsed_transactions.push(parsed_tx);
            }
        }

        Ok(parsed_transactions)
    }

    fn parse_transaction_from_value(
        &self,
        tx_value: &serde_json::Value,
        _tx_index: u32,
    ) -> NozyResult<ParsedTransaction> {
        let txid = tx_value["txid"]
            .as_str()
            .ok_or_else(|| NozyError::InvalidOperation("Missing txid".to_string()))?
            .to_string();

        let raw_data = if let Some(hex_str) = tx_value["hex"].as_str() {
            hex::decode(hex_str).unwrap_or_default()
        } else {
            Vec::new()
        };

        Ok(ParsedTransaction {
            txid,
            height: 0,
            index: 0,
            raw_data,
            orchard_actions: Vec::new(),
        })
    }

    pub fn parse_orchard_actions(&self, tx_data: &Value) -> Vec<OrchardAction> {
        let mut actions = Vec::new();

        if let Some(actions_data) = tx_data["orchard_actions"].as_array() {
            for action_data in actions_data {
                if let Ok(action) = self.parse_orchard_action(action_data) {
                    actions.push(action);
                }
            }
        }

        actions
    }

    pub fn parse_orchard_action(&self, action_data: &Value) -> NozyResult<OrchardAction> {
        let action_type = if action_data["nullifier"].is_string()
            && !action_data["nullifier"].as_str().unwrap_or("").is_empty()
        {
            ActionType::Spend
        } else {
            ActionType::Output
        };

        Ok(OrchardAction {
            action_type,
            nullifier: action_data["nullifier"].as_str().map(|s| s.to_string()),
            commitment: action_data["commitment"].as_str().map(|s| s.to_string()),
            encrypted_note: action_data["encrypted_note"]
                .as_str()
                .map(|s| s.to_string()),
            cv: action_data["cv"].as_str().map(|s| s.to_string()),
            rk: action_data["rk"].as_str().map(|s| s.to_string()),
            cmx: action_data["cmx"].as_str().map(|s| s.to_string()),
            ephemeral_key: action_data["ephemeral_key"].as_str().map(|s| s.to_string()),
            encrypted_ciphertext: action_data["encrypted_ciphertext"]
                .as_str()
                .map(|s| s.to_string()),
            out_ciphertext: action_data["out_ciphertext"]
                .as_str()
                .map(|s| s.to_string()),
        })
    }

    pub fn parse_sapling_spends(&self, tx_data: &Value) -> Vec<SaplingSpend> {
        let mut spends = Vec::new();

        if let Some(spends_data) = tx_data["sapling_spends"].as_array() {
            for spend_data in spends_data {
                if let Ok(spend) = self.parse_sapling_spend(spend_data) {
                    spends.push(spend);
                }
            }
        }

        spends
    }

    pub fn parse_sapling_spend(&self, spend_data: &Value) -> NozyResult<SaplingSpend> {
        Ok(SaplingSpend {
            cv: spend_data["cv"].as_str().unwrap_or("").to_string(),
            anchor: spend_data["anchor"].as_str().unwrap_or("").to_string(),
            nullifier: spend_data["nullifier"].as_str().unwrap_or("").to_string(),
            rk: spend_data["rk"].as_str().unwrap_or("").to_string(),
            zkproof: spend_data["zkproof"].as_str().unwrap_or("").to_string(),
            spend_auth_sig: spend_data["spend_auth_sig"]
                .as_str()
                .unwrap_or("")
                .to_string(),
        })
    }

    pub fn parse_sapling_outputs(&self, tx_data: &Value) -> Vec<SaplingOutput> {
        let mut outputs = Vec::new();

        if let Some(outputs_data) = tx_data["sapling_outputs"].as_array() {
            for output_data in outputs_data {
                if let Ok(output) = self.parse_sapling_output(output_data) {
                    outputs.push(output);
                }
            }
        }

        outputs
    }

    pub fn parse_sapling_output(&self, output_data: &Value) -> NozyResult<SaplingOutput> {
        Ok(SaplingOutput {
            cv: output_data["cv"].as_str().unwrap_or("").to_string(),
            cmu: output_data["cmu"].as_str().unwrap_or("").to_string(),
            ephemeral_key: output_data["ephemeral_key"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            enc_ciphertext: output_data["enc_ciphertext"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            out_ciphertext: output_data["out_ciphertext"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            zkproof: output_data["zkproof"].as_str().unwrap_or("").to_string(),
        })
    }

    pub fn parse_transparent_inputs(&self, tx_data: &Value) -> Vec<TransparentInput> {
        let mut inputs = Vec::new();

        if let Some(inputs_data) = tx_data["vin"].as_array() {
            for input_data in inputs_data {
                if let Ok(input) = self.parse_transparent_input(input_data) {
                    inputs.push(input);
                }
            }
        }

        inputs
    }

    pub fn parse_transparent_input(&self, input_data: &Value) -> NozyResult<TransparentInput> {
        Ok(TransparentInput {
            prevout_hash: input_data["txid"].as_str().unwrap_or("").to_string(),
            prevout_index: input_data["vout"].as_u64().unwrap_or(0) as u32,
            script_sig: input_data["scriptSig"]["asm"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            sequence: input_data["sequence"].as_u64().unwrap_or(0) as u32,
        })
    }

    pub fn parse_transparent_outputs(&self, tx_data: &Value) -> Vec<TransparentOutput> {
        let mut outputs = Vec::new();

        if let Some(outputs_data) = tx_data["vout"].as_array() {
            for output_data in outputs_data {
                if let Ok(output) = self.parse_transparent_output(output_data) {
                    outputs.push(output);
                }
            }
        }

        outputs
    }

    pub fn parse_transparent_output(&self, output_data: &Value) -> NozyResult<TransparentOutput> {
        let value = output_data["value"].as_f64().unwrap_or(0.0) * 100_000_000.0; // Convert to zatoshi

        Ok(TransparentOutput {
            value: value as u64,
            script_pubkey: output_data["scriptPubKey"]["asm"]
                .as_str()
                .unwrap_or("")
                .to_string(),
        })
    }

    pub fn extract_orchard_notes(
        &self,
        _transactions: &[ParsedTransaction],
        _target_address: &str,
    ) -> Vec<crate::notes::OrchardNote> {
        Vec::new()
    }
}
