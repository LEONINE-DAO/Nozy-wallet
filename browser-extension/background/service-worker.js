import initWasm, * as wasm from "../wasm/pkg/nozy_wasm.js";
import {
  MOBILE_SYNC_SCHEMA_VERSION,
  MOBILE_SYNC_PAIRING_TTL_MS,
  cleanupMobileSyncState,
  consumeSession,
  isSessionConsumed,
  migrateMobileSyncState,
  sanitizeDeviceName
} from "./mobile-sync.js";
import {
  buildBuiltTxStateEntry,
  buildFailedTxStateEntry,
  findRecentBuiltTxId,
  nextLifecycleStateFromConfirmation,
  resolveTxidFromBroadcast
} from "./tx-lifecycle.js";

const STORAGE_KEY = "nozy_wallet_state_v1";
const MOBILE_SYNC_KEY = "nozy_mobile_sync_v1";
const TX_STATE_KEY = "nozy_tx_state_v1";
const SESSION_POLICY_KEY = "nozy_session_policy_v1";
const DEFAULT_AUTO_LOCK_MS = 15 * 60 * 1000;

let wasmReady;
let session = {
  unlocked: false,
  mnemonic: null,
  address: null,
  rpcEndpoint: "http://127.0.0.1:8232",
  autoLockMs: DEFAULT_AUTO_LOCK_MS,
  lastActivityAt: 0
};

const pendingApprovals = new Map();
const providerRequestResolvers = new Map();
let worker;
let workerSeq = 0;
const workerPending = new Map();

function nowMs() {
  return Date.now();
}

function touchSession() {
  session.lastActivityAt = nowMs();
}

function isLikelyUnifiedOrchardAddress(value) {
  return typeof value === "string" && /^u1[0-9a-z]{20,}$/i.test(value);
}

function validateMemo(memo) {
  if (typeof memo !== "string") throw new Error("Memo must be a string.");
  const bytes = utf8Encode(memo);
  if (bytes.length > 512) {
    throw new Error(`Memo too long: ${bytes.length} bytes (max 512).`);
  }
  return memo;
}

function validateRecipientAddress(addr) {
  if (!isLikelyUnifiedOrchardAddress(addr)) {
    throw new Error("Invalid recipient address. Expected a unified shielded address (u1...).");
  }
  return addr;
}

function assessOriginRisk(origin) {
  const value = String(origin || "");
  if (!value) return "high";
  if (value.startsWith("https://")) return "low";
  if (value.startsWith("http://localhost") || value.startsWith("http://127.0.0.1")) return "medium";
  return "high";
}

function validateRequestEnvelope(msg) {
  if (!msg || typeof msg !== "object") throw new Error("Invalid request envelope.");
  if (msg.type !== "NOZY_REQUEST") throw new Error("Unsupported message type.");
  if (typeof msg.method !== "string" || !msg.method) throw new Error("Missing request method.");
  if (msg.params !== undefined && (msg.params === null || typeof msg.params !== "object")) {
    throw new Error("Invalid request params.");
  }
}

async function ensureWasm() {
  if (!wasmReady) {
    wasmReady = initWasm();
  }
  await wasmReady;
  return wasm;
}

function ensureWorker() {
  if (!worker) {
    worker = new Worker(chrome.runtime.getURL("background/wallet-worker.js"), {
      type: "module"
    });
    worker.onmessage = (event) => {
      const { id, result, error } = event.data || {};
      const pending = workerPending.get(id);
      if (!pending) return;
      workerPending.delete(id);
      if (error) pending.reject(new Error(error));
      else pending.resolve(result);
    };
  }
}

function callWorker(method, params) {
  ensureWorker();
  return new Promise((resolve, reject) => {
    const id = `w_${++workerSeq}`;
    workerPending.set(id, { resolve, reject });
    worker.postMessage({ id, method, params });
  });
}

function utf8Encode(str) {
  return new TextEncoder().encode(str);
}

function utf8Decode(bytes) {
  return new TextDecoder().decode(bytes);
}

function ok(result) {
  return { result, error: null };
}

