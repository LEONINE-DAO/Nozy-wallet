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
  /** First block height to scan for Orchard notes for this install (create/restore tip, or user-set). */
  orchardBirthdayHeight: number | null;
};

export type TxStateEntry = {
  id: string;
  txid: string | null;
  state: "built" | "broadcast" | "pending" | "confirmed" | "failed" | "expired";
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
  expiryHeight?: number | null;
  priority?: boolean;
  speedUpOf?: string | null;
};

export type PendingApproval = {
  id: string;
  kind: "sign" | "transaction";
  payload: Record<string, unknown>;
  createdAt: number;
};

export type WalletScanProgressResult = {
  status: string;
  startHeight?: number;
  endHeight?: number;
  currentHeight?: number;
  scannedBlocks?: number;
  totalBlocks?: number;
  percent?: number;
  /** Integer 0–100 for stepped progress display. */
  percentInt?: number;
  discoveredNotes?: number;
  totalBalanceZats?: number;
  orchardBalanceZats?: number;
  ironwoodBalanceZats?: number;
  saplingBalanceZats?: number;
  scanError?: string | null;
  /** Last block-level RPC/WASM error while scanning (for diagnostics). */
  lastRpcError?: string | null;
  consecutiveFailures?: number;
  startedAt?: number;
  elapsed?: number;
  /** Set when scan is waiting for session re-hydration after SW restart. */
  sessionWaitingSince?: number | null;
};

export type SendEgressKind =
  | "local"
  | "trusted"
  | "mixnet"
  | "tor"
  | "i2p"
  | "direct_remote"
  | "blocked";

export type SendEgressSnapshot = {
  kind: SendEgressKind;
  label: string;
  connection_mode: string;
  zebra_url: string;
  zebra_url_local: boolean;
  mixnet_requested: boolean;
  mixnet_helper_ok: boolean;
  would_use_mixnet: boolean;
  show_stopgap: boolean;
  stopgap_url: string;
  stopgap_hint: string;
  summary: string;
  detail: string;
};

export type PrivacyNetworkSnapshot = {
  tor_enabled: boolean;
  tor_proxy: string;
  i2p_enabled: boolean;
  i2p_proxy: string;
  preferred_network: string;
  require_privacy_network: boolean;
  broadcast_via_nym_mixnet: boolean;
  sync_via_nym_dvpn: boolean;
  attest_private_network: boolean;
  force_clearnet: boolean;
};

export type NymMixnetReadiness = {
  requested: boolean;
  zebra_url_local: boolean;
  would_use_mixnet: boolean;
  helper_ok: boolean;
  helper_path: string | null;
  helper_error: string | null;
  notes: string[];
};

export type NymDvpnSyncStatus = {
  requested: boolean;
  lwd_url: string;
  lwd_url_local: boolean;
  would_use_dvpn: boolean;
  helper_ok: boolean;
  helper_path: string | null;
  helper_error: string | null;
  mnemonic_env_ok: boolean;
  notes: string[];
};

export type NymDvpnProbeResult = {
  ok: boolean;
  exit_code: number | null;
  helper_path: string;
  lwd_url: string;
  blocks: number;
  stdout_tail: string;
  stderr_tail: string;
  timed_out: boolean;
};

