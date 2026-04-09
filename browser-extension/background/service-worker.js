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
import { normalizeRpcEndpoint, rpcNetworkErrorMessage } from "./rpc-utils.js";
import { selectNotesForSpend, rpcFallbackWithRequester } from "./tx-utils.js";
import {
  companionLwdChainTip,
  companionLwdInfo,
  companionLwdSyncCompact,
  companionStatus
} from "./companion-api.js";

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

let useInlineWorker = false;

function ensureWorker() {
  if (worker) return;
  if (typeof Worker !== "undefined") {
    try {
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
      worker.onerror = () => {};
      return;
    } catch (_) { /* fall through to inline */ }
  }
  useInlineWorker = true;
}

function _toByteArray(value) {
  if (Array.isArray(value)) return value.map((v) => Number(v) & 0xff);
  if (typeof value === "string") {
    const clean = value.startsWith("0x") ? value.slice(2) : value;
    if (clean.length % 2 !== 0) return [];
    const bytes = [];
    for (let i = 0; i < clean.length; i += 2) bytes.push(parseInt(clean.slice(i, i + 2), 16));
    return bytes;
  }
  return [];
}

function _extractActionsFromBlock(block) {
  const actions = [];
  const txs = block?.tx ?? block?.transactions ?? [];
  for (const tx of txs) {
    if (typeof tx === "string") continue;
    const orchard = tx?.orchard || tx?.orchard_bundle || {};
    const candidates = orchard?.actions || orchard?.action || tx?.orchard_actions || [];
    if (Array.isArray(candidates)) {
      for (const c of candidates) {
        const nf = _toByteArray(c?.nullifier ?? c?.nf ?? []);
        const cmx = _toByteArray(c?.cmx ?? c?.note_commitment ?? []);
        const epk = _toByteArray(c?.ephemeralKey ?? c?.ephemeral_key ?? c?.epk ?? []);
        const enc = _toByteArray(c?.encCiphertext ?? c?.encrypted_note ?? c?.enc_ciphertext ?? []);
        if (nf.length === 32 && cmx.length === 32 && epk.length === 32)
          actions.push({ nullifier: nf, cmx, ephemeral_key: epk, encrypted_note: enc });
      }
    }
  }
  return actions;
}

async function _inlineScanNotes(params) {
  await ensureWasm();
  const startHeight = Number(params?.startHeight ?? 0);
  const endHeight = Number(params?.endHeight ?? startHeight);
  const rpcEndpoint = normalizeRpcEndpoint(String(params?.rpcEndpoint ?? ""));
  const mnemonic = String(params?.mnemonic ?? "");
  const address = String(params?.address ?? "");
  let scannedBlocks = 0, totalBalanceZats = 0;
  const discoveredNotes = [];

  let trackerState;
  if (startHeight > 0) {
    const ts = await _inlineRpcRequest(rpcEndpoint, "z_gettreestate", [String(startHeight - 1)]);
    const finalState = ts?.orchard?.commitments?.finalState ?? ts?.orchard?.commitments?.final_state ?? "";
    trackerState = wasm.orchard_scan_tracker_new(typeof finalState === "string" ? finalState : "");
  } else {
    trackerState = wasm.orchard_scan_tracker_new("");
  }

  for (let h = startHeight; h <= endHeight; h += 1) {
    scannedBlocks += 1;
    try {
      const block = await _inlineRpcRequest(rpcEndpoint, "getblock", [String(h), 2]);
      if (!block) continue;
      const blockJson = JSON.stringify(block);
      const out = wasm.orchard_scan_tracker_apply_block(trackerState, mnemonic, address, h, blockJson);
      trackerState = out.tracker_state;
      if (out.notes?.length) {
        for (const n of out.notes) {
          discoveredNotes.push(n);
          totalBalanceZats += Number(n?.value ?? 0);
        }
      }
    } catch (_) { /* continue scanning */ }
  }
  return { scannedBlocks, discoveredNotes, totalBalanceZats };
}

