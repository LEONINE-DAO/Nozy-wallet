import Constants from "expo-constants";
import { Platform } from "react-native";
import type {
  AddressBookEntry,
  AddressResponse,
  ApiError,
  BalanceResponse,
  ConfigResponse,
  CreateWalletResponse,
  FeeEstimateResponse,
  KeystonePrepareResponse,
  KeystoneStatusResponse,
  SendTransactionResponse,
  SyncResponse,
  TransactionHistoryResponse,
  WalletInfo,
  WalletStatusResponse,
} from "../types";

function defaultApiUrl(): string {
  const fromConfig = Constants.expoConfig?.extra?.defaultApiUrl as
    | string
    | undefined;
  if (fromConfig) return fromConfig;
  if (Platform.OS === "android") return "http://10.0.2.2:3000";
  return "http://localhost:3000";
}

let apiBaseUrl = defaultApiUrl();
let apiKey: string | null = null;

export function getApiBaseUrl(): string {
  return apiBaseUrl;
}

export function setApiBaseUrl(url: string): void {
  apiBaseUrl = url.replace(/\/$/, "");
}

export function getApiKey(): string | null {
  return apiKey;
}

export function setApiKey(key: string | null): void {
  const trimmed = key?.trim();
  apiKey = trimmed ? trimmed : null;
}

async function request<T>(
  path: string,
  options: RequestInit = {},
): Promise<T> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(options.headers as Record<string, string> | undefined),
  };
  if (apiKey) {
    headers["X-API-Key"] = apiKey;
  }

  const response = await fetch(`${apiBaseUrl}${path}`, {
    ...options,
    headers,
  });

  const body = await response.json().catch(() => ({}));

  if (!response.ok) {
    const err = body as ApiError;
    throw new Error(err.error ?? err.message ?? `Request failed (${response.status})`);
  }

  return body as T;
}

export const api = {
  health: () => request<{ status: string }>("/health"),

  walletExists: () => request<WalletInfo>("/api/wallet/exists"),

  walletStatus: () => request<WalletStatusResponse>("/api/wallet/status"),

  createWallet: (password?: string) =>
    request<CreateWalletResponse>("/api/wallet/create", {
      method: "POST",
      body: JSON.stringify({ password: password || null }),
    }),

  restoreWallet: (mnemonic: string, password: string) =>
    request<{ success: boolean }>("/api/wallet/restore", {
      method: "POST",
      body: JSON.stringify({ mnemonic, password }),
    }),

  unlockWallet: (password: string) =>
    request<{ success: boolean }>("/api/wallet/unlock", {
      method: "POST",
      body: JSON.stringify({ password }),
    }),

  generateAddress: (password?: string) =>
    request<AddressResponse>("/api/address/generate", {
      method: "POST",
      body: JSON.stringify({ password: password ?? null }),
    }),

  getBalance: () => request<BalanceResponse>("/api/balance"),

  syncWallet: (password?: string, zebraUrl?: string) =>
    request<SyncResponse>("/api/sync", {
      method: "POST",
      body: JSON.stringify({
        password: password ?? null,
        zebra_url: zebraUrl ?? null,
      }),
    }),

  estimateFee: () =>
    request<FeeEstimateResponse>("/api/transaction/fee-estimate"),

  sendTransaction: (params: {
    recipient: string;
    amount: number;
    memo?: string;
    priority?: boolean;
    password?: string;
    zebraUrl?: string;
  }) =>
    request<SendTransactionResponse>("/api/transaction/send", {
      method: "POST",
      body: JSON.stringify({
        recipient: params.recipient,
        amount: params.amount,
        memo: params.memo ?? null,
        priority: params.priority ?? true,
        zebra_url: params.zebraUrl ?? null,
        password: params.password ?? null,
      }),
    }),

  getTransactionHistory: () =>
    request<TransactionHistoryResponse>("/api/transaction/history"),

  getTransaction: (txid: string) =>
    request<Record<string, unknown>>(`/api/transaction/${encodeURIComponent(txid)}`),

  getConfig: () => request<ConfigResponse>("/api/config"),

  setZebraUrl: (url: string) =>
    request<{ success: boolean }>("/api/config/zebra-url", {
      method: "POST",
      body: JSON.stringify({ url }),
    }),

  setTheme: (theme: "dark" | "light") =>
    request<{ success: boolean }>("/api/config/theme", {
      method: "POST",
      body: JSON.stringify({ theme }),
    }),

  testZebra: (zebraUrl?: string) =>
    request<{ ok: boolean; message: string; block_height?: number }>(
      "/api/config/test-zebra",
      {
        method: "POST",
        body: JSON.stringify({ zebra_url: zebraUrl ?? null }),
      },
    ),

  listAddressBook: () => request<AddressBookEntry[]>("/api/address-book"),

  addAddressBookEntry: (name: string, address: string, notes?: string) =>
    request<{ success: boolean; message: string }>("/api/address-book", {
      method: "POST",
      body: JSON.stringify({ name, address, notes: notes ?? null }),
    }),

  removeAddressBookEntry: (name: string) =>
    request<{ success: boolean }>(`/api/address-book/${encodeURIComponent(name)}`, {
      method: "DELETE",
    }),

  keystoneStatus: () => request<KeystoneStatusResponse>("/api/keystone/status"),

  keystoneEnable: (enabled: boolean, deviceLabel?: string) =>
    request<{ success: boolean; enabled: boolean }>("/api/keystone/enable", {
      method: "POST",
      body: JSON.stringify({
        enabled,
        device_label: deviceLabel ?? null,
      }),
    }),

  keystoneExportUfvk: (password?: string) =>
    request<{ success: boolean; ufvk: string }>("/api/keystone/export-ufvk", {
      method: "POST",
      body: JSON.stringify({ password: password ?? null }),
    }),

  keystonePrepareSend: (params: {
    recipient: string;
    amount: number;
    memo?: string;
    priority?: boolean;
    password?: string;
    zebraUrl?: string;
  }) =>
    request<KeystonePrepareResponse>("/api/keystone/prepare-send", {
      method: "POST",
      body: JSON.stringify({
        recipient: params.recipient,
        amount: params.amount,
        memo: params.memo ?? null,
        priority: params.priority ?? true,
        password: params.password ?? null,
        zebra_url: params.zebraUrl ?? null,
      }),
    }),

  keystoneCompleteSend: (params: {
    pcztHex?: string;
    urFrames?: string[];
    broadcast?: boolean;
    zebraUrl?: string;
  }) =>
    request<{ success: boolean; txid?: string; broadcast?: boolean }>(
      "/api/keystone/complete-send",
      {
        method: "POST",
        body: JSON.stringify({
          pczt_hex: params.pcztHex ?? null,
          ur_frames: params.urFrames ?? null,
          broadcast: params.broadcast ?? true,
          zebra_url: params.zebraUrl ?? null,
        }),
      },
    ),
};
