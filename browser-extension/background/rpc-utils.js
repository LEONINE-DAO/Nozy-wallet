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
 * Loopback / RFC1918 / WSL / link-local — mixnet is skipped (Case A1), Chrome RPC fallback is OK.
 * @param {string} endpoint
 * @returns {boolean}
 */
export function isLocalRpcEndpoint(endpoint) {
  try {
    const host = new URL(normalizeRpcEndpoint(endpoint)).hostname.toLowerCase();
    if (host === "localhost" || host === "::1") return true;
    if (host.endsWith(".localhost")) return true;
    if (isWslStyleHost(host)) return true;
    const m = host.match(/^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$/);
    if (!m) return false;
    const a = Number(m[1]);
    const b = Number(m[2]);
    if (a === 127) return true;
    if (a === 10) return true;
    if (a === 192 && b === 168) return true;
    if (a === 172 && b >= 16 && b <= 31) return true;
    if (a === 169 && b === 254) return true;
    return false;
  } catch {
    return false;
  }
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
    const rpcMessage = body.error.message || `RPC ${method} error`;
    const e = new Error(rpcMessage);
    if (typeof body.error.code === "number") e.jsonRpcCode = body.error.code;
    e.jsonRpcMethod = method;
    throw e;
  }
  return body?.result ?? null;
}

/**
 * Compact-sync hosts speak gRPC / compact HTTP. JSON-RPC `getblockcount` against them is HTTP 415.
 * @param {string} endpoint
 * @returns {boolean}
 */
export function isLightwalletdEndpoint(endpoint) {
  try {
    const host = new URL(normalizeRpcEndpoint(endpoint)).hostname.toLowerCase();
    return host === "lwd.nozywallet.org" || host === "zec.rocks";
  } catch {
    return /lwd\.nozywallet\.org|zec\.rocks/i.test(String(endpoint ?? ""));
  }
}

/**
 * These statuses mean "something is listening, but it is not Zebrad JSON-RPC".
 * Windows often has a non-Zebra service on port 8232 that answers 415.
 * @param {number} status
 * @returns {boolean}
 */
export function rpcStatusShouldFailover(status) {
  return status === 404 || status === 405 || status === 415 || status === 501;
}

/**
 * @param {number} [timeoutMs]
 * @returns {AbortSignal | undefined}
 */
function rpcTimeoutSignal(timeoutMs) {
  const ms = Number(timeoutMs);
  if (!Number.isFinite(ms) || ms <= 0) return undefined;
  return AbortSignal.timeout(ms);
}

/**
 * Zebrad and zcashd require `Content-Type: application/json` with no charset.
 * Chrome appends `;charset=UTF-8` when the body is a string, and those nodes answer HTTP 415.
 * A byte body keeps the header exact. A 415 is retried once as `text/plain` for nodes that want that.
 *
 * @param {string} url
 * @param {string} method
 * @param {unknown[]} params
 * @param {number} [timeoutMs]
 * @returns {Promise<Response>}
 */
async function postJsonRpcOnce(url, method, params, timeoutMs) {
  const body = new TextEncoder().encode(
    JSON.stringify({ jsonrpc: "2.0", id: 1, method, params })
  );
  const send = (contentType) =>
    fetch(url, {
      method: "POST",
      headers: {
        "Content-Type": contentType,
        Accept: "application/json"
      },
      body,
      signal: rpcTimeoutSignal(timeoutMs)
    });
  const resp = await send("application/json");
  if (resp.status !== 415) return resp;
  return send("text/plain");
}

/** Last JSON-RPC URL that actually returned a result, so later calls skip a dead port. */
let preferredRpcEndpoint = null;

/** Force the next RPC calls to start at this URL. Spend must not reuse a shorter chain. */
export function pinRpcEndpoint(endpoint) {
  preferredRpcEndpoint = normalizeRpcEndpoint(endpoint);
}

/**
 * @param {string} endpoint
 * @param {string} method
 * @param {unknown[]} [params]
 * @param {{ timeoutMs?: number, extraCandidates?: string[] }} [opts]
 * @returns {Promise<{ result: unknown, endpoint: string }>}
 */
