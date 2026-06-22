//! Client-side ZIP-317 fee policy for Orchard sends (Zebrad has no `estimatefee`).
//!
//! Interim home until Shielded Labs approves moving shared logic into `zeaking`.

/// ZIP-317 marginal fee per logical action (zatoshis).
pub const MARGINAL_FEE_ZATOSHIS: u64 = 5_000;
/// ZIP-317 grace logical actions (minimum billable action count).
pub const GRACE_ACTIONS: u32 = 2;
/// Pilot priority multiplier when the user opts in.
pub const PRIORITY_MULTIPLIER: u64 = 4;
/// Default transaction expiry delta after the mempool build height (~6 minutes at 75s/block).
pub const PILOT_EXPIRY_DELTA_BLOCKS: u32 = 5;
/// Max full rebuild attempts when proving outruns the pilot expiry window.
pub const PILOT_EXPIRY_MAX_REBUILD_ATTEMPTS: u32 = 3;
/// Last-resort fee if shape computation overflows (should not happen in practice).
pub const FALLBACK_FEE_ZATOSHIS: u64 = 10_000;

/// User-visible send options for the dynamic-fee pilot (Phase A1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PilotSendOptions {
    /// When true, fee is [`conventional_fee_zatoshis`] × [`PRIORITY_MULTIPLIER`].
    pub priority: bool,
    /// Blocks after the mempool build height when the tx expires (see [`pilot_expiry_height`]).
    pub expiry_delta_blocks: u32,
}

/// On-chain expiry height for a pilot send given the current chain tip and delta.
///
/// Zebrad validates against the next-block context, so the build height is `tip + 1` and
/// `expiry_height = build_height + expiry_delta_blocks`.
pub fn pilot_expiry_height(tip_height: u32, expiry_delta_blocks: u32) -> u32 {
    tip_height
        .saturating_add(1)
        .saturating_add(expiry_delta_blocks)
}

/// True when the chain tip has moved past a transaction's encoded expiry height.
pub fn pilot_transaction_expired(tip_height: u32, expiry_height: u32) -> bool {
    tip_height > expiry_height
}

/// Detect Zebrad `-25` mempool rejections caused by an expired pilot transaction.
pub fn is_expiry_consensus_error(message: &str) -> bool {
    message.contains("greater than its expiry Height")
        || (message.contains("expiry Height") && message.contains("code: -25"))
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
    /// Orchard bundle actions = max(spends, outputs) (the bundle pads the shorter side).
    pub orchard_actions: u32,
    /// Non-empty memo byte length on the recipient output (0 if none).
    pub memo_len: usize,
}

impl OrchardSendFeeShape {
    /// Shape for `build_single_spend`: one spend, one recipient output, optional change output.
    pub fn single_spend_send(has_change: bool, memo: Option<&[u8]>) -> Self {
        let spends = 1u32;
        let outputs = if has_change { 2 } else { 1 };
        Self {
            // An Orchard bundle has max(spends, outputs) actions (the shorter side
            // is padded with dummies), not spends + outputs. Counting additively
            // over-bills every send (e.g. 1 spend + 2 outputs is 2 actions, not 3).
            orchard_actions: spends.max(outputs),
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
    fn typical_send_is_two_actions_10000_zats() {
        // 1 spend + 2 outputs (recipient + change) => max(1, 2) = 2 Orchard actions.
        let shape = OrchardSendFeeShape::single_spend_send(true, None);
        assert_eq!(shape.orchard_actions, 2);
        assert_eq!(conventional_fee_zatoshis(&shape), 10_000);
    }

    #[test]
    fn priority_multiplies_by_four() {
        let shape = OrchardSendFeeShape::estimate_preview(None);
        assert_eq!(fee_zatoshis(&shape, false), 10_000);
        assert_eq!(fee_zatoshis(&shape, true), 40_000);
    }

    #[test]
    fn pilot_expiry_height_uses_next_block_context() {
        assert_eq!(pilot_expiry_height(100, 5), 106);
        assert_eq!(pilot_expiry_height(3385374, 5), 3385380);
    }

    #[test]
    fn pilot_transaction_expired_when_tip_passes_expiry() {
        assert!(!pilot_transaction_expired(3385380, 3385380));
        assert!(pilot_transaction_expired(3385384, 3385380));
    }

    #[test]
    fn detects_expiry_consensus_error_from_zebra_message() {
        let msg = r#"failed to validate tx: transaction must not be mined at a block Height(3385384) greater than its expiry Height(3385380) (code: -25)"#;
        assert!(is_expiry_consensus_error(msg));
    }

    #[test]
    fn memo_adds_logical_actions_beyond_free_chunks() {
        // No-memo baseline is 2 actions / 10,000; a 1,200-byte memo adds one
        // logical action beyond the 2 free chunks, raising both.
        let shape = OrchardSendFeeShape::single_spend_send(true, Some(&[0u8; 1200]));
        assert!(logical_actions(&shape) > 2);
        assert!(conventional_fee_zatoshis(&shape) > 10_000);
    }
}