/** Consumer NymVPN OS app — companion probe (not mixnet sendraw). */
export type NymVpnAppStatus = {
  daemon_present: boolean;
  vpnc_found: boolean;
  vpnc_path: string | null;
  connected: boolean;
  mode: string;
  tunnel_state: string;
  browse_allowed: boolean;
  source: string;
  detail: string;
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
  walletRestore: (mnemonic: string, password: string, opts?: { birthdayHeight?: number }) =>
    sendMessage<{ address: string }>({
      method: "wallet_restore",
      params: { mnemonic, password, birthdayHeight: opts?.birthdayHeight }
    }),
  walletReset: () => sendMessage<{ exists: boolean }>({ method: "wallet_reset" }),
  walletSetBirthdayHeight: (height: number) =>
    sendMessage<{ orchardBirthdayHeight: number }>({
      method: "wallet_set_birthday_height",
      params: { height }
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
  walletSpeedUp: (id: string, opts?: { companionPassword?: string }) =>
    sendMessage<{ txid: string }>({
      method: "wallet_speed_up",
      params: { id, companionPassword: opts?.companionPassword, allowWasmFallback: true }
    }),
  rpcSetEndpoint: (url: string) =>
    sendMessage<{ rpcEndpoint: string }>({
      method: "rpc_set_endpoint",
      params: { url }
    }),
  rpcAutodetect: () =>
    sendMessage<{ rpcEndpoint: string; blockCount: number }>({ method: "rpc_autodetect" }),
  rpcConnect: (opts?: { url?: string; tryCompanion?: boolean }) =>
    sendMessage<{
      rpcEndpoint: string;
      blockCount: number;
      connected: boolean;
      source: "manual" | "companion" | "autodetect";
    }>({ method: "rpc_connect", params: opts ?? {} }),
  rpcProbeEndpoint: (url: string) =>
    sendMessage<{ endpoint: string; connected: boolean }>({
      method: "rpc_probe_endpoint",
      params: { url }
    }),
  rpcGetStatus: () =>
    sendMessage<{ endpoint: string; connected: boolean; blockCount?: number | null }>({
      method: "rpc_get_status"
    }),
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
  /**
   * Start background Orchard scan. Pass a number for backward compat (last N blocks).
   * Or pass `{ startHeight, endHeight }` for an explicit inclusive range (from RPC tip),
   * optionally with `endHeight` omitted (defaults to current tip).
   */
  walletStartScan: (
    opts:
      | number
      | {
          window?: number;
          startHeight?: number;
          endHeight?: number;
          useBirthdayRange?: boolean;
          /** Companion wallet password (nozywallet-api data dir); empty string if no password. */
          companionPassword?: string;
        } = 20_000
  ) => {
    const params: Record<string, unknown> =
      typeof opts === "number"
        ? { window: opts }
        : {
            window: opts.window,
            startHeight: opts.startHeight,
            endHeight: opts.endHeight,
            companionPassword: opts.companionPassword,
            ...(opts.useBirthdayRange ? { useBirthdayRange: true } : {})
          };
    return sendMessage<{
      started: boolean;
      startHeight: number;
      endHeight: number;
      status: string;
      rpcEndpoint?: string;
    }>({ method: "wallet_start_scan", params });
  },
  walletScanProgress: () =>
    sendMessage<WalletScanProgressResult>({ method: "wallet_scan_progress" }),
  walletStopScan: () =>
    sendMessage<{ status: string }>({ method: "wallet_stop_scan" }),
  rpcSendRawTx: (rawTxHex: string) =>
    sendMessage<string>({ method: "rpc_send_raw_tx", params: { rawTxHex } }),
  walletEstimateSendFee: (params?: { memo?: string; priority?: boolean }) =>
    sendMessage<{
      fee: number;
      expiry_delta_blocks: number;
      core_version: string;
    }>({
      method: "wallet_estimate_send_fee",
      params: params ?? {}
    }),
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

  companionAddressGenerate: (params: { baseUrl?: string; password?: string }) =>
    sendMessage<{ address: string }>({
      method: "companion_address_generate",
      params
    }),

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
    resume?: boolean;
  }) =>
    sendMessage<Record<string, unknown>>({
      method: "companion_lwd_sync_compact",
      params
    }),

  companionLwdSyncCompactToTip: (params: {
    baseUrl?: string;
    lightwalletd_url?: string;
    db_path?: string;
    start_floor?: number;
    persist_progress_every?: number;
  }) =>
    sendMessage<Record<string, unknown>>({
      method: "companion_lwd_sync_compact_to_tip",
      params
    }),

  companionSaplingStatus: (baseUrl?: string) =>
    sendMessage<{
      unspent_notes: number;
      with_rseed: number;
      ready_to_shield: number;
      unspent_balance_zatoshis: number;
      unspent_zec: number;
      fee_zatoshis: number;
      fee_zec: number;
      has_legacy_balance: boolean;
    }>({ method: "companion_sapling_status", params: { baseUrl } }),

  companionSaplingScan: (params?: {
    baseUrl?: string;
    password?: string;
    start_floor?: number;
    full?: boolean;
  }) =>
    sendMessage<Record<string, unknown>>({
      method: "companion_sapling_scan",
      params: params ?? {}
    }),

  companionSaplingShield: (params?: {
    baseUrl?: string;
    password?: string;
    dry_run?: boolean;
    no_broadcast?: boolean;
  }) =>
    sendMessage<{
      success: boolean;
      txid?: string | null;
      message: string;
      value_to_orchard_zatoshis?: number;
      fee_zatoshis?: number;
    }>({
      method: "companion_sapling_shield",
      params: params ?? {}
    }),

  companionVoteStatus: (params?: { baseUrl?: string; env?: string }) =>
    sendMessage<{
      helper_version: string;
      env: string;
      phase: string;
      phase_message: string;
      notes_exported: boolean;
      notes_count: number | null;
      hotkey_ready: boolean;
      signing_request_present: boolean;
      sig_present: boolean;
      forum_url: string;
      snapshot_utc: string;
      vote_start_utc: string;
      vote_end_utc: string;
    }>({ method: "companion_vote_status", params: params ?? {} }),

  companionVoteActive: (params?: { baseUrl?: string; env?: string }) =>
    sendMessage<{
      vote_round_id: string;
      title?: string | null;
      proposals?: Array<{
        id: number;
        title: string;
        options: Array<{ index: number; label: string }>;
      }>;
    }>({ method: "companion_vote_active", params: params ?? {} }),

  companionVoteExportNotes: (params?: {
    baseUrl?: string;
    password?: string;
    env?: string;
  }) =>
    sendMessage<{
      notes_path: string;
      note_count: number;
      total_value_zat: number;
      message: string;
    }>({ method: "companion_vote_export_notes", params: params ?? {} }),

  walletVoteExportNotes: () =>
    sendMessage<{
      notes_json: string;
      note_count: number;
      total_value_zat: number;
      message: string;
    }>({ method: "wallet_vote_export_notes" }),

  companionVoteImportNotes: (params: { baseUrl?: string; notes_json: string }) =>
    sendMessage<{
      notes_path: string;
      note_count: number;
      total_value_zat: number;
      message: string;
    }>({ method: "companion_vote_import_notes", params }),

  companionVoteSigningRequest: (params?: { baseUrl?: string }) =>
    sendMessage<Record<string, unknown>>({
      method: "companion_vote_signing_request",
      params: params ?? {}
    }),

  walletVoteSignDelegation: (requestJson: string) =>
    sendMessage<{ sig_json: string }>({
      method: "wallet_vote_sign_delegation",
      params: { request_json: requestJson }
    }),

  companionVoteSubmitDelegationSig: (params: {
    baseUrl?: string;
    sig_json: string;
    env?: string;
  }) =>
    sendMessage<{ round_id: string; sig_path: string; message: string }>({
      method: "companion_vote_submit_delegation_sig",
      params
    }),

  companionVotePrepare: (params?: { baseUrl?: string; env?: string }) =>
    sendMessage<{ round_id: string; message: string; stdout: string }>({
      method: "companion_vote_prepare",
      params: params ?? {}
    }),

  companionVoteDelegate: (params?: { baseUrl?: string; env?: string }) =>
    sendMessage<{ message: string; stdout: string }>({
      method: "companion_vote_delegate",
      params: params ?? {}
    }),

  companionVoteSignDelegation: (params?: {
    baseUrl?: string;
    password?: string;
    env?: string;
  }) =>
    sendMessage<{ round_id: string; sig_path: string; message: string }>({
      method: "companion_vote_sign_delegation",
      params: params ?? {}
    }),

  companionVoteDelegateFinish: (params?: {
    baseUrl?: string;
    env?: string;
    wait?: boolean;
  }) =>
    sendMessage<{ tx_hash: string; confirmed: boolean; stdout: string }>({
      method: "companion_vote_delegate_finish",
      params: params ?? {}
    }),

  companionVoteCast: (params: {
    baseUrl?: string;
    env?: string;
    choices: Record<string, number>;
    delegation_tx?: string;
    single_share?: boolean;
    wait?: boolean;
  }) =>
    sendMessage<{ proposal_count: number; stdout: string }>({
      method: "companion_vote_cast",
      params
    }),

  companionCrosslinkStatus: (params?: { baseUrl?: string }) =>
    sendMessage<{
      rpc_url: string;
      height: number;
      staking_day: {
        height: number;
        open: boolean;
        blocks_remaining_in_window: number | null;
        blocks_until_next: number | null;
        cycle: number;
        window: number;
      };
      tfl_activated: boolean | null;
      positions: {
        active: Record<
          string,
          Array<{
            pk: string;
            create_height: number | null;
            initial_val: number;
            latest_val: number;
            finalizer: string | null;
          }>
        >;
        withdrawable: Array<{
          pk: string;
          create_height: number | null;
          initial_val: number;
          latest_val: number;
          finalizer: string | null;
        }>;
      };
      finalizer_count: number | null;
      next_action:
        | { wait_for_staking_day: { blocks: number } }
        | { withdraw_ready: { count: number } }
        | "unbond_to_exit"
        | "stake_or_guardian"
        | "retarget_if_needed";
      privacy_notes: string[];
      wallet: {
        sync_height: number;
        tip_height: number;
        user_shielded_spendable_zats: number;
        user_shielded_pending_zats: number;
        user_unshielded_zats: number;
        staked_zats: number;
        withdrawable_zats: number;
      } | null;
    }>({ method: "companion_crosslink_status", params: params ?? {} }),

  companionCrosslinkWalletStatus: (params?: { baseUrl?: string }) =>
    sendMessage<{
      sync_height: number;
      tip_height: number;
      user_shielded_spendable_zats: number;
      user_shielded_pending_zats: number;
      user_unshielded_zats: number;
      staked_zats: number;
      withdrawable_zats: number;
    }>({ method: "companion_crosslink_wallet_status", params: params ?? {} }),

  companionCrosslinkWalletUfvk: (params?: { baseUrl?: string }) =>
    sendMessage<{ ufvk: string }>({
      method: "companion_crosslink_wallet_ufvk",
      params: params ?? {}
    }),

  companionCrosslinkRoster: (params?: { baseUrl?: string; zats?: boolean }) =>
    sendMessage<
      Array<{ finalizer: string; stake_zat: number; share: number }>
    >({ method: "companion_crosslink_roster", params: params ?? {} }),

  companionCrosslinkStake: (params: {
    baseUrl?: string;
    amount_ctaz: number;
    finalizer: string;
    force?: boolean;
  }) =>
    sendMessage<{ action: string; result: unknown }>({
      method: "companion_crosslink_stake",
      params
    }),

  companionCrosslinkRetarget: (params: {
    baseUrl?: string;
    bond: string;
    finalizer: string;
  }) =>
    sendMessage<{ action: string; result: unknown }>({
      method: "companion_crosslink_retarget",
      params
    }),

  companionCrosslinkUnbond: (params: {
    baseUrl?: string;
    bond: string;
    force?: boolean;
  }) =>
    sendMessage<{ action: string; result: unknown }>({
      method: "companion_crosslink_unbond",
      params
    }),

  companionCrosslinkWithdraw: (params: {
    baseUrl?: string;
    bond: string;
    force?: boolean;
  }) =>
    sendMessage<{ action: string; result: unknown }>({
      method: "companion_crosslink_withdraw",
      params
    }),

  companionZnsResolve: (params: {
    name: string;
    network?: "mainnet" | "testnet";
    baseUrl?: string;
  }) =>
    sendMessage<{
      name: string;
      found: boolean;
      registration?: {
        name: string;
        address: string;
        txid?: string;
        height?: number;
        nonce?: number;
        last_action?: string;
      };
    }>({ method: "companion_zns_resolve", params }),

  companionSendEgress: (baseUrl?: string) =>
    sendMessage<SendEgressSnapshot>({ method: "companion_send_egress", params: { baseUrl } }),

  companionPrivacyNetwork: (baseUrl?: string) =>
    sendMessage<PrivacyNetworkSnapshot>({
      method: "companion_privacy_network",
      params: { baseUrl }
    }),

  companionSetPrivacyNetwork: (params: {
    baseUrl?: string;
    patch: Partial<{
      broadcast_via_nym_mixnet: boolean;
      sync_via_nym_dvpn: boolean;
      attest_private_network: boolean;
      force_clearnet: boolean;
      require_privacy_network: boolean;
    }>;
  }) =>
    sendMessage<PrivacyNetworkSnapshot>({
      method: "companion_set_privacy_network",
      params
    }),

  companionNymMixnet: (baseUrl?: string) =>
    sendMessage<NymMixnetReadiness>({ method: "companion_nym_mixnet", params: { baseUrl } }),

  companionNymDvpn: (baseUrl?: string, lightwalletd_url?: string) =>
    sendMessage<NymDvpnSyncStatus>({
      method: "companion_nym_dvpn",
      params: { baseUrl, lightwalletd_url }
    }),

  companionSetNymDvpn: (params: { baseUrl?: string; enabled: boolean }) =>
    sendMessage<NymDvpnSyncStatus>({ method: "companion_set_nym_dvpn", params }),

  companionNymDvpnProbe: (params: {
    baseUrl?: string;
    lightwalletd_url?: string;
    blocks?: number;
  }) =>
    sendMessage<NymDvpnProbeResult>({ method: "companion_nym_dvpn_probe", params }),

  companionNymVpnApp: (baseUrl?: string) =>
    sendMessage<NymVpnAppStatus>({ method: "companion_nym_vpn_app", params: { baseUrl } })
};