export async function rpcCallJson(endpoint, method, params = [], opts = {}) {
  let requested = "";
  try {
    requested = normalizeRpcEndpoint(endpoint);
  } catch (e) {
    if (!preferredRpcEndpoint) throw e instanceof Error ? e : new Error(String(e));
  }

  /** @type {string[]} */
  const candidates = [];
  const push = (u) => {
    if (!u || isLightwalletdEndpoint(u) || candidates.includes(u)) return;
    candidates.push(u);
  };
  if (opts.exact !== true) push(preferredRpcEndpoint);
  push(requested);

  let lastErr = null;
  let sawNetwork = false;
  let expanded = false;
  for (let i = 0; i < candidates.length; i += 1) {
    const url = candidates[i];
    let resp;
    try {
      resp = await postJsonRpcOnce(url, method, params, opts.timeoutMs);
    } catch (err) {
      sawNetwork = true;
      lastErr = err instanceof Error ? err : new Error(String(err));
      if (url === preferredRpcEndpoint) preferredRpcEndpoint = null;
    }
    if (!resp) {
      if (opts.exact === true) continue;
      if (!expanded) {
        expanded = true;
        const found = await findReachableRpcEndpoint(requested || "http://127.0.0.1:18232", {
          extraCandidates: opts.extraCandidates,
          skipEndpoints: candidates
        });
        push(found);
      }
      continue;
    }
    if (resp.status === 401 || resp.status === 403) {
      throw new Error(
        `RPC ${method} returned HTTP ${resp.status} (authentication required). ` +
          `For Zebra local dev, set enable_cookie_auth=false.`
      );
    }
    if (rpcStatusShouldFailover(resp.status)) {
      lastErr = new Error(`RPC ${method} HTTP ${resp.status}`);
      if (url === preferredRpcEndpoint) preferredRpcEndpoint = null;
      if (opts.exact === true) continue;
      if (!expanded) {
        expanded = true;
        const found = await findReachableRpcEndpoint(requested || "http://127.0.0.1:18232", {
          extraCandidates: opts.extraCandidates,
          skipEndpoints: candidates
        });
        push(found);
      }
      continue;
    }
    if (!resp.ok) throw new Error(`RPC ${method} HTTP ${resp.status}`);
    try {
      const result = await parseJsonRpcResponse(resp, url, method);
      preferredRpcEndpoint = url;
      return { result, endpoint: url };
    } catch (parseErr) {
      const msg = String(parseErr?.message ?? parseErr);
      if (!/web page|not JSON|DOCTYPE/i.test(msg)) throw parseErr;
      lastErr = parseErr instanceof Error ? parseErr : new Error(msg);
      if (url === preferredRpcEndpoint) preferredRpcEndpoint = null;
      if (opts.exact === true) continue;
      if (!expanded) {
        expanded = true;
        const found = await findReachableRpcEndpoint(requested || "http://127.0.0.1:18232", {
          extraCandidates: opts.extraCandidates,
          skipEndpoints: candidates
        });
        push(found);
      }
    }
  }

  if (isLightwalletdEndpoint(requested)) {
    throw new Error(
      `RPC ${method} cannot use ${requested}. That host is compact sync, not Zebrad JSON-RPC. ` +
        `Start a local node (port 18232 or 8232), then use Find node in Settings to send.`
    );
  }
  const networkMessage = sawNetwork && lastErr
    ? rpcNetworkErrorMessage(requested || String(endpoint), lastErr)
    : (lastErr instanceof Error ? lastErr.message : `RPC ${method} failed`);
  if (sawNetwork && lastErr) {
    throw new Error(networkMessage);
  }
  throw lastErr instanceof Error ? lastErr : new Error(`RPC ${method} failed`);
}

/**
 * @param {string} endpoint
 * @param {number} [timeoutMs=4000]
 * @returns {Promise<boolean>}
 */
/**
 * Block height from one exact URL. Does not fail over to another port.
 * @param {string} endpoint
 * @param {number} [timeoutMs]
 * @returns {Promise<number>}
 */
export async function rpcBlockCountAt(endpoint, timeoutMs = 2500) {
  const url = normalizeRpcEndpoint(endpoint);
  const resp = await postJsonRpcOnce(url, "getblockcount", [], timeoutMs);
  if (!resp.ok) throw new Error(`RPC getblockcount HTTP ${resp.status}`);
  const result = await parseJsonRpcResponse(resp, url, "getblockcount");
  const n = typeof result === "number" ? result : Number(result);
  if (!Number.isFinite(n) || n < 0) throw new Error("getblockcount was not a height");
  return Math.floor(n);
}

export async function probeZebradRpcEndpoint(endpoint, timeoutMs = 4000) {
  let url;
  try {
    url = normalizeRpcEndpoint(endpoint);
  } catch {
    return false;
  }
  if (isLightwalletdEndpoint(url)) return false;
  const ms = Math.max(500, Number(timeoutMs) || 4000);
  try {
    const resp = await postJsonRpcOnce(url, "getblockcount", [], ms);
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
 * @param {{ extraCandidates?: string[], companionBase?: string, skipEndpoints?: string[] }} [opts]
 * @returns {Promise<string|null>}
 */
export async function findReachableRpcEndpoint(currentEndpoint, opts = {}) {
  /** @type {string[]} */
  const candidates = [];
  const extra = Array.isArray(opts.extraCandidates)
    ? opts.extraCandidates.filter((u) => typeof u === "string")
    : [];
  const skip = new Set(
    (Array.isArray(opts.skipEndpoints) ? opts.skipEndpoints : [])
      .map((u) => {
        try {
          return normalizeRpcEndpoint(u);
        } catch {
          return "";
        }
      })
      .filter(Boolean)
  );
  try {
    const current = normalizeRpcEndpoint(currentEndpoint);
    if (!skip.has(current)) candidates.push(current);
  } catch {
    // ignore invalid current endpoint
  }
  candidates.push(...extra);

  // Local Zebrad / Crosslink JSON-RPC. On this Windows Crosslink GUI setup the
  // node listens on 18232 (8232 is often taken). Prefer the live Crosslink port.
  candidates.push(
    "http://127.0.0.1:18232",
    "http://127.0.0.1:8232"
  );
  // Public lightwalletd (lwd.nozywallet.org) is gRPC. Extension compact scan uses
  // HTTPS /compact (or /api/lwd/compact) — not this JSON-RPC probe list.

  // Deduplicate while preserving order.
  const unique = [...new Set(candidates)];
  let chosen = null;
  for (const endpoint of unique) {
    if (await probeZebradRpcEndpoint(endpoint, 3500)) {
      chosen = endpoint;
      break;
    }
  }
  void opts.companionBase;
  return chosen;
}

/**
 * Single JSON-RPC POST (no retries). Caller handles backoff if needed.
 * @param {string} endpointUrl
 * @param {string} method
 * @param {unknown[]} params
 */
async function jsonRpcPostOnce(endpointUrl, method, params = []) {
  const called = await rpcCallJson(endpointUrl, method, params, { exact: true });
  return called.result;
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
