//! Client-side ZIP-317 fee policy for Orchard sends (Zebrad has no `estimatefee`).
//!
//! ZIP 2009 lowers the marginal fee from 5,000 to 1,000 zatoshis per logical action.
//! Mainnet wallets switch when the block the transaction is mined in is at height
//! [`ZIP2009_MAINNET_ACTIVATION_HEIGHT`]. Test networks use the ZIP 2009 rate at every
//! height. Nozy still multiplies the active conventional fee by [`PRIORITY_MULTIPLIER`].
//!
//! Interim home until Shielded Labs approves moving shared logic into `zeaking`.

/// ZIP-317 marginal fee per logical action before ZIP 2009 (zatoshis).
pub const MARGINAL_FEE_ZATOSHIS: u64 = 5_000;
/// ZIP 2009 marginal fee per logical action (zatoshis).
pub const ZIP2009_MARGINAL_FEE_ZATOSHIS: u64 = 1_000;
/// Mainnet height of the block a transaction is mined in, from which ZIP 2009 applies.
///
/// Zebra `getstandardfee` reports 5,000 until the next block is at this height, then 1,000.
/// The mempool may accept 1,000 earlier; wallets switch together at this height.
pub const ZIP2009_MAINNET_ACTIVATION_HEIGHT: u32 = 3_590_000;

/// Chain position used to pick the ZIP 317 / ZIP 2009 marginal fee.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeeBlock {
    /// `false` for testnet and regtest. Those networks use the ZIP 2009 rate at every height.
    pub mainnet: bool,
    /// Height of the block this transaction is built to be mined in (chain tip + 1).
    pub height: u32,
}

impl FeeBlock {
    /// Fee context for a transaction mined in the block after `tip_height`.
    pub fn next_block(mainnet: bool, tip_height: u32) -> Self {
        Self {
            mainnet,
            height: tip_height.saturating_add(1),
        }
    }

    /// Mainnet before ZIP 2009. Used when the tip is unknown so a send cannot underpay.
    pub fn mainnet_before_zip2009() -> Self {
        Self {
            mainnet: true,
            height: 0,
        }
    }

    /// `network` is a Nozy config or Zebra `chain` name (`main`, `test`, `testnet`, `regtest`).
    pub fn from_network_name(network: &str, tip_height: u32) -> Self {
        Self::next_block(!is_test_network_name(network), tip_height)
    }

    /// Marginal fee in zatoshis per logical action for this block.
    pub fn marginal_fee(self) -> u64 {
        if !self.mainnet || self.height >= ZIP2009_MAINNET_ACTIVATION_HEIGHT {
            ZIP2009_MARGINAL_FEE_ZATOSHIS
        } else {
            MARGINAL_FEE_ZATOSHIS
        }
    }

    pub fn zip2009_active(self) -> bool {
        self.marginal_fee() == ZIP2009_MARGINAL_FEE_ZATOSHIS
    }
}

fn is_test_network_name(network: &str) -> bool {
    matches!(
        network.trim().to_ascii_lowercase().as_str(),
        "test" | "testnet" | "regtest"
    )
}
/// ZIP-317 grace logical actions (minimum billable action count).
pub const GRACE_ACTIONS: u32 = 2;
/// NozyWallet standard fee multiplier (ZIP-317 × 4 on every send surface).
pub const PRIORITY_MULTIPLIER: u64 = 4;
/// All NozyWallet sends use the priority fee; not user-configurable.
pub const NOZY_WALLET_PRIORITY_FEE: bool = true;
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
    /// NozyWallet always sets this to true via [`Self::for_send`].
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
            priority: NOZY_WALLET_PRIORITY_FEE,
            expiry_delta_blocks: PILOT_EXPIRY_DELTA_BLOCKS,
        }
    }
}

