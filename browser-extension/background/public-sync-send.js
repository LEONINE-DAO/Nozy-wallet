/**
 * Public lightwalletd send helpers.
 * lwd.nozywallet.org speaks compact blocks, not Zebra JSON-RPC.
 */

const PUBLIC_SYNC_HOST = "lwd.nozywallet.org";
/** Inclusive heights per compact HTTP request. Server rejects end-start > 500. */
const COMPACT_RANGE_INCLUSIVE = 500;

export function isNozyPublicSyncUrl(url) {
  const raw = String(url || "").trim();
  if (!raw) return false;
  try {
    const withScheme = /^https?:\/\//i.test(raw) ? raw : `http://${raw}`;
    return new URL(withScheme).hostname.toLowerCase() === PUBLIC_SYNC_HOST;
  } catch {
    return raw.toLowerCase().includes(PUBLIC_SYNC_HOST);
  }
}

export function feeBlockFromChainTip(tip, mainnet = true) {
  const n = Number(tip);
  return {
    mainnet: mainnet !== false,
    minedBlockHeight: (Number.isFinite(n) && n >= 0 ? Math.floor(n) : 0) + 1
  };
}

export function mainnetFromNetworkName(network) {
  const n = String(network || "").trim().toLowerCase();
  if (!n) return true;
  return n !== "test" && n !== "testnet" && n !== "regtest" && !n.includes("testnet");
}

/** Zebra-style root fields. A lightwalletd frontier blob is not a root. */
export function explicitAnchorHex(tr) {
  if (!tr || typeof tr !== "object") return "";
  const o = tr.orchard;
  const c = o?.commitments ?? o;
  const fromZebra = c?.finalRoot ?? c?.final_root ?? o?.finalRoot ?? o?.final_root ?? "";
  let hex = String(tr.anchor ?? tr.orchardTree?.anchor ?? fromZebra ?? "").trim();
  if (hex.startsWith("0x") || hex.startsWith("0X")) hex = hex.slice(2);
  return /^[0-9a-f]{64}$/i.test(hex) ? hex.toLowerCase() : "";
}

export function finalStateHex(tr) {
  if (!tr || typeof tr !== "object") return "";
  const o = tr.orchard;
  const c = o?.commitments ?? o;
  const fromCommitments =
    (typeof c?.finalState === "string" && c.finalState) ||
    (typeof c?.final_state === "string" && c.final_state) ||
    "";
  return String(fromCommitments || tr.orchard_tree || "")
    .trim()
    .replace(/^0x/i, "");
}

/**
 * @param {object} tr compact or Zebra treestate
 * @param {(finalStateHex: string) => string} [deriveFromFinalState]
 */
export function anchorHexFromCompactTreestate(tr, deriveFromFinalState) {
  const explicit = explicitAnchorHex(tr);
  if (explicit) return explicit;
  const finalState = finalStateHex(tr);
  if (!finalState || typeof deriveFromFinalState !== "function") return "";
  try {
    const derived = String(deriveFromFinalState(finalState) || "").replace(/^0x/i, "");
    return /^[0-9a-f]{64}$/i.test(derived) ? derived.toLowerCase() : "";
  } catch {
    return "";
  }
}

export async function compactBlocksByHeight(fetchRange, start, end) {
  const s = Math.floor(Number(start));
  const e = Math.floor(Number(end));
  const map = new Map();
  if (!Number.isFinite(s) || !Number.isFinite(e) || e < s) return map;
  let h = s;
  while (h <= e) {
    const chunkEnd = Math.min(e, h + COMPACT_RANGE_INCLUSIVE - 1);
    const blocks = await fetchRange(h, chunkEnd);
    const list = Array.isArray(blocks) ? blocks : [];
    for (const block of list) {
      const height = Number(block?.height);
      if (Number.isFinite(height)) map.set(height, block);
    }
    h = chunkEnd + 1;
  }
  return map;
}

/**
 * Advance an Orchard witness from `fromExclusive` up through `toInclusive`
 * using compact scan blocks. Missing heights contribute no commitments.
 */
export async function advanceWitnessOverRange(
  advance,
  witnessHex,
  fromExclusive,
  toInclusive,
  fetchRange
) {
  let hex = witnessHex;
  const start = Math.floor(Number(fromExclusive)) + 1;
  const end = Math.floor(Number(toInclusive));
  if (!Number.isFinite(start) || !Number.isFinite(end) || start > end) return hex;
  const byHeight = await compactBlocksByHeight(fetchRange, start, end);
  for (let h = start; h <= end; h += 1) {
    const block = byHeight.get(h) || { height: h, tx: [] };
    hex = advance(hex, JSON.stringify(block));
  }
  return hex;
}

/** Display txid: reversed double-SHA256 of the raw transaction bytes. */
export async function zcashTxidFromRawHex(rawHex) {
  const clean = String(rawHex || "")
    .trim()
    .replace(/^0x/i, "");
  if (!clean || clean.length % 2 !== 0 || /[^0-9a-f]/i.test(clean)) {
    throw new Error("raw transaction hex is invalid");
  }
  const bytes = new Uint8Array(clean.length / 2);
  for (let i = 0; i < bytes.length; i += 1) {
    bytes[i] = parseInt(clean.slice(i * 2, i * 2 + 2), 16);
  }
  const first = await crypto.subtle.digest("SHA-256", bytes);
  const second = new Uint8Array(await crypto.subtle.digest("SHA-256", first));
  return Array.from(second)
    .reverse()
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}