function fail(message) {
  return { result: null, error: { message } };
}

function parseNumberMaybeHex(v) {
  if (v === null || v === undefined) return null;
  if (typeof v === "number" && Number.isFinite(v)) return v;
  if (typeof v === "bigint") return Number(v);
  if (typeof v === "string") {
    const s = v.trim();
    if (s.startsWith("0x")) return Number(BigInt(s));
    if (!s) return null;
    return Number(s);
  }
  return null;
}

function parseTxForOrchardV5(tx) {
  // Minimal adapter: accept common EIP-1193 tx shapes.
  // We only need recipient `to`, `value` (zats) and an optional `memo`.
  const to = tx?.to ?? tx?.recipient ?? tx?.receiver ?? tx?.destination ?? null;
  const value = tx?.value ?? tx?.amount ?? tx?.zatoshis ?? tx?.zats ?? null;
  const memo = tx?.memo ?? tx?.data ?? tx?.comment ?? "";

  const recipientAddress = typeof to === "string" ? to : "";
  const amount = parseNumberMaybeHex(value);
  const memoStr = typeof memo === "string" ? memo : "";

  validateRecipientAddress(recipientAddress);
  if (!Number.isFinite(amount) || amount <= 0) {
    throw new Error("Missing/invalid transaction amount (expected tx.value in zats)");
  }
  validateMemo(memoStr);

  return { recipientAddress, amount, memo: memoStr };
}

function parseFeeToZats(v) {
  if (typeof v !== "number" || !Number.isFinite(v)) return null;
  if (v < 0) return null;
  const zats = Math.round(v * 100_000_000);
  return Number.isFinite(zats) ? zats : null;
}

async function buildTxPreflight(tx) {
  const { recipientAddress, amount, memo } = parseTxForOrchardV5(tx);
  const fee = await estimateFeeZats();
  const proving = await callWorker("prove_transaction", {
    recipientAddress,
    walletAddress: session.address,
    mnemonic: session.mnemonic,
    rpcEndpoint: session.rpcEndpoint,
    amount,
    fee,
    memo
  });
  if (!proving?.rawTxHex) {
    throw new Error("Transaction preflight did not return rawTxHex");
  }
  return {
    recipientAddress,
    amount,
    memo,
    fee,
    txid: String(proving.txid || ""),
    rawTxHex: String(proving.rawTxHex || ""),
    inputs_used: Number(proving.inputs_used ?? 0),
    input_mode: String(
      proving.input_mode ?? (Number(proving.inputs_used ?? 0) <= 1 ? "single" : "multi")
    )
  };
}

async function waitForTxConfirmation({ rpcEndpoint, txid, timeoutMs = 60_000, pollMs = 2_500 }) {
  const startedAt = Date.now();
  while (true) {
    try {
      const resp = await rpcCall("getrawtransaction", [txid, true]);
      const height = resp?.blockheight ?? resp?.blockHeight ?? resp?.block_height ?? null;
      const bh = typeof height === "number" ? height : parseNumberMaybeHex(height);
      if (Number.isFinite(bh) && bh > 0) return { confirmed: true, blockHeight: bh };
    } catch (_) {
    }

    if (Date.now() - startedAt > timeoutMs) break;
    await new Promise((r) => setTimeout(r, pollMs));
  }
  return { confirmed: false, blockHeight: null };
}

function storageGet(key) {
  return new Promise((resolve) => {
    chrome.storage.local.get(key, (items) => resolve(items[key]));
  });
}

function storageSet(data) {
  return new Promise((resolve) => {
    chrome.storage.local.set(data, () => resolve());
  });
}

async function loadSessionPolicy() {
  const state = await storageGet(SESSION_POLICY_KEY);
  return state || { autoLockMs: DEFAULT_AUTO_LOCK_MS };
}

async function saveSessionPolicy(state) {
  await storageSet({ [SESSION_POLICY_KEY]: state });
}

async function loadTxState() {
  const state = await storageGet(TX_STATE_KEY);
  return (
    state || {
      txs: [],
      updatedAt: 0
    }
  );
}

async function saveTxState(state) {
  await storageSet({ [TX_STATE_KEY]: state });
}

