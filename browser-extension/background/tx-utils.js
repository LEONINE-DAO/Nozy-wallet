/** Typical ZIP-317 × 4 orchard send with empty memo (NozyWallet policy). */
export const MANDATORY_ORCHARD_FEE_FALLBACK_ZATS = 40_000;

/**
 * ZIP-317 × 4 is required on every Nozy surface (CLI / API / desktop / extension).
 * Callers may pass a `fee` for display only — prove paths must use this value.
 */
export function mandatoryOrchardFeeZats(wasm, memo = "") {
  try {
    const fee = Number(wasm?.estimate_orchard_send_fee_zats?.(String(memo || ""), true));
    if (Number.isFinite(fee) && fee > 0) return Math.floor(fee);
  } catch (_) {
    // Fall through to the known empty-memo floor.
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