impl PilotSendOptions {
    /// NozyWallet mandatory send policy (desktop, mobile, API, CLI).
    pub fn for_send() -> Self {
        Self::default()
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
///
/// `block` selects the ZIP 317 or ZIP 2009 marginal fee.
pub fn conventional_fee_zatoshis(shape: &OrchardSendFeeShape, block: FeeBlock) -> u64 {
    let marginal = block.marginal_fee();
    let actions = logical_actions(shape);
    let billable = actions.max(GRACE_ACTIONS) as u64;
    marginal
        .saturating_mul(billable)
        .max(marginal.saturating_mul(GRACE_ACTIONS as u64))
}

/// Final fee for a NozyWallet send (active ZIP-317 conventional fee × [`PRIORITY_MULTIPLIER`]).
pub fn fee_zatoshis(shape: &OrchardSendFeeShape, _priority: bool, block: FeeBlock) -> u64 {
    conventional_fee_zatoshis(shape, block).saturating_mul(PRIORITY_MULTIPLIER)
}

/// Fee estimate for UI / CLI preview (assumes change output exists).
pub fn estimate_orchard_send_fee_zatoshis(
    memo: Option<&[u8]>,
    priority: bool,
    block: FeeBlock,
) -> u64 {
    let shape = OrchardSendFeeShape::estimate_preview(memo);
    let fee = fee_zatoshis(&shape, priority, block);
    if fee == 0 {
        block
            .marginal_fee()
            .saturating_mul(GRACE_ACTIONS as u64)
            .saturating_mul(PRIORITY_MULTIPLIER)
            .max(FALLBACK_FEE_ZATOSHIS)
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

    fn legacy() -> FeeBlock {
        FeeBlock::mainnet_before_zip2009()
    }

    #[test]
    fn grace_minimum_fee_is_10000_zats() {
        let shape = OrchardSendFeeShape {
            orchard_actions: 1,
            memo_len: 0,
        };
        assert_eq!(conventional_fee_zatoshis(&shape, legacy()), 10_000);
    }

    #[test]
    fn typical_send_is_two_actions_10000_zats() {
        // 1 spend + 2 outputs (recipient + change) => max(1, 2) = 2 Orchard actions.
        let shape = OrchardSendFeeShape::single_spend_send(true, None);
        assert_eq!(shape.orchard_actions, 2);
        assert_eq!(conventional_fee_zatoshis(&shape, legacy()), 10_000);
    }

    #[test]
    fn nozy_wallet_fee_is_always_priority_times_four() {
        let shape = OrchardSendFeeShape::estimate_preview(None);
        assert_eq!(fee_zatoshis(&shape, false, legacy()), 40_000);
        assert_eq!(fee_zatoshis(&shape, true, legacy()), 40_000);
    }

    #[test]
    fn estimate_preview_matches_typical_send_floor() {
        assert_eq!(
            estimate_orchard_send_fee_zatoshis(None, false, legacy()),
            40_000
        );
        assert_eq!(
            estimate_orchard_send_fee_zatoshis(None, true, legacy()),
            40_000
        );
        assert_eq!(
            estimate_orchard_send_fee_zatoshis(Some(b"hi"), true, legacy()),
            40_000
        );
    }

    #[test]
    fn zip2009_switches_on_the_mined_block_not_the_tip() {
        let before = FeeBlock::next_block(true, ZIP2009_MAINNET_ACTIVATION_HEIGHT - 2);
        let at_activation = FeeBlock::next_block(true, ZIP2009_MAINNET_ACTIVATION_HEIGHT - 1);
        assert_eq!(before.height, ZIP2009_MAINNET_ACTIVATION_HEIGHT - 1);
        assert_eq!(before.marginal_fee(), MARGINAL_FEE_ZATOSHIS);
        assert_eq!(at_activation.height, ZIP2009_MAINNET_ACTIVATION_HEIGHT);
        assert_eq!(at_activation.marginal_fee(), ZIP2009_MARGINAL_FEE_ZATOSHIS);
        assert!(at_activation.zip2009_active());
    }

    #[test]
    fn zip2009_testnet_uses_1000_before_mainnet_activation() {
        let test = FeeBlock::from_network_name("testnet", 100);
        assert_eq!(test.marginal_fee(), ZIP2009_MARGINAL_FEE_ZATOSHIS);
        assert_eq!(estimate_orchard_send_fee_zatoshis(None, true, test), 8_000);
    }

    #[test]
    fn zip2009_mainnet_priority_fee_is_four_times_the_new_conventional_fee() {
        let block = FeeBlock::next_block(true, ZIP2009_MAINNET_ACTIVATION_HEIGHT);
        let shape = OrchardSendFeeShape::estimate_preview(None);
        assert_eq!(conventional_fee_zatoshis(&shape, block), 2_000);
        assert_eq!(fee_zatoshis(&shape, true, block), 8_000);
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
        assert!(conventional_fee_zatoshis(&shape, legacy()) > 10_000);
    }
}