async function appendTxState(entry) {
  const state = await loadTxState();
  const txs = Array.isArray(state.txs) ? state.txs : [];
  txs.push(entry);
  await saveTxState({ txs, updatedAt: nowMs() });
}

async function patchTxState(txid, patch) {
  const state = await loadTxState();
  const txs = Array.isArray(state.txs) ? state.txs : [];
  const next = txs.map((tx) => (tx.txid === txid ? { ...tx, ...patch, updatedAt: nowMs() } : tx));
  await saveTxState({ txs: next, updatedAt: nowMs() });
}

async function patchTxStateById(id, patch) {
  const state = await loadTxState();
  const txs = Array.isArray(state.txs) ? state.txs : [];
  const next = txs.map((tx) => (tx.id === id ? { ...tx, ...patch, updatedAt: nowMs() } : tx));
  await saveTxState({ txs: next, updatedAt: nowMs() });
}

async function retryBroadcastById(id) {
  const state = await loadTxState();
  const txs = Array.isArray(state.txs) ? state.txs : [];
  const tx = txs.find((t) => t.id === id);
  if (!tx) throw new Error("Transaction record not found.");
  if (!tx.rawTxHex) throw new Error("No raw transaction available for retry.");
  const broadcastResult = await rpcCallWithRetry("sendrawtransaction", [tx.rawTxHex], {
    retries: 2,
    baseDelayMs: 500
  });
  const txid = resolveTxidFromBroadcast(broadcastResult, tx.txid);
  await patchTxStateById(id, {
    txid: String(txid),
    state: "broadcast",
    error: null
  });
  return String(txid);
}

async function loadWalletState() {
  const state = await storageGet(STORAGE_KEY);
  return state || null;
}

async function saveWalletState(state) {
  await storageSet({ [STORAGE_KEY]: state });
}

async function loadMobileSyncState() {
  const state = await storageGet(MOBILE_SYNC_KEY);
  return migrateMobileSyncState(state, nowMs());
}

async function saveMobileSyncState(state) {
  await storageSet({ [MOBILE_SYNC_KEY]: state });
}

async function cleanupStaleMobileSyncState() {
  const loaded = await loadMobileSyncState();
  const { state, changed } = cleanupMobileSyncState(loaded, nowMs());
  if (changed) {
    await saveMobileSyncState(state);
  }
  return state;
}

function randomHex(bytes = 16) {
  const arr = crypto.getRandomValues(new Uint8Array(bytes));
  return Array.from(arr, (b) => b.toString(16).padStart(2, "0")).join("");
}

async function mobileSyncInitPairing(params = {}) {
  if (!session.unlocked || !session.address) throw new Error("Unlock wallet first.");

  const now = nowMs();
  const ttlMs = Number(params.ttlMs ?? MOBILE_SYNC_PAIRING_TTL_MS);
  const boundedTtlMs = Math.max(60_000, Math.min(ttlMs, 30 * 60 * 1000));
  const state = await cleanupStaleMobileSyncState();
  const sessionId = `ms_${randomHex(12)}`;
  const verifyCode = randomHex(3).toUpperCase();
  const challenge = randomHex(24);

  const pairing = {
    sessionId,
    walletAddress: session.address,
    verifyCode,
    challenge,
    createdAt: now,
    expiresAt: now + boundedTtlMs
  };

  const payload = JSON.stringify({
    v: MOBILE_SYNC_SCHEMA_VERSION,
    sigAlg: "nozy-sign-message-v1",
    replayProtection: "session-consume-v1",
    sessionId,
    walletAddress: session.address,
    challenge,
    verifyCode,
    expiresAt: pairing.expiresAt
  });

  const next = {
    ...state,
    activePairing: pairing,
    pairingPayload: payload,
    updatedAt: now
  };
  await saveMobileSyncState(next);

  return {
    sessionId,
    verifyCode,
    expiresAt: pairing.expiresAt,
    payload
  };
}

