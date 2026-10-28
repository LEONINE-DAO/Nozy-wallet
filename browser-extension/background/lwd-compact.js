/**
 * Read-only compact block HTTP client for public lightwalletd.
 * Keys never leave the extension — these endpoints only return public compact ciphertext.
 */

const PUBLIC_COMPACT_HTTP_BASES = [
  "https://lwd.nozywallet.org/compact",
  // Fallback until Caddy /compact is deployed on the LWD host.
  "https://api.nozywallet.org/api/lwd/compact"
];

let preferredCompactBase = null;

async function compactFetch(pathAndQuery) {
  const bases = preferredCompactBase
    ? [preferredCompactBase, ...PUBLIC_COMPACT_HTTP_BASES.filter((b) => b !== preferredCompactBase)]
    : PUBLIC_COMPACT_HTTP_BASES;
  let lastErr = null;
  for (const base of bases) {
    const url = `${base.replace(/\/$/, "")}${pathAndQuery}`;
    try {
      const resp = await fetch(url, { method: "GET", cache: "no-store" });
      if (!resp.ok) {
        lastErr = new Error(`Compact HTTP ${resp.status} from ${base}`);
        continue;
      }
      const body = await resp.json();
      preferredCompactBase = base;
      return body;
    } catch (e) {
      lastErr = e instanceof Error ? e : new Error(String(e));
    }
  }
  throw lastErr || new Error("Compact HTTP unreachable");
}

export async function compactChainTip() {
  const body = await compactFetch("/tip");
  const h = Number(body?.height);
  if (!Number.isFinite(h) || h < 0) throw new Error("Compact tip missing height");
  return Math.floor(h);
}

export async function compactTreeState(height) {
  const q =
    height === undefined || height === null || height === ""
      ? "/treestate"
      : `/treestate?height=${encodeURIComponent(String(Math.floor(Number(height))))}`;
  return compactFetch(q);
}

/**
 * Broadcast a raw transaction through lightwalletd `SendTransaction`.
 * The compact HTTP host proxies this; it is not Zebra `sendrawtransaction`.
 * @param {string} rawTxHex
 * @returns {Promise<{ success?: boolean, txid?: string }>}
 */
export async function compactSubmitRawTx(rawTxHex) {
  const clean = String(rawTxHex || "")
    .trim()
    .replace(/^0x/i, "");
  if (!clean || clean.length % 2 !== 0) {
    throw new Error("raw transaction hex is invalid");
  }
  const bases = preferredCompactBase
    ? [preferredCompactBase, ...PUBLIC_COMPACT_HTTP_BASES.filter((b) => b !== preferredCompactBase)]
    : PUBLIC_COMPACT_HTTP_BASES;
  let lastErr = null;
  for (const base of bases) {
    const url = `${base.replace(/\/$/, "")}/submit`;
    try {
      const resp = await fetch(url, {
        method: "POST",
        headers: { "content-type": "application/json" },
        cache: "no-store",
        body: JSON.stringify({ raw_tx_hex: clean })
      });
      if (!resp.ok) {
        lastErr = new Error(`Compact submit HTTP ${resp.status} from ${base}`);
        continue;
      }
      const body = await resp.json();
      preferredCompactBase = base;
      return body && typeof body === "object" ? body : { success: true };
    } catch (e) {
      lastErr = e instanceof Error ? e : new Error(String(e));
    }
  }
  throw lastErr || new Error("Compact submit unreachable");
}

/**
 * @param {number} start inclusive
 * @param {number} end inclusive
 * @returns {Promise<object[]>} slim scan blocks
 */
export async function compactFetchBlocks(start, end) {
  const s = Math.floor(Number(start));
  const e = Math.floor(Number(end));
  if (!Number.isFinite(s) || !Number.isFinite(e) || e < s) {
    throw new Error(`Invalid compact range ${start}..${end}`);
  }
  const body = await compactFetch(
    `/blocks?start=${encodeURIComponent(String(s))}&end=${encodeURIComponent(String(e))}`
  );
  return Array.isArray(body?.blocks) ? body.blocks : [];
}

export function compactHttpBases() {
  return [...PUBLIC_COMPACT_HTTP_BASES];
}
