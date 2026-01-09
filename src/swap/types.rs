use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SwapDirection {
    XmrToZec,
    ZecToXmr,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SwapStatus {
    Pending,
    WaitingForDeposit,
    Processing,
    Completed,
    Failed,
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapRequest {
    pub direction: SwapDirection,
    pub amount: f64,
    pub from_address: String,
    pub to_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapResponse {
    pub swap_id: String,
    pub deposit_address: String,
    pub deposit_amount: f64,
    pub status: SwapStatus,
    pub estimated_time: Option<u64>, // seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapStatusResponse {
    pub swap_id: String,
    pub status: SwapStatus,
    pub progress: f64, // 0.0 to 1.0
    pub txid: Option<String>,
}