async function mobileSyncConfirmPairing(params = {}) {
  if (!session.unlocked || !session.address) throw new Error("Unlock wallet first.");
  const sessionId = String(params.sessionId ?? "");
  const deviceName = sanitizeDeviceName(params.deviceName);
  const platform = String(params.platform ?? "unknown");
  const challengeSignature = String(params.challengeSignature ?? "");
  const now = nowMs();

  const state = await cleanupStaleMobileSyncState();
  const active = state.activePairing;
  if (!active) throw new Error("No active pairing session.");
  if (isSessionConsumed(state, sessionId, now)) {
    throw new Error("Replay detected: pairing session already consumed.");
  }
  if (active.sessionId !== sessionId) throw new Error("Pairing session mismatch.");
  if (active.expiresAt < now) throw new Error("Pairing session expired.");
  if (!challengeSignature) throw new Error("Missing challenge signature.");

  // Seed-on-device handshake: mobile must prove it can sign the challenge.
  const expectedSignature = wasm.sign_message(session.mnemonic, active.challenge);
  if (challengeSignature !== expectedSignature) {
    throw new Error("Invalid pairing signature for challenge.");
  }

  const existing = (state.pairedDevices || []).find((d) => d.sessionId === sessionId && d.status !== "revoked");
  const pairedDevice = {
    id: existing?.id || `dev_${randomHex(10)}`,
    name: deviceName,
    platform,
    sessionId,
    signaturePrefix: challengeSignature.slice(0, 12),
    pairedAt: existing?.pairedAt || now,
    renamedAt: existing?.renamedAt || null,
    revokedAt: null,
    lastSeenAt: now,
    trustLevel: "signed-challenge-v1",
    status: "paired"
  };

  const remainingDevices = (state.pairedDevices || []).filter((d) => d.id !== pairedDevice.id);
  const withConsumedSession = consumeSession(state, sessionId, now);
  const next = {
    ...withConsumedSession,
    pairedDevices: [...remainingDevices, pairedDevice],
    activePairing: null,
    pairingPayload: null,
    updatedAt: now
  };
  await saveMobileSyncState(next);
  return pairedDevice;
}

async function mobileSyncUnpair(params = {}) {
  if (!session.unlocked) throw new Error("Unlock wallet first.");
  const deviceId = String(params.deviceId ?? "");
  if (!deviceId) throw new Error("Missing deviceId.");
  const state = await loadMobileSyncState();
  const next = {
    ...state,
    pairedDevices: (state.pairedDevices || []).filter((d) => d.id !== deviceId),
    updatedAt: Date.now()
  };
  await saveMobileSyncState(next);
  return { removed: true, deviceId };
}

async function mobileSyncRenameDevice(params = {}) {
  if (!session.unlocked) throw new Error("Unlock wallet first.");
  const deviceId = String(params.deviceId ?? "");
  const name = sanitizeDeviceName(params.name);
  if (!deviceId) throw new Error("Missing deviceId.");
  const now = nowMs();
  const state = await loadMobileSyncState();
  const devices = Array.isArray(state.pairedDevices) ? state.pairedDevices : [];
  const index = devices.findIndex((d) => d.id === deviceId);
  if (index < 0) throw new Error("Device not found.");
  const nextDevices = [...devices];
  nextDevices[index] = {
    ...nextDevices[index],
    name,
    renamedAt: now,
    lastSeenAt: now
  };
  const next = { ...state, pairedDevices: nextDevices, updatedAt: now };
  await saveMobileSyncState(next);
  return nextDevices[index];
}

async function mobileSyncRevokeDevice(params = {}) {
  if (!session.unlocked) throw new Error("Unlock wallet first.");
  const deviceId = String(params.deviceId ?? "");
  if (!deviceId) throw new Error("Missing deviceId.");
  const now = nowMs();
  const state = await loadMobileSyncState();
  const devices = Array.isArray(state.pairedDevices) ? state.pairedDevices : [];
  const index = devices.findIndex((d) => d.id === deviceId);
  if (index < 0) throw new Error("Device not found.");
  const nextDevices = [...devices];
  nextDevices[index] = {
    ...nextDevices[index],
    status: "revoked",
    revokedAt: now,
    lastSeenAt: now
  };
  const next = { ...state, pairedDevices: nextDevices, updatedAt: now };
  await saveMobileSyncState(next);
  return nextDevices[index];
}

