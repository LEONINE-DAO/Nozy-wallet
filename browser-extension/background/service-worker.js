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
  isLightwalletdEndpoint,
  isLocalRpcEndpoint,
  isWslStyleHost,
  normalizeRpcEndpoint,
  probeZebradRpcEndpoint,
  pinRpcEndpoint,
  rpcBlockCountAt,
  rpcCallJson,
  rpcGetBlockVerboseByHeight
} from "./rpc-utils.js";
import {
  compactChainTip,
  compactFetchBlocks,
  compactTreeState
} from "./lwd-compact.js";
import {
  addAccount,
  emptyAccountStore,
  normalizeAccountStore,
  renameAccount,
  setActiveAccount
} from "./wallet-accounts.js";
import {
  feeBlockFromRpc,
  insufficientFundsMessage,
  mandatoryOrchardFeeZats,
  payForInclusionRequested,
  rpcFallbackWithRequester
} from "./tx-utils.js";
import { nullifierHex, resolveSpendWitness, selectNotesSamePool } from "./witness-spend.js";
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
  companionCrosslinkPayoutClaim,
  companionCrosslinkDoctor,
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
  companionNymVpnApp,
  probeLwdMixnetProxy
} from "./companion-api.js";
import { scanStatusDefersAutoLock } from "./session-lock.js";

const STORAGE_KEY = "nozy_wallet_state_v1";
const ACCOUNTS_KEY = "nozy_wallet_accounts_v1";
const RPC_CACHE_KEY = "nozy_zebra_rpc_cache_v1";
const COMPANION_BASE_KEY = "nozy_companion_base_url";
const STORAGE_LWD_URL = "nozy_lightwalletd_url";
const STORAGE_PUBLIC_LWD_OPT_IN = "nozy_public_lwd_opt_in_v1";
const PUBLIC_LWD_URL = "https://lwd.nozywallet.org:443";
const PUBLIC_NODE_STATUS_URL = "https://nozywallet.org/status.json";
const MOBILE_SYNC_KEY = "nozy_mobile_sync_v1";
const TX_STATE_KEY = "nozy_tx_state_v1";
const CONNECTED_ORIGINS_KEY = "nozy_connected_origins_v1";
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
const SESSION_ACTIVITY_KEY = "nozy_session_activity_v1";
const LOCK_ALARM = "nozy-auto-lock";

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
  /** In-memory only: SHA-256 hex of vault password for fast step-up checks (never persisted). */
  passwordFingerprint: null,
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
  const at = nowMs();
  session.lastActivityAt = at;
  const api = sessionStorageApi();
  if (!api) return;
  try {
    api.set({ [SESSION_ACTIVITY_KEY]: at }, () => {});
  } catch (_) {
    /* session storage can be unavailable in tests */
  }
}

async function loadActivityAt() {
  const api = sessionStorageApi();
  if (!api) return session.lastActivityAt || 0;
  return new Promise((resolve) => {
    api.get([SESSION_ACTIVITY_KEY], (v) => {
      const n = Number(v?.[SESSION_ACTIVITY_KEY]);
      resolve(Number.isFinite(n) && n > 0 ? n : 0);
    });
  });
}

function clearSessionActivity() {
  session.lastActivityAt = 0;
  const api = sessionStorageApi();
  if (!api) return;
  try {
    api.remove([SESSION_ACTIVITY_KEY], () => {});
  } catch (_) {
    /* ignore */
  }
}

/** True when the idle timer has elapsed. A missing activity stamp is not idle. */
function idleLockDue() {
  if (!session.lastActivityAt) return false;
  return nowMs() - session.lastActivityAt >= session.autoLockMs;
}

/**
 * Lock when the user has been idle. A catch-up scan defers the lock so the
 * popout and full-page dashboard can sit idle without wiping the seed and
 * stopping sync. Returns true when the caller must not re-hydrate the session.
 */
async function lockIfIdle() {
  if (!idleLockDue()) return false;
  const scan = await loadScanState();
  if (scanStatusDefersAutoLock(scan?.status)) return false;
  await walletLock();
  return true;
}

/** Background polls must not count as the user still being at the keyboard. */
const ACTIVITY_SKIP_METHODS = new Set([
  "wallet_status",
  "wallet_scan_progress",
  "wallet_get_transactions",
  "wallet_get_pending_approvals",
  "rpc_get_status",
  "companion_status",
  "companion_check_confirmations"
]);

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
  "zcash_sendTransaction",
  "zcash_openPayment"
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
    const initPromise = initWasm();
    wasmReady = Promise.race([
      initPromise,
      new Promise((_, reject) => {
        setTimeout(() => reject(new Error("WASM init timed out after 12s")), 12_000);
      })
    ]).then(
      (v) => {
        return v;
      },
      (err) => {
        wasmReady = null;
        throw err;
      }
    );
  }
  await wasmReady;
  return wasm;
}

/** Node/status methods must not wait on WASM — hung init freezes the load page. */
const WASM_OPTIONAL_METHODS = new Set([
  "rpc_get_status",
  "rpc_connect",
  "rpc_autodetect",
  "rpc_probe_endpoint",
  "rpc_set_endpoint",
  "connect_public_lwd",
  "clear_public_lwd",
  "wallet_status",
  "wallet_get_scan_progress",
  "wallet_open_window",
  "rpc_tx_status",
  "wallet_lock",
  "companion_status",
  "zcash_openPayment"
]);

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

/**
 * Normalize a stored scan row to `{ note, height, txid, value }` for spend selection.
 * Supports wrapped rows and flat Orchard decryption payloads.
 */