function _bytesToHex(bytes) {
  const arr = Array.isArray(bytes) || bytes instanceof Uint8Array ? bytes : [];
  return Array.from(arr, (b) => (Number(b) & 0xff).toString(16).padStart(2, "0")).join("");
}

/** Zebra `z_gettreestate` uses orchard.commitments.finalRoot; zcashd-style may use anchor. */
function _orchardAnchorHexFromRpc(tr) {
  if (!tr || typeof tr !== "object") return "";
  const o = tr.orchard;
  const c = o?.commitments ?? o;
  const fromZebra =
    c?.finalRoot ?? c?.final_root ?? o?.finalRoot ?? o?.final_root ?? "";
  let hex = String(tr.anchor ?? tr.orchardTree?.anchor ?? fromZebra ?? "").trim();
  if (hex.startsWith("0x") || hex.startsWith("0X")) hex = hex.slice(2);
  return hex;
}

/** Zebrad has no z_findnoteposition / z_getauthpath; replace cryptic JSON-RPC errors. */
function rewriteMissingWitnessRpcError(err) {
  const m = String(err?.message || err || "");
  const code = typeof err?.jsonRpcCode === "number" ? err.jsonRpcCode : null;
  // jsonrpsee (Zebra) often uses "The method does not exist / is not available." (-32601), not the string "method not found".
  const looksLikeMissingMethod =
    code === -32601 ||
    /method not found/i.test(m) ||
    /\bmethod\b.*\bnot found\b/i.test(m) ||
    /does not exist|is not available/i.test(m);
  if (!looksLikeMissingMethod) return err instanceof Error ? err : new Error(String(err));
  return new Error(
    "Zebrad (Zebra) does not implement z_findnoteposition or z_getauthpath, which Nozy needs to build Orchard spends. " +
      "Scanning and receiving work with Zebrad; for shielded sends, use a zcashd JSON-RPC (or another node that exposes those methods)."
  );
}