async function mobileSyncGetState() {
  const state = await cleanupStaleMobileSyncState();
  const active = state.activePairing;
  return {
    schemaVersion: state.schemaVersion || MOBILE_SYNC_SCHEMA_VERSION,
    pairedDevices: state.pairedDevices || [],
    activePairing: active,
    pairingPayload: active ? state.pairingPayload || null : null
  };
}

function mobileSyncGetPairingSchema() {
  return {
    type: "nozy.mobile_sync.pairing.v2",
    required: ["v", "sessionId", "walletAddress", "challenge", "verifyCode", "expiresAt"],
    fields: {
      v: "number",
      sessionId: "string",
      walletAddress: "string",
      challenge: "string",
      verifyCode: "string",
      expiresAt: "number",
      replayProtection: "string"
    },
    notes: "Seed and private keys are never included in pairing payload. Session IDs are one-time-use."
  };
}

async function walletCreate(password) {
  await ensureWasm();
  const created = wasm.create_wallet(password);
  const mnemonic = created.mnemonic;
  const address = created.address;
  const encryptedMnemonic = Array.from(
    wasm.encrypt_for_storage(utf8Encode(mnemonic), password)
  );

  await saveWalletState({
    encryptedMnemonic,
    address,
    createdAt: Date.now(),
    rpcEndpoint: session.rpcEndpoint
  });

  session.unlocked = true;
  session.mnemonic = mnemonic;
  session.address = address;
  touchSession();

  return { address };
}

async function walletRestore(mnemonic, password) {
  await ensureWasm();
  const restored = wasm.restore_wallet(mnemonic, password);
  const address = restored.address;
  const encryptedMnemonic = Array.from(
    wasm.encrypt_for_storage(utf8Encode(mnemonic), password)
  );

  await saveWalletState({
    encryptedMnemonic,
    address,
    createdAt: Date.now(),
    rpcEndpoint: session.rpcEndpoint
  });

  session.unlocked = true;
  session.mnemonic = mnemonic;
  session.address = address;
  touchSession();

  return { address };
}

async function walletUnlock(password) {
  await ensureWasm();
  const state = await loadWalletState();
  if (!state?.encryptedMnemonic) {
    throw new Error("No wallet found. Create or restore first.");
  }

  const decrypted = wasm.decrypt_from_storage(
    new Uint8Array(state.encryptedMnemonic),
    password
  );
  const mnemonic = utf8Decode(decrypted);
  const address = wasm.generate_address(mnemonic, 0, 0);

  session.unlocked = true;
  session.mnemonic = mnemonic;
  session.address = address;
  session.rpcEndpoint = state.rpcEndpoint || session.rpcEndpoint;
  touchSession();

  return { address };
}

function walletLock() {
  session.unlocked = false;
  session.mnemonic = null;
  session.address = null;
  return true;
}

async function getWalletStatus() {
  const state = await loadWalletState();
  return {
    exists: !!state,
    unlocked: session.unlocked,
    address: session.address || state?.address || null,
    rpcEndpoint: session.rpcEndpoint
  };
}

async function getAccounts() {
  if (!session.unlocked || !session.address) return [];
  touchSession();
  return [session.address];
}

async function requestApproval(kind, payload) {
  const id = crypto.randomUUID();
  const approval = { id, kind, payload, createdAt: Date.now() };
  pendingApprovals.set(id, approval);
  return approval;
}

async function ensureSessionInitialized() {
  const wallet = (await loadWalletState()) || {};
  session.rpcEndpoint = wallet.rpcEndpoint || session.rpcEndpoint;
  const policy = await loadSessionPolicy();
  session.autoLockMs = Number(policy.autoLockMs) || DEFAULT_AUTO_LOCK_MS;
  if (!session.lastActivityAt) touchSession();
}

