import { invoke as tauriInvokeRaw } from "@tauri-apps/api/core";
import {
  WalletExistsResponse,
  WalletProfileInfo,
  NetworkWalletStatusResponse,
  ConfigureNetworkWalletRequest,
  DesktopTestnetWalletRequest,
  DesktopTestnetWalletResponse,
  CreateWalletRequest,
  RestoreWalletRequest,
  UnlockWalletRequest,
  ChangePasswordRequest,
  GenerateAddressResponse,
  BalanceResponse,
  SendTransactionRequest,
  ConfigResponse,
  SetZebraUrlRequest,
  SetThemeRequest,
  ProvingStatusResponse,
  VerifyPasswordRequest as _VerifyPasswordRequest,
  SignMessageRequest,
  SignMessageResponse,
  AddressBookEntry,
  AddAddressBookRequest,
  BackupPathRequest,
  BackupActionResponse,
  SyncStatusResponse,
  OrchardPoolStatsResponse,
  IronwoodDesktopStatusResponse,
  IronwoodStatusRequest,
  IronwoodPlanSaveResponse,
  IronwoodMigrateRequest,
  IronwoodMigrateResponse,
  IronwoodSplitRequest,
  IronwoodSplitResponse,
  IronwoodBroadcastRequest,
  IronwoodBroadcastResponse,
  PrepareCosignRequest,
  PrepareCosignResponse,
  SignCosignRequest,
  CompleteCosignSendRequest,
  KeystoneStatusResponse,
  KeystonePrepareResponse,
  SyncWalletResponse,
} from "./types";

