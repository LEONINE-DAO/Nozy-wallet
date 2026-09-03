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
  buildSpeedUpTxStateEntry,
  canSpeedUpTx,
  findRecentBuiltTxId,
  isPilotTxExpired,
  nextLifecycleStateFromConfirmation,
  resolveTxidFromBroadcast
} from "./tx-lifecycle.js";
import {
  findReachableRpcEndpoint,
  isLocalRpcEndpoint,
  isWslStyleHost,
  normalizeRpcEndpoint,
  parseJsonRpcResponse,
  probeZebradRpcEndpoint,
  rpcGetBlockVerboseByHeight,
  rpcNetworkErrorMessage
} from "./rpc-utils.js";
import {
  mandatoryOrchardFeeZats,
  selectNotesForSpend,
  rpcFallbackWithRequester
} from "./tx-utils.js";
import {
  companionCheckConfirmations,
  companionLwdChainTip,
  companionLwdInfo,
  companionLwdSyncCompact,
  companionLwdSyncCompactToTip,
  companionSaplingScan,
  companionSaplingShield,
  companionSaplingStatus,
  companionSpeedUpTransaction,
  companionStatus,
  companionVoteActive,
  companionVoteCast,
  companionVoteDelegate,
  companionVoteDelegateFinish,
  companionVoteExportNotes,
  companionVoteImportNotes,
  companionVotePrepare,
  companionVoteSignDelegation,
  companionVoteSigningRequest,
  companionVoteStatus,
  companionVoteSubmitDelegationSig,
  companionCrosslinkStatus,
  companionCrosslinkPositions,
  companionCrosslinkRoster,
  companionCrosslinkStake,
  companionCrosslinkRetarget,
  companionCrosslinkUnbond,
  companionCrosslinkWithdraw,
  companionCrosslinkWalletStatus,
  companionCrosslinkWalletUfvk,
  companionZnsResolve,
  companionGetConfig,
  companionGenerateAddress,
  companionBroadcastRaw,
  companionSendEgress,
  companionPrivacyNetwork,
  companionSetPrivacyNetwork,
  companionNymMixnet,
  companionNymDvpn,
  companionSetNymDvpn,
  companionNymDvpnProbe,
  companionNymVpnApp
} from "./companion-api.js";

const STORAGE_KEY = "nozy_wallet_state_v1";
const RPC_CACHE_KEY = "nozy_zebra_rpc_cache_v1";
const COMPANION_BASE_KEY = "nozy_companion_base_url";
const STORAGE_LWD_URL = "nozy_lightwalletd_url";
const MOBILE_SYNC_KEY = "nozy_mobile_sync_v1";
const TX_STATE_KEY = "nozy_tx_state_v1";
const SESSION_POLICY_KEY = "nozy_session_policy_v1";
const DEFAULT_AUTO_LOCK_MS = 15 * 60 * 1000;

/**
 * Session-storage key for the encrypted mnemonic blob.
 * chrome.storage.session survives MV3 service-worker restarts within the same
 * browser session but is wiped when the browser closes — so:
 *   - SW killed by Chrome idle timer → auto-resume without user interaction ✓
 *   - Browser restart → one unlock needed (same as any wallet) ✓
 * We store the same AES-GCM ciphertext that lives on disk; the password is
 * never stored. Auto-resume decrypts with the password derived from the blob
 * header — but we can't do that without the password. Instead we cache the
 * decrypted mnemonic as a byte array that is only kept in session storage
 * during a running browser session.
 *
 * Security: chrome.storage.session is inaccessible to web pages and other
 * extensions. It is cleared on browser close. This matches what a typical
 * software wallet keeps in process memory while running.
 */
const SESSION_MNEMONIC_KEY = "nozy_session_mnemonic_v1";

function sessionStorageApi() {
  return typeof chrome !== "undefined" && chrome.storage && chrome.storage.session
    ? chrome.storage.session
    : null;
}

/** Persist the plaintext mnemonic in session storage so SW restart can resume scan. */
async function saveSessionMnemonic(mnemonic) {
  const api = sessionStorageApi();
  if (!api || !mnemonic) return;
  await new Promise((resolve) => {
    api.set({ [SESSION_MNEMONIC_KEY]: mnemonic }, () => resolve());
  });
}

/** Read back the session mnemonic (null if browser was restarted or never set). */
async function loadSessionMnemonic() {
  const api = sessionStorageApi();
  if (!api) return null;
  return new Promise((resolve) => {
    api.get([SESSION_MNEMONIC_KEY], (v) => {
      const m = v?.[SESSION_MNEMONIC_KEY];
      resolve(typeof m === "string" && m.trim() ? m.trim() : null);
    });
  });
}

/** Wipe the session mnemonic (on explicit lock or vault reset). */
function clearSessionMnemonic() {
  const api = sessionStorageApi();
  if (!api) return;
  try { api.remove([SESSION_MNEMONIC_KEY], () => {}); } catch (_) { /* ignore */ }
}

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

/** dApp / page provider methods (content script only, or extension page for debugging). */
const DAPP_PROVIDER_METHODS = new Set([
  "eth_chainId",
  "zcash_chainId",
  "eth_getBalance",
  "wallet_watchAsset",
  "eth_accounts",
  "zcash_accounts",
  "eth_requestAccounts",
  "zcash_requestAccounts",
  "personal_sign",
  "zcash_signMessage",
  "eth_sendTransaction",
  "zcash_sendTransaction"
]);

function extensionOriginPrefix() {
  return `chrome-extension://${chrome.runtime.id}/`;
}

function mozExtensionOriginPrefix() {
  return `moz-extension://${chrome.runtime.id}/`;
}

function isExtensionPageUrl(url) {
  if (!url || typeof url !== "string") return false;
  return url.startsWith(extensionOriginPrefix()) || url.startsWith(mozExtensionOriginPrefix());
}

/** Popup / options / other extension pages — may call privileged methods. */
function isExtensionPageSender(sender) {
  if (!sender || sender.id !== chrome.runtime.id) return false;
  if (sender.tab) {
    return isExtensionPageUrl(sender.tab.url || sender.url || "");
  }
  return isExtensionPageUrl(sender.url || "") || !sender.url;
}

/** Injected content script on a web page — dApp methods only. */
function isContentScriptSender(sender) {
  if (!sender || sender.id !== chrome.runtime.id || !sender.tab) return false;
  return !isExtensionPageUrl(sender.tab.url || sender.url || "");
}

