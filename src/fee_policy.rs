//! Client-side ZIP-317 fee policy for Orchard sends (Zebrad has no `estimatefee`).
//!
//! Interim home until Shielded Labs approves moving shared logic into `zeaking`.

/// ZIP-317 marginal fee per logical action (zatoshis).
pub const MARGINAL_FEE_ZATOSHIS: u64 = 5_000;
/// ZIP-317 grace logical actions (minimum billable action count).
pub const GRACE_ACTIONS: u32 = 2;
/// Pilot priority multiplier when the user opts in.
pub const PRIORITY_MULTIPLIER: u64 = 4;
/// Default transaction expiry delta (~2 minutes at 75s/block).
pub const PILOT_EXPIRY_DELTA_BLOCKS: u32 = 2;
/// Last-resort fee if shape computation overflows (should not happen in practice).
pub const FALLBACK_FEE_ZATOSHIS: u64 = 10_000;

/// User-visible send options for the dynamic-fee pilot (Phase A1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PilotSendOptions {
    /// When true, fee is [`conventional_fee_zatoshis`] × [`PRIORITY_MULTIPLIER`].
    pub priority: bool,
    /// Blocks after chain tip at build time when the tx expires (`expiry_height = tip + delta`).
    pub expiry_delta_blocks: u32,
}

impl Default for PilotSendOptions {
    fn default() -> Self {
        Self {
            priority: false,
            expiry_delta_blocks: PILOT_EXPIRY_DELTA_BLOCKS,
        }
    }
}

/// ZIP-317 inputs for a typical Orchard-only shielded send built by `orchard_tx`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrchardSendFeeShape {
    /// Orchard bundle actions (spend + outputs).
    pub orchard_actions: u32,
    /// Non-empty memo byte length on the recipient output (0 if none).
    pub memo_len: usize,
}

impl OrchardSendFeeShape {
    /// Shape for `build_single_spend`: one spend, one recipient output, optional change output.
    pub fn single_spend_send(has_change: bool, memo: Option<&[u8]>) -> Self {
        let outputs = if has_change { 2 } else { 1 };
        Self {
            orchard_actions: 1 + outputs,
            memo_len: memo.map(|m| m.len()).unwrap_or(0),
        }
    }

    /// Conservative preview shape when note values are not yet known (assumes change output).
    pub fn estimate_preview(memo: Option<&[u8]>) -> Self {
        Self::single_spend_send(true, memo)
    }
}

/// ZIP-317 logical actions for an Orchard-only transaction shape.
pub fn logical_actions(shape: &OrchardSendFeeShape) -> u32 {
    let memo_chunks = if shape.memo_len == 0 {
        0
    } else {
        div_ceil(shape.memo_len, 512) as u32
    };
    // Orchard outputs exist → 2 free memo chunks (ZIP-317).
    let free_memo = if shape.orchard_actions > 0 { 2 } else { 0 };
    let memo_contribution = memo_chunks.saturating_sub(free_memo);
    shape.orchard_actions.saturating_add(memo_contribution)
}

/// ZIP-317 conventional fee in zatoshis: `marginal_fee × max(grace_actions, logical_actions)`.
pub fn conventional_fee_zatoshis(shape: &OrchardSendFeeShape) -> u64 {
    let actions = logical_actions(shape);
    let billable = actions.max(GRACE_ACTIONS) as u64;
    MARGINAL_FEE_ZATOSHIS
        .saturating_mul(billable)
        .max(MARGINAL_FEE_ZATOSHIS.saturating_mul(GRACE_ACTIONS as u64))
}

/// Final fee for a send, optionally multiplied for pilot priority.
pub fn fee_zatoshis(shape: &OrchardSendFeeShape, priority: bool) -> u64 {
    let base = conventional_fee_zatoshis(shape);
    if priority {
        base.saturating_mul(PRIORITY_MULTIPLIER)
    } else {
        base
    }
}

/// Fee estimate for UI / CLI preview (assumes change output exists).
pub fn estimate_orchard_send_fee_zatoshis(memo: Option<&[u8]>, priority: bool) -> u64 {
    let shape = OrchardSendFeeShape::estimate_preview(memo);
    let fee = fee_zatoshis(&shape, priority);
    if fee == 0 {
        FALLBACK_FEE_ZATOSHIS
    } else {
        fee
    }
}

fn div_ceil(a: usize, b: usize) -> usize {
    if b == 0 {
        return a;
    }
    a.saturating_add(b - 1) / b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grace_minimum_fee_is_10000_zats() {
        let shape = OrchardSendFeeShape {
            orchard_actions: 1,
            memo_len: 0,
        };
        assert_eq!(conventional_fee_zatoshis(&shape), 10_000);
    }

    #[test]
    fn typical_three_action_send_is_15000_zats() {
        let shape = OrchardSendFeeShape::single_spend_send(true, None);
        assert_eq!(shape.orchard_actions, 3);
        assert_eq!(conventional_fee_zatoshis(&shape), 15_000);
    }

    #[test]
    fn priority_multiplies_by_four() {
        let shape = OrchardSendFeeShape::estimate_preview(None);
        assert_eq!(fee_zatoshis(&shape, false), 15_000);
        assert_eq!(fee_zatoshis(&shape, true), 60_000);
    }

    #[test]
    fn memo_adds_logical_actions_beyond_free_chunks() {
        let shape = OrchardSendFeeShape::single_spend_send(true, Some(&[0u8; 1200]));
        assert!(logical_actions(&shape) > 3);
        assert!(conventional_fee_zatoshis(&shape) > 15_000);
    }
}
