export function resolveTxidFromBroadcast(broadcastResult, fallbackTxid = "") {
  const txid =
    broadcastResult?.txid ??
    broadcastResult?.result?.txid ??
    broadcastResult?.result ??
    broadcastResult ??
    fallbackTxid;
  return String(txid || "");
}

export function inferInputMode(inputsUsed, explicitMode) {
  if (explicitMode === "single" || explicitMode === "multi") return explicitMode;
  return Number(inputsUsed ?? 0) <= 1 ? "single" : "multi";
}

export function buildBuiltTxStateEntry({ id, origin, proving, createdAt }) {
  const inputsUsed = Number(proving?.inputs_used ?? 0);
  return {
    id,
    txid: String(proving?.txid || ""),
    state: "built",
    origin: String(origin || ""),
    recipientAddress: String(proving.recipientAddress || ""),
    amount: Number(proving.amount ?? 0),
    fee: Number(proving.fee ?? 0),
    memo: String(proving.memo || ""),
    inputsUsed,
    inputMode: inferInputMode(inputsUsed, String(proving?.input_mode || "")),
    rawTxHex: String(proving.rawTxHex || ""),
    createdAt,
    updatedAt: createdAt,
    error: null
  };
}

export function buildFailedTxStateEntry({ id, origin, tx, preflight, error, createdAt, parseAmount }) {
  const inputsUsed = Number(preflight?.inputs_used ?? 0);
  return {
    id,
    txid: null,
    state: "failed",
    origin: String(origin || ""),
    recipientAddress: String(tx?.to ?? ""),
    amount: Number(parseAmount(tx?.value ?? tx?.amount) ?? 0),
    fee: null,
    memo: String(tx?.memo ?? ""),
    inputsUsed,
    inputMode: inferInputMode(inputsUsed, String(preflight?.input_mode || "")),
    rawTxHex: null,
    createdAt,
    updatedAt: createdAt,
    error: String(error || "Unknown error")
  };
}

export function findRecentBuiltTxId(txs, origin, now, windowMs = 5 * 60 * 1000) {
  if (!Array.isArray(txs)) return null;
  const targetOrigin = String(origin || "");
  const candidate = txs
    .slice()
    .reverse()
    .find((t) => t.state === "built" && t.origin === targetOrigin && t.updatedAt >= now - windowMs);
  return candidate?.id || null;
}

export function nextLifecycleStateFromConfirmation(confirmation) {
  return confirmation?.confirmed ? "confirmed" : "pending";
}
