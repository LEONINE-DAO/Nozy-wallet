import { invoke } from "@tauri-apps/api/tauri";
import {
  WalletExistsResponse,
  CreateWalletRequest,
  RestoreWalletRequest,
  UnlockWalletRequest,
  GenerateAddressResponse,
  BalanceResponse,
  SendTransactionRequest,
  ConfigResponse,
  SetZebraUrlRequest,
  SetThemeRequest,
  ProvingStatusResponse,
} from "./types";

export const walletApi = {
  // Health - Tauri doesn't need this, wallet operations are direct
  checkHealth: async () => {
    // For Tauri, we can check wallet status instead
    return { data: { status: "ok" } };
  },

  // Wallet
  checkWalletExists: async (): Promise<{ data: WalletExistsResponse }> => {
    const result = await invoke<WalletExistsResponse>("wallet_exists");
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

  // Address
  generateAddress: async (): Promise<{ data: GenerateAddressResponse }> => {
    const result = await invoke<GenerateAddressResponse>("generate_address");
    return { data: result };
  },

  // Balance
  getBalance: async (): Promise<{ data: BalanceResponse }> => {
    const result = await invoke<BalanceResponse>("get_balance");
    return { data: result };
  },

  // Sync
  syncWallet: async (data?: { start_height?: number; end_height?: number; zebra_url?: string; password?: string }) => {
    const result = await invoke("sync_wallet", { request: data || {} });
    return { data: result };
  },

  // Transactions
  sendTransaction: async (data: SendTransactionRequest): Promise<{ data: { success: boolean; txid?: string; message: string } }> => {
    const result = await invoke<{ success: boolean; txid?: string; message: string }>("send_transaction", { request: data });
    return { data: result };
  },

  estimateFee: async (zebraUrl?: string): Promise<{ data: number }> => {
    const result = await invoke<number>("estimate_fee", { zebraUrl });
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

  // Config
  getConfig: async (): Promise<{ data: ConfigResponse }> => {
    const result = await invoke<ConfigResponse>("get_config");
    return { data: result };
  },

  setZebraUrl: async (data: SetZebraUrlRequest) => {
    await invoke("set_zebra_url", { request: data });
    return { data: null };
  },

  setTheme: async (data: SetThemeRequest) => {
    // Tauri doesn't have theme setting in our current implementation
    // This could be added later as a frontend-only feature
    return { data: null };
  },

  testZebraConnection: async (zebraUrl?: string): Promise<{ data: string }> => {
    const result = await invoke<string>("test_zebra_connection", { request: { zebra_url: zebraUrl } });
    return { data: result };
  },

  // Proving
  getProvingStatus: async (): Promise<{ data: ProvingStatusResponse }> => {
    const result = await invoke<ProvingStatusResponse>("check_proving_status");
    return { data: result };
  },

  downloadProvingParams: async () => {
    const result = await invoke<string>("download_proving_parameters");
    return { data: result };
  },

  // Additional methods for wallet status
  getWalletStatus: async (): Promise<{ data: { exists: boolean; unlocked: boolean; has_password: boolean; address: string | null } }> => {
    const result = await invoke<{ exists: boolean; unlocked: boolean; has_password: boolean; address: string | null }>("get_wallet_status");
    return { data: result };
  },
};