function normalizeDiscoveredScanEntry(raw) {
  if (!raw || typeof raw !== "object" || raw.spent) return null;
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

async function _inlineRpcRequest(endpoint, method, params = [], opts = {}) {
  const called = await rpcCallJson(endpoint, method, params, opts.exact ? { exact: true } : {});
  if (!opts.exact && called.endpoint && called.endpoint !== session.rpcEndpoint) {
    void persistRpcEndpoint(called.endpoint);
  }
  return called.result;
}

function noteDiscoveryHeight(note) {
  const h = Number(
    note?.block_height ??
      note?.blockHeight ??
      note?.orchard_witness_tip_height ??
      note?.orchardWitnessTipHeight
  );
  return Number.isFinite(h) && h >= 0 ? Math.floor(h) : NaN;
}

const SPEND_REBUILD_KEY = "nozy_spend_rebuild_v1";

function loadSpendRebuild() {
  return new Promise((resolve) => {
    chrome.storage.local.get(SPEND_REBUILD_KEY, (v) => resolve(v[SPEND_REBUILD_KEY] || null));
  });
}

function saveSpendRebuild(checkpoint) {
  return new Promise((resolve) => {
    chrome.storage.local.set({ [SPEND_REBUILD_KEY]: checkpoint }, () => resolve());
  });
}

function clearSpendRebuild() {
  return new Promise((resolve) => {
    chrome.storage.local.remove(SPEND_REBUILD_KEY, () => resolve());
  });
}

/** True while a send is rebuilding a witness, so a 45s scan tick cannot starve that message. */
let spendRebuildActive = false;

/**
 * The saved scan tracker can be an Orchard tree with Ironwood notes appended.
 * That root never matches the node's Ironwood anchor. Rebuild from the node's
 * treestate at the note's block, then apply the blocks after it.
 */
async function rebuildSpendTracker({
  notes,
  chainTip,
  endpoint,
  spendRpc,
  mnemonic,
  address,
  accountIndicesJson
}) {
  const heights = (Array.isArray(notes) ? notes : [])
    .map(noteDiscoveryHeight)
    .filter((h) => Number.isFinite(h));
  const noteHeight = heights.length ? Math.min(...heights) : NaN;
  const gap = chainTip - noteHeight;
  if (!Number.isFinite(noteHeight) || gap < 0 || gap > 25000) {
    return null;
  }
  const saved = await loadSpendRebuild();
  const canResume =
    saved &&
    saved.noteHeight === noteHeight &&
    typeof saved.nextHeight === "number" &&
    saved.nextHeight >= noteHeight &&
    typeof saved.tracker === "string" &&
    saved.tracker.length > 2;
  let tracker;
  let cursor = noteHeight;
  if (canResume && saved.nextHeight > chainTip) {
    await clearSpendRebuild();
    return { done: true, tracker: saved.tracker, nextHeight: saved.nextHeight, blocks: 0 };
  }
  if (canResume) {
    tracker = saved.tracker;
    cursor = Math.floor(saved.nextHeight);
  } else {
    const prior = await spendRpc("z_gettreestate", [String(Math.max(0, noteHeight - 1))]);
    tracker = wasm.shielded_scan_tracker_new(
      poolFinalStateFromTreestate(prior, "orchard"),
      poolFinalStateFromTreestate(prior, "ironwood")
    );
  }
  const applyBlock = (height, block) => {
    const json = compactBlockJsonForScan(block || { tx: [] });
    const { nextTracker } = applyShieldedScanBlock(
      tracker,
      mnemonic,
      address,
      height,
      json,
      accountIndicesJson
    );
    if (nextTracker) tracker = nextTracker;
  };
  spendRebuildActive = true;
  let next = cursor;
  let blocks = 0;
  let compactGot = 0;
  try {
    if (next <= chainTip) {
      const end = Math.min(chainTip, next + 99);
      const compactBlocks = await compactFetchBlocks(next, end);
      compactGot = Array.isArray(compactBlocks) ? compactBlocks.length : 0;
      if (compactGot === 0) {
        throw new Error(`No compact blocks returned for heights ${next}-${end}.`);
      }
      const byH = new Map();
      for (const block of compactBlocks) {
        const hh = Math.floor(Number(block?.height));
        if (Number.isFinite(hh)) byH.set(hh, block);
      }
      const packed = [];
      for (let h = next; h <= end; h += 1) {
        const parsed = JSON.parse(compactBlockJsonForScan(byH.get(h) || { tx: [] }));
        parsed.height = h;
        packed.push(parsed);
      }
      if (typeof wasm.orchard_scan_tracker_apply_blocks !== "function") {
        throw new Error("Reload the extension after the witness rebuild update.");
      }
      const applied = wasmJsonValue(
        wasm.orchard_scan_tracker_apply_blocks(
          tracker,
          mnemonic,
          address,
          JSON.stringify(packed),
          accountIndicesJson
        )
      );
      const nextTracker = applied?.tracker_state ?? applied?.trackerState ?? null;
      if (nextTracker) tracker = nextTracker;
      blocks = end - next + 1;
      next = end + 1;
    }
    const done = next > chainTip;
    if (done) await clearSpendRebuild();
    else {
      await saveSpendRebuild({
        noteHeight,
        nextHeight: next,
        tracker,
        updatedAt: Date.now()
      });
    }
    return { done, tracker, nextHeight: next, blocks };
  } finally {
    spendRebuildActive = false;
  }
}

async function _inlineProveTransaction(params) {
  await ensureWasm();
  const recipientAddress = String(params?.recipientAddress ?? params?.to ?? "");
  const walletAddress = String(params?.walletAddress ?? "");
  const mnemonic = String(params?.mnemonic ?? "");
  const rpcEndpoint = String(params?.rpcEndpoint ?? "");
  const requestedAmount = Number(params?.amount ?? 0);
  const memo = String(params?.memo ?? "");

  if (!rpcEndpoint) throw new Error("Missing rpcEndpoint.");
  if (!mnemonic) throw new Error("Wallet locked.");
  if (!walletAddress) throw new Error("Missing wallet address.");
  if (!recipientAddress) throw new Error("Missing recipient address.");

  const endpoint = normalizeRpcEndpoint(rpcEndpoint);
  pinRpcEndpoint(endpoint);
  const spendRpc = (method, rpcParams = []) =>
    _inlineRpcRequest(endpoint, method, rpcParams, { exact: true });
  const feeBlock = await feeBlockFromRpc(spendRpc);
  const payForInclusion = payForInclusionRequested(params);
  const requestedFee = mandatoryOrchardFeeZats(wasm, memo, feeBlock, payForInclusion);
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
  const selected = selectNotesSamePool(candidates, requiredValue);
  if (selected.length === 0) {
    throw new Error(insufficientFundsMessage(requestedAmount, requestedFee, scannedValue));
  }

  const spendValue = selected.reduce((acc, n) => acc + n.value, 0);
  const chainTip = Number(await spendRpc("getblockcount", []));
  const txBuildHeight = chainTip + 1;
  const scanHeight = Number(scanState?.heightProgress);
  const tsCache = new Map();
  const fetchTreestate = async (h) => {
    const key = Math.floor(Number(h));
    if (tsCache.has(key)) return tsCache.get(key);
    let ts;
    try {
      ts = await spendRpc("z_gettreestate", [String(key)]);
    } catch (err) {
      const msg = String(err?.message ?? err);
      const missing = err?.jsonRpcCode === -32601 || /method not found/i.test(msg);
      if (!missing) throw err;
      ts = await spendRpc("z_getorchardtree", [String(key)]);
    }
    tsCache.set(key, ts);
    return ts;
  };

  const selectedWitnesses = [];
  const witnessFields = (resolved) => ({
    incremental_witness_hex: resolved.witnessHex,
    anchor_hex: resolved.anchorHex,
    target_height: txBuildHeight,
    fee_block_height: feeBlock.minedBlockHeight,
    fee_mainnet: feeBlock.mainnet,
    marginal_fee: feeBlock.marginalFee || 0,
    pay_for_inclusion: payForInclusion
  });
  let witnessesReady = false;
  if (scanState?.trackerState) {
    try {
      const trial = [];
      for (const noteSel of selected) {
        trial.push(
          await resolveSpendWitness({
            wasm,
            note: noteSel?.note,
            noteHeight: noteSel?.height,
            trackerState: scanState.trackerState,
            scanHeight,
            chainTip,
            fetchBlock: (h) => rpcGetBlockVerboseByHeight(endpoint, h),
            fetchTreestate
          })
        );
      }
      for (const resolved of trial) selectedWitnesses.push(witnessFields(resolved));
      witnessesReady = true;
    } catch (err) {
      selectedWitnesses.length = 0;
    }
  }
  if (!witnessesReady) {
  const rebuilt = await rebuildSpendTracker({
    notes: selected.map((s) => s.note),
    chainTip,
    endpoint,
    spendRpc,
    mnemonic,
    address: walletAddress,
    accountIndicesJson: String(params?.accountIndicesJson ?? "[0]")
  });
  if (rebuilt && rebuilt.done === false) {
    return {
      rebuilding: true,
      nextHeight: rebuilt.nextHeight,
      chainTip,
      noteHeight: noteDiscoveryHeight(selected[0]?.note)
    };
  }
  const rebuiltTracker = rebuilt && rebuilt.done ? rebuilt.tracker : "";
  const spendTracker = rebuiltTracker || scanState?.trackerState || "";
  const spendScanHeight = rebuiltTracker ? chainTip : scanHeight;
  for (const noteSel of selected) {
    const resolved = await resolveSpendWitness({
      wasm,
      note: noteSel?.note,
      noteHeight: noteSel?.height,
      trackerState: spendTracker,
      scanHeight: spendScanHeight,
      chainTip,
      fetchBlock: (h) => rpcGetBlockVerboseByHeight(endpoint, h),
      fetchTreestate
    });
    selectedWitnesses.push(witnessFields(resolved));
  }
  if (rebuiltTracker && scanState) {
    scanState.trackerState = rebuiltTracker;
    scanState.heightProgress = chainTip;
    scanState.currentHeight = chainTip + 1;
    scanState.updatedAt = nowMs();
    await saveScanState(scanState);
  }
  }

  const provingResult = wasm.build_orchard_v5_tx_from_note(
    mnemonic,
    recipientAddress,
    BigInt(requestedAmount),
    BigInt(requestedFee),
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
    if (method === "apply_scan_block") {
      return Promise.resolve(
        applyShieldedScanBlock(
          String(params?.trackerState ?? ""),
          String(params?.mnemonic ?? ""),
          String(params?.address ?? ""),
          Number(params?.height ?? 0),
          String(params?.blockJson ?? ""),
          String(params?.accountIndicesJson ?? "[0]")
        )
      );
    }
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

/** Memory-only password fingerprint for step-up UX (reveal seed without a second Argon2). */
async function passwordFingerprint(password) {
  const data = utf8Encode(String(password ?? ""));
  const digest = await crypto.subtle.digest("SHA-256", data);
  return Array.from(new Uint8Array(digest), (b) => b.toString(16).padStart(2, "0")).join("");
}

async function rememberSessionPassword(password) {
  try {
    session.passwordFingerprint = await passwordFingerprint(password);
  } catch (_) {
    session.passwordFingerprint = null;
  }
}

function clearSessionPassword() {
  session.passwordFingerprint = null;
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
  await assertSeedBackupConfirmed();
  const { recipientAddress, amount, memo } = parseTxForOrchardV5(tx);
  const payForInclusion = tx?.priority !== false;
  const rpcEndpoint = await ensureSpendRpcEndpoint();
  const fee = await estimateFeeZats(memo, payForInclusion);
  const proving = await callWorker("prove_transaction", {
    recipientAddress,
    walletAddress: session.address,
    mnemonic: session.mnemonic,
    rpcEndpoint,
    amount,
    fee,
    memo,
    priority: payForInclusion,
    accountIndicesJson: await accountIndicesJson()
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

function readTxConfirmation(resp) {
  const raw = resp?.height ?? resp?.blockheight ?? resp?.blockHeight ?? resp?.block_height ?? null;
  const parsed = typeof raw === "number" ? raw : parseNumberMaybeHex(raw);
  const blockHeight = Number.isFinite(parsed) && parsed > 0 ? Math.floor(parsed) : null;
  const confirmations = Number(resp?.confirmations);
  const confirmed = blockHeight != null || (Number.isFinite(confirmations) && confirmations > 0);
  return {
    confirmed,
    blockHeight,
    confirmations: Number.isFinite(confirmations) ? confirmations : null,
    sawHeight: resp?.height != null
  };
}

async function waitForTxConfirmation({ rpcEndpoint, txid, timeoutMs = 60_000, pollMs = 2_500 }) {
  const startedAt = Date.now();
  while (true) {
    try {
      const resp = await rpcCall("getrawtransaction", [txid, true]);
      const mined = readTxConfirmation(resp);
      if (mined.confirmed) return { confirmed: true, blockHeight: mined.blockHeight };
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
  if (scan?.status === "scanning" || scan?.status === "watching") {
    await stopBackgroundScan();
  }
  clearScanResumeForBackground();
  await storageRemove([SCAN_STATE_KEY, TX_STATE_KEY, CONNECTED_ORIGINS_KEY]);
  await resetAccountStore();
}

/** Remove the Chrome vault so Welcome can restore the Desktop seed. */
async function walletReset() {
  await wipeDerivedWalletData();
  session.unlocked = false;
  session.mnemonic = null;
  session.address = null;
  clearSessionMnemonic();
  clearSessionActivity();
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

function noteCmxHex(note) {
  return _bytesToHex(note?.cmx ?? []).toLowerCase();
}

function txidHexCandidates(txid) {
  const id = String(txid || "").trim().toLowerCase();
  const bytes = id.match(/../g);
  if (!bytes || bytes.length !== 32) return [id];
  const flipped = bytes.reverse().join("");
  return flipped === id ? [id] : [id, flipped];
}

async function backfillScanNoteMemos() {
  if (!session.unlocked || !session.mnemonic) return;
  if (typeof wasm.decrypt_memos_from_raw_tx !== "function") return;
  const state = await loadScanState();
  if (!state || !Array.isArray(state.discoveredNotes) || !state.discoveredNotes.length) return;
  const pending = [];
  const seen = new Set();
  for (const row of state.discoveredNotes) {
    const note = row?.note ?? row;
    if (!note || typeof note.memo === "string") continue;
    const txid = String(row?.txid ?? note?.txid ?? "").trim();
    if (!/^[0-9a-fA-F]{64}$/.test(txid) || seen.has(txid.toLowerCase())) continue;
    seen.add(txid.toLowerCase());
    pending.push(txid);
    if (pending.length >= 4) break;
  }
  if (!pending.length) return;
  let changed = false;
  for (const txid of pending) {
    let found = null;
    for (const candidate of txidHexCandidates(txid)) {
      let rawHex = "";
      try {
        const raw = await rpcCall("getrawtransaction", [candidate]);
        rawHex = typeof raw === "string" ? raw.trim() : "";
      } catch (_) {
        continue;
      }
      if (rawHex.length < 20) continue;
      try {
        found = wasmJsonValue(wasm.decrypt_memos_from_raw_tx(session.mnemonic, rawHex));
      } catch (_) {
        found = null;
        continue;
      }
      if (Array.isArray(found)) break;
    }
    if (!Array.isArray(found) || !found.length) continue;
    for (const row of state.discoveredNotes) {
      const note = row?.note ?? row;
      if (!note || typeof note.memo === "string") continue;
      const rowTx = String(row?.txid ?? note?.txid ?? "").toLowerCase();
      if (rowTx !== txid.toLowerCase()) continue;
      const cmx = noteCmxHex(note);
      const sameValue = found.filter((item) => Number(item?.value) === Number(note.value));
      const hit =
        found.find((item) => String(item?.cmx || "").toLowerCase() === cmx) ||
        (sameValue.length === 1 ? sameValue[0] : null);
      if (!hit) continue;
      const text = typeof hit.memo === "string" ? hit.memo : "";
      if (row.note && typeof row.note === "object") row.note.memo = text;
      else row.memo = text;
      changed = true;
    }
  }
  if (changed) await saveScanState(state);
}

function receivedActivityEntries(scanState) {
  const rows = Array.isArray(scanState?.discoveredNotes) ? scanState.discoveredNotes : [];
  const entries = [];
  rows.forEach((row, i) => {
    const note = row?.note ?? row;
    const amount = Number(row?.value ?? note?.value ?? 0);
    if (!Number.isFinite(amount) || amount <= 0) return;
    const txid = String(row?.txid ?? note?.txid ?? "");
    const height = Number(row?.height ?? note?.block_height ?? 0);
    entries.push({
      id: `received-${txid}-${Number.isFinite(height) ? height : 0}-${i}`,
      txid: txid || null,
      state: "confirmed",
      origin: "received",
      direction: "received",
      recipientAddress: "",
      amount,
      fee: null,
      memo: typeof note?.memo === "string" ? note.memo : "",
      createdAt: Number.isFinite(height) ? height : 0,
      updatedAt: Number.isFinite(height) ? height : 0,
      error: null,
      blockHeight: Number.isFinite(height) ? height : null
    });
  });
  entries.sort((a, b) => (b.blockHeight || 0) - (a.blockHeight || 0));
  return entries;
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
      if (readTxConfirmation(resp).confirmed) continue;
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

  const rpcEndpoint = await ensureSpendRpcEndpoint();
  const priorityFee = await estimateFeeZats(tx.memo || "", true);
  const proving = await callWorker("prove_transaction", {
    recipientAddress: tx.recipientAddress,
    walletAddress: session.address,
    mnemonic: session.mnemonic,
    rpcEndpoint,
    amount: tx.amount,
    fee: priorityFee,
    memo: tx.memo || "",
    priority: true,
    accountIndicesJson: await accountIndicesJson()
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

async function loadAccountStore() {
  return normalizeAccountStore(await storageGet(ACCOUNTS_KEY));
}

async function saveAccountStore(store) {
  const normalized = normalizeAccountStore(store);
  await storageSet({ [ACCOUNTS_KEY]: normalized });
  return normalized;
}

async function resetAccountStore() {
  return saveAccountStore(emptyAccountStore());
}

function activeAccountOf(store) {
  return (
    store.accounts.find((account) => account.index === store.activeIndex) || store.accounts[0]
  );
}

function accountListResult(store, address) {
  return {
    accounts: store.accounts,
    activeIndex: store.activeIndex,
    activeAccount: activeAccountOf(store),
    address: address || null
  };
}

async function accountIndicesJson() {
  const store = await loadAccountStore();
  return JSON.stringify(store.accounts.map((account) => account.index));
}

async function addressForActiveAccount(mnemonic) {
  const store = await loadAccountStore();
  return wasm.generate_address(mnemonic, store.activeIndex, 0);
}

async function persistActiveAddress(mnemonic) {
  const address = await addressForActiveAccount(mnemonic);
  session.address = address;
  const state = await loadWalletState();
  if (state && state.address !== address) {
    await saveWalletState({ ...state, address });
  }
  return address;
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
  let broadcastResult;
  try {
    broadcastResult = await rpcCallWithRetry("sendrawtransaction", [hex, false], {
      retries: opts.retries ?? 2,
      baseDelayMs: opts.baseDelayMs ?? 500
    });
  } catch (err) {
    const retired = await retireDuplicateNullifier(err);
    if (retired > 0) {
      throw new Error(
        "That note is already spent on chain. It was removed from the spendable balance. Go back and review the send again."
      );
    }
    throw err;
  }
  const directTxid = String(resolveTxidFromBroadcast(broadcastResult, ""));
  return directTxid;
}

/** Probe nozywallet.org/status.json for public LWD health (zec.rocks-style). */
async function probePublicNodeStatus(timeoutMs = 6000) {
  const ctrl = new AbortController();
  const timer = setTimeout(() => ctrl.abort(), timeoutMs);
  try {
    const resp = await fetch(PUBLIC_NODE_STATUS_URL, {
      cache: "no-store",
      signal: ctrl.signal
    });
    if (!resp.ok) {
      throw new Error(`Public node status HTTP ${resp.status}`);
    }
    const body = await resp.json();
    const lwdOk = Boolean(body?.lightwalletd?.ok);
    const heightRaw = body?.lightwalletd?.height ?? body?.zebra?.blocks;
    const blockCount =
      typeof heightRaw === "number" && Number.isFinite(heightRaw) && heightRaw >= 0
        ? Math.floor(heightRaw)
        : null;
    if (!lwdOk) {
      throw new Error(
        "Public lightwalletd is not ready yet. Try again later or connect a local Zebrad."
      );
    }
    return { blockCount, lwdOk: true };
  } catch (e) {
    throw e;
  } finally {
    clearTimeout(timer);
  }
}

async function isPublicLwdOptedIn() {
  const opted = await storageGet(STORAGE_PUBLIC_LWD_OPT_IN);
  if (!opted) return false;
  const lwd = String((await storageGet(STORAGE_LWD_URL)) ?? "").trim();
  return !lwd || /lwd\.nozywallet\.org/i.test(lwd) || /zec\.rocks/i.test(lwd);
}

async function connectPublicLwd() {
  const status = await probePublicNodeStatus();
  await storageSet({
    [STORAGE_PUBLIC_LWD_OPT_IN]: true,
    [STORAGE_LWD_URL]: PUBLIC_LWD_URL
  });
  const blockCount = status.blockCount;
  return {
    endpoint: PUBLIC_LWD_URL,
    blockCount,
    connected: true,
    source: "public_lwd",
    message:
      blockCount != null
        ? `Public sync connected — ${blockCount.toLocaleString()} blocks (lwd.nozywallet.org). Keys stay on this device.`
        : "Public sync connected (lwd.nozywallet.org). Keys stay on this device."
  };
}

async function clearPublicLwd() {
  await storageSet({
    [STORAGE_PUBLIC_LWD_OPT_IN]: false,
    [STORAGE_LWD_URL]: "http://127.0.0.1:9067"
  });
  return { cleared: true };
}

/**
 * Zebrad RPC if available; otherwise allow public LWD opt-in (create/restore only).
 * Scan/send still prefer a live Zebrad when present.
 */
async function ensureNodeForWalletLifecycle() {
  try {
    return { rpcUrl: await ensureReachableZebradRpc(), mode: "zebrad" };
  } catch (zebraErr) {
    if (await isPublicLwdOptedIn()) {
      try {
        await probePublicNodeStatus();
        return { rpcUrl: null, mode: "public_lwd" };
      } catch (_) {
        /* fall through */
      }
    }
    throw zebraErr instanceof Error
      ? zebraErr
      : new Error(String(zebraErr));
  }
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
      "Could not find a local Zebrad or Zakura node. Start one, then click Find my node — or choose Yes to connect to the Nozy sync server (https://lwd.nozywallet.org:443).\n\n" +
        "• Zebrad on this PC: use port 8232\n" +
        "• Zebrad in WSL: we auto-detect the WSL IP (not 127.0.0.1 from Chrome)\n" +
        "• Public sync: Yes on the offer, or pick Public sync in the chips\n" +
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

function endpointHost(url) {
  const raw = String(url || "").trim();
  if (!raw) return "";
  try {
    const withScheme = /^https?:\/\//i.test(raw) ? raw : `http://${raw}`;
    return new URL(withScheme).hostname.toLowerCase();
  } catch (_) {
    return "";
  }
}

function isLocalNodeHost(host) {
  if (host === "localhost" || host === "127.0.0.1" || host === "::1" || host === "10.0.2.2") {
    return true;
  }
  const v4 = /^(\d+)\.(\d+)\.(\d+)\.(\d+)$/.exec(host);
  if (!v4) return false;
  const a = Number(v4[1]);
  const b = Number(v4[2]);
  if (a === 10) return true;
  if (a === 192 && b === 168) return true;
  if (a === 172 && b >= 16 && b <= 31) return true;
  return false;
}

function isNozyPublicSyncUrl(url) {
  return endpointHost(url) === "lwd.nozywallet.org";
}

/**
 * One-shot node connect for extension onboarding.
 * @param {{ url?: string, tryCompanion?: boolean }} [opts]
 */
async function connectZebradRpc(opts = {}) {
  const explicitUrl = String(opts?.url ?? "").trim();
  if (explicitUrl && !isNozyPublicSyncUrl(explicitUrl) && !isLocalNodeHost(endpointHost(explicitUrl))) {
    throw new Error(
      "Only a local node or the Nozy server (lwd.nozywallet.org) can be used. Other servers, including zec.rocks, are not allowed."
    );
  }
  if (explicitUrl && isNozyPublicSyncUrl(explicitUrl)) {
    const publicRes = await connectPublicLwd();
    return {
      rpcEndpoint: publicRes.endpoint,
      blockCount: publicRes.blockCount ?? 0,
      connected: true,
      source: "public_lwd"
    };
  }
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
  const live = await probeZebradRpcEndpoint(endpoint, 3500);
  if (live) {
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
  // Prefer live Zebrad; allow create after public LWD opt-in (zec.rocks-style).
  const node = await ensureNodeForWalletLifecycle();
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
    rpcEndpoint: node.rpcUrl || session.rpcEndpoint,
    orchardBirthdayHeight,
    restoredFromPhrase: false,
    seedBackupConfirmed: false,
    syncMode: node.mode
  });

  session.unlocked = true;
  session.mnemonic = mnemonic;
  session.address = address;
  touchSession();
  await rememberSessionPassword(password);
  await saveSessionMnemonic(mnemonic);
  await startAutoBackgroundScan();

  return { address, mnemonic };
}

async function walletRestore(mnemonic, password, opts = {}) {
  const node = await ensureNodeForWalletLifecycle();
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
    // A blank birthday must still find notes that already exist. Chain tip only
    // sees new blocks and leaves a restored wallet looking empty.
    orchardBirthdayHeight = await defaultRestoreBirthdayHeight();
  }

  await saveWalletState({
    encryptedMnemonic,
    address,
    createdAt: Date.now(),
    rpcEndpoint: node.rpcUrl || session.rpcEndpoint,
    orchardBirthdayHeight,
    restoredFromPhrase: true,
    seedBackupConfirmed: true,
    syncMode: node.mode
  });

  session.unlocked = true;
  session.mnemonic = mnemonic;
  session.address = address;
  touchSession();
  await rememberSessionPassword(password);
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
  const address = await persistActiveAddress(mnemonic);

  session.unlocked = true;
  session.mnemonic = mnemonic;
  session.address = address;
  session.rpcEndpoint = state.rpcEndpoint || session.rpcEndpoint;
  touchSession();
  await rememberSessionPassword(password);
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

/**
 * Re-show the recovery phrase after unlock. Requires the vault password again
 * (even if the session is unlocked) so casual shoulder-surfing is harder.
 *
 * Fast path: if this browser session already unlocked with the same password,
 * skip a second Argon2 decrypt (that was freezing the UI on "Checking…").
 */
async function walletRevealMnemonic(password) {
  if (!session.unlocked) {
    throw new Error("Unlock the wallet first.");
  }
  const pwd = String(password ?? "");
  if (!pwd) {
    throw new Error("Enter your wallet password to view the recovery phrase.");
  }

  // Prefer mnemonic already held in the unlocked session.
  let mnemonic = typeof session.mnemonic === "string" ? session.mnemonic.trim() : "";
  if (!mnemonic) {
    mnemonic = String((await loadSessionMnemonic()) ?? "").trim();
    if (mnemonic) session.mnemonic = mnemonic;
  }

  const fp = await passwordFingerprint(pwd);
  if (session.passwordFingerprint && fp === session.passwordFingerprint && mnemonic) {
    return { mnemonic };
  }

  // Fallback (SW restarted mid-session, or fingerprint missing): full vault decrypt.
  await ensureWasm();
  const state = await loadWalletState();
  if (!state?.encryptedMnemonic) {
    throw new Error("No wallet found.");
  }
  let decryptedMnemonic;
  try {
    const decrypted = wasm.decrypt_from_storage(
      new Uint8Array(state.encryptedMnemonic),
      pwd
    );
    decryptedMnemonic = utf8Decode(decrypted).trim();
  } catch (_) {
    throw new Error("Wrong password.");
  }
  if (!decryptedMnemonic) {
    throw new Error("Could not decrypt recovery phrase.");
  }
  session.mnemonic = decryptedMnemonic;
  await rememberSessionPassword(pwd);
  await saveSessionMnemonic(decryptedMnemonic);
  return { mnemonic: decryptedMnemonic };
}

async function walletLock() {
  const scan = await loadScanState();
  if (scan?.status === "scanning" || scan?.status === "watching") {
    await stopBackgroundScan();
  }
  session.unlocked = false;
  session.mnemonic = null;
  clearSessionPassword();
  session.address = null;
  clearScanResumeForBackground();
  clearSessionMnemonic();
  clearSessionActivity();
  return true;
}

async function getWalletStatus() {
  const state = await loadWalletState();
  const accountStore = await loadAccountStore();
  const bh = state?.orchardBirthdayHeight;
  const orchardBirthdayHeight =
    typeof bh === "number" && Number.isFinite(bh) && bh >= 0 ? Math.floor(bh) : null;
  return {
    exists: !!state,
    unlocked: session.unlocked,
    address: session.address || state?.address || null,
    rpcEndpoint: session.rpcEndpoint,
    accounts: accountStore.accounts,
    activeAccount: activeAccountOf(accountStore),
    orchardBirthdayHeight,
    restoredFromPhrase:
      typeof state?.restoredFromPhrase === "boolean" ? state.restoredFromPhrase : null,
    seedBackupConfirmed: state?.seedBackupConfirmed === false ? false : true,
    autoLockMs: session.autoLockMs
  };
}

async function assertSeedBackupConfirmed() {
  const state = await loadWalletState();
  if (state?.seedBackupConfirmed === false) {
    throw new Error("Confirm your recovery phrase backup before using the wallet.");
  }
}

async function walletConfirmSeedBackup(answers) {
  const state = await loadWalletState();
  if (!state?.encryptedMnemonic) throw new Error("No wallet found.");
  if (!session.unlocked || !session.mnemonic) throw new Error("Unlock wallet first.");
  if (state.seedBackupConfirmed !== false) {
    return { seedBackupConfirmed: true };
  }
  const words = String(session.mnemonic).trim().split(/\s+/).filter(Boolean);
  const entries = answers && typeof answers === "object" ? Object.entries(answers) : [];
  if (entries.length < 3) {
    throw new Error("Confirm at least three words from your recovery phrase.");
  }
  for (const [pos, word] of entries) {
    const index = Number(pos) - 1;
    const expected = words[index];
    const given = String(word ?? "").trim().toLowerCase();
    if (!expected || given !== expected.toLowerCase()) {
      throw new Error("One or more words don’t match. Check your written copy and try again.");
    }
  }
  await saveWalletState({ ...state, seedBackupConfirmed: true });
  return { seedBackupConfirmed: true };
}

async function walletPendingBackupPhrase() {
  const state = await loadWalletState();
  if (!state?.encryptedMnemonic) throw new Error("No wallet found.");
  if (state.seedBackupConfirmed !== false) {
    throw new Error("Recovery phrase backup is already confirmed.");
  }
  if (!session.unlocked || !session.mnemonic) throw new Error("Unlock wallet first.");
  return { mnemonic: session.mnemonic };
}

async function getAccounts() {
  if (!session.unlocked || !session.address) return [];
  touchSession();
  return [session.address];
}

/** Real page origin from the content-script sender. Falls back to the message field. */
function pageOrigin(sender, fallback) {
  const raw = sender?.tab?.url || sender?.url || fallback || "";
  try {
    return new URL(String(raw)).origin;
  } catch (_) {
    return "";
  }
}

async function loadConnectedOrigins() {
  const raw = await storageGet(CONNECTED_ORIGINS_KEY);
  const list = Array.isArray(raw?.origins) ? raw.origins : [];
  return list.filter((origin) => typeof origin === "string" && origin);
}

async function saveConnectedOrigins(origins) {
  await storageSet({ [CONNECTED_ORIGINS_KEY]: { origins } });
}

async function isOriginConnected(origin) {
  if (!origin) return false;
  const list = await loadConnectedOrigins();
  return list.includes(origin);
}

async function rememberConnectedOrigin(origin) {
  if (!origin) return;
  const list = await loadConnectedOrigins();
  if (list.includes(origin)) return;
  list.push(origin);
  await saveConnectedOrigins(list);
}

async function forgetConnectedOrigin(origin) {
  const list = await loadConnectedOrigins();
  await saveConnectedOrigins(list.filter((item) => item !== origin));
}

function findPendingConnect(origin) {
  for (const approval of pendingApprovals.values()) {
    if (approval.kind === "connect" && approval.payload?.origin === origin) return approval;
  }
  return null;
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

async function ensureSessionInitialized(opts = {}) {
  const wallet = (await loadWalletState()) || {};
  session.rpcEndpoint = wallet.rpcEndpoint || session.rpcEndpoint;
  const policy = await loadSessionPolicy();
  session.autoLockMs = Number(policy.autoLockMs) || DEFAULT_AUTO_LOCK_MS;
  if (!session.lastActivityAt) {
    session.lastActivityAt = await loadActivityAt();
  }
  if (await lockIfIdle()) return;

  // Auto-hydrate from session storage after a service-worker restart.
  // chrome.storage.session survives SW idle kills within the same browser session.
  // Skip WASM hydrate for lightweight UI methods so Public sync never waits on initWasm.
  if (!opts.skipWasmHydrate && !session.unlocked && wallet.encryptedMnemonic) {
    try {
      const sessionMnemonic = await loadSessionMnemonic();
      if (sessionMnemonic && typeof sessionMnemonic === "string") {
        await ensureWasm();
        // Account 0 still matches a vault that has not switched accounts yet.
        const store = await loadAccountStore();
        const derivedActive = wasm.generate_address(sessionMnemonic, store.activeIndex, 0);
        const derivedAccount0 =
          store.activeIndex === 0
            ? derivedActive
            : wasm.generate_address(sessionMnemonic, 0, 0);
        if (
          derivedActive === wallet.address ||
          derivedAccount0 === wallet.address ||
          !wallet.address
        ) {
          session.unlocked = true;
          session.mnemonic = sessionMnemonic;
          session.address = derivedActive;
          if (!session.lastActivityAt) touchSession();
          if (wallet.address !== derivedActive) {
            await saveWalletState({ ...wallet, address: derivedActive });
          }
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
  const called = await rpcCallJson(endpoint, method, params, {
    timeoutMs: 8000,
    extraCandidates: await loadRpcEndpointCache()
  });
  if (called.endpoint && called.endpoint !== endpoint) {
    await persistRpcEndpoint(called.endpoint);
  } else if (called.endpoint) {
    try {
      const host = new URL(called.endpoint).hostname;
      if (isWslStyleHost(host) || host === "127.0.0.1" || host === "localhost") {
        await rememberRpcEndpoint(called.endpoint);
      }
    } catch {
      // ignore cache errors
    }
  }
  return called.result;
}

/**
 * Spend/prove must talk to Zebrad. A saved lightwalletd URL or a non-Zebra process
 * on port 8232 answers `getblockcount` with HTTP 415.
 * @returns {Promise<string>}
 */
/**
 * The first port that answers getblockcount can be a short chain.
 * A mainnet scan then asks that node for an anchor it does not have,
 * and the fallback method comes back as "Method not found".
 * Prefer the reachable node whose tip has reached the scan.
 * @param {number} scanHeight
 * @returns {Promise<string>}
 */
async function ensureSpendRpcEndpointForScan(scanHeight) {
  const wanted = Number(scanHeight);
  /** @type {string[]} */
  const candidates = [];
  const push = (value) => {
    if (!value || typeof value !== "string") return;
    let url = "";
    try {
      url = normalizeRpcEndpoint(value);
    } catch {
      return;
    }
    if (!url || isLightwalletdEndpoint(url) || candidates.includes(url)) return;
    candidates.push(url);
  };
  push(session.rpcEndpoint);
  try {
    const cached = await loadRpcEndpointCache();
    if (Array.isArray(cached)) cached.forEach(push);
  } catch {
    /* cache is optional */
  }
  push("http://127.0.0.1:18232");
  push("http://127.0.0.1:8232");

  let bestUrl = "";
  let bestTip = -1;
  const probed = [];
  for (const url of candidates) {
    try {
      const tip = await rpcBlockCountAt(url, 2000);
      probed.push({ url, tip });
      if (tip > bestTip) {
        bestUrl = url;
        bestTip = tip;
      }
      if (Number.isFinite(wanted) && wanted > 0 && tip + 20 >= wanted) break;
    } catch {
      probed.push({ url, tip: null });
    }
  }
  if (bestUrl) {
    pinRpcEndpoint(bestUrl);
    return persistRpcEndpoint(bestUrl);
  }
  return ensureSpendRpcEndpoint();
}

async function ensureSpendRpcEndpoint() {
  const current = String(session.rpcEndpoint || "");
  const lwd = !current || isLightwalletdEndpoint(current);
  if (!lwd && (await probeZebradRpcEndpoint(current, 2500))) {
    return normalizeRpcEndpoint(current);
  }
  const fromCompanion = await tryCompanionZebraUrl();
  if (fromCompanion) return persistRpcEndpoint(fromCompanion);
  try {
    return await autodetectZebradRpcEndpoint();
  } catch (err) {
    if (lwd) {
      throw new Error(
        "Sending needs a local Zebrad or Zakura node. Public sync can scan, but getblockcount is JSON-RPC and lwd.nozywallet.org is not that. Use Find node in Settings."
      );
    }
    throw err instanceof Error ? err : new Error(String(err));
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
  try {
    if (await isPublicLwdOptedIn()) {
      const status = await probePublicNodeStatus();
      if (status.blockCount != null) return status.blockCount;
    }
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

function scanCoveredRestoreFloor(scanState, floor) {
  if (!scanState) return false;
  if (scanState.scannedFromBirthdayFloor === true) return true;
  const start = Number(scanState.startHeight);
  return Number.isFinite(start) && Math.floor(start) <= Math.floor(floor) + 64;
}

/**
 * Each wallet scans from its own saved birthday. 3,050,000 is not applied to
 * every restore — only a wallet whose stored birthday is that height (the first one).
 */
async function resolveScanBirthday(ws, chainTip, scanState) {
  const tip = Number.isFinite(chainTip) && chainTip >= 0 ? Math.floor(chainTip) : 0;
  const bh = Number(ws?.orchardBirthdayHeight);
  const notes = Array.isArray(scanState?.discoveredNotes) ? scanState.discoveredNotes.length : 0;
  let birthday = Number.isFinite(bh) && bh >= 0 ? Math.floor(bh) : tip;
  // A restore that stored "now" as the birthday finishes sync with an empty balance.
  // Rewind once to the shielded history floor so the phrase can see existing notes.
  if (
    ws?.restoredFromPhrase === true &&
    tip > 0 &&
    notes === 0 &&
    birthdayLooksLikeChainTip(birthday, tip)
  ) {
    const floor = await defaultRestoreBirthdayHeight();
    if (birthday <= floor + RESTORE_BIRTHDAY_NEAR_TIP) {
      return { birthday: Math.min(tip, birthday), rewound: false };
    }
    const next = Math.min(tip, floor);
    if (next < birthday) {
      await saveWalletState({ ...ws, orchardBirthdayHeight: next });
      birthday = next;
      return { birthday, rewound: true };
    }
  }
  return { birthday: tip > 0 ? Math.min(tip, birthday) : birthday, rewound: false };
}

async function estimateFeeZats(memo = "", priority = true) {
  // ZIP-317 fee. Marginal fee comes from getstandardfee when the node has it.
  // `priority` selects the 10× inclusion fee; false pays the standard fee.
  try {
    await ensureWasm();
  } catch (_) {
    return mandatoryOrchardFeeZats(null, memo, null, priority);
  }
  let feeBlock = null;
  try {
    if (session.rpcEndpoint) {
      feeBlock = await feeBlockFromRpc((method, rpcParams) =>
        rpcCall(method, rpcParams)
      );
    }
  } catch (_) {
    feeBlock = null;
  }
  return mandatoryOrchardFeeZats(wasm, memo, feeBlock, priority);
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
  await assertSeedBackupConfirmed();
  await ensureWasm();
  if (typeof wasm.sign_vote_delegation !== "function") {
    throw new Error("Reload the unpacked extension after rebuilding WASM (vote sign).");
  }
  return wasm.sign_vote_delegation(session.mnemonic, String(requestJson || ""));
}

setInterval(() => {
  lockIfIdle().catch(() => undefined);
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
const SCAN_FIRST_BATCH = 200;
/** Persist tracker/notes periodically; large tracker JSON makes frequent saves costly. */
const SCAN_SAVE_EVERY_BLOCKS = 40;
/** Near the start, save often enough that a service-worker kill loses only a few blocks. */
const SCAN_SAVE_EVERY_BLOCKS_BOOTSTRAP = 8;
/** Yield a tick before Chrome kills the worker so currentHeight can be persisted. */
const SCAN_TICK_BUDGET_MS = 45_000;
/** Parallel Zebrad `getblock` fetches per sub-batch (witness apply stays sequential). */
const SCAN_PARALLEL_FETCH = 8;
/**
 * Compact HTTP blocks per request. The public endpoint rejects spans over 500.
 * One request of 100 replaces dozens of 3-block round trips.
 */
const COMPACT_FETCH_SPAN = 100;
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

function actionNullifierHexes(block) {
  const out = new Set();
  const txs = Array.isArray(block?.tx)
    ? block.tx
    : Array.isArray(block?.transactions)
      ? block.transactions
      : [];
  for (const tx of txs) {
    if (!tx || typeof tx === "string") continue;
    const lists = [
      tx?.orchard?.actions,
      tx?.ironwood?.actions,
      tx?.ironwood_actions,
      tx?.ironwoodActions
    ];
    for (const list of lists) {
      if (!Array.isArray(list)) continue;
      for (const action of list) {
        const hex = nullifierHex(action);
        if (hex) out.add(hex);
      }
    }
  }
  return out;
}

function markNotesSpent(state, hexes) {
  let marked = 0;
  for (const row of state?.discoveredNotes ?? []) {
    if (!row || row.spent) continue;
    const note = row.note ?? row;
    const nf = nullifierHex(note);
    if (!nf) continue;
    const hit =
      hexes instanceof Set
        ? hexes.has(nf)
        : hexes.some((h) => h.length === 64 && nf === h);
    if (!hit) continue;
    row.spent = true;
    marked += 1;
  }
  return marked;
}

function markNotesSpentByBlock(state, block) {
  return markNotesSpent(state, actionNullifierHexes(block));
}

async function applyCanonicalNullifiers(state) {
  if (!session.mnemonic || typeof wasm?.canonical_nullifier_hex !== "function") return 0;
  await ensureWasm();
  let updated = 0;
  for (const row of state?.discoveredNotes ?? []) {
    const note = row?.note ?? row;
    if (!note || typeof note !== "object") continue;
    let hex = "";
    try {
      hex = String(
        wasm.canonical_nullifier_hex(session.mnemonic, JSON.stringify(note)) || ""
      ).toLowerCase();
    } catch (_) {
      continue;
    }
    if (!/^[0-9a-f]{64}$/.test(hex)) continue;
    if (nullifierHex(note) === hex) continue;
    note.nullifier = hex.match(/../g).map((b) => parseInt(b, 16));
    updated += 1;
  }
  return updated;
}

function reverseHex32(hex) {
  if (!/^[0-9a-f]{64}$/.test(hex)) return "";
  return hex.match(/../g).reverse().join("");
}

async function retireDuplicateNullifier(err) {
  const msg = String(err?.message ?? err ?? "");
  if (!/duplicate nullifier/i.test(msg)) return 0;
  const displayed = [];
  for (const match of msg.matchAll(/0x([0-9a-fA-F]{64})/g)) {
    displayed.push(match[1].toLowerCase());
  }
  if (!displayed.length) return 0;
  // Zebra prints orchard nullifiers with `Fp`'s Debug, which reverses to_repr().
  // Stored and canonical nullifiers are the little-endian consensus bytes.
  const hexes = [];
  for (const shown of displayed) {
    hexes.push(shown);
    const wire = reverseHex32(shown);
    if (wire && wire !== shown) hexes.push(wire);
  }
  const state = await loadScanState();
  if (!state) return 0;
  const updated = await applyCanonicalNullifiers(state);
  const marked = markNotesSpent(state, hexes);
  if (marked || updated) {
    recomputePoolBalances(state);
    state.updatedAt = nowMs();
    await saveScanState(state);
  }
  return marked;
}

function recomputePoolBalances(state) {
  let orchard = 0;
  let ironwood = 0;
  const sapling = Number(state.saplingBalanceZats ?? 0);
  for (const row of state.discoveredNotes ?? []) {
    if (row?.spent) continue;
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
    if (await isPublicLwdOptedIn()) {
      try {
        const ts = await compactTreeState(startHeight - 1);
        orchardFinal =
          poolFinalStateFromTreestate(ts, "orchard") ||
          String(ts?.orchard_tree ?? "").trim();
        ironwoodFinal =
          poolFinalStateFromTreestate(ts, "ironwood") ||
          String(ts?.ironwood_tree ?? "").trim();
        // A compact server that copies the Orchard tree into Ironwood poisons every witness.
        if (ironwoodFinal && orchardFinal && ironwoodFinal === orchardFinal) {
          ironwoodFinal = "";
        }
      } catch (_) {
        /* try Zebrad below */
      }
    }
    if (!orchardFinal) {
      const ts = rpcEndpoint
        ? await _inlineRpcRequest(rpcEndpoint, "z_gettreestate", [String(startHeight - 1)])
        : await rpcCall("z_gettreestate", [String(startHeight - 1)]);
      orchardFinal = poolFinalStateFromTreestate(ts, "orchard");
      ironwoodFinal = poolFinalStateFromTreestate(ts, "ironwood");
    }
  }
  if (typeof wasm.shielded_scan_tracker_new === "function") {
    return wasm.shielded_scan_tracker_new(orchardFinal, ironwoodFinal);
  }
  return serializeShieldedTrackerState({
    orchard: wasm.orchard_scan_tracker_new(orchardFinal),
    ironwood: wasm.orchard_scan_tracker_new(ironwoodFinal)
  });
}

function wasmJsonValue(v) {
  if (v instanceof Map) {
    const o = {};
    for (const [k, val] of v) o[String(k)] = wasmJsonValue(val);
    return o;
  }
  if (Array.isArray(v)) return v.map(wasmJsonValue);
  return v;
}

function applyShieldedScanBlock(trackerJson, mnemonic, address, height, blockJson, accountIndicesJson) {
  const out = wasmJsonValue(
    wasm.orchard_scan_tracker_apply_block(
      trackerJson,
      mnemonic,
      address,
      height,
      blockJson,
      accountIndicesJson || "[0]"
    )
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

/** Keep only fields orchard_scan_tracker_apply_block needs — drops multi‑MB verbose tx junk. */
function slimActionForScan(action) {
  if (!action || typeof action !== "object") return null;
  const nullifier = action.nullifier;
  const cmx = action.cmx;
  const ephemeralKey = action.ephemeralKey ?? action.ephemeral_key;
  const encCiphertext = action.encCiphertext ?? action.enc_ciphertext;
  if (
    typeof nullifier !== "string" ||
    typeof cmx !== "string" ||
    typeof ephemeralKey !== "string" ||
    typeof encCiphertext !== "string"
  ) {
    return null;
  }
  return { nullifier, cmx, ephemeralKey, encCiphertext };
}

function slimActionsForScan(actions) {
  if (!Array.isArray(actions) || actions.length === 0) return null;
  const out = [];
  for (const a of actions) {
    const slim = slimActionForScan(a);
    if (slim) out.push(slim);
  }
  return out.length ? out : null;
}

function compactBlockJsonForScan(block) {
  const txs = Array.isArray(block?.tx) ? block.tx : [];
  const outTx = [];
  for (const tx of txs) {
    if (!tx || typeof tx === "string") continue;
    const orchardActions = slimActionsForScan(tx?.orchard?.actions);
    let ironwoodActions = slimActionsForScan(tx?.ironwood?.actions);
    if (!ironwoodActions) {
      ironwoodActions = slimActionsForScan(
        Array.isArray(tx?.ironwood_actions) ? tx.ironwood_actions : tx?.ironwoodActions
      );
    }
    if (!ironwoodActions && tx?.ironwood && typeof tx.ironwood === "object") {
      ironwoodActions = slimActionsForScan(tx.ironwood.actions);
    }
    if (!orchardActions && !ironwoodActions) continue;
    const slimTx = { txid: String(tx.txid ?? tx.hash ?? "") };
    if (orchardActions) slimTx.orchard = { actions: orchardActions };
    if (ironwoodActions) slimTx.ironwood = { actions: ironwoodActions };
    outTx.push(slimTx);
  }
  return JSON.stringify({ tx: outTx });
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
/** Session-only marker that a background scan needs an unlocked RAM mnemonic (no seed persisted). */
const SCAN_RESUME_SESSION_KEY = "nozy_scan_resume_wallet_v1";
let emptyRestoreRewindKickInFlight = false;

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
  return new Promise((r) =>
    chrome.storage.local.set({ [SCAN_STATE_KEY]: s }, () => {
      void chrome.runtime.lastError;
      r();
    })
  );
}

let scanRunning = false;
let tipWatchKickInFlight = false;
let tipWatchLastKickAt = 0;

function scheduleScanAlarm(delayMinutes) {
  chrome.alarms.create(SCAN_ALARM, { delayInMinutes: delayMinutes });
}

async function readScanChainTip() {
  let latestTip = null;
  if (await isPublicLwdOptedIn()) {
    try {
      latestTip = await compactChainTip();
    } catch (_) {
      /* local RPC below */
    }
  }
  if (latestTip == null) {
    latestTip = Number(
      await rpcCallWithRetry("getblockcount", [], { retries: 1, baseDelayMs: 150 })
    );
  }
  if (!Number.isFinite(latestTip)) return null;
  return Math.max(0, Math.floor(latestTip));
}

/** Caught up: keep the cursor and notes, and poll the tip instead of marking the scan finished. */
async function saveTipWatchState(existing, chainTip) {
  const now = nowMs();
  const state = {
    ...(existing && typeof existing === "object" ? existing : {}),
    status: "watching",
    scanMode: SCAN_MODE_AUTO,
    followTip: true,
    endHeight: chainTip,
    currentHeight: chainTip + 1,
    finishedAt: null,
    scanError: null,
    updatedAt: now,
    rpcEndpoint: scanRpcEndpoint(existing || {})
  };
  if (typeof state.heightProgress !== "number" || !Number.isFinite(state.heightProgress)) {
    state.heightProgress = chainTip;
  }
  await saveScanState(state);
  scheduleScanAlarm(0.5);
  return state;
}

/**
 * A finished sync used to sit at "done" until Settings → Sync to tip.
 * While the wallet is unlocked, resume from the cursor and then watch the tip.
 */
async function maybeEnsureTipWatch() {
  if (!session.unlocked || !session.mnemonic || !session.address) return;
  if (tipWatchKickInFlight || emptyRestoreRewindKickInFlight) return;
  if (nowMs() - tipWatchLastKickAt < 20_000) return;
  const existing = await loadScanState();
  if (!existing || existing.status !== "done") return;
  tipWatchKickInFlight = true;
  tipWatchLastKickAt = nowMs();
  try {
    await startAutoBackgroundScan();
  } catch (_) {
    /* next progress poll or scan completion retries */
  } finally {
    tipWatchKickInFlight = false;
  }
}

async function scanTick() {
  if (spendRebuildActive) {
    scheduleScanAlarm(0.5);
    return;
  }
  if (scanRunning) {
    scheduleScanAlarm(0.05);
    return;
  }
  const state = await loadScanState();
  if (!state || (state.status !== "scanning" && state.status !== "watching")) return;
  const noteCount = Array.isArray(state.discoveredNotes) ? state.discoveredNotes.length : 0;
  if (noteCount === 0) {
    const ws = await loadWalletState();
    const tip = Number(state.endHeight ?? 0);
    const bh = Number(ws?.orchardBirthdayHeight);
    const floor = await defaultRestoreBirthdayHeight();
    if (
      ws?.restoredFromPhrase === true &&
      Number.isFinite(tip) &&
      tip > 64 &&
      bh > floor + RESTORE_BIRTHDAY_NEAR_TIP &&
      birthdayLooksLikeChainTip(bh, tip)
    ) {
      await startAutoBackgroundScan();
      return;
    }
  }

  scanRunning = true;
  const tickStarted = Date.now();
  try {
    await ensureSessionInitialized();
    const resume = await readScanResumeForBackground();
    const mnemonicForScan = session.mnemonic || null;
    const addressForScan = session.address || resume?.address || null;
    if (state.status === "watching") {
      // Tip follow must not refresh lastActivityAt. Catch-up ("scanning") defers
      // auto-lock; once status is "watching", idle time can lock again.
      if (!mnemonicForScan || !addressForScan) {
        return;
      }
      let latestTip = null;
      try {
        latestTip = await readScanChainTip();
      } catch (_) {
        latestTip = null;
      }
      const progress =
        typeof state.heightProgress === "number" && Number.isFinite(state.heightProgress)
          ? Math.floor(state.heightProgress)
          : Math.max(-1, Number(state.endHeight ?? 0));
      if (!Number.isFinite(latestTip) || latestTip <= progress) {
        if (Number.isFinite(latestTip)) {
          state.endHeight = Math.max(Number(state.endHeight ?? 0), latestTip);
          state.currentHeight = latestTip + 1;
          state.updatedAt = nowMs();
          await saveScanState(state);
        }
        return;
      }
      state.status = "scanning";
      state.scanMode = SCAN_MODE_AUTO;
      state.followTip = true;
      state.endHeight = latestTip;
      state.currentHeight = progress + 1;
      state.finishedAt = null;
      state.scanError = null;
      state.updatedAt = nowMs();
      await saveScanState(state);
    }
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
        let latestTip = null;
        let tipSource = null;
        const publicOptIn = await isPublicLwdOptedIn();
        if (publicOptIn) {
          try {
            latestTip = await compactChainTip();
            tipSource = "compact";
          } catch (compactErr) {
          }
        }
        if (latestTip == null) {
          latestTip = Number(
            await rpcCallWithRetry("getblockcount", [], { retries: 1, baseDelayMs: 150 })
          );
          tipSource = "rpc";
        }
        if (Number.isFinite(latestTip)) {
          state.endHeight = Math.max(state.endHeight ?? 0, Math.floor(latestTip));
        }
      } catch (_) {
        // Ignore transient tip lookup errors; block fetch path records actual RPC errors.
      }
    }
    // MV3 service workers are often killed mid-tick. heightProgress is saved during the
    // loop; currentHeight was only advanced at tick end — so resume forever restarted
    // at the same height. Always continue from the farther of the two cursors.
    if (typeof state.heightProgress === "number" && Number.isFinite(state.heightProgress)) {
      const resumeAt = Math.floor(state.heightProgress) + 1;
      if (resumeAt > (state.currentHeight ?? 0) && resumeAt >= (state.startHeight ?? 0)) {
        state.currentHeight = resumeAt;
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

    const useCompact = await isPublicLwdOptedIn();
    const rpcEndpoint = scanRpcEndpoint(state);
    let lastCompletedHeight = loopStart - 1;
    let stoppedEarly = false;
    const fetchSpan = useCompact ? COMPACT_FETCH_SPAN : SCAN_PARALLEL_FETCH;
    const accountIndices = await accountIndicesJson();
    const loadBlocks = async (batchStart, batchEnd) => {
      const heights = [];
      for (let h = batchStart; h <= batchEnd; h += 1) heights.push(h);
      if (useCompact) {
        const compactBlocks = await compactFetchBlocks(batchStart, batchEnd);
        const byH = new Map();
        for (const b of compactBlocks) {
          const hh = Number(b?.height);
          if (Number.isFinite(hh)) byH.set(Math.floor(hh), b);
        }
        return { heights, blocks: heights.map((h) => byH.get(h) || { height: h, tx: [] }) };
      }
      const blocks = await Promise.all(
        heights.map((h) => rpcGetBlockVerboseByHeight(rpcEndpoint, h).catch(() => null))
      );
      return { heights, blocks };
    };
    scheduleScanAlarm(0.05);
    const keepAlive = setInterval(() => scheduleScanAlarm(0.05), 4000);
    try {
    let prefetch = null;
    for (let batchStart = loopStart; batchStart <= end; batchStart += fetchSpan) {
      if (Date.now() - tickStarted > SCAN_TICK_BUDGET_MS) {
        stoppedEarly = true;
        break;
      }
      const batchEnd = Math.min(batchStart + fetchSpan - 1, end);
      let loaded;
      try {
        loaded = prefetch ? await prefetch : await loadBlocks(batchStart, batchEnd);
        prefetch = null;
        if (loaded?.error) throw loaded.error;
      } catch (e) {
        if (!useCompact) throw e;
        state.consecutiveFailures = (state.consecutiveFailures || 0) + 1;
        state.lastRpcError = String(e?.message || e).slice(0, 400);
        if (state.consecutiveFailures >= failLimit) {
          state.status = "failed";
          state.scanError = `Scan stopped: compact HTTP failures. ${state.lastRpcError}`;
          state.finishedAt = nowMs();
          clearScanResumeForBackground();
          await saveScanState(state);
          return;
        }
        scheduleScanAlarm(0.5);
        return;
      }
      const upcoming = batchEnd + 1;
      if (upcoming <= end) {
        const upcomingEnd = Math.min(upcoming + fetchSpan - 1, end);
        prefetch = loadBlocks(upcoming, upcomingEnd).catch((error) => ({ error }));
      }
      const { heights, blocks } = loaded;
      for (let i = 0; i < heights.length; i += 1) {
        const h = heights[i];
        const block = blocks[i];
        try {
          if (block) {
            const blockJson = useCompact
              ? JSON.stringify(block)
              : compactBlockJsonForScan(block);
            let out;
            let nextTracker;
            const applied = await callWorker("apply_scan_block", {
              trackerState,
              mnemonic: mnemonicForScan,
              address: addressForScan,
              height: h,
              blockJson,
              accountIndicesJson: accountIndices
            });
            out = applied?.out;
            nextTracker = applied?.nextTracker || null;
            if (nextTracker) {
              trackerState = nextTracker;
              state.trackerState = trackerState;
            }
            state.consecutiveFailures = 0;
            if (out?.notes?.length) {
              for (const n of out.notes) {
                const v = Number(n?.value ?? 0);
                if (!Number.isFinite(v) || v <= 0) continue;
                const txid = String(n?.txid ?? block?.hash ?? `h${h}`);
                const pool = notePoolTag(n);
                const existing = (state.discoveredNotes ?? []).find((row) => {
                  const prev = row?.note ?? row;
                  return (
                    Number(row?.height) === h &&
                    String(row?.txid ?? prev?.txid ?? "") === txid &&
                    notePoolTag(prev) === pool &&
                    Number(row?.value ?? prev?.value ?? 0) === v
                  );
                });
                if (existing) {
                  const prev = existing.note ?? existing;
                  if (typeof prev.memo !== "string" && typeof n.memo === "string") {
                    if (existing.note && typeof existing.note === "object") existing.note.memo = n.memo;
                    else existing.memo = n.memo;
                  }
                  continue;
                }
                state.discoveredNotes.push({
                  note: n,
                  height: h,
                  txid,
                  value: v,
                  pool
                });
              }
              recomputePoolBalances(state);
            }
            const markedSpent = markNotesSpentByBlock(state, block);
            if (markedSpent) {
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
          state.currentHeight = h + 1;
          lastCompletedHeight = h;
          state.scannedBlocks = h - state.startHeight + 1;
          state.updatedAt = nowMs();
          blocksSinceProgressSave += 1;
          const pctInt = scanPercentInt(state);
          const crossedPercent = pctInt > (state.lastSavedPercentInt ?? -1);
          const saveEvery =
            h < (state.startHeight ?? 0) + 500
              ? SCAN_SAVE_EVERY_BLOCKS_BOOTSTRAP
              : SCAN_SAVE_EVERY_BLOCKS;
          if (blocksSinceProgressSave >= saveEvery || crossedPercent) {
            blocksSinceProgressSave = 0;
            if (crossedPercent) state.lastSavedPercentInt = pctInt;
            await saveScanState(state);
          }
        }
      }
    }
    } finally {
      clearInterval(keepAlive);
    }

    if (!stoppedEarly) {
      state.currentHeight = end + 1;
    } else if (lastCompletedHeight >= loopStart) {
      state.currentHeight = lastCompletedHeight + 1;
    }
    state.updatedAt = nowMs();

    if (state.currentHeight > state.endHeight) {
      const followTip = state.scanMode === SCAN_MODE_AUTO || state.followTip === true;
      if (followTip) {
        try {
          const latestTip = await readScanChainTip();
          if (Number.isFinite(latestTip) && latestTip > state.endHeight) {
            state.endHeight = latestTip;
          }
        } catch (_) {
          /* keep the tip already on the scan */
        }
      }
      if (state.currentHeight > state.endHeight && (state.scanMode === SCAN_MODE_AUTO || state.followTip === true)) {
        // Idle at the tip. The next alarm looks for new blocks without a Settings sync.
        state.status = "watching";
        state.followTip = true;
        state.scanMode = SCAN_MODE_AUTO;
        state.currentHeight = state.endHeight + 1;
        state.finishedAt = null;
      } else if (state.currentHeight > state.endHeight) {
        state.status = "done";
        state.finishedAt = nowMs();
        clearScanResumeForBackground();
      }
    }
    await refreshSaplingBalanceInScanState(state, {
      force: state.status === "done" || state.status === "watching"
    });
    await saveScanState(state);
  } catch (err) {
    const failMessage = String(err?.message || err).slice(0, 300);
    try {
      const latest = await loadScanState();
      if (latest && (latest.status === "scanning" || latest.status === "watching")) {
        latest.lastRpcError = failMessage;
        latest.updatedAt = nowMs();
        await saveScanState(latest);
      }
    } catch {
      /* keep the alarm */
    }
    scheduleScanAlarm(0.25);
  } finally {
    scanRunning = false;
  }

  if (state.status === "watching") {
    if (session.unlocked && session.mnemonic) {
      scheduleScanAlarm(0.5);
    }
  } else if (state.status === "scanning") {
    const hasMoreBlocks = (state.currentHeight ?? 0) <= (state.endHeight ?? -1);
    if (hasMoreBlocks) {
      scheduleScanAlarm(0.5);
      setTimeout(() => {
        void scanTick();
      }, 20);
    } else {
      scheduleScanAlarm(0.5);
    }
  } else if (state.status === "done" && session.unlocked && session.mnemonic) {
    setTimeout(() => {
      void maybeEnsureTipWatch();
    }, 50);
  }
}

async function startBackgroundScan(startHeight, endHeight, opts = {}) {
  // User explicitly started a range (window / custom / birthday). Always replace any
  // in-progress job — otherwise "Last 500" is ignored while auto-sync is still "scanning".
  if (session.unlocked && session.mnemonic && session.address) {
    await persistScanResumeForBackground(session.mnemonic, session.address);
  }
  const floor = await defaultRestoreBirthdayHeight();
  const ws = await loadWalletState();
  const ownBirthday = Number(ws?.orchardBirthdayHeight);
  const atOwnBirthday = Number.isFinite(ownBirthday) && startHeight <= ownBirthday + 64;
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
    scannedFromBirthdayFloor: atOwnBirthday && Number.isFinite(ownBirthday) && ownBirthday <= floor + 64,
    scannedFromWalletBirthday: atOwnBirthday,
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

async function maybeKickEmptyRestoreRewind() {
  if (!session.unlocked || !session.mnemonic || !session.address) return;
  if (emptyRestoreRewindKickInFlight) return;
  const ws = await loadWalletState();
  const existing = await loadScanState();
  if (!existing) return;
  const notes = Array.isArray(existing.discoveredNotes) ? existing.discoveredNotes.length : 0;
  const start = Number(existing.startHeight);
  const birthday = Number(ws?.orchardBirthdayHeight);
  if (!Number.isFinite(birthday) || !Number.isFinite(start) || start >= birthday - 64) return;
  emptyRestoreRewindKickInFlight = true;
  try {
    await startAutoBackgroundScan();
  } catch (err) {
  } finally {
    emptyRestoreRewindKickInFlight = false;
  }
}

async function startAutoBackgroundScan() {
  if (!session.unlocked || !session.mnemonic || !session.address) {
    return null;
  }
  await persistScanResumeForBackground(session.mnemonic, session.address);

  let tip = null;
  if (await isPublicLwdOptedIn()) {
    try {
      tip = await compactChainTip();
    } catch (_) {
      /* local RPC below */
    }
  }
  if (tip == null) {
    tip = Number(await rpcCallWithRetry("getblockcount", [], { retries: 1, baseDelayMs: 200 }));
  }
  const chainTip = Number.isFinite(tip) ? Math.max(0, Math.floor(tip)) : 0;
  const existing = await loadScanState();
  const now = nowMs();
  const ws = await loadWalletState();
  const resolved = await resolveScanBirthday(ws, chainTip, existing);
  const priorNoteCount = Array.isArray(existing?.discoveredNotes) ? existing.discoveredNotes.length : 0;
  const emptyHighStart =
    priorNoteCount === 0 &&
    existing &&
    typeof existing.startHeight === "number" &&
    Math.floor(existing.startHeight) > resolved.birthday + 64;
  const scanStartsBeforeBirthday =
    existing &&
    typeof existing.startHeight === "number" &&
    Math.floor(existing.startHeight) < resolved.birthday - 64;

  if (
    existing &&
    (existing.status === "scanning" || existing.status === "watching") &&
    !resolved.rewound &&
    !emptyHighStart &&
    !scanStartsBeforeBirthday
  ) {
    existing.scanMode = SCAN_MODE_AUTO;
    existing.followTip = true;
    existing.endHeight = Math.max(existing.endHeight ?? 0, chainTip);
    const progress =
      typeof existing.heightProgress === "number" && Number.isFinite(existing.heightProgress)
        ? Math.floor(existing.heightProgress)
        : Number(existing.currentHeight ?? 0) - 1;
    if (chainTip > progress) {
      existing.status = "scanning";
      existing.currentHeight = Math.max(Number(existing.currentHeight ?? 0), progress + 1);
      existing.finishedAt = null;
    } else {
      existing.status = "watching";
      existing.currentHeight = chainTip + 1;
      existing.finishedAt = null;
    }
    existing.updatedAt = now;
    await saveScanState(existing);
    if (existing.status === "watching") {
      scheduleScanAlarm(0.5);
      return existing;
    }
    if (scanRunning) {
      scheduleScanAlarm(0.2);
      return existing;
    }
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

  if (resolved.rewound || emptyHighStart || scanStartsBeforeBirthday) {
    startHeight = resolved.birthday;
    resumedFrom = startHeight;
    rewoundForSafety = true;
  } else if (priorDone && typeof existing.heightProgress === "number" && existing.heightProgress >= 0) {
    const nextHeight = Math.max(0, Math.floor(existing.heightProgress) + 1);
    resumedFrom = nextHeight;
    startHeight = nextHeight;
  } else if (priorDone && typeof existing.currentHeight === "number" && existing.currentHeight >= 0) {
    resumedFrom = Math.max(0, Math.floor(existing.currentHeight));
    startHeight = resumedFrom;
  } else {
    startHeight = Math.min(chainTip, resolved.birthday);
    resumedFrom = startHeight;
  }

  if (
    priorDone &&
    priorNoteCount === 0 &&
    !rewoundForSafety &&
    startHeight > resolved.birthday + 64
  ) {
    startHeight = resolved.birthday;
    resumedFrom = startHeight;
    rewoundForSafety = true;
  }

  if (!rewoundForSafety && priorDone && existing && startHeight > chainTip) {
    return saveTipWatchState(existing, chainTip);
  }

  const scannedFromBirthdayFloor =
    Boolean(existing?.scannedFromBirthdayFloor) || startHeight <= resolved.birthday + 64;
  const scannedFromWalletBirthday =
    Boolean(existing?.scannedFromWalletBirthday) || startHeight <= resolved.birthday + 64;
  const keepHistory = priorDone && Array.isArray(existing?.discoveredNotes) && !rewoundForSafety;
  const state = {
    status: "scanning",
    scanMode: SCAN_MODE_AUTO,
    followTip: true,
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
    scannedFromBirthdayFloor,
    scannedFromWalletBirthday,
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
  await startAutoBackgroundScan();
}

function stopBackgroundScan() {
  chrome.alarms.clear(SCAN_ALARM);
  return loadScanState().then((s) => {
    if (s && (s.status === "scanning" || s.status === "watching")) {
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
  if (alarm.name === LOCK_ALARM) {
    ensureSessionInitialized().catch(() => undefined);
  }
});

chrome.alarms.create(LOCK_ALARM, { periodInMinutes: 1 });

chrome.runtime.onInstalled.addListener((details) => {
  if (details?.reason !== "update") return;
  chrome.alarms.clear(SCAN_ALARM);
  clearScanResumeForBackground();
  loadScanState()
    .then((s) => {
      if (!s || (s.status !== "scanning" && s.status !== "watching")) return;
      s.status = "stopped";
      s.finishedAt = nowMs();
      s.scanError = null;
      s.lastRpcError = null;
      s.updatedAt = nowMs();
      return saveScanState(s);
    })
    .catch(() => undefined);
});

loadScanState().then(async (s) => {
  await ensureSessionInitialized();
  if (session.unlocked) {
    try {
      await startAutoBackgroundScan();
    } catch {
      /* RPC may be down; scanTick still waits */
    }
    return;
  }
  if (s && s.status === "scanning") {
    scheduleScanAlarm(0.01);
    void scanTick();
  }
});

async function openNozyPaymentWindow(uri) {
  const payment = String(uri || "").trim();
  if (!/^zcash:/i.test(payment) || payment.length > 4096) {
    throw new Error("Not a shielded Nozy payment request.");
  }
  await chrome.storage.session.set({
    nozyPendingPayment: { uri: payment, at: Date.now() }
  });
  const url = chrome.runtime.getURL("wasm-core/popup/dist/index.html?view=send");
  const win = await chrome.windows.create({
    url,
    type: "popup",
    width: 420,
    height: 720,
    focused: true
  });
  return win && win.id ? win.id : 0;
}

chrome.runtime.onMessageExternal.addListener((msg, sender, sendResponse) => {
  if (!msg || msg.type !== "NOZY_REQUEST" || msg.method !== "zcash_openPayment") return;
  let origin = "";
  try {
    origin = new URL(sender.url || sender.origin || "").origin;
  } catch (_) {
    origin = "";
  }
  const allowed = origin === "http://127.0.0.1:8080" || origin === "http://localhost:8080";
  if (!allowed) {
    sendResponse(fail("This page cannot open NozyWallet."));
    return true;
  }
  openNozyPaymentWindow(msg.params && msg.params.uri)
    .then(() => {
      sendResponse(ok(true));
    })
    .catch((err) => {
      sendResponse(fail(err && err.message ? err.message : String(err)));
    });
  return true;
});

chrome.runtime.onMessage.addListener((msg, sender, sendResponse) => {
  if (!msg || msg.type !== "NOZY_REQUEST") return;

  (async () => {
    try {
      validateRequestEnvelope(msg);
      assertMethodAllowedForSender(msg.method, sender);
      const method = msg.method;
      const params = msg.params ?? {};
      const skipWasm = WASM_OPTIONAL_METHODS.has(method);
      await ensureSessionInitialized({ skipWasmHydrate: skipWasm });
      if (!skipWasm) {
        await ensureWasm();
      }
      if (!ACTIVITY_SKIP_METHODS.has(method)) touchSession();

      // Popup/UI control methods.
      switch (method) {
        case "zcash_openPayment": {
          const uri = String(params.uri || "").trim();
          await openNozyPaymentWindow(uri);
          sendResponse(ok(true));
          return;
        }
        case "wallet_create":
          sendResponse(ok(await walletCreate(params.password)));
          return;
        case "wallet_restore":
          sendResponse(
            ok(await walletRestore(params.mnemonic, params.password, { birthdayHeight: params.birthdayHeight }))
          );
          return;
        case "wallet_reveal_mnemonic":
          sendResponse(ok(await walletRevealMnemonic(params.password)));
          return;
        case "wallet_confirm_seed_backup":
          sendResponse(ok(await walletConfirmSeedBackup(params.answers)));
          return;
        case "wallet_pending_backup_phrase":
          sendResponse(ok(await walletPendingBackupPhrase()));
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
        case "wallet_open_window": {
          const page = new URL(chrome.runtime.getURL("wasm-core/popup/dist/index.html"));
          page.searchParams.set("page", "1");
          if (params?.view) page.searchParams.set("view", String(params.view));
          if (params?.finalizer) page.searchParams.set("finalizer", String(params.finalizer));
          if (params?.bond) page.searchParams.set("bond", String(params.bond));
          const target = page.toString();
          const existing = await chrome.windows.getAll({ populate: true });
          for (const win of existing) {
            const tab = (win.tabs || []).find((t) => typeof t.url === "string" && t.url.includes("page=1") && t.url.includes("popup/dist/index.html"));
            if (tab?.id != null && win.id != null) {
              await chrome.windows.update(win.id, { focused: true });
              await chrome.tabs.update(tab.id, { active: true });
              sendResponse(ok({ id: win.id, focused: true }));
              return;
            }
          }
          const created = await chrome.windows.create({
            url: target,
            type: "popup",
            width: 1080,
            height: 760,
            focused: true
          });
          sendResponse(ok({ id: created?.id ?? null, focused: true }));
          return;
        }
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
        case "companion_crosslink_payout_claim":
          sendResponse(
            ok(
              await companionCrosslinkPayoutClaim(params.baseUrl, {
                payout_address: params.payout_address,
                mobile_ufvk: params.mobile_ufvk,
                cutoff_height: params.cutoff_height
              })
            )
          );
          return;
        case "companion_crosslink_doctor":
          sendResponse(
            ok(
              await companionCrosslinkDoctor(params.baseUrl, {
                observer: params.observer !== false
              })
            )
          );
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
        case "probe_lwd_mixnet_proxy":
          sendResponse(
            ok(
              await probeLwdMixnetProxy(params.baseUrl, {
                grpc_url: params.grpc_url,
                metrics_url: params.metrics_url
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
          await refreshTxExpiryStates();
          try {
            await backfillScanNoteMemos();
          } catch (_) {
            /* history still returns without a memo if the node is down */
          }
          {
            const sent = await loadTxState();
            const scan = await loadScanState();
            const outgoing = Array.isArray(sent.txs) ? sent.txs : [];
            sendResponse(
              ok({
                txs: [...receivedActivityEntries(scan), ...outgoing],
                updatedAt: sent.updatedAt || 0
              })
            );
          }
          return;
        case "wallet_retry_broadcast":
          sendResponse(ok({ txid: await retryBroadcastById(String(params.id ?? "")) }));
          return;
        case "wallet_speed_up":
          await assertSeedBackupConfirmed();
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
        case "wallet_list_accounts": {
          const store = await loadAccountStore();
          sendResponse(ok(accountListResult(store, session.address)));
          return;
        }
        case "wallet_add_account": {
          if (!session.unlocked || !session.mnemonic) throw new Error("Unlock wallet first.");
          const store = addAccount(await loadAccountStore(), params?.name);
          await saveAccountStore(store);
          const address = await persistActiveAddress(session.mnemonic);
          sendResponse(ok(accountListResult(store, address)));
          return;
        }
        case "wallet_rename_account": {
          const store = renameAccount(await loadAccountStore(), params?.index, params?.name);
          await saveAccountStore(store);
          sendResponse(ok(accountListResult(store, session.address)));
          return;
        }
        case "wallet_set_active_account": {
          if (!session.unlocked || !session.mnemonic) throw new Error("Unlock wallet first.");
          const store = setActiveAccount(await loadAccountStore(), params?.index);
          await saveAccountStore(store);
          const address = await persistActiveAddress(session.mnemonic);
          sendResponse(ok(accountListResult(store, address)));
          return;
        }
        case "wallet_generate_address":
          if (!session.unlocked || !session.mnemonic) throw new Error("Wallet is locked");
          sendResponse(ok(wasm.generate_address(session.mnemonic, params.account ?? 0, params.index ?? 0)));
          return;
        case "wallet_sign_message":
          if (!session.unlocked || !session.mnemonic) throw new Error("Wallet is locked");
          await assertSeedBackupConfirmed();
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
                if (approval.kind === "connect") {
                  if (!session.unlocked || !session.address) {
                    throw new Error("Unlock wallet first.");
                  }
                  const origin = String(approval.payload?.origin ?? "");
                  await rememberConnectedOrigin(origin);
                  resolver.sendResponse(ok([session.address]));
                  return;
                }

                if (approval.kind === "sign") {
                  const message = String(approval.payload?.message ?? "");
                  if (!message) throw new Error("Missing message for signing");
                  await assertSeedBackupConfirmed();
                  const signature = wasm.sign_message(session.mnemonic, message);
                  resolver.sendResponse(ok(signature));
                  return;
                }

                if (approval.kind === "transaction") {
                  await assertSeedBackupConfirmed();
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
        case "wallet_list_connected_sites":
          sendResponse(ok(await loadConnectedOrigins()));
          return;
        case "wallet_disconnect_site": {
          const origin = String(params?.origin ?? "");
          if (!origin) throw new Error("Missing origin.");
          await forgetConnectedOrigin(origin);
          sendResponse(ok({ origin, disconnected: true }));
          return;
        }
        case "rpc_set_endpoint": {
          const next = params.url || session.rpcEndpoint;
          if (isLightwalletdEndpoint(next)) {
            await connectPublicLwd();
            if (isLightwalletdEndpoint(session.rpcEndpoint)) {
              try {
                await autodetectZebradRpcEndpoint();
              } catch {
                /* Public sync stays on; send looks for a local node later. */
              }
            }
            sendResponse(ok({ rpcEndpoint: session.rpcEndpoint, publicLwd: true }));
            return;
          }
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
        case "connect_public_lwd": {
          const publicRes = await connectPublicLwd();
          sendResponse(ok(publicRes));
          return;
        }
        case "clear_public_lwd": {
          sendResponse(ok(await clearPublicLwd()));
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
          let endpoint = session.rpcEndpoint;
          let mode = "none";
          const publicOptIn =
            (await isPublicLwdOptedIn()) || isNozyPublicSyncUrl(endpoint);
          if (publicOptIn) {
            try {
              const status = await probePublicNodeStatus();
              connected = true;
              blockCount = status.blockCount;
              endpoint = PUBLIC_LWD_URL;
              mode = "public_lwd";
            } catch (_) {
              connected = false;
            }
          }
          if (!connected) {
            try {
              const raw = await rpcCallWithRetry("getblockcount", [], { retries: 0 });
              const n = typeof raw === "number" ? raw : Number(raw);
              if (Number.isFinite(n) && n >= 0) {
                connected = true;
                blockCount = Math.floor(n);
                mode = "zebrad";
                endpoint = session.rpcEndpoint;
              }
            } catch (_) {
              connected = false;
            }
          }
          sendResponse(
            ok({
              endpoint,
              connected,
              blockCount,
              mode
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
          if (params?.record === true) {
            const now = nowMs();
            await appendTxState({
              id: crypto.randomUUID(),
              txid: String(txid),
              state: "pending",
              origin: "popup-send",
              direction: "sent",
              recipientAddress: String(params.recipientAddress || params.to || ""),
              amount: Number(params.amount || 0),
              fee: Number.isFinite(Number(params.fee)) ? Number(params.fee) : null,
              memo: String(params.memo || ""),
              createdAt: now,
              updatedAt: now,
              error: null,
              rawTxHex: hex
            });
          }
          sendResponse(ok(txid));
          return;
        }
        case "rpc_tx_status": {
          const txid = String(params?.txid || "").trim();
          if (!/^[0-9a-fA-F]{64}$/.test(txid)) throw new Error("Missing txid.");
          try {
            const resp = await rpcCall("getrawtransaction", [txid, 1]);
            const mined = readTxConfirmation(resp);
            if (mined.confirmed) {
              await patchTxState(txid, {
                state: "confirmed",
                blockHeight: mined.blockHeight,
                error: null
              });
              sendResponse(
                ok({ confirmed: true, blockHeight: mined.blockHeight, pendingReason: null })
              );
              return;
            }
            sendResponse(
              ok({
                confirmed: false,
                blockHeight: null,
                pendingReason:
                  "Zebrad has the transaction in the mempool. It stays pending until a miner includes it in the next block."
              })
            );
            return;
          } catch (err) {
            const msg = String(err?.message || err || "");
            sendResponse(
              ok({
                confirmed: false,
                blockHeight: null,
                pendingReason: /not found|No such mempool|code: -5/i.test(msg)
                  ? "Zebrad has not seen this transaction in the mempool yet. It stays pending until the node accepts it."
                  : "Still pending. Zebrad is waiting for the next block to include this transaction."
              })
            );
            return;
          }
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
          if (params.useBirthdayRange === true) {
            const s = await startAutoBackgroundScan();
            if (!s) throw new Error("Unlock wallet first.");
            sendResponse(
              ok({
                started: true,
                startHeight: s.startHeight,
                endHeight: s.endHeight,
                status: s.status,
                rpcEndpoint: s.rpcEndpoint || session.rpcEndpoint || null
              })
            );
            return;
          }
          await persistScanResumeForBackground(session.mnemonic, session.address);
          let rpcUrl = null;
          let blockCount;
          if (await isPublicLwdOptedIn()) {
            try {
              blockCount = await compactChainTip();
              rpcUrl = PUBLIC_LWD_URL;
            } catch (_) {
              /* fall through to local Zebrad/Crosslink */
            }
          }
          if (blockCount == null) {
            rpcUrl = await ensureReachableZebradRpc();
            blockCount = await rpcCallWithRetry("getblockcount", []);
          }

          const rawEnd = params?.endHeight;
          let endH = blockCount;
          if (rawEnd !== undefined && rawEnd !== null && rawEnd !== "") {
            const n = Number(rawEnd);
            if (Number.isFinite(n)) {
              endH = Math.max(0, Math.min(Math.floor(n), blockCount));
            }
          }

          let startH;
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
            if (scan && (scan.status === "done" || scan.status === "stopped" || scan.status === "failed" || scan.status === "watching")) {
              await chrome.storage.local.remove(SCAN_STATE_KEY);
            }
          }
          sendResponse(ok({ orchardBirthdayHeight }));
          return;
        }
        case "wallet_scan_progress": {
          void (async () => {
            await maybeKickEmptyRestoreRewind();
            await maybeEnsureTipWatch();
          })();
          const scanState = await loadScanState();
          if (!scanState) {
            sendResponse(ok({ status: "idle", sessionUnlocked: session.unlocked }));
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
                : nowMs() - scanState.startedAt,
              sessionWaitingSince: scanState.sessionWaitingSince ?? null,
              sessionUnlocked: session.unlocked
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
          const payForInclusion = payForInclusionRequested(params);
          const fee = await estimateFeeZats(memo, payForInclusion);
          sendResponse(
            ok({
              fee,
              priority: payForInclusion,
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
          await assertSeedBackupConfirmed();
          {
            const scanState = await loadScanState();
            const rpcEndpoint = await ensureSpendRpcEndpointForScan(scanState?.heightProgress);
            const proveParams = {
                ...params,
                recipientAddress: params?.recipientAddress ?? params?.to ?? session.address,
                walletAddress: session.address,
                mnemonic: session.mnemonic,
                rpcEndpoint,
                accountIndicesJson: await accountIndicesJson(),
                priority: payForInclusionRequested(params),
                fee: await estimateFeeZats(
                  String(params?.memo ?? ""),
                  payForInclusionRequested(params)
                ),
                trackerState: scanState?.trackerState ?? "",
                scanHeight: scanState?.heightProgress,
                discoveredNotes: Array.isArray(scanState?.discoveredNotes)
                  ? scanState.discoveredNotes
                  : []
              };
            const proved = await callWorker("prove_transaction", proveParams);
            sendResponse(ok(proved));
            return;
          }
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
        case "zcash_accounts": {
          if (isExtensionPageSender(sender)) {
            sendResponse(ok(await getAccounts()));
            return;
          }
          const accountsOrigin = pageOrigin(sender, msg.origin);
          if (accountsOrigin && (await isOriginConnected(accountsOrigin))) {
            sendResponse(ok(await getAccounts()));
            return;
          }
          sendResponse(ok([]));
          return;
        }
        case "eth_requestAccounts":
        case "zcash_requestAccounts": {
          if (!session.unlocked || !session.address) {
            try {
              if (chrome.action?.openPopup) await chrome.action.openPopup();
            } catch (_) {
              /* user can open the popup themselves */
            }
            throw new Error("Unlock NozyWallet, then connect again.");
          }
          if (isExtensionPageSender(sender)) {
            sendResponse(ok(await getAccounts()));
            return;
          }
          const origin = pageOrigin(sender, msg.origin);
          if (!origin) throw new Error("Could not determine which site is asking to connect.");
          if (await isOriginConnected(origin)) {
            sendResponse(ok([session.address]));
            return;
          }
          const existingConnect = findPendingConnect(origin);
          if (existingConnect) {
            const prev = providerRequestResolvers.get(existingConnect.id);
            providerRequestResolvers.set(existingConnect.id, {
              sendResponse: (response) => {
                try {
                  prev?.sendResponse(response);
                } catch (_) {
                  /* the first request may already have settled */
                }
                sendResponse(response);
              }
            });
            return;
          }
          const approval = await requestApproval("connect", {
            method,
            origin,
            address: session.address
          });
          providerRequestResolvers.set(approval.id, { sendResponse });
          return;
        }
        case "personal_sign":
        case "zcash_signMessage": {
          if (!session.unlocked) throw new Error("Unlock wallet first.");
          const origin = pageOrigin(sender, msg.origin);
          const approval = await requestApproval("sign", {
            method,
            origin,
            message: params?.message || params?.[0] || ""
          });
          providerRequestResolvers.set(approval.id, { sendResponse });
          return;
        }
        case "eth_sendTransaction":
        case "zcash_sendTransaction": {
          if (!session.unlocked) throw new Error("Unlock wallet first.");
          await assertSeedBackupConfirmed();
          const origin = pageOrigin(sender, msg.origin);
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
      const failMessage = e?.message ?? String(e);
      sendResponse(fail(failMessage));
    }
  })();

  return true;
});

