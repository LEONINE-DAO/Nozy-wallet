/**
 * Shared helpers for JSON-RPC calls to a local Zebrad node from the extension.
 */

const WSL_HOST_RE = /^172\.(1[6-9]|2\d|3[0-1])\.\d+\.\d+$/;

/**
 * @param {string} host
 * @returns {boolean}
 */
export function isWslStyleHost(host) {
  return WSL_HOST_RE.test(String(host ?? "").trim());
}

/**
 * @param {Response} resp
 * @param {string} endpoint
 * @param {string} method
 * @returns {Promise<unknown>}
 */
export async function parseJsonRpcResponse(resp, endpoint, method) {
  const text = await resp.text();
  const trimmed = text.trim();
  if (!trimmed) {
    throw new Error(`RPC ${method} at ${endpoint} returned an empty body.`);
  }
  if (/^\s*</.test(trimmed) || /<!DOCTYPE/i.test(trimmed)) {
    throw new Error(
      `RPC ${method} at ${endpoint} returned a web page, not JSON. ` +
        `Check the URL and port (Zebrad JSON-RPC is usually http://127.0.0.1:8232).`
    );
  }
  let body;
  try {
    body = JSON.parse(trimmed);
  } catch {
    throw new Error(
      `RPC ${method} at ${endpoint} returned non-JSON (${trimmed.slice(0, 120)}…).`
    );
  }
  if (body?.error) {
    const e = new Error(body.error.message || `RPC ${method} error`);
    if (typeof body.error.code === "number") e.jsonRpcCode = body.error.code;
    e.jsonRpcMethod = method;
    throw e;
  }
  return body?.result ?? null;
}

/**
 * @param {string} endpoint
 * @param {number} [timeoutMs=4000]
 * @returns {Promise<boolean>}
 */
export async function probeZebradRpcEndpoint(endpoint, timeoutMs = 4000) {
  let url;
  try {
    url = normalizeRpcEndpoint(endpoint);
  } catch {
    return false;
  }
  const ms = Math.max(500, Number(timeoutMs) || 4000);
  try {
    const resp = await fetch(url, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method: "getblockcount",
        params: []
      }),
      signal: AbortSignal.timeout(ms)
    });
    if (!resp.ok) return false;
    const result = await parseJsonRpcResponse(resp, url, "getblockcount");
    return typeof result === "number" || typeof result === "string";
  } catch {
    return false;
  }
}

/**
 * @param {unknown} raw
 * @returns {string}
 */
export function normalizeRpcEndpoint(raw) {
  let s = String(raw ?? "").trim();
  if (!s) {
    throw new Error("RPC URL is empty. Set it in Settings (e.g. http://127.0.0.1:8232).");
  }
  // Accept shorthand like `zec.rocks443` by converting to `zec.rocks:443`.
  if (!/^[a-z][a-z0-9+.-]*:\/\//i.test(s)) {
    const m = s.match(/^([a-z0-9.-]*[a-z.-])(\d{2,5})(\/.*)?$/i);
    if (m && m[1].includes(".")) {
      const host = m[1];
      const port = m[2];
      const tail = m[3] || "";
      s = `${host}:${port}${tail}`;
    }
    const isLocalHostLike =
      /^localhost(?::\d+)?(\/|$)/i.test(s) ||
      /^127\.\d+\.\d+\.\d+(?::\d+)?(\/|$)/.test(s) ||
      /^10\.\d+\.\d+\.\d+(?::\d+)?(\/|$)/.test(s) ||
      /^192\.168\.\d+\.\d+(?::\d+)?(\/|$)/.test(s) ||
      /^172\.(1[6-9]|2\d|3[0-1])\.\d+\.\d+(?::\d+)?(\/|$)/.test(s);
    s = `${isLocalHostLike ? "http" : "https"}://${s}`;
  }
  let u;
  try {
    u = new URL(s);
  } catch {
    throw new Error(`Invalid RPC URL: ${s}`);
  }
  if (u.protocol !== "http:" && u.protocol !== "https:") {
    throw new Error("RPC URL must use http:// or https://");
  }
  if (!u.hostname) {
    throw new Error("RPC URL must include a hostname (e.g. 127.0.0.1 or your WSL IP).");
  }
  const out = u.toString();
  const normalized = out.endsWith("/") ? out.slice(0, -1) : out;
  return normalized;
}

