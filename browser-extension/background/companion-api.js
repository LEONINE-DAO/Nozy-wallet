/**
 * HTTP client for the Nozy desktop companion (`nozywallet-api` / api-server on localhost).
 * Same LWD operations as Tauri and `zeaking::lwd`; MV3 service worker must not run gRPC/SQLite itself.
 */

const DEFAULT_COMPANION_BASE = "http://127.0.0.1:3000";

export function normalizeCompanionBase(url) {
  const s = String(url ?? "").trim().replace(/\/+$/, "");
  return s || DEFAULT_COMPANION_BASE;
}

async function readErrorBody(r) {
  try {
    const t = await r.text();
    return t || `HTTP ${r.status}`;
  } catch (_) {
    return `HTTP ${r.status}`;
  }
}

/**
 * @param {string} [baseUrl]
 * @returns {Promise<{ companionReachable: boolean, healthStatus: number, lwdChainTip: object | null }>}
 */
export async function companionStatus(baseUrl) {
  const base = normalizeCompanionBase(baseUrl);
  let healthStatus = 0;
  let companionReachable = false;
  try {
    const health = await fetch(`${base}/health`, { method: "GET" });
    healthStatus = health.status;
    companionReachable = health.ok;
  } catch (_) {
    companionReachable = false;
  }
  let lwdChainTip = null;
  if (companionReachable) {
    try {
      const r = await fetch(`${base}/api/lwd/chain-tip`);
      if (r.ok) lwdChainTip = await r.json();
    } catch (_) { /* ignore */ }
  }
  return { companionReachable, healthStatus, lwdChainTip };
}

/**
 * @param {string} [baseUrl]
 * @param {string} [lightwalletdUrl]
 */
export async function companionLwdInfo(baseUrl, lightwalletdUrl) {
  const base = normalizeCompanionBase(baseUrl);
  const q =
    lightwalletdUrl && String(lightwalletdUrl).trim()
      ? `?lightwalletd_url=${encodeURIComponent(String(lightwalletdUrl).trim())}`
      : "";
  const r = await fetch(`${base}/api/lwd/info${q}`);
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/**
 * @param {string} [baseUrl]
 * @param {string} [lightwalletdUrl]
 */
export async function companionLwdChainTip(baseUrl, lightwalletdUrl) {
  const base = normalizeCompanionBase(baseUrl);
  const q =
    lightwalletdUrl && String(lightwalletdUrl).trim()
      ? `?lightwalletd_url=${encodeURIComponent(String(lightwalletdUrl).trim())}`
      : "";
  const r = await fetch(`${base}/api/lwd/chain-tip${q}`);
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/**
 * Triggers compact sync on the companion machine (desktop SQLite path).
 * @param {string} [baseUrl]
 * @param {{ start: number, end?: number, lightwalletd_url?: string, db_path?: string }} body
 */
export async function companionLwdSyncCompact(baseUrl, body) {
  const base = normalizeCompanionBase(baseUrl);
  const r = await fetch(`${base}/api/lwd/sync/compact`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      start: body.start,
      end: body.end,
      lightwalletd_url: body.lightwalletd_url,
      db_path: body.db_path
    })
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}
