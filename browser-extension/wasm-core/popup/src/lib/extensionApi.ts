type ApiRequest = {
  method: string;
  params?: Record<string, unknown>;
};

type ApiResponse<T> = {
  result: T | null;
  error: { message: string } | null;
};

export type WalletStatus = {
  exists: boolean;
  unlocked: boolean;
  address: string | null;
  rpcEndpoint: string;
};

export type TxStateEntry = {
  id: string;
  txid: string | null;
  state: "built" | "broadcast" | "pending" | "confirmed" | "failed";
  origin: string;
  recipientAddress: string;
  amount: number;
  fee: number | null;
  memo: string;
  createdAt: number;
  updatedAt: number;
  error: string | null;
  blockHeight?: number | null;
  rawTxHex?: string | null;
  inputsUsed?: number;
  inputMode?: "single" | "multi" | string;
};

export type PendingApproval = {
  id: string;
  kind: "sign" | "transaction";
  payload: Record<string, unknown>;
  createdAt: number;
};

export type MobileSyncDevice = {
  id: string;
  name: string;
  platform: string;
  sessionId: string;
  pairedAt: number;
  status: "paired" | "revoked";
  renamedAt?: number | null;
  revokedAt?: number | null;
  lastSeenAt?: number;
  trustLevel?: string;
};

export type MobileSyncState = {
  schemaVersion: number;
  pairedDevices: MobileSyncDevice[];
  activePairing: {
    sessionId: string;
    walletAddress: string;
    verifyCode: string;
    challenge: string;
    createdAt: number;
    expiresAt: number;
  } | null;
  pairingPayload: string | null;
};

function sendMessage<T>(request: ApiRequest): Promise<T> {
  return new Promise((resolve, reject) => {
    chrome.runtime.sendMessage(
      {
        type: "NOZY_REQUEST",
        method: request.method,
        params: request.params ?? {}
      },
      (response: ApiResponse<T>) => {
        if (chrome.runtime.lastError) {
          reject(new Error(chrome.runtime.lastError.message));
          return;
        }
        if (!response) {
          reject(new Error("No response from Nozy background worker"));
          return;
        }
        if (response.error) {
          reject(new Error(response.error.message));
          return;
        }
        resolve(response.result as T);
      }
    );
  });
}