async function rpcCall(method, params = []) {
  const endpoint = session.rpcEndpoint;
  const resp = await fetch(endpoint, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method,
      params
    })
  });
  if (!resp.ok) throw new Error(`RPC HTTP ${resp.status}`);
  const body = await resp.json();
  if (body.error) throw new Error(body.error.message || "RPC error");
  return body.result;
}

async function rpcCallWithRetry(method, params = [], opts = {}) {
  const retries = Number.isFinite(opts.retries) ? opts.retries : 3;
  const baseDelayMs = Number.isFinite(opts.baseDelayMs) ? opts.baseDelayMs : 250;
  let lastErr;
  for (let attempt = 0; attempt <= retries; attempt += 1) {
    try {
      return await rpcCall(method, params);
    } catch (err) {
      lastErr = err;
      if (attempt < retries) {
        await new Promise((r) => setTimeout(r, baseDelayMs * 2 ** attempt));
      }
    }
  }
  throw lastErr || new Error(`RPC ${method} failed`);
}

async function estimateFeeZats() {
  try {
    const feeResult = await rpcCallWithRetry("estimatefee", [1], { retries: 1, baseDelayMs: 200 });
    const fromRoot = parseFeeToZats(feeResult);
    const fromObj = parseFeeToZats(feeResult?.feerate);
    const candidate = fromRoot ?? fromObj;
    if (candidate && candidate > 0) {
      return Math.max(1_000, Math.min(candidate, 250_000));
    }
  } catch (_) {
    // ignore and fallback
  }
  return 10_000;
}

setInterval(() => {
  if (!session.unlocked) return;
  if (!session.lastActivityAt) return;
  if (nowMs() - session.lastActivityAt >= session.autoLockMs) {
    walletLock();
  }
}, 20_000);

setInterval(() => {
  cleanupStaleMobileSyncState().catch(() => undefined);
}, 30_000);