const invoke = async <T>(command: string, args?: Record<string, unknown>): Promise<T> => {
  const hasTauriRuntime =
    typeof window !== "undefined" && typeof (window as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ !== "undefined";

  if (!hasTauriRuntime) {
    throw new Error(
      "Nozy desktop backend is unavailable in browser mode. Launch the Tauri desktop app with `cargo tauri dev`."
    );
  }

  return tauriInvokeRaw<T>(command, args);
};

export const walletApi = {
  checkHealth: async () => {
    return { data: { status: "ok" } };
  },

  checkWalletExists: async (): Promise<{ data: WalletExistsResponse }> => {
    const result = await invoke<WalletExistsResponse>("wallet_exists");
    return { data: result };
  },

  listWalletProfiles: async (): Promise<{ data: WalletProfileInfo[] }> => {
    const result = await invoke<WalletProfileInfo[]>("list_wallet_profiles_cmd");
    return { data: result };
  },

  switchWalletProfile: async (profileId: string) => {
    await invoke("switch_wallet_profile", { request: { profile_id: profileId } });
    return { data: null };
  },

  getNetworkWalletStatus: async (): Promise<{ data: NetworkWalletStatusResponse }> => {
    const result = await invoke<NetworkWalletStatusResponse>("get_network_wallet_status");
    return { data: result };
  },

  configureNetworkWallet: async (
    data: ConfigureNetworkWalletRequest
  ): Promise<{ data: NetworkWalletStatusResponse }> => {
    const result = await invoke<NetworkWalletStatusResponse>("configure_network_wallet", {
      request: data,
    });
    return { data: result };
  },

  createOrRestoreTestnetWallet: async (
    data: DesktopTestnetWalletRequest
  ): Promise<{ data: DesktopTestnetWalletResponse }> => {
    const result = await invoke<DesktopTestnetWalletResponse>("create_or_restore_testnet_wallet", {
      request: data,
    });
    return { data: result };
  },

  createWallet: async (data: CreateWalletRequest): Promise<{ data: string }> => {
    const result = await invoke<string>("create_wallet", { request: data });
    return { data: result };
  },

  restoreWallet: async (data: RestoreWalletRequest) => {
    await invoke("restore_wallet", { request: data });
    return { data: null };
  },

  unlockWallet: async (data: UnlockWalletRequest): Promise<{ data: { exists: boolean; unlocked: boolean; has_password: boolean; address: string } }> => {
    const result = await invoke<{ exists: boolean; unlocked: boolean; has_password: boolean; address: string }>("unlock_wallet", { request: data });
    return { data: result };
  },

  lockWallet: async () => {
    await invoke("lock_wallet");
    return { data: null };
  },

  changePassword: async (data: ChangePasswordRequest) => {
    await invoke("change_password", { request: data });
    return { data: null };
  },

  generateAddress: async (): Promise<{ data: GenerateAddressResponse }> => {
    const result = await invoke<GenerateAddressResponse>("generate_address");
    return { data: result };
  },

  getBalance: async (): Promise<{ data: BalanceResponse }> => {
    const result = await invoke<BalanceResponse>("get_balance");
    return { data: result };
  },

  getSyncStatus: async (): Promise<{ data: SyncStatusResponse }> => {
    const result = await invoke<SyncStatusResponse>("get_sync_status");
    return { data: result };
  },

  getOrchardPoolStats: async (): Promise<{ data: OrchardPoolStatsResponse }> => {
    const result = await invoke<OrchardPoolStatsResponse>("get_orchard_pool_stats");
    return { data: result };
  },

  getIronwoodStatus: async (
    request: IronwoodStatusRequest = {}
  ): Promise<{ data: IronwoodDesktopStatusResponse }> => {
    const result = await invoke<IronwoodDesktopStatusResponse>("get_ironwood_status", {
      request,
    });
    return { data: result };
  },

  ironwoodPlanSave: async (): Promise<{ data: IronwoodPlanSaveResponse }> => {
    const result = await invoke<IronwoodPlanSaveResponse>("ironwood_plan_save");
    return { data: result };
  },

  ironwoodMigrate: async (
    request: IronwoodMigrateRequest = {}
  ): Promise<{ data: IronwoodMigrateResponse }> => {
    const result = await invoke<IronwoodMigrateResponse>("ironwood_migrate", { request });
    return { data: result };
  },

  ironwoodSplit: async (
    request: IronwoodSplitRequest = {}
  ): Promise<{ data: IronwoodSplitResponse }> => {
    const result = await invoke<IronwoodSplitResponse>("ironwood_split", { request });
    return { data: result };
  },

  ironwoodBroadcast: async (
    request: IronwoodBroadcastRequest = {}
  ): Promise<{ data: IronwoodBroadcastResponse }> => {
    const result = await invoke<IronwoodBroadcastResponse>("ironwood_broadcast", { request });
    return { data: result };
  },

  prepareCosignRequest: async (
    data: PrepareCosignRequest
  ): Promise<{ data: PrepareCosignResponse }> => {
    const result = await invoke<PrepareCosignResponse>("prepare_cosign_request", { request: data });
    return { data: result };
  },

  signCosignRequest: async (
    data: SignCosignRequest
  ): Promise<{ data: { pczt_hex: string } }> => {
    const result = await invoke<{ pczt_hex: string }>("sign_cosign_request", { request: data });
    return { data: result };
  },

  completeCosignSend: async (
    data: CompleteCosignSendRequest
  ): Promise<{ data: { success: boolean; txid?: string; message: string } }> => {
    const result = await invoke<{ success: boolean; txid?: string; message: string }>(
      "complete_cosign_send",
      { request: data }
    );
    if (!result.success) {
      throw new Error(result.message || "Co-sign broadcast failed");
    }
    return { data: result };
  },

  getKeystoneStatus: async (): Promise<{ data: KeystoneStatusResponse }> => {
    const result = await invoke<KeystoneStatusResponse>("get_keystone_status");
    return { data: result };
  },

  setKeystoneEnabled: async (
    enabled: boolean,
    deviceLabel?: string
  ): Promise<{ data: { success: boolean; enabled: boolean } }> => {
    const result = await invoke<{ success: boolean; enabled: boolean }>("set_keystone_enabled", {
      request: { enabled, device_label: deviceLabel ?? null },
    });
    return { data: result };
  },

  exportKeystoneUfvk: async (
    password?: string
  ): Promise<{ data: { success: boolean; ufvk: string } }> => {
    const result = await invoke<{ success: boolean; ufvk: string }>("export_keystone_ufvk", {
      request: { password: password ?? null },
    });
    return { data: result };
  },

  keystonePrepareSend: async (params: {
    recipient: string;
    amount: number;
    memo?: string;
    priority?: boolean;
    password?: string;
    zebraUrl?: string;
  }): Promise<{ data: KeystonePrepareResponse }> => {
    const result = await invoke<KeystonePrepareResponse>("keystone_prepare_send", {
      request: {
        recipient: params.recipient,
        amount: params.amount,
        memo: params.memo ?? null,
        priority: params.priority ?? true,
        password: params.password ?? null,
        zebra_url: params.zebraUrl ?? null,
      },
    });
    return { data: result };
  },

  keystoneCompleteSend: async (params: {
    pcztHex?: string;
    urFrames?: string[];
    broadcast?: boolean;
    zebraUrl?: string;
  }): Promise<{ data: { success: boolean; txid?: string; broadcast?: boolean } }> => {
    const result = await invoke<{ success: boolean; txid?: string; broadcast?: boolean }>(
      "keystone_complete_send",
      {
        request: {
          pczt_hex: params.pcztHex ?? null,
          ur_frames: params.urFrames ?? null,
          broadcast: params.broadcast ?? true,
          zebra_url: params.zebraUrl ?? null,
        },
      }
    );
    return { data: result };
  },

  syncWallet: async (data?: {
    start_height?: number;
    end_height?: number;
    zebra_url?: string;
    password?: string;
  }): Promise<{ data: SyncWalletResponse }> => {
    const result = await invoke<SyncWalletResponse>("sync_wallet", { request: data || {} });
    return { data: result };
  },

  sendTransaction: async (data: SendTransactionRequest): Promise<{ data: { success: boolean; txid?: string; message: string } }> => {
    const result = await invoke<{ success: boolean; txid?: string; message: string }>("send_transaction", { request: data });
    if (!result.success) {
      throw new Error(result.message || "Transaction failed");
    }
    return { data: result };
  },

  estimateFee: async (opts?: {
    zebraUrl?: string;
    priority?: boolean;
  }): Promise<{ data: number }> => {
    const result = await invoke<number>("estimate_fee", {
      zebra_url: opts?.zebraUrl,
      priority: opts?.priority ?? true,
    });
    return { data: result };
  },

  getTransactionHistory: async (): Promise<{ data: any[] }> => {
    const result = await invoke<any[]>("get_transaction_history");
    return { data: result };
  },

  getTransaction: async (txid: string): Promise<{ data: any }> => {
    const result = await invoke<any>("get_transaction", { txid });
    return { data: result };
  },

  checkTransactionConfirmations: async (zebraUrl?: string): Promise<{
    data: { pending_updated: number; expired_updated: number; confirmations_updated: number };
  }> => {
    const result = await invoke<{
      pending_updated: number;
      expired_updated: number;
      confirmations_updated: number;
    }>("check_transaction_confirmations", { zebraUrl: zebraUrl ?? null });
    return { data: result };
  },

  speedUpTransaction: async (opts: {
    originalTxid: string;
    password?: string;
    zebraUrl?: string;
  }): Promise<{ data: { success: boolean; txid?: string; original_txid: string; message: string } }> => {
    const result = await invoke<{
      success: boolean;
      txid?: string;
      original_txid: string;
      message: string;
    }>("speed_up_transaction", {
      request: {
        original_txid: opts.originalTxid,
        password: opts.password ?? "",
        zebra_url: opts.zebraUrl ?? null,
      },
    });
    return { data: result };
  },

  getConfig: async (): Promise<{ data: ConfigResponse }> => {
    const result = await invoke<ConfigResponse>("get_config");
    return { data: result };
  },

  setZebraUrl: async (data: SetZebraUrlRequest) => {
    await invoke("set_zebra_url", { request: data });
    return { data: null };
  },

  setTheme: async (_data: SetThemeRequest) => {
    
    return { data: null };
  },

  testZebraConnection: async (zebraUrl?: string): Promise<{ data: string }> => {
    const result = await invoke<string>("test_zebra_connection", { request: { zebra_url: zebraUrl } });
    return { data: result };
  },

  getProvingStatus: async (): Promise<{ data: ProvingStatusResponse }> => {
    const result = await invoke<ProvingStatusResponse>("check_proving_status");
    return { data: result };
  },

  downloadProvingParams: async () => {
    const result = await invoke<string>("download_proving_parameters");
    return { data: result };
  },

  getWalletStatus: async (): Promise<{ data: { exists: boolean; unlocked: boolean; has_password: boolean; address: string | null } }> => {
    const result = await invoke<{ exists: boolean; unlocked: boolean; has_password: boolean; address: string | null }>("get_wallet_status");
    return { data: result };
  },

  getMnemonic: async (data: { password: string }): Promise<{ data: string }> => {
    const result = await invoke<string>("get_mnemonic", { request: data });
    return { data: result };
  },

  getPrivateKey: async (data: { password: string }): Promise<{ data: string }> => {
    const result = await invoke<string>("get_private_key", { request: data });
    return { data: result };
  },

  signMessage: async (data: SignMessageRequest): Promise<{ data: SignMessageResponse }> => {
    const result = await invoke<SignMessageResponse>("sign_message", { request: data });
    return { data: result };
  },

  listAddressBook: async (): Promise<{ data: AddressBookEntry[] }> => {
    const result = await invoke<AddressBookEntry[]>("address_book_list");
    return { data: result ?? [] };
  },

  addAddressBookEntry: async (data: AddAddressBookRequest): Promise<{ data: null }> => {
    await invoke("address_book_add", { request: data });
    return { data: null };
  },

  removeAddressBookEntry: async (name: string): Promise<{ data: boolean }> => {
    const result = await invoke<boolean>("address_book_remove", { name });
    return { data: result ?? false };
  },

  getAddressBookEntry: async (name: string): Promise<{ data: AddressBookEntry | null }> => {
    const result = await invoke<AddressBookEntry | null>("address_book_get", { name });
    return { data: result ?? null };
  },

  searchAddressBook: async (query: string): Promise<{ data: AddressBookEntry[] }> => {
    const result = await invoke<AddressBookEntry[]>("address_book_search", { query });
    return { data: result ?? [] };
  },

  exportBackup: async (data: BackupPathRequest): Promise<{ data: BackupActionResponse }> => {
    const result = await invoke<BackupActionResponse>("export_backup", { request: data });
    return { data: result };
  },

  restoreFromBackup: async (data: BackupPathRequest): Promise<{ data: BackupActionResponse }> => {
    const result = await invoke<BackupActionResponse>("restore_from_backup", { request: data });
    return { data: result };
  },

  listBackups: async (): Promise<{ data: string[] }> => {
    const result = await invoke<string[]>("list_backups");
    return { data: result ?? [] };
  },
};