/**
 * Turn low-level fetch failures into actionable messages for wallet users.
 * @param {string} endpoint
 * @param {unknown} err
 * @returns {string}
 */
export function rpcNetworkErrorMessage(endpoint, err) {
  const base = String(err?.message ?? err ?? "unknown");
  if (
    base === "Failed to fetch" ||
    /NetworkError|network failed|Load failed|ECONNREFUSED|ERR_CONNECTION|aborted|timed out/i.test(
      base
    )
  ) {
    return (
      `Cannot reach RPC at ${endpoint}. ` +
      `Confirm the node is running and the port matches your config. ` +
      `If the node runs in WSL or Docker, try your VM/container IP instead of 127.0.0.1 from Windows. ` +
      `For Zebra local dev, set enable_cookie_auth=false (or supply auth your node expects).`
    );
  }
  return base;
}

/**
 * Probe candidate RPC endpoints and return the first reachable URL.
 * @param {string} currentEndpoint
 * @param {{ extraCandidates?: string[], companionBase?: string }} [opts]
 * @returns {Promise<string|null>}
 */
export async function findReachableRpcEndpoint(currentEndpoint, opts = {}) {
  /** @type {string[]} */
  const candidates = [];
  const extra = Array.isArray(opts.extraCandidates)
    ? opts.extraCandidates.filter((u) => typeof u === "string")
    : [];
  try {
    candidates.push(normalizeRpcEndpoint(currentEndpoint));
  } catch {
    // ignore invalid current endpoint
  }
  candidates.push(...extra);

  // Common local Zebrad RPC ports/configs.
  candidates.push(
    "http://127.0.0.1:18232",
    "http://127.0.0.1:8232",
    "https://zec.rocks:443",
    "https://testnet.zec.rocks:443"
  );

  // Deduplicate while preserving order.
  const unique = [...new Set(candidates)];
  for (const endpoint of unique) {
    if (await probeZebradRpcEndpoint(endpoint, 3500)) {
      return endpoint;
    }
  }
  void opts.companionBase;
  return null;
}

/**
 * Single JSON-RPC POST (no retries). Caller handles backoff if needed.
 * @param {string} endpointUrl
 * @param {string} method
 * @param {unknown[]} params
 */
async function jsonRpcPostOnce(endpointUrl, method, params = []) {
  const resp = await fetch(endpointUrl, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params })
  });
  if (!resp.ok) throw new Error(`RPC ${method} HTTP ${resp.status}`);
  const body = await resp.json();
  if (body?.error) {
    const e = new Error(body.error.message || `RPC ${method} error`);
    if (typeof body.error.code === "number") e.jsonRpcCode = body.error.code;
    e.jsonRpcMethod = method;
    throw e;
  }
  return body?.result ?? null;
}

/**
 * Verbose block JSON (verbosity 2) at chain height: `getblockhash` then `getblock(hash, 2)`.
 * Matches nozy `ZebraClient` — Zebrad expects a block hash for verbose `getblock`, not a numeric height string.
 *
 * @param {string} endpoint RPC base URL (same rules as {@link normalizeRpcEndpoint})
 * @param {number} height block height
 * @returns {Promise<unknown>} RPC `result` (block object)
 */
export async function rpcGetBlockVerboseByHeight(endpoint, height) {
  const url = normalizeRpcEndpoint(String(endpoint ?? "").trim());
  const h = Math.max(0, Math.floor(Number(height)));
  if (!Number.isFinite(h)) {
    throw new Error(`Invalid block height: ${height}`);
  }
  const hash = await jsonRpcPostOnce(url, "getblockhash", [h]);
  if (typeof hash !== "string" || !hash) {
    throw new Error(`getblockhash(${h}) returned empty`);
  }
  return jsonRpcPostOnce(url, "getblock", [hash, 2]);
}