chrome.runtime.onMessage.addListener((msg, _sender, sendResponse) => {
  if (!msg || msg.type !== "NOZY_REQUEST") return;

  (async () => {
    try {
      validateRequestEnvelope(msg);
      await ensureSessionInitialized();
      await ensureWasm();
      const method = msg.method;
      const params = msg.params ?? {};
      touchSession();

      // Popup/UI control methods.
      switch (method) {
        case "wallet_create":
          sendResponse(ok(await walletCreate(params.password)));
          return;
        case "wallet_restore":
          sendResponse(ok(await walletRestore(params.mnemonic, params.password)));
          return;
        case "wallet_unlock":
          sendResponse(ok(await walletUnlock(params.password)));
          return;
        case "wallet_lock":
          sendResponse(ok(walletLock()));
          return;
        case "wallet_status":
          sendResponse(ok(await getWalletStatus()));
          return;
        case "wallet_set_session_policy": {
          const autoLockMs = Number(params.autoLockMs ?? DEFAULT_AUTO_LOCK_MS);
          const bounded = Math.max(60_000, Math.min(autoLockMs, 24 * 60 * 60 * 1000));
          session.autoLockMs = bounded;
          await saveSessionPolicy({ autoLockMs: bounded });
          sendResponse(ok({ autoLockMs: bounded }));
          return;
        }
        case "wallet_get_transactions":
          sendResponse(ok(await loadTxState()));
          return;
        case "wallet_retry_broadcast":
          sendResponse(ok({ txid: await retryBroadcastById(String(params.id ?? "")) }));
          return;
        case "wallet_generate_address":
          if (!session.unlocked || !session.mnemonic) throw new Error("Wallet is locked");
          sendResponse(ok(wasm.generate_address(session.mnemonic, params.account ?? 0, params.index ?? 0)));
          return;
        case "wallet_sign_message":
          if (!session.unlocked || !session.mnemonic) throw new Error("Wallet is locked");
          sendResponse(ok(wasm.sign_message(session.mnemonic, params.message || "")));
          return;
        case "wallet_get_pending_approvals":
          sendResponse(ok(Array.from(pendingApprovals.values())));
          return;
        case "wallet_approve_request": {
          const approval = pendingApprovals.get(params.id);
          if (!approval) throw new Error("Approval request not found");
          pendingApprovals.delete(params.id);
          sendResponse(ok({ approved: true, id: params.id }));

          const resolver = providerRequestResolvers.get(params.id);
          if (resolver) {
            providerRequestResolvers.delete(params.id);
            (async () => {
              try {
                if (approval.kind === "sign") {
                  const message = String(approval.payload?.message ?? "");
                  if (!message) throw new Error("Missing message for signing");
                  const signature = wasm.sign_message(session.mnemonic, message);
                  resolver.sendResponse(ok(signature));
                  return;
                }

                if (approval.kind === "transaction") {
                  const tx = approval.payload?.tx ?? {};
                  const createdAt = nowMs();
                  let proving = approval.payload?.preflight ?? null;
                  if (!proving?.rawTxHex) {
                    proving = await buildTxPreflight(tx);
                  }
                  if (!proving?.rawTxHex) {
                    throw new Error("Transaction proving did not return rawTxHex");
                  }

                  const txStateId = crypto.randomUUID();
                  await appendTxState(
                    buildBuiltTxStateEntry({
                      id: txStateId,
                      origin: String(approval.payload?.origin ?? ""),
                      proving,
                      createdAt
                    })
                  );

                  const broadcastResult = await rpcCallWithRetry("sendrawtransaction", [
                    proving.rawTxHex
                  ], { retries: 3, baseDelayMs: 400 });

                  const txid = resolveTxidFromBroadcast(broadcastResult, proving.txid);
                  await patchTxStateById(txStateId, {
                    txid: String(txid),
                    state: "broadcast",
                    error: null
                  });

                  const confirmation = await waitForTxConfirmation({
                    rpcEndpoint: session.rpcEndpoint,
                    txid: String(txid),
                    timeoutMs: 120_000,
                    pollMs: 2_000
                  });
                  await patchTxStateById(txStateId, {
                    txid: String(txid),
                    state: nextLifecycleStateFromConfirmation(confirmation),
                    blockHeight: confirmation.blockHeight ?? null
                  });

                  resolver.sendResponse(ok(String(txid)));
                  return;
                }

                resolver.sendResponse(fail(`Unsupported approval kind: ${approval.kind}`));
              } catch (e) {
                const errMsg = e?.message ?? "Failed to fulfill approved request";
                if (approval?.kind === "transaction") {
                  const now = nowMs();
                  const existingBuiltId = await (async () => {
                    const state = await loadTxState();
                    const txs = Array.isArray(state.txs) ? state.txs : [];
                    return findRecentBuiltTxId(txs, String(approval.payload?.origin ?? ""), now);
                  })();
                  if (existingBuiltId) {
                    await patchTxStateById(existingBuiltId, {
                      state: "failed",
                      error: errMsg
                    });
                  } else {
                    await appendTxState(
                      buildFailedTxStateEntry({
                        id: crypto.randomUUID(),
                        origin: String(approval.payload?.origin ?? ""),
                        tx: approval.payload?.tx ?? {},
                        preflight: approval.payload?.preflight ?? {},
                        error: errMsg,
                        createdAt: now,
                        parseAmount: parseNumberMaybeHex
                      })
                    );
                  }
                }
                resolver.sendResponse(fail(errMsg));
              }
            })();
          }
          return;
        }
        case "wallet_reject_request":
          pendingApprovals.delete(params.id);
          if (providerRequestResolvers.has(params.id)) {
            const resolver = providerRequestResolvers.get(params.id);
            providerRequestResolvers.delete(params.id);
            resolver.sendResponse(fail("Request rejected by user"));
          }
          sendResponse(ok({ approved: false, id: params.id }));
          return;
        case "rpc_set_endpoint":
          session.rpcEndpoint = params.url || session.rpcEndpoint;
          {
            const existing = (await loadWalletState()) || {};
            await saveWalletState({ ...existing, rpcEndpoint: session.rpcEndpoint });
          }
          sendResponse(ok({ rpcEndpoint: session.rpcEndpoint }));
          return;
        case "rpc_get_status":
          sendResponse(
            ok({
              endpoint: session.rpcEndpoint,
              connected: !!(await rpcCallWithRetry("getblockcount", [], { retries: 1 }))
            })
          );
          return;
        case "rpc_get_block_count":
          sendResponse(ok(await rpcCallWithRetry("getblockcount", [])));
          return;
        case "rpc_get_block":
          sendResponse(ok(await rpcCallWithRetry("getblock", [params.height])));
          return;
        case "wallet_scan_notes":
          if (!session.unlocked || !session.mnemonic || !session.address) {
            throw new Error("Unlock wallet first.");
          }
          sendResponse(
            ok(
              await callWorker("scan_notes", {
                startHeight: params.startHeight ?? 0,
                endHeight: params.endHeight ?? params.startHeight ?? 0,
                rpcEndpoint: session.rpcEndpoint,
                mnemonic: session.mnemonic,
                address: session.address
              })
            )
          );
          return;
        case "wallet_prove_transaction":
          if (!session.unlocked || !session.address) {
            throw new Error("Unlock wallet first.");
          }
          sendResponse(
            ok(
              await callWorker("prove_transaction", {
                ...params,
                recipientAddress: params?.recipientAddress ?? params?.to ?? session.address,
                walletAddress: session.address,
                mnemonic: session.mnemonic,
                rpcEndpoint: session.rpcEndpoint
              })
            )
          );
          return;
        case "mobile_sync_get_state":
          sendResponse(ok(await mobileSyncGetState()));
          return;
        case "mobile_sync_get_pairing_schema":
          sendResponse(ok(mobileSyncGetPairingSchema()));
          return;
        case "mobile_sync_init_pairing":
          sendResponse(ok(await mobileSyncInitPairing(params)));
          return;
        case "mobile_sync_confirm_pairing":
          sendResponse(ok(await mobileSyncConfirmPairing(params)));
          return;
        case "mobile_sync_unpair":
          sendResponse(ok(await mobileSyncUnpair(params)));
          return;
        case "mobile_sync_rename_device":
          sendResponse(ok(await mobileSyncRenameDevice(params)));
          return;
        case "mobile_sync_revoke_device":
          sendResponse(ok(await mobileSyncRevokeDevice(params)));
          return;
      }

      // dApp provider methods.
      switch (method) {
        case "eth_chainId":
        case "zcash_chainId":
          sendResponse(ok(wasm.get_zcash_chain_id()));
          return;
        case "eth_getBalance":
          sendResponse(ok("0x0"));
          return;
        case "wallet_watchAsset":
          sendResponse(ok(false));
          return;
        case "eth_accounts":
        case "zcash_accounts":
          sendResponse(ok(await getAccounts()));
          return;
        case "eth_requestAccounts":
        case "zcash_requestAccounts": {
          const accounts = await getAccounts();
          if (accounts.length === 0) throw new Error("Unlock wallet in popup first.");
          sendResponse(ok(accounts));
          return;
        }
        case "personal_sign":
        case "zcash_signMessage": {
          if (!session.unlocked) throw new Error("Unlock wallet first.");
          const approval = await requestApproval("sign", {
            method,
            origin: msg.origin || "",
            message: params?.message || params?.[0] || ""
          });
          providerRequestResolvers.set(approval.id, { sendResponse });
          return;
        }
        case "eth_sendTransaction":
        case "zcash_sendTransaction": {
          if (!session.unlocked) throw new Error("Unlock wallet first.");
          const origin = String(msg.origin || "");
          const txPayload = params?.tx || params?.[0] || params;
          let preflight = null;
          let preflightError = null;
          try {
            preflight = await buildTxPreflight(txPayload);
          } catch (e) {
            preflightError = e?.message ?? "Transaction preflight failed";
          }
          const approval = await requestApproval("transaction", {
            method,
            origin,
            risk: assessOriginRisk(origin),
            tx: txPayload,
            preflight,
            preflightError
          });
          providerRequestResolvers.set(approval.id, { sendResponse });
          return;
        }
      }

      sendResponse(fail(`Unsupported method: ${method}`));
    } catch (e) {
      sendResponse(fail(e?.message ?? String(e)));
    }
  })();

  return true;
});