async function _inlineRpcRequest(endpoint, method, params = []) {
  const resp = await fetch(endpoint, {
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

async function _inlineProveTransaction(params) {
  await ensureWasm();
  const recipientAddress = String(params?.recipientAddress ?? params?.to ?? "");
  const walletAddress = String(params?.walletAddress ?? "");
  const mnemonic = String(params?.mnemonic ?? "");
  const rpcEndpoint = String(params?.rpcEndpoint ?? "");
  const requestedAmount = Number(params?.amount ?? 0);
  const requestedFee = Number(params?.fee ?? 10000);
  const memo = String(params?.memo ?? "");

  if (!rpcEndpoint) throw new Error("Missing rpcEndpoint.");
  if (!mnemonic) throw new Error("Wallet locked.");
  if (!walletAddress) throw new Error("Missing wallet address.");
  if (!recipientAddress) throw new Error("Missing recipient address.");

  const endpoint = normalizeRpcEndpoint(rpcEndpoint);
  const requiredValue = requestedAmount + requestedFee;
  if (!Number.isFinite(requiredValue) || requiredValue <= 0) {
    throw new Error(`Invalid amount/fee (amount=${requestedAmount}, fee=${requestedFee}).`);
  }

  const scanState = await loadScanState();
  let candidates = [];
  if (scanState?.discoveredNotes?.length) {
    candidates = scanState.discoveredNotes.filter(
      (n) => Number.isFinite(n.value) && n.value > 0
    );
  }

  if (candidates.length === 0) {
    const status = scanState?.status ?? "idle";
    if (status === "scanning") {
      throw new Error("Scan in progress — no spendable notes found yet. Wait for the scan to find notes, then try again.");
    }
    throw new Error("No spendable notes found. Run a block scan from the Receive tab first.");
  }

  const scannedValue = candidates.reduce((acc, n) => acc + n.value, 0);
  const selected = selectNotesForSpend(candidates, requiredValue);
  if (selected.length === 0) {
    throw new Error(`Insufficient funds (need ${requiredValue}, have ${scannedValue}).`);
  }

  const spendValue = selected.reduce((acc, n) => acc + n.value, 0);
  const targetHeight = Number(await _inlineRpcRequest(endpoint, "getblockcount", []));
  const heightStr = String(targetHeight);

  const rpcReq = (at) => _inlineRpcRequest(endpoint, at.method, at.params ?? []);
  const orchardTree = await rpcFallbackWithRequester(rpcReq, [
    { method: "z_getorchardtree", params: [heightStr] },
    { method: "z_gettreestate", params: [heightStr] }
  ]);
  const anchorHex = _orchardAnchorHexFromRpc(orchardTree);
  if (!anchorHex || anchorHex.length < 64) throw new Error("RPC did not return a valid Orchard anchor.");

  const selectedWitnesses = [];
  for (const noteSel of selected) {
    let witnessHex = noteSel?.note?.orchard_incremental_witness_hex;
    const tip = Number(noteSel?.note?.orchard_witness_tip_height ?? noteSel?.height ?? 0);
    if (!witnessHex || typeof witnessHex !== "string") {
      throw new Error(
        "Note missing orchard_incremental_witness_hex. Rescan with the updated extension that records Orchard witnesses (Zebrad-compatible)."
      );
    }
    if (!Number.isFinite(tip) || tip < 0) {
      throw new Error("Invalid orchard_witness_tip_height on note.");
    }
    for (let h = tip + 1; h <= targetHeight; h += 1) {
      const block = await _inlineRpcRequest(endpoint, "getblock", [String(h), 2]);
      witnessHex = wasm.advance_orchard_witness_hex(witnessHex, JSON.stringify(block));
    }
    if (!wasm.orchard_witness_matches_anchor_hex(witnessHex, anchorHex)) {
      throw new Error("Orchard witness does not match anchor at tip (rescan or wait for sync).");
    }
    selectedWitnesses.push({
      incremental_witness_hex: witnessHex,
      anchor_hex: anchorHex,
      target_height: targetHeight
    });
  }

  const provingResult = wasm.build_orchard_v5_tx_from_note(
    mnemonic,
    recipientAddress,
    requestedAmount,
    requestedFee,
    memo,
    JSON.stringify(selected.map((s) => s.note)),
    JSON.stringify(selectedWitnesses)
  );

  return {
    txid: provingResult?.txid ?? "",
    chainId: wasm.get_zcash_chain_id(),
    rawTxHex: provingResult?.rawTxHex ?? "",
    proving: "inline_orchard_v5_tx_build_wasm",
    bundle_actions: provingResult?.bundle_actions ?? 0,
    proof_generated: provingResult?.proof_generated ?? true,
    selected_notes_count: selected.length,
    selected_notes_total_value: spendValue,
    selected_notes: selected.map((s) => ({
      value: Number(s?.note?.value ?? 0),
      cmx: _bytesToHex(s?.note?.cmx ?? []).slice(0, 16),
      block_height: s.height
    })),
    selected_witnesses_count: selectedWitnesses.length,
    inputs_used: selected.length,
    input_mode: selected.length <= 1 ? "single" : "multi",
    fee: requestedFee
  };
}

function callWorker(method, params) {
  ensureWorker();
  if (useInlineWorker) {
    if (method === "scan_notes") return _inlineScanNotes(params);
    if (method === "prove_transaction") return _inlineProveTransaction(params);
    return Promise.reject(new Error(`Inline fallback does not support method: ${method}`));
  }
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

  void resumeBackgroundScanAfterUnlock();

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
  let endpoint;
  try {
    endpoint = normalizeRpcEndpoint(session.rpcEndpoint);
  } catch (e) {
    throw e instanceof Error ? e : new Error(String(e));
  }
  let resp;
  try {
    resp = await fetch(endpoint, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method,
        params
      })
    });
  } catch (err) {
    throw new Error(rpcNetworkErrorMessage(endpoint, err));
  }
  if (resp.status === 401 || resp.status === 403) {
    throw new Error(
      `RPC returned HTTP ${resp.status} (authentication required). ` +
        `For Zebra, set enable_cookie_auth=false in zebrad.toml for local JSON-RPC, or run a small proxy that adds the expected credentials.`
    );
  }
  if (!resp.ok) throw new Error(`RPC HTTP ${resp.status}`);
  const body = await resp.json();
  if (body.error) {
    throw new Error(body.error.message || "RPC error");
  }
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

// ── Background scan ────────────────────────────────────────────────
const SCAN_STATE_KEY = "nozy_scan_state_v1";
const SCAN_BATCH = 50;
const SCAN_ALARM = "nozy_scan_tick";

function loadScanState() {
  return new Promise((r) =>
    chrome.storage.local.get(SCAN_STATE_KEY, (v) => r(v[SCAN_STATE_KEY] || null))
  );
}
function saveScanState(s) {
  return new Promise((r) => chrome.storage.local.set({ [SCAN_STATE_KEY]: s }, r));
}

let scanRunning = false;

function scheduleScanAlarm(delayMinutes) {
  chrome.alarms.create(SCAN_ALARM, { delayInMinutes: delayMinutes });
}

async function scanTick() {
  if (scanRunning) {
    scheduleScanAlarm(0.05);
    return;
  }
  const state = await loadScanState();
  if (!state || state.status !== "scanning") return;
  if (!session.unlocked || !session.mnemonic || !session.address) {
    // SW restarted or wallet locked: keep alarms alive until the user unlocks again.
    scheduleScanAlarm(1);
    return;
  }
  scanRunning = true;

  try {
    await ensureWasm();
    const endpoint = normalizeRpcEndpoint(session.rpcEndpoint);
    const end = Math.min(state.currentHeight + SCAN_BATCH - 1, state.endHeight);

    for (let h = state.currentHeight; h <= end; h++) {
      try {
        const resp = await fetch(endpoint, {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "getblock", params: [String(h), 2] })
        });
        if (!resp.ok) continue;
        const body = await resp.json();
        if (body?.error || !body?.result) continue;
        const block = body.result;
        const actions = _extractActionsFromBlock(block);
        if (actions.length === 0) continue;
        const txid = block?.hash || `h${h}`;
        const scan = wasm.scan_orchard_actions(session.mnemonic, session.address, JSON.stringify(actions), h, txid);
        if (scan?.notes?.length) {
          for (const n of scan.notes) {
            const v = Number(n?.value ?? 0);
            state.discoveredNotes.push({ note: n, height: h, txid, value: v });
          }
          state.totalBalanceZats += Number(scan.total_value_zats || 0);
        }
      } catch (_) { /* skip block */ }
    }

    state.scannedBlocks += (end - state.currentHeight + 1);
    state.currentHeight = end + 1;
    state.updatedAt = nowMs();

    if (state.currentHeight > state.endHeight) {
      state.status = "done";
      state.finishedAt = nowMs();
    }
    await saveScanState(state);
  } finally {
    scanRunning = false;
  }

  if (state.status === "scanning") {
    scheduleScanAlarm(0.02);
  }
}

