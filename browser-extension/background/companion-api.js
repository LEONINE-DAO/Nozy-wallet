/**
 * HTTP client for the Nozy desktop companion (`nozywallet-api` / api-server on localhost).
 * Same LWD operations as Tauri and `zeaking::lwd`; MV3 service worker must not run gRPC/SQLite itself.
 */

const DEFAULT_COMPANION_BASE = "http://127.0.0.1:3000";
export const COMPANION_API_KEY_STORAGE = "nozy_companion_api_key_v1";

export function normalizeCompanionBase(url) {
  const s = String(url ?? "").trim().replace(/\/+$/, "");
  return s || DEFAULT_COMPANION_BASE;
}

export async function getCompanionApiKey() {
  return new Promise((resolve) => {
    try {
      chrome.storage.local.get([COMPANION_API_KEY_STORAGE], (v) => {
        const key = v?.[COMPANION_API_KEY_STORAGE];
        resolve(typeof key === "string" ? key.trim() : "");
      });
    } catch (_) {
      resolve("");
    }
  });
}

export async function setCompanionApiKey(key) {
  const value = String(key ?? "").trim();
  return new Promise((resolve, reject) => {
    chrome.storage.local.set({ [COMPANION_API_KEY_STORAGE]: value }, () => {
      if (chrome.runtime.lastError) reject(chrome.runtime.lastError);
      else resolve();
    });
  });
}

async function companionHeaders(extra = {}) {
  const headers = { ...extra };
  const key = await getCompanionApiKey();
  if (key) headers["X-API-Key"] = key;
  return headers;
}