const STORAGE_COMPANION_BASE = "nozy_companion_base_url";
const STORAGE_LWD_URL = "nozy_lightwalletd_url";
const STORAGE_COMPANION_API_KEY = "nozy_companion_api_key_v1";
const DEFAULT_LWD_URL = "http://127.0.0.1:9067";

function normalizeStoredLwdUrl(raw: string): string {
  const s = String(raw ?? "").trim();
  if (!s || /zec\.rocks/i.test(s)) {
    return DEFAULT_LWD_URL;
  }
  return s;
}

/** Local Nozy API + optional lightwalletd URL (popup chrome.storage; not sent to sites). */
export async function getCompanionPrefs(): Promise<{
  baseUrl: string;
  lightwalletdUrl: string;
  apiKey: string;
}> {
  return new Promise((resolve, reject) => {
    chrome.storage.local.get(
      {
        [STORAGE_COMPANION_BASE]: "http://127.0.0.1:3000",
        [STORAGE_LWD_URL]: DEFAULT_LWD_URL,
        [STORAGE_COMPANION_API_KEY]: ""
      },
      (items) => {
        if (chrome.runtime.lastError) {
          reject(new Error(chrome.runtime.lastError.message));
          return;
        }
        resolve({
          baseUrl: String(items[STORAGE_COMPANION_BASE]),
          lightwalletdUrl: normalizeStoredLwdUrl(String(items[STORAGE_LWD_URL] ?? "")),
          apiKey: String(items[STORAGE_COMPANION_API_KEY] ?? "")
        });
      }
    );
  });
}

export async function setCompanionPrefs(prefs: {
  baseUrl?: string;
  lightwalletdUrl?: string;
  apiKey?: string;
}): Promise<void> {
  return new Promise((resolve, reject) => {
    const patch: Record<string, string> = {};
    if (prefs.baseUrl !== undefined) {
      const u = prefs.baseUrl.trim().replace(/\/+$/, "");
      patch[STORAGE_COMPANION_BASE] = u || "http://127.0.0.1:3000";
    }
    if (prefs.lightwalletdUrl !== undefined) {
      patch[STORAGE_LWD_URL] = normalizeStoredLwdUrl(prefs.lightwalletdUrl);
    }
    if (prefs.apiKey !== undefined) {
      patch[STORAGE_COMPANION_API_KEY] = prefs.apiKey.trim();
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

