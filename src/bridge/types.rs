// Bridge Types
// Common types for Monero-Zcash bridge operations

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwapDirection {
    XmrToZec,
    ZecToXmr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