function assertMethodAllowedForSender(method, sender) {
  if (DAPP_PROVIDER_METHODS.has(method)) {
    if (isContentScriptSender(sender) || isExtensionPageSender(sender)) return;
    throw new Error("Provider method not allowed from this context.");
  }
  if (!isExtensionPageSender(sender)) {
    throw new Error("This wallet method is only available from the NozyWallet extension UI.");
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
  trackerState = await initShieldedTrackerState(startHeight, rpcEndpoint);

  for (let h = startHeight; h <= endHeight; h += 1) {
    scannedBlocks += 1;
    try {
      const block = await rpcGetBlockVerboseByHeight(rpcEndpoint, h);
      if (!block) continue;
      const blockJson = JSON.stringify(block);
      const { out, nextTracker } = applyShieldedScanBlock(
        trackerState,
        mnemonic,
        address,
        h,
        blockJson
      );
      if (nextTracker) trackerState = nextTracker;
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

/** Zebra `z_gettreestate` uses orchard.commitments.finalRoot; legacy nodes may use anchor. */
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

/**
 * Normalize a stored scan row to `{ note, height, txid, value }` for spend selection.
 * Supports wrapped rows and flat Orchard decryption payloads.
 */
function normalizeDiscoveredScanEntry(raw) {
  if (!raw || typeof raw !== "object") return null;
  if (raw.note != null && typeof raw.note === "object") {
    const v = Number(raw.value ?? raw.note?.value ?? 0);
    if (!Number.isFinite(v) || v <= 0) return null;
    const height = Number(raw.height ?? raw.note?.block_height ?? 0);
    const txid = String(raw.txid ?? raw.note?.txid ?? "");
    return { note: raw.note, height, txid, value: v };
  }
  const v = Number(raw.value ?? 0);
  if (!Number.isFinite(v) || v <= 0) return null;
  const height = Number(raw.block_height ?? 0);
  const txid = String(raw.txid ?? "");
  return { note: raw, height, txid, value: v };
}

/** Read witness fields (snake_case or camelCase from serde_wasm_bindgen). */
function orchardWitnessFieldsFromNote(noteObj) {
  if (!noteObj || typeof noteObj !== "object") {
    return { witnessHex: "", tipHeight: NaN };
  }
  const witnessHex =
    (typeof noteObj.orchard_incremental_witness_hex === "string" &&
      noteObj.orchard_incremental_witness_hex) ||
    (typeof noteObj.orchardIncrementalWitnessHex === "string" &&
      noteObj.orchardIncrementalWitnessHex) ||
    "";
  const tipHeight = Number(
    noteObj.orchard_witness_tip_height ?? noteObj.orchardWitnessTipHeight ?? NaN
  );
  return { witnessHex, tipHeight };
}

async function _inlineRpcRequest(endpoint, method, params = []) {
  const url = normalizeRpcEndpoint(endpoint);
  const resp = await fetch(url, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params })
  });
  if (!resp.ok) throw new Error(`RPC ${method} HTTP ${resp.status}`);
  return parseJsonRpcResponse(resp, url, method);
}

async function _inlineProveTransaction(params) {
  await ensureWasm();
  const recipientAddress = String(params?.recipientAddress ?? params?.to ?? "");
  const walletAddress = String(params?.walletAddress ?? "");
  const mnemonic = String(params?.mnemonic ?? "");
  const rpcEndpoint = String(params?.rpcEndpoint ?? "");
  const requestedAmount = Number(params?.amount ?? 0);
  const memo = String(params?.memo ?? "");
  const requestedFee = mandatoryOrchardFeeZats(wasm, memo);

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
  const rawList = Array.isArray(scanState?.discoveredNotes) ? scanState.discoveredNotes : [];
  const candidates = rawList.map(normalizeDiscoveredScanEntry).filter(Boolean);

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
  const chainTip = Number(await _inlineRpcRequest(endpoint, "getblockcount", []));
  const txBuildHeight = chainTip + 1;
  const heightStr = String(chainTip);

  const rpcReq = (at) => _inlineRpcRequest(endpoint, at.method, at.params ?? []);
  const orchardTree = await rpcFallbackWithRequester(rpcReq, [
    { method: "z_getorchardtree", params: [heightStr] },
    { method: "z_gettreestate", params: [heightStr] }
  ]);
  const anchorHex = _orchardAnchorHexFromRpc(orchardTree);
  if (!anchorHex || anchorHex.length < 64) throw new Error("RPC did not return a valid Orchard anchor.");

  const selectedWitnesses = [];
  for (const noteSel of selected) {
    const { witnessHex: w0, tipHeight: tipFromNote } = orchardWitnessFieldsFromNote(noteSel?.note);
    let witnessHex = w0;
    let tip = Number.isFinite(tipFromNote)
      ? tipFromNote
      : Number(noteSel?.height ?? 0);
    if (!witnessHex || typeof witnessHex !== "string") {
      throw new Error(
        "Note missing orchard_incremental_witness_hex. Rescan with the updated extension that records Orchard witnesses (Zebrad-compatible)."
      );
    }
    if (!Number.isFinite(tip) || tip < 0) {
      throw new Error("Invalid orchard_witness_tip_height on note.");
    }
    for (let h = tip + 1; h <= chainTip; h += 1) {
      const block = await rpcGetBlockVerboseByHeight(endpoint, h);
      witnessHex = wasm.advance_orchard_witness_hex(witnessHex, JSON.stringify(block));
    }
    if (!wasm.orchard_witness_matches_anchor_hex(witnessHex, anchorHex)) {
      throw new Error("Orchard witness does not match anchor at tip (rescan or wait for sync).");
    }
    selectedWitnesses.push({
      incremental_witness_hex: witnessHex,
      anchor_hex: anchorHex,
      target_height: txBuildHeight
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
  // Always priority ×4 — matches CLI / api-server / desktop.
  const fee = await estimateFeeZats(memo, true);
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

function storageRemove(keys) {
  return new Promise((resolve) => {
    chrome.storage.local.remove(keys, () => resolve());
  });
}

/** Drop scan/tx caches that belong to a previous seed. */
async function wipeDerivedWalletData() {
  const scan = await loadScanState();
  if (scan?.status === "scanning") {
    await stopBackgroundScan();
  }
  clearScanResumeForBackground();
  await storageRemove([SCAN_STATE_KEY, TX_STATE_KEY]);
}

/** Remove the Chrome vault so Welcome can restore the Desktop seed. */
async function walletReset() {
  await wipeDerivedWalletData();
  session.unlocked = false;
  session.mnemonic = null;
  session.address = null;
  clearSessionMnemonic();
  await storageRemove([STORAGE_KEY]);
  return { exists: false };
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
  const txid = await broadcastRawHex(tx.rawTxHex, { retries: 2, baseDelayMs: 500 });
  await patchTxStateById(id, {
    txid: String(txid),
    state: "broadcast",
    error: null
  });
  return String(txid);
}

async function pilotExpiryHeightForTip(chainTip) {
  await ensureWasm();
  const delta = Number(wasm.pilot_expiry_delta_blocks?.() ?? 5);
  const tip = Number(chainTip);
  if (!Number.isFinite(tip)) return null;
  return tip + delta;
}

async function refreshTxExpiryStates() {
  if (!session.rpcEndpoint) return;
  let chainTip;
  try {
    chainTip = Number(await rpcCall("getblockcount", []));
  } catch (_) {
    return;
  }
  if (!Number.isFinite(chainTip)) return;

  const state = await loadTxState();
  const txs = Array.isArray(state.txs) ? state.txs : [];
  let changed = false;

  for (const tx of txs) {
    if (!tx?.txid) continue;
    if (tx.state !== "pending" && tx.state !== "broadcast") continue;
    if (!tx.expiryHeight) continue;
    if (!isPilotTxExpired(chainTip, tx.expiryHeight)) continue;
    try {
      const resp = await rpcCall("getrawtransaction", [tx.txid, true]);
      const height = resp?.blockheight ?? resp?.blockHeight ?? resp?.block_height ?? null;
      const bh = typeof height === "number" ? height : parseNumberMaybeHex(height);
      if (Number.isFinite(bh) && bh > 0) continue;
    } catch (_) {
      // Not mined — eligible for expired.
    }
    tx.state = "expired";
    tx.updatedAt = nowMs();
    changed = true;
  }

  if (changed) {
    await saveTxState({ txs, updatedAt: nowMs() });
  }
}

async function speedUpTxById(id, opts = {}) {
  await refreshTxExpiryStates();
  const state = await loadTxState();
  const txs = Array.isArray(state.txs) ? state.txs : [];
  const tx = txs.find((t) => t.id === id);
  if (!tx) throw new Error("Transaction record not found.");
  if (!canSpeedUpTx(tx)) {
    throw new Error(`Speed-up is not available for transaction state: ${tx.state}`);
  }
  if (!tx.txid) throw new Error("Speed-up requires a broadcast txid.");
  if (!session.unlocked || !session.mnemonic) throw new Error("Wallet is locked.");

  const companionBase = await loadCompanionBaseUrl();
  const companionPassword = String(opts.companionPassword ?? "").trim();
  if (companionPassword) {
    try {
      const result = await companionSpeedUpTransaction(companionBase, {
        original_txid: tx.txid,
        password: companionPassword,
        zebra_url: session.rpcEndpoint
      });
      if (result?.success && result?.txid) {
        await patchTxStateById(id, { state: "expired", error: null });
        const chainTip = Number(await rpcCall("getblockcount", []));
        const expiryHeight = await pilotExpiryHeightForTip(chainTip);
        await appendTxState(
          buildSpeedUpTxStateEntry({
            id: crypto.randomUUID(),
            origin: tx.origin || "",
            proving: {
              txid: result.txid,
              recipientAddress: tx.recipientAddress,
              amount: tx.amount,
              fee: await estimateFeeZats(tx.memo || "", true),
              memo: tx.memo || "",
              inputs_used: tx.inputsUsed ?? 1,
              rawTxHex: ""
            },
            createdAt: nowMs(),
            speedUpOf: tx.txid,
            expiryHeight
          })
        );
        return String(result.txid);
      }
      if (result?.message) throw new Error(result.message);
    } catch (e) {
      if (!opts.allowWasmFallback) throw e;
    }
  }

  const priorityFee = await estimateFeeZats(tx.memo || "", true);
  const proving = await callWorker("prove_transaction", {
    recipientAddress: tx.recipientAddress,
    walletAddress: session.address,
    mnemonic: session.mnemonic,
    rpcEndpoint: session.rpcEndpoint,
    amount: tx.amount,
    fee: priorityFee,
    memo: tx.memo || ""
  });
  if (!proving?.rawTxHex) {
    throw new Error("Speed-up proving did not return rawTxHex");
  }

  const newTxid = await broadcastRawHex(proving.rawTxHex, { retries: 3, baseDelayMs: 400 });
  const chainTip = Number(await rpcCall("getblockcount", []));
  const expiryHeight = await pilotExpiryHeightForTip(chainTip);

  await patchTxStateById(id, { state: "expired", error: null });
  await appendTxState(
    buildSpeedUpTxStateEntry({
      id: crypto.randomUUID(),
      origin: tx.origin || "",
      proving: { ...proving, fee: priorityFee },
      createdAt: nowMs(),
      speedUpOf: tx.txid,
      expiryHeight
    })
  );

  return String(newTxid);
}

async function loadWalletState() {
  const state = await storageGet(STORAGE_KEY);
  return state || null;
}

async function saveWalletState(state) {
  await storageSet({ [STORAGE_KEY]: state });
}

async function loadRpcEndpointCache() {
  const cached = await storageGet(RPC_CACHE_KEY);
  return Array.isArray(cached) ? cached.filter((u) => typeof u === "string") : [];
}

async function rememberRpcEndpoint(endpoint) {
  const url = String(endpoint ?? "").trim();
  if (!url) return;
  const prev = await loadRpcEndpointCache();
  const next = [url, ...prev.filter((u) => u !== url)].slice(0, 8);
  await storageSet({ [RPC_CACHE_KEY]: next });
}

async function loadCompanionPrefs() {
  return new Promise((resolve) => {
    chrome.storage.local.get(
      {
        [COMPANION_BASE_KEY]: "http://127.0.0.1:3000",
        [STORAGE_LWD_URL]: ""
      },
      (items) => {
        resolve({
          baseUrl: String(items[COMPANION_BASE_KEY] || "http://127.0.0.1:3000").replace(
            /\/+$/,
            ""
          ),
          lightwalletdUrl: String(items[STORAGE_LWD_URL] ?? "").trim()
        });
      }
    );
  });
}

async function loadCompanionBaseUrl() {
  const prefs = await loadCompanionPrefs();
  return prefs.baseUrl;
}

function companionErrorMessage(err) {
  const raw = err instanceof Error ? err.message : String(err);
  try {
    const j = JSON.parse(raw);
    if (typeof j?.error === "string") return j.error;
  } catch (_) {
    /* plain text */
  }
  return raw;
}

/**
 * Submit via companion `ZebraClient` so remote sendraw uses the same Nym mixnet helper
 * as desktop/CLI. Local/LAN RPC may fall back to Chrome JSON-RPC if the companion is down.
 */
async function broadcastRawHex(hex, opts = {}) {
  const companionBase = await loadCompanionBaseUrl();
  const local = isLocalRpcEndpoint(session.rpcEndpoint);
  try {
    const result = await companionBroadcastRaw(companionBase, {
      raw_transaction_hex: hex,
      zebra_url: session.rpcEndpoint
    });
    const txid = resolveTxidFromBroadcast(result, result?.txid ?? "");
    if (!txid) {
      throw new Error(result?.message || "Companion broadcast returned no txid");
    }
    return String(txid);
  } catch (e) {
    const msg = companionErrorMessage(e);
    if (!local) {
      throw new Error(
        `Remote send must go through the companion API (same Nym mixnet path as desktop/CLI). ${msg}`
      );
    }
  }
  const broadcastResult = await rpcCallWithRetry("sendrawtransaction", [hex, false], {
    retries: opts.retries ?? 2,
    baseDelayMs: opts.baseDelayMs ?? 500
  });
  return String(resolveTxidFromBroadcast(broadcastResult, ""));
}

async function persistRpcEndpoint(found) {
  session.rpcEndpoint = found;
  await rememberRpcEndpoint(found);
  const existing = (await loadWalletState()) || {};
  await saveWalletState({ ...existing, rpcEndpoint: session.rpcEndpoint });
  return found;
}

async function autodetectZebradRpcEndpoint() {
  const cached = await loadRpcEndpointCache();
  const companionBase = await loadCompanionBaseUrl();
  const found = await findReachableRpcEndpoint(session.rpcEndpoint, {
    extraCandidates: cached,
    companionBase
  });
  if (!found) {
    throw new Error(
      "Could not find Zebrad. Start your node first, then click Find my node in the extension.\n\n" +
        "• Zebrad on this PC: use port 8232\n" +
        "• Zebrad in WSL: we auto-detect the WSL IP (not 127.0.0.1 from Chrome)\n" +
        "• Remote VPS: paste your RPC URL under Remote VPS\n\n" +
        "Running Nozy Desktop? Start nozywallet-api — we can read its zebra_url config."
    );
  }
  return persistRpcEndpoint(found);
}

async function readRpcBlockCount() {
  const raw = await rpcCallWithRetry("getblockcount", [], { retries: 1 });
  const n = typeof raw === "number" ? raw : Number(raw);
  return Number.isFinite(n) && n >= 0 ? Math.floor(n) : 0;
}

/** Try Nozy Desktop / api-server config for zebra_url (same stack as CLI). */
async function tryCompanionZebraUrl() {
  try {
    const base = await loadCompanionBaseUrl();
    const cfg = await companionGetConfig(base);
    const zebraUrl = cfg?.zebra_url;
    if (!zebraUrl || typeof zebraUrl !== "string") return null;
    const url = normalizeRpcEndpoint(zebraUrl.trim());
    if (await probeZebradRpcEndpoint(url, 4000)) {
      return url;
    }
  } catch (_) {
    /* companion optional */
  }
  return null;
}

/**
 * One-shot node connect for extension onboarding.
 * @param {{ url?: string, tryCompanion?: boolean }} [opts]
 */
async function connectZebradRpc(opts = {}) {
  const explicitUrl = String(opts?.url ?? "").trim();
  if (explicitUrl) {
    const url = normalizeRpcEndpoint(explicitUrl);
    if (!(await probeZebradRpcEndpoint(url, 4500))) {
      throw new Error(
        `Cannot reach Zebrad at ${url}. Check the node is running and the URL is correct. ` +
          `WSL/Docker: use the VM IP, not 127.0.0.1 from Windows Chrome.`
      );
    }
    await persistRpcEndpoint(url);
    return {
      rpcEndpoint: url,
      blockCount: await readRpcBlockCount(),
      connected: true,
      source: "manual"
    };
  }

  if (opts.tryCompanion !== false) {
    const fromCompanion = await tryCompanionZebraUrl();
    if (fromCompanion) {
      await persistRpcEndpoint(fromCompanion);
      return {
        rpcEndpoint: fromCompanion,
        blockCount: await readRpcBlockCount(),
        connected: true,
        source: "companion"
      };
    }
  }

  const found = await autodetectZebradRpcEndpoint();
  return {
    rpcEndpoint: found,
    blockCount: await readRpcBlockCount(),
    connected: true,
    source: "autodetect"
  };
}

/** Probe current RPC; auto-detect WSL/local Zebrad if saved URL (often 127.0.0.1) is wrong. */
async function ensureReachableZebradRpc() {
  let endpoint;
  try {
    endpoint = normalizeRpcEndpoint(session.rpcEndpoint);
  } catch (e) {
    throw e instanceof Error ? e : new Error(String(e));
  }
  if (await probeZebradRpcEndpoint(endpoint, 3500)) {
    return endpoint;
  }
  return autodetectZebradRpcEndpoint();
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
  // Extension wallets need a live Zebrad RPC for birthday + Orchard scan — refuse offline create.
  const rpcUrl = await ensureReachableZebradRpc();
  await ensureWasm();
  await wipeDerivedWalletData();
  const created = wasm.create_wallet(password);
  const mnemonic = created.mnemonic;
  const address = created.address;
  const encryptedMnemonic = Array.from(
    wasm.encrypt_for_storage(utf8Encode(mnemonic), password)
  );

  const orchardBirthdayHeight = await tryGetChainTipForBirthday();
  await saveWalletState({
    encryptedMnemonic,
    address,
    createdAt: Date.now(),
    rpcEndpoint: rpcUrl || session.rpcEndpoint,
    orchardBirthdayHeight,
    restoredFromPhrase: false
  });

  session.unlocked = true;
  session.mnemonic = mnemonic;
  session.address = address;
  touchSession();
  await saveSessionMnemonic(mnemonic);
  await startAutoBackgroundScan();

  return { address };
}

async function walletRestore(mnemonic, password, opts = {}) {
  // Same as create: restore without a node leaves an unscannable wallet.
  const rpcUrl = await ensureReachableZebradRpc();
  await ensureWasm();
  await wipeDerivedWalletData();
  const restored = wasm.restore_wallet(mnemonic, password);
  const address = restored.address;
  const encryptedMnemonic = Array.from(
    wasm.encrypt_for_storage(utf8Encode(mnemonic), password)
  );

  let orchardBirthdayHeight = null;
  const rawBh = opts?.birthdayHeight;
  if (rawBh !== undefined && rawBh !== null && rawBh !== "") {
    const bh = Number(rawBh);
    if (Number.isFinite(bh) && bh >= 0) orchardBirthdayHeight = Math.floor(bh);
  }
  if (orchardBirthdayHeight === null) {
    // Never use chain tip — that scans one block and misses the restored wallet's notes.
    orchardBirthdayHeight = await defaultRestoreBirthdayHeight();
  }

  await saveWalletState({
    encryptedMnemonic,
    address,
    createdAt: Date.now(),
    rpcEndpoint: rpcUrl || session.rpcEndpoint,
    orchardBirthdayHeight,
    restoredFromPhrase: true
  });

  session.unlocked = true;
  session.mnemonic = mnemonic;
  session.address = address;
  touchSession();
  await saveSessionMnemonic(mnemonic);
  await startAutoBackgroundScan();

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
  await saveSessionMnemonic(mnemonic);

  try {
    await resumeBackgroundScanAfterUnlock();
  } catch {
    // Auto-sync may fail if RPC is unreachable; unlock still succeeds.
  }

  const scanSt = await loadScanState();
  if (scanSt?.status === "scanning") {
    await persistScanResumeForBackground(mnemonic, address);
  }

  try {
    await ensureReachableZebradRpc();
  } catch {
    // User can fix RPC in Settings; unlock should still succeed.
  }

  return { address };
}

async function walletLock() {
  const scan = await loadScanState();
  if (scan?.status === "scanning") {
    await stopBackgroundScan();
  }
  session.unlocked = false;
  session.mnemonic = null;
  session.address = null;
  clearScanResumeForBackground();
  clearSessionMnemonic();
  return true;
}

async function getWalletStatus() {
  const state = await loadWalletState();
  const bh = state?.orchardBirthdayHeight;
  const orchardBirthdayHeight =
    typeof bh === "number" && Number.isFinite(bh) && bh >= 0 ? Math.floor(bh) : null;
  return {
    exists: !!state,
    unlocked: session.unlocked,
    address: session.address || state?.address || null,
    rpcEndpoint: session.rpcEndpoint,
    orchardBirthdayHeight
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
  try {
    if (chrome.action?.openPopup) {
      await chrome.action.openPopup();
    }
  } catch (_) {
    try {
      await chrome.windows.create({
        url: chrome.runtime.getURL("wasm-core/popup/dist/index.html"),
        type: "popup",
        width: 420,
        height: 680
      });
    } catch (_) {
      /* user must open the popup manually */
    }
  }
  return approval;
}

async function ensureSessionInitialized() {
  const wallet = (await loadWalletState()) || {};
  session.rpcEndpoint = wallet.rpcEndpoint || session.rpcEndpoint;
  const policy = await loadSessionPolicy();
  session.autoLockMs = Number(policy.autoLockMs) || DEFAULT_AUTO_LOCK_MS;
  if (!session.lastActivityAt) touchSession();

  // Auto-hydrate from session storage after a service-worker restart.
  // chrome.storage.session survives SW idle kills within the same browser session.
  if (!session.unlocked && wallet.encryptedMnemonic) {
    try {
      const sessionMnemonic = await loadSessionMnemonic();
      if (sessionMnemonic && typeof sessionMnemonic === "string") {
        await ensureWasm();
        // Verify the mnemonic still matches this vault by deriving the address.
        const derivedAddress = wasm.generate_address(sessionMnemonic, 0, 0);
        if (derivedAddress === wallet.address || !wallet.address) {
          session.unlocked = true;
          session.mnemonic = sessionMnemonic;
          session.address = derivedAddress;
          touchSession();
        } else {
          // Address mismatch — vault was replaced; wipe the stale session key.
          clearSessionMnemonic();
        }
      }
    } catch (_) {
      // Never block startup on auto-hydration errors.
    }
  }
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
    // Auto-recover from common local RPC port mismatch (8232 vs 18232).
    const fallbackEndpoint = await findReachableRpcEndpoint(endpoint, {
      extraCandidates: await loadRpcEndpointCache(),
      companionBase: await loadCompanionBaseUrl()
    });
    if (fallbackEndpoint && fallbackEndpoint !== endpoint) {
      session.rpcEndpoint = fallbackEndpoint;
      await rememberRpcEndpoint(fallbackEndpoint);
      const existing = (await loadWalletState()) || {};
      await saveWalletState({ ...existing, rpcEndpoint: session.rpcEndpoint });
      resp = await fetch(fallbackEndpoint, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          jsonrpc: "2.0",
          id: 1,
          method,
          params
        })
      });
    } else {
      throw new Error(rpcNetworkErrorMessage(endpoint, err));
    }
  }
  if (resp.status === 401 || resp.status === 403) {
    throw new Error(
      `RPC returned HTTP ${resp.status} (authentication required). ` +
        `For Zebra, set enable_cookie_auth=false in zebrad.toml for local JSON-RPC, or run a small proxy that adds the expected credentials.`
    );
  }
  if (!resp.ok) throw new Error(`RPC HTTP ${resp.status}`);
  try {
    const result = await parseJsonRpcResponse(resp, endpoint, method);
    try {
      const host = new URL(endpoint).hostname;
      if (isWslStyleHost(host) || host === "127.0.0.1" || host === "localhost") {
        await rememberRpcEndpoint(endpoint);
      }
    } catch {
      // ignore cache errors
    }
    return result;
  } catch (parseErr) {
    const msg = String(parseErr?.message ?? parseErr);
    if (!/web page|not JSON|DOCTYPE|127\.0\.0\.1|localhost/i.test(msg)) {
      throw parseErr;
    }
    const found = await autodetectZebradRpcEndpoint();
    if (found === endpoint) throw parseErr;
    return rpcCall(method, params);
  }
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

/** Best-effort chain tip for persisting Orchard scan birthday (null if RPC unset/offline). */
async function tryGetChainTipForBirthday() {
  try {
    const n = Number(await rpcCallWithRetry("getblockcount", []));
    if (Number.isFinite(n) && n >= 0) return Math.floor(n);
  } catch (_) {
    /* ignore */
  }
  return null;
}

/**
 * Orchard activation (NU5) fallback scan floor.
 * Mainnet: 1,687,104
 * Testnet: 1,842,420
 */
async function getOrchardActivationHeight() {
  const MAINNET_NU5 = 1_687_104;
  const TESTNET_NU5 = 1_842_420;
  try {
    const info = await rpcCallWithRetry("getblockchaininfo", [], { retries: 1, baseDelayMs: 150 });
    const chain = String(info?.chain ?? "").toLowerCase();
    if (chain.includes("test")) return TESTNET_NU5;
    return MAINNET_NU5;
  } catch (_) {
    return MAINNET_NU5;
  }
}

/** Same floor as `nozy::wallet_sync::MAINNET_DEFAULT_SCAN_START` (Desktop restore). */
const MAINNET_RESTORE_SCAN_FLOOR = 3_050_000;
const RESTORE_BIRTHDAY_NEAR_TIP = 64;

async function defaultRestoreBirthdayHeight() {
  const act = await getOrchardActivationHeight();
  if (act === 1_842_420) return 1;
  return MAINNET_RESTORE_SCAN_FLOOR;
}

function birthdayLooksLikeChainTip(birthday, chainTip) {
  if (!Number.isFinite(birthday) || !Number.isFinite(chainTip) || chainTip < 0) return false;
  return Math.floor(birthday) >= Math.floor(chainTip) - RESTORE_BIRTHDAY_NEAR_TIP;
}

/**
 * Restore used to save birthday = chain tip, so auto-sync scanned one block.
 * Rewind to Desktop's scan floor when that happens.
 */
async function resolveScanBirthday(ws, chainTip, scanState) {
  const floor = await defaultRestoreBirthdayHeight();
  const tip = Number.isFinite(chainTip) && chainTip >= 0 ? Math.floor(chainTip) : floor;
  const bh = Number(ws?.orchardBirthdayHeight);
  const notes = Array.isArray(scanState?.discoveredNotes) ? scanState.discoveredNotes.length : 0;
  const scanned = Number(scanState?.scannedBlocks ?? 0);
  const restored = ws?.restoredFromPhrase === true;
  const finishedEmpty =
    scanState &&
    (scanState.status === "done" || scanState.status === "stopped" || scanState.status === "failed") &&
    notes === 0 &&
    scanned <= 2;
  const tinyRange =
    scanState &&
    typeof scanState.startHeight === "number" &&
    typeof scanState.endHeight === "number" &&
    Number(scanState.endHeight) - Number(scanState.startHeight) <= RESTORE_BIRTHDAY_NEAR_TIP &&
    notes === 0;
  const tipBug =
    (restored && (!Number.isFinite(bh) || birthdayLooksLikeChainTip(bh, tip))) ||
    (birthdayLooksLikeChainTip(bh, tip) && (finishedEmpty || tinyRange));

  if (tipBug) {
    const birthday = Math.min(tip, floor);
    if (ws && typeof ws === "object") {
      await saveWalletState({
        ...ws,
        orchardBirthdayHeight: birthday,
        restoredFromPhrase: true
      });
    }
    return { birthday, rewound: true };
  }
  if (Number.isFinite(bh) && bh >= 0) {
    return { birthday: Math.min(tip, Math.floor(bh)), rewound: false };
  }
  if (restored) {
    return { birthday: Math.min(tip, floor), rewound: false };
  }
  const orchardActivation = await getOrchardActivationHeight();
  return { birthday: Math.max(0, Math.min(tip, orchardActivation)), rewound: false };
}

async function estimateFeeZats(memo = "", _priority = true) {
  // Always ZIP-317 × 4; `_priority` kept for call-site compat and ignored.
  try {
    await ensureWasm();
    return mandatoryOrchardFeeZats(wasm, memo);
  } catch (_) {
    return mandatoryOrchardFeeZats(null, memo);
  }
}

function ironwoodNotesFromScan(state) {
  const rows = Array.isArray(state?.discoveredNotes) ? state.discoveredNotes : [];
  return rows
    .filter((row) => {
      const n = row?.note ?? row;
      return (row?.pool || n?.pool) === "ironwood";
    })
    .map((row) => {
      const n = row?.note ?? row;
      return {
        ...n,
        pool: "ironwood",
        height: row.height ?? n.block_height,
        txid: row.txid ?? n.txid,
        value: row.value ?? n.value
      };
    });
}

async function walletVoteExportNotes() {
  if (!session.unlocked || !session.mnemonic) throw new Error("Unlock wallet first.");
  await ensureWasm();
  if (typeof wasm.export_ironwood_vote_notes_json !== "function") {
    throw new Error("Reload the unpacked extension after rebuilding WASM (vote export).");
  }
  const scan = await loadScanState();
  const notesJson = wasm.export_ironwood_vote_notes_json(
    session.mnemonic,
    JSON.stringify(ironwoodNotesFromScan(scan)),
    "mainnet"
  );
  const parsed = JSON.parse(notesJson);
  const noteCount = Array.isArray(parsed.notes) ? parsed.notes.length : 0;
  const total = (parsed.notes || []).reduce((acc, n) => acc + Number(n.value || 0), 0);
  return {
    notes_json: notesJson,
    note_count: noteCount,
    total_value_zat: total,
    message: `Exported ${noteCount} Ironwood note(s) from this extension wallet.`
  };
}

async function walletVoteSignDelegation(requestJson) {
  if (!session.unlocked || !session.mnemonic) throw new Error("Unlock wallet first.");
  await ensureWasm();
  if (typeof wasm.sign_vote_delegation !== "function") {
    throw new Error("Reload the unpacked extension after rebuilding WASM (vote sign).");
  }
  return wasm.sign_vote_delegation(session.mnemonic, String(requestJson || ""));
}

setInterval(() => {
  if (!session.unlocked) return;
  if (!session.lastActivityAt) return;
  // Keep wallet unlocked while a background Orchard scan is running (popup may be closed).
  loadScanState()
    .then((scan) => {
      if (scan?.status === "scanning") return;
      if (nowMs() - session.lastActivityAt >= session.autoLockMs) {
        walletLock();
      }
    })
    .catch(() => {
      if (nowMs() - session.lastActivityAt >= session.autoLockMs) {
        walletLock();
      }
    });
}, 20_000);

setInterval(() => {
  cleanupStaleMobileSyncState().catch(() => undefined);
}, 30_000);

// ── Background scan ────────────────────────────────────────────────
const SCAN_STATE_KEY = "nozy_scan_state_v1";
/**
 * Blocks processed per tick. The scan continues immediately after each tick (see scanTick tail)
 * so throughput is batch-size × ticks-per-second, not throttled by Chrome's ~30s alarm floor.
 * Kept large enough to amortize per-tick overhead but small enough that one tick stays well
 * within a service-worker task budget.
 */
const SCAN_BATCH = 800;
/** First wake processes fewer blocks so the first `saveScanState` (and UI poll) happens sooner. */
const SCAN_FIRST_BATCH = 80;
/** Persist tracker/notes periodically; large tracker JSON makes frequent saves costly. */
const SCAN_SAVE_EVERY_BLOCKS = 50;
/** Parallel Zebrad `getblock` fetches per sub-batch (witness apply stays sequential). */
const SCAN_PARALLEL_FETCH = 3;
/** Min interval between companion Sapling balance polls during an active scan. */
const SAPLING_REFRESH_MS = 120_000;

function scanPercentInt(state) {
  const total = Math.max(1, (state.endHeight ?? 0) - (state.startHeight ?? 0) + 1);
  const done =
    typeof state.heightProgress === "number"
      ? Math.min(total, Math.max(0, state.heightProgress - state.startHeight + 1))
      : Math.min(total, state.scannedBlocks ?? 0);
  return Math.min(100, Math.max(0, Math.floor((done / total) * 100)));
}

function scanRpcEndpoint(state) {
  return session.rpcEndpoint || state?.rpcEndpoint || "http://127.0.0.1:8232";
}

function poolFinalStateFromTreestate(ts, pool) {
  const c = ts?.[pool]?.commitments ?? ts?.[pool];
  return (
    (typeof c?.finalState === "string" && c.finalState) ||
    (typeof c?.final_state === "string" && c.final_state) ||
    ""
  );
}

function parseShieldedTrackerState(raw) {
  if (typeof raw === "string" && raw.trim().startsWith("{")) {
    try {
      const o = JSON.parse(raw);
      if (o && typeof o === "object") {
        return {
          orchard: typeof o.orchard === "string" ? o.orchard : "",
          ironwood: typeof o.ironwood === "string" ? o.ironwood : ""
        };
      }
    } catch (_) {
      /* legacy string below */
    }
  }
  if (typeof raw === "string") return { orchard: raw, ironwood: "" };
  return { orchard: "", ironwood: "" };
}

function serializeShieldedTrackerState(trackers) {
  return JSON.stringify({
    orchard: trackers.orchard || "",
    ironwood: trackers.ironwood || ""
  });
}

function notePoolTag(note) {
  const p = note?.pool;
  return p === "ironwood" ? "ironwood" : "orchard";
}

function recomputePoolBalances(state) {
  let orchard = 0;
  let ironwood = 0;
  const sapling = Number(state.saplingBalanceZats ?? 0);
  for (const row of state.discoveredNotes ?? []) {
    const note = row?.note ?? row;
    const v = Number(row?.value ?? note?.value ?? 0);
    if (!Number.isFinite(v) || v <= 0) continue;
    if (notePoolTag(note) === "ironwood") ironwood += v;
    else orchard += v;
  }
  state.orchardBalanceZats = orchard;
  state.ironwoodBalanceZats = ironwood;
  state.totalBalanceZats = orchard + ironwood + sapling;
}

async function initShieldedTrackerState(startHeight, rpcEndpoint) {
  let orchardFinal = "";
  let ironwoodFinal = "";
  if (startHeight > 0) {
    const ts = rpcEndpoint
      ? await _inlineRpcRequest(rpcEndpoint, "z_gettreestate", [String(startHeight - 1)])
      : await rpcCall("z_gettreestate", [String(startHeight - 1)]);
    orchardFinal = poolFinalStateFromTreestate(ts, "orchard");
    ironwoodFinal = poolFinalStateFromTreestate(ts, "ironwood");
  }
  if (typeof wasm.shielded_scan_tracker_new === "function") {
    return wasm.shielded_scan_tracker_new(orchardFinal, ironwoodFinal);
  }
  return serializeShieldedTrackerState({
    orchard: wasm.orchard_scan_tracker_new(orchardFinal),
    ironwood: wasm.orchard_scan_tracker_new(ironwoodFinal)
  });
}

function applyShieldedScanBlock(trackerJson, mnemonic, address, height, blockJson) {
  const out = wasm.orchard_scan_tracker_apply_block(
    trackerJson,
    mnemonic,
    address,
    height,
    blockJson
  );
  const nextTracker =
    out?.tracker_state ??
    out?.trackerState ??
    (out?.orchard_tracker_state && out?.ironwood_tracker_state
      ? serializeShieldedTrackerState({
          orchard: out.orchard_tracker_state,
          ironwood: out.ironwood_tracker_state
        })
      : null);
  return { out, nextTracker };
}

async function refreshSaplingBalanceInScanState(state, opts = {}) {
  const force = opts.force === true;
  const now = nowMs();
  if (
    !force &&
    typeof state.saplingPipelineAt === "number" &&
    now - state.saplingPipelineAt < SAPLING_REFRESH_MS
  ) {
    return false;
  }
  try {
    const base = await loadCompanionBaseUrl();
    const status = await companionSaplingStatus(base);
    const zats = Number(status?.unspent_zatoshis ?? status?.unspentZatoshis ?? 0);
    if (Number.isFinite(zats) && zats >= 0) {
      state.saplingBalanceZats = zats;
      state.saplingPipelineAt = now;
      recomputePoolBalances(state);
      return true;
    }
  } catch (_) {
    /* companion optional */
  }
  return false;
}

let saplingPipelineRunning = false;

/** LWD compact sync + Sapling scan via local nozywallet-api (best-effort, non-blocking). */
async function runCompanionSaplingPipeline(opts = {}) {
  if (saplingPipelineRunning) return { ok: false, reason: "busy" };
  saplingPipelineRunning = true;
  try {
    const prefs = await loadCompanionPrefs();
    await companionStatus(prefs.baseUrl);

    const lwdBody = {};
    if (prefs.lightwalletdUrl) lwdBody.lightwalletd_url = prefs.lightwalletdUrl;

    try {
      await companionLwdSyncCompactToTip(prefs.baseUrl, lwdBody);
    } catch (_) {
      /* LWD offline — scan may still work on cached compact blocks */
    }

    const password = String(opts.password ?? opts.companionPassword ?? "");
    try {
      await companionSaplingScan(prefs.baseUrl, { password, full: false });
    } catch (_) {
      /* companion wallet may not be loaded with the same seed */
    }

    const state = await loadScanState();
    if (state) {
      await refreshSaplingBalanceInScanState(state, { force: true });
      state.saplingPipelineAt = nowMs();
      await saveScanState(state);
    }
    return { ok: true };
  } catch (e) {
    return { ok: false, reason: e instanceof Error ? e.message : String(e) };
  } finally {
    saplingPipelineRunning = false;
  }
}

function kickCompanionSaplingPipeline(opts) {
  void runCompanionSaplingPipeline(opts).catch(() => undefined);
}
const SCAN_ALARM = "nozy_scan_tick";
const SCAN_MODE_MANUAL = "manual";
const SCAN_MODE_AUTO = "auto";
/** Auto mode rewind window when prior scan found no notes (safety for missed receives near tip). */
const AUTO_SYNC_RESCAN_OVERLAP_BLOCKS = 100;
/** Session-only marker that a background scan needs an unlocked RAM mnemonic (no seed persisted). */
const SCAN_RESUME_SESSION_KEY = "nozy_scan_resume_wallet_v1";

function scanResumeSessionApi() {
  return typeof chrome !== "undefined" && chrome.storage && chrome.storage.session
    ? chrome.storage.session
    : null;
}

async function persistScanResumeForBackground(_mnemonic, address) {
  const api = scanResumeSessionApi();
  // Never persist mnemonic to session storage — only the UA so UI can show context.
  // Background scan after SW restart requires the user to unlock again (mnemonic stays in RAM only).
  if (!api || !address) return;
  await new Promise((resolve, reject) => {
    api.set(
      { [SCAN_RESUME_SESSION_KEY]: { address, mnemonicPersisted: false } },
      () => {
        if (chrome.runtime.lastError) reject(chrome.runtime.lastError);
        else resolve();
      }
    );
  }).catch(() => {});
}

async function readScanResumeForBackground() {
  const api = scanResumeSessionApi();
  if (!api) return null;
  return new Promise((resolve) => {
    api.get([SCAN_RESUME_SESSION_KEY], (v) => {
      const row = v?.[SCAN_RESUME_SESSION_KEY];
      if (row && typeof row === "object" && typeof row.address === "string") {
        resolve({ address: row.address, mnemonic: null });
      } else resolve(null);
    });
  });
}

function clearScanResumeForBackground() {
  const api = scanResumeSessionApi();
  if (!api) return;
  try {
    api.remove([SCAN_RESUME_SESSION_KEY], () => {});
  } catch (_) {
    /* ignore */
  }
}

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

  scanRunning = true;
  try {
    await ensureSessionInitialized();
    if (session.unlocked) touchSession();
    const resume = await readScanResumeForBackground();
    const mnemonicForScan = session.mnemonic || null;
    const addressForScan = session.address || resume?.address || null;
    if (!mnemonicForScan || !addressForScan) {
      // Session was not hydrated (browser was restarted and no session key).
      // Keep status = "scanning" so the alarm keeps firing; each tick tries
      // ensureSessionInitialized() which will auto-resume once unlocked.
      // Only hard-fail after 10 minutes of waiting so the user gets a clear message.
      const waitingSince = state.sessionWaitingSince || nowMs();
      state.sessionWaitingSince = waitingSince;
      const waitedMs = nowMs() - waitingSince;
      if (waitedMs > 10 * 60 * 1000) {
        state.status = "failed";
        state.scanError =
          "Sync paused: unlock the wallet in the popup to resume. (Session expired after browser restart.)";
        state.finishedAt = nowMs();
        clearScanResumeForBackground();
      }
      await saveScanState(state);
      scheduleScanAlarm(0.5);
      return;
    }
    // Clear waiting marker once session is hydrated.
    if (state.sessionWaitingSince) {
      delete state.sessionWaitingSince;
    }

    await ensureWasm();
    if (
      typeof wasm.orchard_scan_tracker_new !== "function" ||
      typeof wasm.orchard_scan_tracker_apply_block !== "function"
    ) {
      state.status = "failed";
      state.scanError =
        "WASM bundle is missing orchard_scan_tracker_* exports. Rebuild browser-extension wasm-core (wasm-pack) and reload the extension.";
      state.finishedAt = nowMs();
      clearScanResumeForBackground();
      await saveScanState(state);
      return;
    }
    if (state.scanMode === SCAN_MODE_AUTO) {
      try {
        const latestTip = Number(
          await rpcCallWithRetry("getblockcount", [], { retries: 1, baseDelayMs: 150 })
        );
        if (Number.isFinite(latestTip)) {
          state.endHeight = Math.max(state.endHeight ?? 0, Math.floor(latestTip));
        }
      } catch (_) {
        // Ignore transient tip lookup errors; block fetch path records actual RPC errors.
      }
    }
    const batchSize =
      state.currentHeight === state.startHeight && SCAN_FIRST_BATCH < SCAN_BATCH
        ? SCAN_FIRST_BATCH
        : SCAN_BATCH;
    const end = Math.min(state.currentHeight + batchSize - 1, state.endHeight);

    // Legacy scans used scan_orchard_actions (no incremental witness). Reset and rescan from start.
    if (!state.trackerState) {
      state.discoveredNotes = [];
      state.totalBalanceZats = 0;
      state.orchardBalanceZats = 0;
      state.ironwoodBalanceZats = 0;
      state.saplingBalanceZats = state.saplingBalanceZats ?? 0;
      state.currentHeight = state.startHeight;
      state.scannedBlocks = 0;
      state.heightProgress = state.startHeight - 1;
      state.consecutiveFailures = 0;
      state.lastRpcError = null;
    }

    let trackerState = state.trackerState;
    if (!trackerState || (typeof trackerState === "string" && !trackerState.trim())) {
      trackerState = await initShieldedTrackerState(state.startHeight);
      state.trackerState = trackerState;
      state.updatedAt = nowMs();
      await saveScanState(state);
    }

    let blocksSinceProgressSave = 0;
    const loopStart = state.currentHeight;
    const failLimit = 30;
    if (
      (state.consecutiveFailures ?? 0) > 0 &&
      typeof state.lastRpcError === "string" &&
      /DOCTYPE|web page|not JSON|127\.0\.0\.1/i.test(state.lastRpcError)
    ) {
      try {
        const fixed = await ensureReachableZebradRpc();
        state.rpcEndpoint = fixed;
        state.consecutiveFailures = 0;
        state.lastRpcError = null;
        await saveScanState(state);
      } catch {
        // keep failing with existing errors
      }
    }

    const rpcEndpoint = scanRpcEndpoint(state);
    for (let batchStart = loopStart; batchStart <= end; batchStart += SCAN_PARALLEL_FETCH) {
      const batchEnd = Math.min(batchStart + SCAN_PARALLEL_FETCH - 1, end);
      const heights = [];
      for (let h = batchStart; h <= batchEnd; h += 1) heights.push(h);
      const blocks = await Promise.all(
        heights.map((h) =>
          rpcGetBlockVerboseByHeight(rpcEndpoint, h).catch(() => null)
        )
      );
      for (let i = 0; i < heights.length; i += 1) {
        const h = heights[i];
        const block = blocks[i];
        try {
          if (block) {
            const blockJson = JSON.stringify(block);
            const { out, nextTracker } = applyShieldedScanBlock(
              trackerState,
              mnemonicForScan,
              addressForScan,
              h,
              blockJson
            );
            if (nextTracker) {
              trackerState = nextTracker;
              state.trackerState = trackerState;
            }
            state.consecutiveFailures = 0;
            if (out.notes?.length) {
              for (const n of out.notes) {
                const v = Number(n?.value ?? 0);
                if (!Number.isFinite(v) || v <= 0) continue;
                const txid = String(n?.txid ?? block?.hash ?? `h${h}`);
                state.discoveredNotes.push({
                  note: n,
                  height: h,
                  txid,
                  value: v,
                  pool: notePoolTag(n)
                });
              }
              recomputePoolBalances(state);
            }
          } else {
            state.consecutiveFailures = (state.consecutiveFailures || 0) + 1;
            state.lastRpcError = "getblock returned empty";
            if (state.consecutiveFailures >= failLimit) {
              state.status = "failed";
              state.scanError = `Scan stopped: ${failLimit} consecutive block fetch failures. Check RPC URL and Zebrad (see Settings). Last: ${state.lastRpcError}`;
              state.finishedAt = nowMs();
              clearScanResumeForBackground();
              await saveScanState(state);
              return;
            }
          }
        } catch (e) {
          const msg = e instanceof Error ? e.message : String(e);
          state.consecutiveFailures = (state.consecutiveFailures || 0) + 1;
          state.lastRpcError = msg.slice(0, 400);
          if (state.consecutiveFailures >= failLimit) {
            state.status = "failed";
            state.scanError = `Scan stopped: ${failLimit} consecutive errors. Check RPC URL / cookie auth / network. Last: ${msg.slice(0, 220)}`;
            state.finishedAt = nowMs();
            clearScanResumeForBackground();
            await saveScanState(state);
            return;
          }
        } finally {
          state.heightProgress = h;
          state.scannedBlocks = h - state.startHeight + 1;
          state.updatedAt = nowMs();
          blocksSinceProgressSave += 1;
          const pctInt = scanPercentInt(state);
          const crossedPercent = pctInt > (state.lastSavedPercentInt ?? -1);
          if (blocksSinceProgressSave >= SCAN_SAVE_EVERY_BLOCKS || crossedPercent) {
            blocksSinceProgressSave = 0;
            if (crossedPercent) state.lastSavedPercentInt = pctInt;
            await saveScanState(state);
          }
        }
      }
    }

    state.currentHeight = end + 1;
    state.updatedAt = nowMs();

    if (state.currentHeight > state.endHeight) {
      if (state.scanMode === SCAN_MODE_AUTO) {
        // Stay active at chain tip so newly arrived notes are discovered automatically.
        state.currentHeight = state.endHeight + 1;
      } else {
        state.status = "done";
        state.finishedAt = nowMs();
        clearScanResumeForBackground();
      }
    }
    await refreshSaplingBalanceInScanState(state, {
      force: state.status === "done"
    });
    await saveScanState(state);
  } finally {
    scanRunning = false;
  }

  if (state.status === "scanning") {
    const hasMoreBlocks = (state.currentHeight ?? 0) <= (state.endHeight ?? -1);
    if (hasMoreBlocks) {
    
      scheduleScanAlarm(0.5);
      
      setTimeout(() => {
        void scanTick();
      }, 20);
    } else {
    
      scheduleScanAlarm(0.5);
    }
  }
}

async function startBackgroundScan(startHeight, endHeight, opts = {}) {
  // User explicitly started a range (window / custom / birthday). Always replace any
  // in-progress job — otherwise "Last 500" is ignored while auto-sync is still "scanning".
  if (session.unlocked && session.mnemonic && session.address) {
    await persistScanResumeForBackground(session.mnemonic, session.address);
  }
  const state = {
    status: "scanning",
    scanMode: SCAN_MODE_MANUAL,
    startHeight,
    endHeight,
    currentHeight: startHeight,
    scannedBlocks: 0,
    heightProgress: startHeight - 1,
    lastSavedPercentInt: -1,
    rpcEndpoint: scanRpcEndpoint({}),
    discoveredNotes: [],
    totalBalanceZats: 0,
    orchardBalanceZats: 0,
    ironwoodBalanceZats: 0,
    saplingBalanceZats: 0,
    trackerState: "",
    startedAt: nowMs(),
    updatedAt: nowMs(),
    finishedAt: null,
    consecutiveFailures: 0,
    lastRpcError: null
  };
  await saveScanState(state);
  scheduleScanAlarm(0.02);
  void scanTick();
  kickCompanionSaplingPipeline({
    companionPassword: opts?.companionPassword ?? opts?.password ?? ""
  });
  return state;
}

async function startAutoBackgroundScan() {
  if (!session.unlocked || !session.mnemonic || !session.address) return null;
  await persistScanResumeForBackground(session.mnemonic, session.address);

  const tip = Number(await rpcCallWithRetry("getblockcount", [], { retries: 1, baseDelayMs: 200 }));
  const chainTip = Number.isFinite(tip) ? Math.max(0, Math.floor(tip)) : 0;
  const existing = await loadScanState();
  const now = nowMs();
  const ws = await loadWalletState();
  const resolved = await resolveScanBirthday(ws, chainTip, existing);

  if (existing && existing.status === "scanning" && !resolved.rewound) {
    existing.scanMode = SCAN_MODE_AUTO;
    existing.endHeight = Math.max(existing.endHeight ?? 0, chainTip);
    existing.updatedAt = now;
    await saveScanState(existing);
    scheduleScanAlarm(0.02);
    void scanTick();
    return existing;
  }

  let startHeight = chainTip;
  const priorDone =
    existing &&
    (existing.status === "done" || existing.status === "stopped" || existing.status === "failed");
  let resumedFrom = chainTip;
  let rewoundForSafety = false;

  if (resolved.rewound) {
    startHeight = resolved.birthday;
    resumedFrom = startHeight;
    rewoundForSafety = true;
  } else if (priorDone && typeof existing.heightProgress === "number" && existing.heightProgress >= 0) {
    resumedFrom = Math.min(chainTip, Math.max(0, Math.floor(existing.heightProgress) + 1));
    startHeight = resumedFrom;
  } else if (priorDone && typeof existing.currentHeight === "number" && existing.currentHeight >= 0) {
    resumedFrom = Math.min(chainTip, Math.max(0, Math.floor(existing.currentHeight)));
    startHeight = resumedFrom;
  } else {
    startHeight = Math.min(chainTip, resolved.birthday);
    resumedFrom = startHeight;
  }

  
  const priorNoteCount = Array.isArray(existing?.discoveredNotes) ? existing.discoveredNotes.length : 0;
  if (priorDone && priorNoteCount === 0) {
    const rewound = Math.max(0, startHeight - AUTO_SYNC_RESCAN_OVERLAP_BLOCKS);
    if (rewound < startHeight) {
      startHeight = rewound;
      rewoundForSafety = true;
    }
  }

  const keepHistory = priorDone && Array.isArray(existing?.discoveredNotes) && !rewoundForSafety;
  const state = {
    status: "scanning",
    scanMode: SCAN_MODE_AUTO,
    startHeight,
    endHeight: chainTip,
    currentHeight: startHeight,
    scannedBlocks: 0,
    heightProgress: startHeight - 1,
    discoveredNotes: keepHistory ? existing.discoveredNotes : [],
    totalBalanceZats: keepHistory ? Number(existing.totalBalanceZats ?? 0) : 0,
    orchardBalanceZats: keepHistory ? Number(existing.orchardBalanceZats ?? 0) : 0,
    ironwoodBalanceZats: keepHistory ? Number(existing.ironwoodBalanceZats ?? 0) : 0,
    saplingBalanceZats: keepHistory ? Number(existing.saplingBalanceZats ?? 0) : 0,
    trackerState: keepHistory && typeof existing.trackerState === "string" ? existing.trackerState : "",
    startedAt: now,
    updatedAt: now,
    finishedAt: null,
    consecutiveFailures: 0,
    lastRpcError: null,
    scanError: null,
    lastSavedPercentInt: -1,
    rpcEndpoint: scanRpcEndpoint({})
  };
  await saveScanState(state);
  scheduleScanAlarm(0.02);
  void scanTick();
  kickCompanionSaplingPipeline({});
  return state;
}

async function resumeBackgroundScanAfterUnlock() {
  const s = await loadScanState();
  if (!s || s.status !== "scanning") {
    await startAutoBackgroundScan();
    return;
  }
  s.scanMode = s.scanMode || SCAN_MODE_AUTO;
  s.updatedAt = nowMs();
  await saveScanState(s);
  scheduleScanAlarm(0.02);
  void scanTick();
}

function stopBackgroundScan() {
  chrome.alarms.clear(SCAN_ALARM);
  return loadScanState().then((s) => {
    if (s && s.status === "scanning") {
      s.status = "stopped";
      s.finishedAt = nowMs();
      return saveScanState(s).then(() => {
        clearScanResumeForBackground();
        return s;
      });
    }
    return s;
  });
}

chrome.alarms.onAlarm.addListener((alarm) => {
  if (alarm.name === SCAN_ALARM) scanTick();
});

chrome.runtime.onInstalled.addListener((details) => {
  if (details?.reason !== "update") return;
  chrome.alarms.clear(SCAN_ALARM);
  clearScanResumeForBackground();
  loadScanState()
    .then((s) => {
      if (!s || s.status !== "scanning") return;
      s.status = "stopped";
      s.finishedAt = nowMs();
      s.scanError = null;
      s.lastRpcError = null;
      s.updatedAt = nowMs();
      return saveScanState(s);
    })
    .catch(() => undefined);
});

loadScanState().then((s) => {
  if (s && s.status === "scanning") {
    scheduleScanAlarm(0.01);
    void scanTick();
  }
});

chrome.runtime.onMessage.addListener((msg, sender, sendResponse) => {
  if (!msg || msg.type !== "NOZY_REQUEST") return;

  (async () => {
    try {
      validateRequestEnvelope(msg);
      assertMethodAllowedForSender(msg.method, sender);
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
          sendResponse(
            ok(await walletRestore(params.mnemonic, params.password, { birthdayHeight: params.birthdayHeight }))
          );
          return;
        case "wallet_reset":
          sendResponse(ok(await walletReset()));
          return;
        case "wallet_unlock":
          sendResponse(ok(await walletUnlock(params.password)));
          return;
        case "wallet_lock":
          sendResponse(ok(await walletLock()));
          return;
        case "wallet_status":
          sendResponse(ok(await getWalletStatus()));
          return;
        case "companion_status":
          sendResponse(ok(await companionStatus(params.baseUrl)));
          return;
        case "companion_address_generate":
          sendResponse(
            ok(await companionGenerateAddress(params.baseUrl, params.password))
          );
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
                db_path: params.db_path,
                resume: params.resume === true
              })
            )
          );
          return;
        case "companion_lwd_sync_compact_to_tip":
          sendResponse(
            ok(
              await companionLwdSyncCompactToTip(params.baseUrl, {
                lightwalletd_url: params.lightwalletd_url,
                db_path: params.db_path,
                start_floor:
                  params.start_floor !== undefined && params.start_floor !== null
                    ? Number(params.start_floor)
                    : undefined,
                persist_progress_every:
                  params.persist_progress_every !== undefined &&
                  params.persist_progress_every !== null
                    ? Number(params.persist_progress_every)
                    : undefined
              })
            )
          );
          return;
        case "companion_sapling_status":
          sendResponse(ok(await companionSaplingStatus(params.baseUrl)));
          return;
        case "companion_sapling_scan":
          sendResponse(
            ok(
              await companionSaplingScan(params.baseUrl, {
                password: params.password,
                start_floor: params.start_floor,
                full: params.full === true
              })
            )
          );
          return;
        case "companion_sapling_shield":
          sendResponse(
            ok(
              await companionSaplingShield(params.baseUrl, {
                password: params.password,
                dry_run: params.dry_run === true,
                no_broadcast: params.no_broadcast === true
              })
            )
          );
          return;
        case "companion_vote_status":
          sendResponse(ok(await companionVoteStatus(params.baseUrl, params.env)));
          return;
        case "companion_vote_active":
          sendResponse(ok(await companionVoteActive(params.baseUrl, params.env)));
          return;
        case "companion_vote_export_notes":
          sendResponse(
            ok(
              await companionVoteExportNotes(params.baseUrl, {
                password: params.password,
                env: params.env
              })
            )
          );
          return;
        case "wallet_vote_export_notes":
          sendResponse(ok(await walletVoteExportNotes()));
          return;
        case "companion_vote_import_notes":
          sendResponse(
            ok(
              await companionVoteImportNotes(params.baseUrl, {
                notes_json: params.notes_json
              })
            )
          );
          return;
        case "companion_vote_signing_request":
          sendResponse(ok(await companionVoteSigningRequest(params.baseUrl)));
          return;
        case "wallet_vote_sign_delegation":
          sendResponse(
            ok({
              sig_json: await walletVoteSignDelegation(params.request_json ?? params.requestJson)
            })
          );
          return;
        case "companion_vote_submit_delegation_sig":
          sendResponse(
            ok(
              await companionVoteSubmitDelegationSig(params.baseUrl, {
                sig_json: params.sig_json,
                env: params.env
              })
            )
          );
          return;
        case "companion_vote_prepare":
          sendResponse(
            ok(await companionVotePrepare(params.baseUrl, { env: params.env }))
          );
          return;
        case "companion_vote_delegate":
          sendResponse(
            ok(await companionVoteDelegate(params.baseUrl, { env: params.env }))
          );
          return;
        case "companion_vote_sign_delegation":
          sendResponse(
            ok(
              await companionVoteSignDelegation(params.baseUrl, {
                password: params.password,
                env: params.env
              })
            )
          );
          return;
        case "companion_vote_delegate_finish":
          sendResponse(
            ok(
              await companionVoteDelegateFinish(params.baseUrl, {
                env: params.env,
                wait: params.wait !== false
              })
            )
          );
          return;
        case "companion_vote_cast":
          sendResponse(
            ok(
              await companionVoteCast(params.baseUrl, {
                env: params.env,
                choices: params.choices ?? {},
                delegation_tx: params.delegation_tx,
                single_share: params.single_share === true,
                wait: params.wait !== false
              })
            )
          );
          return;
        case "companion_crosslink_status":
          sendResponse(ok(await companionCrosslinkStatus(params.baseUrl)));
          return;
        case "companion_crosslink_positions":
          sendResponse(ok(await companionCrosslinkPositions(params.baseUrl)));
          return;
        case "companion_crosslink_roster":
          sendResponse(
            ok(await companionCrosslinkRoster(params.baseUrl, params.zats === true))
          );
          return;
        case "companion_crosslink_stake":
          sendResponse(
            ok(
              await companionCrosslinkStake(params.baseUrl, {
                amount_ctaz: params.amount_ctaz,
                finalizer: params.finalizer,
                force: params.force === true
              })
            )
          );
          return;
        case "companion_crosslink_retarget":
          sendResponse(
            ok(
              await companionCrosslinkRetarget(params.baseUrl, {
                bond: params.bond,
                finalizer: params.finalizer
              })
            )
          );
          return;
        case "companion_crosslink_unbond":
          sendResponse(
            ok(
              await companionCrosslinkUnbond(params.baseUrl, {
                bond: params.bond,
                force: params.force === true
              })
            )
          );
          return;
        case "companion_crosslink_withdraw":
          sendResponse(
            ok(
              await companionCrosslinkWithdraw(params.baseUrl, {
                bond: params.bond,
                force: params.force === true
              })
            )
          );
          return;
        case "companion_crosslink_wallet_status":
          sendResponse(ok(await companionCrosslinkWalletStatus(params.baseUrl)));
          return;
        case "companion_crosslink_wallet_ufvk":
          sendResponse(ok(await companionCrosslinkWalletUfvk(params.baseUrl)));
          return;
        case "companion_zns_resolve":
          sendResponse(
            ok(
              await companionZnsResolve(params.baseUrl, {
                name: params.name,
                network: params.network
              })
            )
          );
          return;
        case "companion_send_egress":
          sendResponse(ok(await companionSendEgress(params.baseUrl)));
          return;
        case "companion_privacy_network":
          sendResponse(ok(await companionPrivacyNetwork(params.baseUrl)));
          return;
        case "companion_set_privacy_network":
          sendResponse(ok(await companionSetPrivacyNetwork(params.baseUrl, params.patch ?? {})));
          return;
        case "companion_nym_mixnet":
          sendResponse(ok(await companionNymMixnet(params.baseUrl)));
          return;
        case "companion_nym_dvpn":
          sendResponse(ok(await companionNymDvpn(params.baseUrl, params.lightwalletd_url)));
          return;
        case "companion_set_nym_dvpn":
          sendResponse(ok(await companionSetNymDvpn(params.baseUrl, params.enabled === true)));
          return;
        case "companion_nym_dvpn_probe":
          sendResponse(
            ok(
              await companionNymDvpnProbe(params.baseUrl, {
                lightwalletd_url: params.lightwalletd_url,
                blocks: params.blocks !== undefined && params.blocks !== null ? Number(params.blocks) : undefined
              })
            )
          );
          return;
        case "companion_nym_vpn_app":
          sendResponse(ok(await companionNymVpnApp(params.baseUrl)));
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
          await refreshTxExpiryStates();
          sendResponse(ok(await loadTxState()));
          return;
        case "wallet_retry_broadcast":
          sendResponse(ok({ txid: await retryBroadcastById(String(params.id ?? "")) }));
          return;
        case "wallet_speed_up":
          sendResponse(
            ok({
              txid: await speedUpTxById(String(params.id ?? ""), {
                companionPassword: params.companionPassword,
                allowWasmFallback: params.allowWasmFallback !== false
              })
            })
          );
          return;
        case "companion_check_confirmations":
          sendResponse(ok(await companionCheckConfirmations(params.baseUrl)));
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

                  const txid = await broadcastRawHex(proving.rawTxHex, {
                    retries: 3,
                    baseDelayMs: 400
                  });
                  const chainTip = Number(await rpcCall("getblockcount", []));
                  const expiryHeight = await pilotExpiryHeightForTip(chainTip);
                  await patchTxStateById(txStateId, {
                    txid: String(txid),
                    state: "broadcast",
                    error: null,
                    expiryHeight
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
          await rememberRpcEndpoint(session.rpcEndpoint);
          {
            const existing = (await loadWalletState()) || {};
            await saveWalletState({ ...existing, rpcEndpoint: session.rpcEndpoint });
          }
          sendResponse(ok({ rpcEndpoint: session.rpcEndpoint }));
          return;
        }
        case "rpc_autodetect": {
          const found = await autodetectZebradRpcEndpoint();
          const blockCount = await readRpcBlockCount();
          sendResponse(
            ok({
              rpcEndpoint: found,
              blockCount
            })
          );
          return;
        }
        case "rpc_connect": {
          const res = await connectZebradRpc({
            url: params?.url,
            tryCompanion: params?.tryCompanion
          });
          sendResponse(ok(res));
          return;
        }
        case "rpc_probe_endpoint": {
          const url = normalizeRpcEndpoint(String(params?.url ?? session.rpcEndpoint));
          const okProbe = await probeZebradRpcEndpoint(url, 4000);
          if (!okProbe) {
            throw new Error(
              `Zebrad not reachable at ${url}. If zebrad runs in WSL, use your WSL IP instead of 127.0.0.1.`
            );
          }
          sendResponse(ok({ endpoint: url, connected: true }));
          return;
        }
        case "rpc_get_status": {
          let connected = false;
          let blockCount = null;
          try {
            const raw = await rpcCallWithRetry("getblockcount", [], { retries: 1 });
            const n = typeof raw === "number" ? raw : Number(raw);
            if (Number.isFinite(n) && n >= 0) {
              connected = true;
              blockCount = Math.floor(n);
            }
          } catch (_) {
            connected = false;
          }
          sendResponse(
            ok({
              endpoint: session.rpcEndpoint,
              connected,
              blockCount
            })
          );
          return;
        }
        case "rpc_get_block_count":
          sendResponse(ok(await rpcCallWithRetry("getblockcount", [])));
          return;
        case "rpc_get_block": {
          const bh = Number(params?.height ?? 0);
          sendResponse(ok(await rpcGetBlockVerboseByHeight(session.rpcEndpoint, bh)));
          return;
        }
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
          const txid = await broadcastRawHex(hex, { retries: 2, baseDelayMs: 500 });
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
          await persistScanResumeForBackground(session.mnemonic, session.address);
          const rpcUrl = await ensureReachableZebradRpc();
          const blockCount = await rpcCallWithRetry("getblockcount", []);

          const rawEnd = params?.endHeight;
          let endH = blockCount;
          if (rawEnd !== undefined && rawEnd !== null && rawEnd !== "") {
            const n = Number(rawEnd);
            if (Number.isFinite(n)) {
              endH = Math.max(0, Math.min(Math.floor(n), blockCount));
            }
          }

          let startH;
          if (params.useBirthdayRange === true) {
            const ws = await loadWalletState();
            const existingScan = await loadScanState();
            const blockTip = Number(endH);
            const resolved = await resolveScanBirthday(ws, blockTip, existingScan);
            let birthdayStart = resolved.birthday;
            startH = birthdayStart;
            const priorScanStart =
              typeof existingScan?.startHeight === "number"
                ? Math.floor(existingScan.startHeight)
                : birthdayStart;
            const birthdayWasLowered = resolved.rewound || birthdayStart < priorScanStart;
            if (
              !birthdayWasLowered &&
              existingScan &&
              (existingScan.status === "done" ||
                existingScan.status === "stopped" ||
                existingScan.status === "failed") &&
              typeof existingScan.heightProgress === "number" &&
              existingScan.heightProgress >= birthdayStart - 1
            ) {
              startH = Math.min(endH, Math.floor(existingScan.heightProgress) + 1);
            }
            if (startH > endH) {
              startH = endH;
            }
          } else {
            const rawStart = params?.startHeight;
            if (rawStart !== undefined && rawStart !== null && rawStart !== "") {
              const n = Number(rawStart);
              if (!Number.isFinite(n)) {
                throw new Error("Invalid startHeight");
              }
              startH = Math.max(0, Math.floor(n));
            } else {
              const scanWindow = Number(params?.window ?? 20_000);
              const w = Math.max(1, scanWindow);
              startH = Math.max(0, blockCount - w);
            }
          }

          if (startH > endH) {
            throw new Error(
              `startHeight (${startH}) must be ≤ endHeight (${endH}). Refresh chain tip or adjust the range.`
            );
          }

          const s = await startBackgroundScan(startH, endH, {
            companionPassword: params?.companionPassword ?? params?.password ?? ""
          });
          sendResponse(
            ok({
              started: true,
              startHeight: startH,
              endHeight: endH,
              status: s.status,
              rpcEndpoint: rpcUrl
            })
          );
          return;
        }
        case "wallet_set_birthday_height": {
          if (!session.unlocked) throw new Error("Unlock wallet first.");
          const existing = (await loadWalletState()) || {};
          if (!existing.encryptedMnemonic) throw new Error("No wallet found.");
          const n = Number(params?.height);
          if (!Number.isFinite(n) || n < 0) {
            throw new Error("height must be a non-negative integer (Orchard scan birthday).");
          }
          const orchardBirthdayHeight = Math.floor(n);
          const prevBh = Number(existing.orchardBirthdayHeight);
          await saveWalletState({ ...existing, orchardBirthdayHeight });
          // Lower birthday requires rescanning older blocks; clear stale "done" scan metadata.
          if (Number.isFinite(prevBh) && orchardBirthdayHeight < prevBh) {
            const scan = await loadScanState();
            if (scan && (scan.status === "done" || scan.status === "stopped" || scan.status === "failed")) {
              await chrome.storage.local.remove(SCAN_STATE_KEY);
            }
          }
          sendResponse(ok({ orchardBirthdayHeight }));
          return;
        }
        case "wallet_scan_progress": {
          const scanState = await loadScanState();
          if (!scanState) {
            sendResponse(ok({ status: "idle" }));
          } else {
            const total = Math.max(1, scanState.endHeight - scanState.startHeight + 1);
            let done;
            if (typeof scanState.heightProgress === "number") {
              done = Math.min(
                total,
                Math.max(0, scanState.heightProgress - scanState.startHeight + 1)
              );
            } else {
              done = Math.min(total, scanState.scannedBlocks ?? 0);
            }
            // 4 decimals: 2 collapses to 0.00 on million-block scans and the UI reads as stuck.
            const pct = Number(((done / total) * 100).toFixed(4));
            const percentInt = Math.min(100, Math.max(0, Math.floor(pct)));
            sendResponse(ok({
              status: scanState.status,
              startHeight: scanState.startHeight,
              endHeight: scanState.endHeight,
              currentHeight: scanState.currentHeight,
              scannedBlocks: done,
              totalBlocks: total,
              percent: pct,
              percentInt,
              discoveredNotes: scanState.discoveredNotes?.length ?? 0,
              totalBalanceZats: scanState.totalBalanceZats ?? 0,
              orchardBalanceZats: scanState.orchardBalanceZats ?? 0,
              ironwoodBalanceZats: scanState.ironwoodBalanceZats ?? 0,
              saplingBalanceZats: scanState.saplingBalanceZats ?? 0,
              scanError: scanState.scanError ?? null,
              lastRpcError: scanState.lastRpcError ?? null,
              consecutiveFailures: scanState.consecutiveFailures ?? 0,
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
        case "wallet_estimate_send_fee": {
          await ensureWasm();
          const memo = String(params?.memo ?? "");
          // Ignore caller priority — mandatory ×4 (same as native surfaces).
          sendResponse(
            ok({
              fee: wasm.estimate_orchard_send_fee_zats(memo, true),
              priority: true,
              expiry_delta_blocks: wasm.pilot_expiry_delta_blocks(),
              core_version: wasm.nozy_version_display()
            })
          );
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
                rpcEndpoint: session.rpcEndpoint,
                // Ignore caller fee — ZIP-317 × 4 is required (same as CLI/API/desktop).
                fee: await estimateFeeZats(String(params?.memo ?? ""), true)
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