async function startBackgroundScan(startHeight, endHeight) {
  const existing = await loadScanState();
  if (existing && existing.status === "scanning") {
    scheduleScanAlarm(0.02);
    void scanTick();
    return existing;
  }
  const state = {
    status: "scanning",
    startHeight,
    endHeight,
    currentHeight: startHeight,
    scannedBlocks: 0,
    discoveredNotes: [],
    totalBalanceZats: 0,
    startedAt: nowMs(),
    updatedAt: nowMs(),
    finishedAt: null
  };
  await saveScanState(state);
  scheduleScanAlarm(0.02);
  void scanTick();
  return state;
}

async function resumeBackgroundScanAfterUnlock() {
  const s = await loadScanState();
  if (!s || s.status !== "scanning") return;
  scheduleScanAlarm(0.02);
  void scanTick();
}

function stopBackgroundScan() {
  chrome.alarms.clear(SCAN_ALARM);
  return loadScanState().then((s) => {
    if (s && s.status === "scanning") {
      s.status = "stopped";
      s.finishedAt = nowMs();
      return saveScanState(s).then(() => s);
    }
    return s;
  });
}

chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === SCAN_ALARM) scanTick();
});

// Resume scan on service worker restart
loadScanState().then((s) => {
  if (s && s.status === "scanning") {
    scheduleScanAlarm(0.05);
  }
});

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
        case "companion_status":
          sendResponse(ok(await companionStatus(params.baseUrl)));
          return;
        case "companion_lwd_info":
          sendResponse(
            ok(await companionLwdInfo(params.baseUrl, params.lightwalletd_url))
          );
          return;
        case "companion_lwd_chain_tip":
          sendResponse(
            ok(await companionLwdChainTip(params.baseUrl, params.lightwalletd_url))
          );
          return;
        case "companion_lwd_sync_compact":
          sendResponse(
            ok(
              await companionLwdSyncCompact(params.baseUrl, {
                start: Number(params.start ?? 0),
                end: params.end !== undefined && params.end !== null ? Number(params.end) : undefined,
                lightwalletd_url: params.lightwalletd_url,
                db_path: params.db_path
              })
            )
          );
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
        case "rpc_set_endpoint": {
          const next = params.url || session.rpcEndpoint;
          try {
            session.rpcEndpoint = normalizeRpcEndpoint(next);
          } catch (e) {
            throw e instanceof Error ? e : new Error(String(e));
          }
          {
            const existing = (await loadWalletState()) || {};
            await saveWalletState({ ...existing, rpcEndpoint: session.rpcEndpoint });
          }
          sendResponse(ok({ rpcEndpoint: session.rpcEndpoint }));
          return;
        }
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
          sendResponse(
            ok(await rpcCallWithRetry("getblock", [String(params.height ?? ""), 2]))
          );
          return;
        case "rpc_send_raw_tx": {
          const raw =
            typeof params?.rawTxHex === "string"
              ? params.rawTxHex
              : typeof params?.raw_tx_hex === "string"
                ? params.raw_tx_hex
                : "";
          const hex = raw.trim().replace(/^0x/i, "");
          if (!hex || !/^[0-9a-fA-F]+$/.test(hex)) {
            throw new Error(
              "Missing transaction hex. Close the popup and run Preview again, then broadcast without switching tabs."
            );
          }
          const txResult = await rpcCallWithRetry("sendrawtransaction", [hex, false]);
          const txid =
            typeof txResult === "string"
              ? txResult
              : txResult && typeof txResult === "object" && "txid" in txResult
                ? String(txResult.txid)
                : String(txResult ?? "");
          sendResponse(ok(txid));
          return;
        }
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
        case "wallet_start_scan": {
          if (!session.unlocked || !session.mnemonic || !session.address) {
            throw new Error("Unlock wallet first.");
          }
          const blockCount = await rpcCallWithRetry("getblockcount", []);
          const scanWindow = Number(params.window ?? 20_000);
          const startH = Math.max(0, blockCount - scanWindow);
          const s = await startBackgroundScan(startH, blockCount);
          sendResponse(ok({ started: true, startHeight: startH, endHeight: blockCount, status: s.status }));
          return;
        }
        case "wallet_scan_progress": {
          const scanState = await loadScanState();
          if (!scanState) {
            sendResponse(ok({ status: "idle" }));
          } else {
            const total = scanState.endHeight - scanState.startHeight + 1;
            const done = scanState.scannedBlocks;
            const pct = total > 0 ? Math.round((done / total) * 100) : 0;
            sendResponse(ok({
              status: scanState.status,
              startHeight: scanState.startHeight,
              endHeight: scanState.endHeight,
              currentHeight: scanState.currentHeight,
              scannedBlocks: done,
              totalBlocks: total,
              percent: pct,
              discoveredNotes: scanState.discoveredNotes?.length ?? 0,
              totalBalanceZats: scanState.totalBalanceZats ?? 0,
              startedAt: scanState.startedAt,
              elapsed: scanState.finishedAt
                ? scanState.finishedAt - scanState.startedAt
                : nowMs() - scanState.startedAt
            }));
          }
          return;
        }
        case "wallet_stop_scan": {
          const stopped = await stopBackgroundScan();
          sendResponse(ok({ status: stopped?.status ?? "idle" }));
          return;
        }
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

