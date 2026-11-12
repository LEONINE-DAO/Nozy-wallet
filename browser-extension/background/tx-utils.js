/** ZIP-317 marginal fee before ZIP 2009, zatoshis per logical action. */
export const ZIP317_MARGINAL_FEE_ZATS = 5_000;
/** ZIP 2009 marginal fee, zatoshis per logical action. */
export const ZIP2009_MARGINAL_FEE_ZATS = 1_000;
/** Mainnet block the transaction is mined in, from which ZIP 2009 applies. */
export const ZIP2009_MAINNET_ACTIVATION_HEIGHT = 3_590_000;
/** Inclusion boost on top of the ZIP-317 conventional fee. */
export const PRIORITY_MULTIPLIER = 10;
/** ZIP-317 grace actions. A normal send bills at least this many. */
export const GRACE_ACTIONS = 2;

/** Pre-ZIP-2009 empty-memo inclusion floor (5,000 × 2 × 10). Used when the chain tip is unknown. */
export const MANDATORY_ORCHARD_FEE_FALLBACK_ZATS = 100_000;

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

/** Empty-memo inclusion floor for the active marginal fee. */
export function orchardFeeFloorZats(mainnet, minedBlockHeight, marginalFee) {
  const quoted = Number(marginalFee);
  const marginal =
    Number.isFinite(quoted) && quoted > 0
      ? Math.floor(quoted)
      : marginalFeeForMinedBlock(mainnet, minedBlockHeight);
  return marginal * GRACE_ACTIONS * PRIORITY_MULTIPLIER;
}

/**
 * Ask the node which fee block the next transaction is mined in, and the
 * `getstandardfee` marginal fee when that RPC exists.
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
  let marginalFee = null;
  try {
    const quote = await request("getstandardfee", []);
    const fee = Number(quote?.standard_fee);
    if (Number.isFinite(fee) && fee > 0) marginalFee = Math.floor(fee);
  } catch (_) {
    // Older nodes keep the local ZIP 317 / ZIP 2009 schedule.
  }
  const minedBlockHeight = (Number.isFinite(tip) && tip >= 0 ? Math.floor(tip) : 0) + 1;
  return { mainnet, minedBlockHeight, marginalFee };
}

function memoUtf8Length(memo) {
  const text = String(memo || "");
  if (typeof TextEncoder !== "undefined") return new TextEncoder().encode(text).length;
  return text.length;
}

/**
 * ZIP-317 fee for one shielded spend with change.
 * `priority` selects the 10× inclusion fee. A positive `marginalFee` is `getstandardfee`.
 */
/**
 * Extension Send uses the standard ZIP-317 fee unless the caller explicitly
 * opts into 10× inclusion (`priority: true`). That 10× path waits for NU7.
 */
export function payForInclusionRequested(params) {
  return params?.priority === true;
}

export function zatsToZecString(zats) {
  const n = Number(zats);
  if (!Number.isFinite(n) || n < 0) return "0.00000000";
  return (n / 1e8).toFixed(8);
}

export function insufficientFundsMessage(amountZats, feeZats, haveZats) {
  const amount = Math.max(0, Math.floor(Number(amountZats) || 0));
  const fee = Math.max(0, Math.floor(Number(feeZats) || 0));
  const have = Math.max(0, Math.floor(Number(haveZats) || 0));
  const need = amount + fee;
  return (
    `Insufficient funds. This send needs ${zatsToZecString(need)} ZEC ` +
    `(${zatsToZecString(amount)} + ${zatsToZecString(fee)} fee) but this wallet has ` +
    `${zatsToZecString(have)} ZEC spendable. Lower the amount so the network fee still fits.`
  );
}

export function orchardSendFeeZats(memo, feeBlock, priority = true) {
  const chunks = memoUtf8Length(memo) === 0 ? 0 : Math.ceil(memoUtf8Length(memo) / 512);
  const logical = GRACE_ACTIONS + Math.max(0, chunks - 2);
  const billable = Math.max(logical, GRACE_ACTIONS);
  const quoted = Number(feeBlock?.marginalFee);
  const marginal =
    Number.isFinite(quoted) && quoted > 0
      ? Math.floor(quoted)
      : marginalFeeForMinedBlock(feeBlock?.mainnet !== false, feeBlock?.minedBlockHeight);
  return marginal * billable * (priority ? PRIORITY_MULTIPLIER : 1);
}

/**
 * ZIP-317 fee for `feeBlock` from `feeBlockFromRpc`.
 * Uses the WASM quote export when this build has it, otherwise the same formula in JS
 * so a node `getstandardfee` quote is not ignored by an older WASM schedule.
 */
export function mandatoryOrchardFeeZats(wasm, memo = "", feeBlock = null, priority = true) {
  const memoStr = String(memo || "");
  const payForInclusion = priority !== false;
  const marginalFee = Number(feeBlock?.marginalFee) > 0 ? Math.floor(Number(feeBlock.marginalFee)) : 0;
  if (feeBlock && typeof wasm?.estimate_orchard_send_fee_zats_quoted === "function") {
    try {
      const fee = Number(
        wasm.estimate_orchard_send_fee_zats_quoted(
          memoStr,
          payForInclusion,
          !!feeBlock.mainnet,
          Number(feeBlock.minedBlockHeight) || 0,
          marginalFee
        )
      );
      if (Number.isFinite(fee) && fee > 0) return Math.floor(fee);
    } catch (_) {
      // Fall through to the JS formula, which still honors the node quote.
    }
  }
  if (feeBlock) {
    return orchardSendFeeZats(memoStr, feeBlock, payForInclusion);
  }
  try {
    const fee = Number(wasm?.estimate_orchard_send_fee_zats?.(memoStr, payForInclusion));
    if (Number.isFinite(fee) && fee > 0) return Math.floor(fee);
  } catch (_) {
    // Fall through to the known empty-memo floor.
  }
  return payForInclusion
    ? MANDATORY_ORCHARD_FEE_FALLBACK_ZATS
    : Math.floor(MANDATORY_ORCHARD_FEE_FALLBACK_ZATS / PRIORITY_MULTIPLIER);
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