export const extensionApi = {
  walletStatus: () => sendMessage<WalletStatus>({ method: "wallet_status" }),
  walletCreate: (password: string) =>
    sendMessage<{ address: string }>({ method: "wallet_create", params: { password } }),
  walletRestore: (mnemonic: string, password: string) =>
    sendMessage<{ address: string }>({
      method: "wallet_restore",
      params: { mnemonic, password }
    }),
  walletUnlock: (password: string) =>
    sendMessage<{ address: string }>({ method: "wallet_unlock", params: { password } }),
  walletLock: () => sendMessage<boolean>({ method: "wallet_lock" }),
  walletGenerateAddress: (account = 0, index = 0) =>
    sendMessage<string>({
      method: "wallet_generate_address",
      params: { account, index }
    }),
  walletSignMessage: (message: string) =>
    sendMessage<string>({
      method: "wallet_sign_message",
      params: { message }
    }),
  walletGetPendingApprovals: () =>
    sendMessage<PendingApproval[]>({ method: "wallet_get_pending_approvals" }),
  walletApproveRequest: (id: string) =>
    sendMessage<{ approved: boolean; id: string }>({
      method: "wallet_approve_request",
      params: { id }
    }),
  walletRejectRequest: (id: string) =>
    sendMessage<{ approved: boolean; id: string }>({
      method: "wallet_reject_request",
      params: { id }
    }),
  walletSetSessionPolicy: (autoLockMs: number) =>
    sendMessage<{ autoLockMs: number }>({
      method: "wallet_set_session_policy",
      params: { autoLockMs }
    }),
  walletGetTransactions: () =>
    sendMessage<{ txs: TxStateEntry[]; updatedAt: number }>({ method: "wallet_get_transactions" }),
  walletRetryBroadcast: (id: string) =>
    sendMessage<{ txid: string }>({ method: "wallet_retry_broadcast", params: { id } }),
  rpcSetEndpoint: (url: string) =>
    sendMessage<{ rpcEndpoint: string }>({
      method: "rpc_set_endpoint",
      params: { url }
    }),
  rpcGetStatus: () =>
    sendMessage<{ endpoint: string; connected: boolean }>({ method: "rpc_get_status" }),
  rpcGetBlockCount: () => sendMessage<number>({ method: "rpc_get_block_count" }),
  walletScanNotes: (startHeight: number, endHeight: number) =>
    sendMessage<{
      scannedBlocks: number;
      discoveredNotes: unknown[];
      totalBalanceZats: number;
    }>({
      method: "wallet_scan_notes",
      params: { startHeight, endHeight }
    }),
  walletStartScan: (window = 20_000) =>
    sendMessage<{
      started: boolean;
      startHeight: number;
      endHeight: number;
      status: string;
    }>({ method: "wallet_start_scan", params: { window } }),
  walletScanProgress: () =>
    sendMessage<{
      status: string;
      startHeight?: number;
      endHeight?: number;
      currentHeight?: number;
      scannedBlocks?: number;
      totalBlocks?: number;
      percent?: number;
      discoveredNotes?: number;
      totalBalanceZats?: number;
      startedAt?: number;
      elapsed?: number;
    }>({ method: "wallet_scan_progress" }),
  walletStopScan: () =>
    sendMessage<{ status: string }>({ method: "wallet_stop_scan" }),
  rpcSendRawTx: (rawTxHex: string) =>
    sendMessage<string>({ method: "rpc_send_raw_tx", params: { rawTxHex } }),
  walletProveTransaction: (tx: Record<string, unknown>) =>
    sendMessage<{
      txid: string;
      chainId: string;
      rawTxHex: string;
      proving: string;
      selected_notes_count?: number;
      selected_notes_total_value?: number;
      selected_notes?: Array<{
        value: number;
        cmx: string;
        block_height: number;
      }>;
      selected_witnesses_count?: number;
      inputs_used?: number;
      input_mode?: "single" | "multi";
      fee?: number;
    }>({
      method: "wallet_prove_transaction",
      params: tx
    }),
  mobileSyncGetState: () =>
    sendMessage<MobileSyncState>({ method: "mobile_sync_get_state" }),
  mobileSyncGetPairingSchema: () =>
    sendMessage<{
      type: string;
      required: string[];
      fields: Record<string, string>;
      notes: string;
    }>({ method: "mobile_sync_get_pairing_schema" }),
  mobileSyncInitPairing: () =>
    sendMessage<{
      sessionId: string;
      verifyCode: string;
      expiresAt: number;
      payload: string;
    }>({ method: "mobile_sync_init_pairing" }),
  mobileSyncConfirmPairing: (
    sessionId: string,
    deviceName: string,
    platform: string,
    challengeSignature: string
  ) =>
    sendMessage<MobileSyncDevice>({
      method: "mobile_sync_confirm_pairing",
      params: { sessionId, deviceName, platform, challengeSignature }
    }),
  mobileSyncUnpair: (deviceId: string) =>
    sendMessage<{ removed: boolean; deviceId: string }>({
      method: "mobile_sync_unpair",
      params: { deviceId }
    }),
  mobileSyncRenameDevice: (deviceId: string, name: string) =>
    sendMessage<MobileSyncDevice>({
      method: "mobile_sync_rename_device",
      params: { deviceId, name }
    }),
  mobileSyncRevokeDevice: (deviceId: string) =>
    sendMessage<MobileSyncDevice>({
      method: "mobile_sync_revoke_device",
      params: { deviceId }
    }),

  companionStatus: (baseUrl?: string) =>
    sendMessage<{
      companionReachable: boolean;
      healthStatus: number;
      lwdChainTip: unknown;
    }>({ method: "companion_status", params: { baseUrl } }),

  companionLwdInfo: (baseUrl?: string, lightwalletd_url?: string) =>
    sendMessage<Record<string, unknown>>({
      method: "companion_lwd_info",
      params: { baseUrl, lightwalletd_url }
    }),

  companionLwdChainTip: (baseUrl?: string, lightwalletd_url?: string) =>
    sendMessage<Record<string, unknown>>({
      method: "companion_lwd_chain_tip",
      params: { baseUrl, lightwalletd_url }
    }),

  companionLwdSyncCompact: (params: {
    baseUrl?: string;
    start: number;
    end?: number;
    lightwalletd_url?: string;
    db_path?: string;
  }) =>
    sendMessage<Record<string, unknown>>({
      method: "companion_lwd_sync_compact",
      params
    })
};

const STORAGE_COMPANION_BASE = "nozy_companion_base_url";
const STORAGE_LWD_URL = "nozy_lightwalletd_url";

/** Local Nozy API + optional lightwalletd URL (popup chrome.storage; not sent to sites). */
export async function getCompanionPrefs(): Promise<{ baseUrl: string; lightwalletdUrl: string }> {
  return new Promise((resolve, reject) => {
    chrome.storage.local.get(
      { [STORAGE_COMPANION_BASE]: "http://127.0.0.1:3000", [STORAGE_LWD_URL]: "" },
      (items) => {
        if (chrome.runtime.lastError) {
          reject(new Error(chrome.runtime.lastError.message));
          return;
        }
        resolve({
          baseUrl: String(items[STORAGE_COMPANION_BASE]),
          lightwalletdUrl: String(items[STORAGE_LWD_URL] ?? "")
        });
      }
    );
  });
}

export async function setCompanionPrefs(prefs: {
  baseUrl?: string;
  lightwalletdUrl?: string;
}): Promise<void> {
  return new Promise((resolve, reject) => {
    const patch: Record<string, string> = {};
    if (prefs.baseUrl !== undefined) {
      const u = prefs.baseUrl.trim().replace(/\/+$/, "");
      patch[STORAGE_COMPANION_BASE] = u || "http://127.0.0.1:3000";
    }
    if (prefs.lightwalletdUrl !== undefined) {
      patch[STORAGE_LWD_URL] = prefs.lightwalletdUrl.trim();
    }
    chrome.storage.local.set(patch, () => {
      if (chrome.runtime.lastError) {
        reject(new Error(chrome.runtime.lastError.message));
        return;
      }
      resolve();
    });
  });
}

