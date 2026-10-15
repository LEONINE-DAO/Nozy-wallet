/** ZIP-317 marginal fee before ZIP 2009, zatoshis per logical action. */
export const ZIP317_MARGINAL_FEE_ZATS = 5_000;
/** ZIP 2009 marginal fee, zatoshis per logical action. */
export const ZIP2009_MARGINAL_FEE_ZATS = 1_000;
/** Mainnet block the transaction is mined in, from which ZIP 2009 applies. */
export const ZIP2009_MAINNET_ACTIVATION_HEIGHT = 3_590_000;
/** Nozy multiplies the active conventional fee by this on every send. */
export const PRIORITY_MULTIPLIER = 4;
/** ZIP-317 grace actions. A normal send bills at least this many. */
export const GRACE_ACTIONS = 2;

/** Pre-ZIP-2009 empty-memo floor (5,000 × 2 × 4). Used when the chain tip is unknown. */
export const MANDATORY_ORCHARD_FEE_FALLBACK_ZATS = 40_000;

/**
 * Marginal fee for a transaction mined in `minedBlockHeight` (chain tip + 1).
 * Test networks use ZIP 2009 at every height. Mainnet switches at 3,590,000.
 */
export function marginalFeeForMinedBlock(mainnet, minedBlockHeight) {
  const height = Number(minedBlockHeight);
  if (!mainnet || (Number.isFinite(height) && height >= ZIP2009_MAINNET_ACTIVATION_HEIGHT)) {
    return ZIP2009_MARGINAL_FEE_ZATS;
  }
  return ZIP317_MARGINAL_FEE_ZATS;
}

/** Empty-memo ZIP-317 ×4 floor for the active marginal fee. */
export function orchardFeeFloorZats(mainnet, minedBlockHeight) {
  return marginalFeeForMinedBlock(mainnet, minedBlockHeight) * GRACE_ACTIONS * PRIORITY_MULTIPLIER;
}

/**
 * Ask the node which ZIP 2009 fee block the next transaction is mined in.
 * `request(method, params)` returns the JSON-RPC result.
 */
export async function feeBlockFromRpc(request) {
  const tip = Number(await request("getblockcount", []));
  let mainnet = true;
  try {
    const info = await request("getblockchaininfo", []);
    const chain = String(info?.chain ?? "main").toLowerCase();
    mainnet = chain !== "test" && chain !== "testnet" && chain !== "regtest";
  } catch (_) {
    // A missing chain name stays on mainnet so the wallet cannot underpay.
  }
  const minedBlockHeight = (Number.isFinite(tip) && tip >= 0 ? Math.floor(tip) : 0) + 1;
  return { mainnet, minedBlockHeight };
}

/**
 * ZIP-317 × 4 of the active ZIP 2009 schedule.
 * `feeBlock` is `{ mainnet, minedBlockHeight }` from `feeBlockFromRpc`.
 * Without a rebuilt WASM that exports `estimate_orchard_send_fee_zats_at`, this
 * stays on the pre-2009 floor so it matches the prove-path clamp.
 */
export function mandatoryOrchardFeeZats(wasm, memo = "", feeBlock = null) {
  const memoStr = String(memo || "");
  if (
    feeBlock &&
    typeof wasm?.estimate_orchard_send_fee_zats_at === "function"
  ) {
    try {
      const fee = Number(
        wasm.estimate_orchard_send_fee_zats_at(
          memoStr,
          true,
          !!feeBlock.mainnet,
          Number(feeBlock.minedBlockHeight) || 0
        )
      );
      if (Number.isFinite(fee) && fee > 0) return Math.floor(fee);
    } catch (_) {
      // Fall through to the legacy export.
    }
  }
  try {
    const fee = Number(wasm?.estimate_orchard_send_fee_zats?.(memoStr, true));
    if (Number.isFinite(fee) && fee > 0) return Math.floor(fee);
  } catch (_) {
    // Fall through to the known empty-memo floor.
  }
  if (feeBlock && typeof wasm?.estimate_orchard_send_fee_zats_at === "function") {
    return orchardFeeFloorZats(!!feeBlock.mainnet, feeBlock.minedBlockHeight);
  }
  return MANDATORY_ORCHARD_FEE_FALLBACK_ZATS;
}

export function selectNotesForSpend(notes, requiredValue) {
  const usable = notes
    .filter((n) => Number.isFinite(n.value) && n.value > 0)
    .sort((a, b) => a.value - b.value || a.height - b.height);

  if (usable.length === 0) return [];

  // Prefer a single smallest-sufficient note first.
  const single = usable.find((n) => n.value >= requiredValue);
  if (single) return [single];

  // Otherwise, accumulate larger notes first to minimize input count.
  const desc = [...usable].sort((a, b) => b.value - a.value || a.height - b.height);
  const selected = [];
  let running = 0;
  for (const note of desc) {
    selected.push(note);
    running += note.value;
    if (running >= requiredValue) break;
  }
  return running >= requiredValue ? selected : [];
}

export async function rpcFallbackWithRequester(requester, attempts) {
  let lastErr;
  for (const at of attempts) {
    try {
      return await requester(at);
    } catch (err) {
      lastErr = err;
    }
  }
  throw lastErr || new Error("All RPC fallbacks failed");
}