async function companionFetch(baseUrl, path, init = {}) {
  const base = normalizeCompanionBase(baseUrl);
  const headers = await companionHeaders(init.headers || {});
  return fetch(`${base}${path}`, { ...init, headers });
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
 * Desktop / CLI receive UA (account 0) from the companion wallet.dat.
 * Used to confirm an extension restore matches this PC's Nozy wallet.
 * @param {string} [baseUrl]
 * @param {string} [password]
 * @returns {Promise<{ address: string }>}
 */
export async function companionGenerateAddress(baseUrl, password) {
  const r = await companionFetch(baseUrl, "/api/address/generate", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ password: password || null, account: 0 })
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/**
 * Read Nozy Desktop / api-server config (zebra_url for RPC autoconnect).
 * @param {string} [baseUrl]
 * @returns {Promise<{ zebra_url?: string } | null>}
 */
export async function companionGetConfig(baseUrl) {
  const base = normalizeCompanionBase(baseUrl);
  try {
    const r = await companionFetch(base, "/api/config", { method: "GET" });
    if (!r.ok) return null;
    const body = await r.json();
    return body && typeof body === "object" ? body : null;
  } catch (_) {
    return null;
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
    const health = await companionFetch(base, "/health", { method: "GET" });
    healthStatus = health.status;
    companionReachable = health.ok;
  } catch (_) {
    companionReachable = false;
  }
  let lwdChainTip = null;
  if (companionReachable) {
    try {
      const r = await companionFetch(base, "/api/lwd/chain-tip");
      if (r.ok) lwdChainTip = await r.json();
    } catch (_) {
      /* ignore */
    }
  }
  return { companionReachable, healthStatus, lwdChainTip };
}

/**
 * @param {string} [baseUrl]
 * @param {string} [lightwalletdUrl]
 */
export async function companionLwdInfo(baseUrl, lightwalletdUrl) {
  const q =
    lightwalletdUrl && String(lightwalletdUrl).trim()
      ? `?lightwalletd_url=${encodeURIComponent(String(lightwalletdUrl).trim())}`
      : "";
  const r = await companionFetch(baseUrl, `/api/lwd/info${q}`);
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/**
 * @param {string} [baseUrl]
 * @param {string} [lightwalletdUrl]
 */
export async function companionLwdChainTip(baseUrl, lightwalletdUrl) {
  const q =
    lightwalletdUrl && String(lightwalletdUrl).trim()
      ? `?lightwalletd_url=${encodeURIComponent(String(lightwalletdUrl).trim())}`
      : "";
  const r = await companionFetch(baseUrl, `/api/lwd/chain-tip${q}`);
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/**
 * Triggers compact sync on the companion machine (desktop SQLite path).
 * @param {string} [baseUrl]
 * @param {{ start: number, end?: number, lightwalletd_url?: string, db_path?: string, resume?: boolean }} body
 */
export async function companionLwdSyncCompact(baseUrl, body) {
  const r = await companionFetch(baseUrl, "/api/lwd/sync/compact", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      start: body.start,
      end: body.end,
      lightwalletd_url: body.lightwalletd_url,
      db_path: body.db_path,
      resume: body.resume
    })
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/**
 * Sync compact blocks from next missing height through chain tip (companion desktop DB).
 * @param {string} [baseUrl]
 * @param {{ lightwalletd_url?: string, db_path?: string, start_floor?: number, persist_progress_every?: number }} body
 */
export async function companionLwdSyncCompactToTip(baseUrl, body) {
  const r = await companionFetch(baseUrl, "/api/lwd/sync/compact-to-tip", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      lightwalletd_url: body.lightwalletd_url,
      db_path: body.db_path,
      start_floor: body.start_floor,
      persist_progress_every: body.persist_progress_every
    })
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/**
 * Refresh pending pilot txs (mark expired, release notes on companion wallet).
 * @param {string} [baseUrl]
 */
export async function companionCheckConfirmations(baseUrl) {
  const r = await companionFetch(baseUrl, "/api/transaction/check-confirmations", {
    method: "POST"
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/**
 * Rebuild an expired transaction at priority fee via companion api-server.
 * @param {string} [baseUrl]
 * @param {{ original_txid: string, password?: string, zebra_url?: string }} body
 */
export async function companionSpeedUpTransaction(baseUrl, body) {
  const r = await companionFetch(baseUrl, "/api/transaction/speed-up", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      original_txid: body.original_txid,
      password: body.password ?? "",
      zebra_url: body.zebra_url
    })
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/**
 * Quiet Sapling legacy status from companion wallet notes.
 * @param {string} [baseUrl]
 */
export async function companionSaplingStatus(baseUrl) {
  const r = await companionFetch(baseUrl, "/api/sapling/status");
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/**
 * Scan companion LWD compact cache for Sapling notes.
 * @param {string} [baseUrl]
 * @param {{ password?: string, start_floor?: number, full?: boolean }} [body]
 */
export async function companionSaplingScan(baseUrl, body = {}) {
  const r = await companionFetch(baseUrl, "/api/sapling/scan", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      password: body.password ?? null,
      start_floor: body.start_floor ?? null,
      full: body.full ?? false
    })
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/**
 * Move companion legacy Sapling notes into Orchard/Ironwood.
 * @param {string} [baseUrl]
 * @param {{ password?: string, dry_run?: boolean, no_broadcast?: boolean }} [body]
 */
export async function companionSaplingShield(baseUrl, body = {}) {
  const r = await companionFetch(baseUrl, "/api/sapling/shield", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      password: body.password ?? null,
      dry_run: body.dry_run ?? false,
      no_broadcast: body.no_broadcast ?? false
    })
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/**
 * Resolve a Zcash name via companion `/api/zns/resolve`.
 * @param {string} [baseUrl]
 * @param {{ name: string, network?: string }} body
 */
export async function companionZnsResolve(baseUrl, body) {
  const r = await companionFetch(baseUrl, "/api/zns/resolve", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      name: body.name,
      network: body.network ?? "mainnet"
    })
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/** @param {string} [baseUrl] @param {string} [env] */
export async function companionVoteStatus(baseUrl, env = "prod") {
  const q = new URLSearchParams({ env });
  const r = await companionFetch(baseUrl, `/api/vote/status?${q}`);
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/** @param {string} [baseUrl] @param {string} [env] */
export async function companionVoteActive(baseUrl, env = "prod") {
  const q = new URLSearchParams({ env });
  const r = await companionFetch(baseUrl, `/api/vote/active?${q}`);
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/** @param {string} [baseUrl] @param {{ password?: string, env?: string }} [body] */
export async function companionVoteExportNotes(baseUrl, body = {}) {
  const r = await companionFetch(baseUrl, "/api/vote/export-notes", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      password: body.password ?? "",
      env: body.env ?? "prod"
    })
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/** @param {string} [baseUrl] @param {{ notes_json: string }} body */
export async function companionVoteImportNotes(baseUrl, body) {
  const r = await companionFetch(baseUrl, "/api/vote/import-notes", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ notes_json: body.notes_json ?? "" })
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/** @param {string} [baseUrl] */
export async function companionVoteSigningRequest(baseUrl) {
  const r = await companionFetch(baseUrl, "/api/vote/signing-request");
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/** @param {string} [baseUrl] @param {{ sig_json: string, env?: string }} body */
export async function companionVoteSubmitDelegationSig(baseUrl, body) {
  const r = await companionFetch(baseUrl, "/api/vote/submit-delegation-sig", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      sig_json: body.sig_json ?? "",
      env: body.env ?? "prod"
    })
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/** @param {string} [baseUrl] @param {{ env?: string }} [body] */
export async function companionVotePrepare(baseUrl, body = {}) {
  const r = await companionFetch(baseUrl, "/api/vote/prepare", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ env: body.env ?? "prod" })
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/** @param {string} [baseUrl] @param {{ env?: string }} [body] */
export async function companionVoteDelegate(baseUrl, body = {}) {
  const r = await companionFetch(baseUrl, "/api/vote/delegate", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ env: body.env ?? "prod" })
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/** @param {string} [baseUrl] @param {{ password?: string, env?: string }} [body] */
export async function companionVoteSignDelegation(baseUrl, body = {}) {
  const r = await companionFetch(baseUrl, "/api/vote/sign-delegation", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      password: body.password ?? "",
      env: body.env ?? "prod"
    })
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/** @param {string} [baseUrl] @param {{ env?: string, wait?: boolean }} [body] */
export async function companionVoteDelegateFinish(baseUrl, body = {}) {
  const r = await companionFetch(baseUrl, "/api/vote/delegate-finish", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      env: body.env ?? "prod",
      wait: body.wait !== false
    })
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/**
 * @param {string} [baseUrl]
 * @param {{ env?: string, choices: Record<string, number>, delegation_tx?: string, single_share?: boolean, wait?: boolean }} body
 */
export async function companionVoteCast(baseUrl, body) {
  const r = await companionFetch(baseUrl, "/api/vote/cast", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      env: body.env ?? "prod",
      choices: body.choices ?? {},
      delegation_tx: body.delegation_tx ?? null,
      single_share: body.single_share === true,
      wait: body.wait !== false
    })
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/** @param {string} [baseUrl] */
export async function companionCrosslinkStatus(baseUrl) {
  const r = await companionFetch(baseUrl, "/api/crosslink/status");
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/** @param {string} [baseUrl] */
export async function companionCrosslinkPositions(baseUrl) {
  const r = await companionFetch(baseUrl, "/api/crosslink/positions");
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/** @param {string} [baseUrl] @param {boolean} [zats] */
export async function companionCrosslinkRoster(baseUrl, zats = false) {
  const q = new URLSearchParams();
  if (zats) q.set("zats", "true");
  const r = await companionFetch(baseUrl, `/api/crosslink/roster?${q}`);
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/**
 * @param {string} [baseUrl]
 * @param {{ amount_ctaz: number, finalizer: string, force?: boolean }} body
 */
export async function companionCrosslinkStake(baseUrl, body) {
  const r = await companionFetch(baseUrl, "/api/crosslink/stake", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      amount_ctaz: body.amount_ctaz,
      finalizer: body.finalizer,
      force: body.force === true
    })
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/**
 * @param {string} [baseUrl]
 * @param {{ bond: string, finalizer: string }} body
 */
export async function companionCrosslinkRetarget(baseUrl, body) {
  const r = await companionFetch(baseUrl, "/api/crosslink/retarget", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      bond: body.bond,
      finalizer: body.finalizer
    })
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/**
 * @param {string} [baseUrl]
 * @param {{ bond: string, force?: boolean }} body
 */
export async function companionCrosslinkUnbond(baseUrl, body) {
  const r = await companionFetch(baseUrl, "/api/crosslink/unbond", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      bond: body.bond,
      force: body.force === true
    })
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/**
 * @param {string} [baseUrl]
 * @param {{ bond: string, force?: boolean }} body
 */
export async function companionCrosslinkWithdraw(baseUrl, body) {
  const r = await companionFetch(baseUrl, "/api/crosslink/withdraw", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      bond: body.bond,
      force: body.force === true
    })
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/** @param {string} [baseUrl] */
export async function companionCrosslinkWalletStatus(baseUrl) {
  const r = await companionFetch(baseUrl, "/api/crosslink/wallet-status");
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/** @param {string} [baseUrl] */
export async function companionCrosslinkWalletUfvk(baseUrl) {
  const r = await companionFetch(baseUrl, "/api/crosslink/wallet-ufvk");
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/**
 * Broadcast raw tx hex through companion `ZebraClient` (Nym mixnet / Tor / local — same as desktop).
 * @param {string} [baseUrl]
 * @param {{ raw_transaction_hex: string, zebra_url?: string }} body
 */
export async function companionBroadcastRaw(baseUrl, body) {
  const r = await companionFetch(baseUrl, "/api/transaction/broadcast", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      raw_transaction_hex: body.raw_transaction_hex,
      zebra_url: body.zebra_url || undefined
    })
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/** @param {string} [baseUrl] */
export async function companionSendEgress(baseUrl) {
  const r = await companionFetch(baseUrl, "/api/privacy/send-egress");
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/** @param {string} [baseUrl] */
export async function companionPrivacyNetwork(baseUrl) {
  const r = await companionFetch(baseUrl, "/api/config/privacy-network");
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/**
 * @param {string} [baseUrl]
 * @param {Record<string, boolean | string | null | undefined>} patch
 */
export async function companionSetPrivacyNetwork(baseUrl, patch) {
  const r = await companionFetch(baseUrl, "/api/config/privacy-network", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(patch ?? {})
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/** @param {string} [baseUrl] */
export async function companionNymMixnet(baseUrl) {
  const r = await companionFetch(baseUrl, "/api/privacy/nym-mixnet");
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/**
 * @param {string} [baseUrl]
 * @param {string} [lightwalletdUrl]
 */
export async function companionNymDvpn(baseUrl, lightwalletdUrl) {
  const q =
    lightwalletdUrl && String(lightwalletdUrl).trim()
      ? `?lightwalletd_url=${encodeURIComponent(String(lightwalletdUrl).trim())}`
      : "";
  const r = await companionFetch(baseUrl, `/api/privacy/nym-dvpn${q}`);
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/**
 * @param {string} [baseUrl]
 * @param {boolean} enabled
 */
export async function companionSetNymDvpn(baseUrl, enabled) {
  const r = await companionFetch(baseUrl, "/api/privacy/nym-dvpn", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ enabled: Boolean(enabled) })
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/** @param {string} [baseUrl] */
export async function companionNymVpnApp(baseUrl) {
  const r = await companionFetch(baseUrl, "/api/privacy/nym-vpn-app");
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}

/**
 * @param {string} [baseUrl]
 * @param {{ lightwalletd_url?: string, blocks?: number }} [body]
 */
export async function companionNymDvpnProbe(baseUrl, body) {
  const r = await companionFetch(baseUrl, "/api/privacy/nym-dvpn/probe", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      lightwalletd_url: body?.lightwalletd_url,
      blocks: body?.blocks
    })
  });
  if (!r.ok) throw new Error(await readErrorBody(r));
  return r.json();
}
