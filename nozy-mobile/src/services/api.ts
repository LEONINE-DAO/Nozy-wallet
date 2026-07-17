import Constants from "expo-constants";
import { Platform } from "react-native";
import {
  defaultHostedApiUrl,
  defaultSelfHostedApiUrl,
} from "../lib/connectionPresets";
import { isProductionBuild } from "../lib/buildProfile";
import type {
  AddressBookEntry,
  AddressResponse,
  ApiError,
  BalanceResponse,
  CheckConfirmationsResponse,
  ConfigResponse,
  CreateWalletResponse,
  FeeEstimateResponse,
  IronwoodBroadcastResponse,
  IronwoodMigrateResponse,
  IronwoodPlanSaveResponse,
  IronwoodSplitResponse,
  IronwoodStatusResponse,
  KeystonePrepareResponse,
  KeystoneStatusResponse,
  NetworkWalletStatusResponse,
  PrivacyNetworkResponse,
  ProvingStatusResponse,
  SendTransactionResponse,
  SpeedUpTransactionResponse,
  SyncResponse,
  TestnetWalletResponse,
  TransactionHistoryResponse,
  WalletInfo,
  WalletStatusResponse,
} from "../types";

function defaultApiUrl(): string {
  const fromConfig = Constants.expoConfig?.extra?.defaultApiUrl as
    | string
    | undefined;
  if (fromConfig) return fromConfig;
  if (isProductionBuild()) return defaultHostedApiUrl();
  if (Platform.OS === "android") return defaultSelfHostedApiUrl();
  return "http://localhost:3000";
}

let apiBaseUrl = defaultApiUrl();
let apiKey: string | null = null;

// Preview + production: bake hosted API key so physical-device installs
// can reach nozywallet.leoninedao.org without manual Settings entry.
const bakedHostedApiKey = (Constants.expoConfig?.extra
  ?.hostedApiKey as string | undefined)?.trim();
if (bakedHostedApiKey) {
  apiKey = bakedHostedApiKey;
}

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

  let response: Response;
  try {
    response = await fetch(`${apiBaseUrl}${path}`, {
      ...options,
      headers,
    });
  } catch (e) {
    const detail = e instanceof Error ? e.message : "Network request failed";
    const hint =
      Platform.OS === "android" &&
      (apiBaseUrl.includes("localhost") || apiBaseUrl.includes("127.0.0.1"))
        ? " On the Android emulator use http://10.0.2.2:3000 (not localhost)."
        : " Check that nozywallet-api is running and the API URL in Settings is correct.";
    throw new Error(`${detail}.${hint}`);
  }

  const body = await response.json().catch(() => ({}));

  if (!response.ok) {
    const err = body as ApiError & { message?: string };
    const msg =
      err.error ?? err.message ?? `Request failed (${response.status})`;
    if (
      typeof msg === "string" &&
      msg.toLowerCase().includes("witness does not match")
    ) {
      throw new Error(
        `${msg} Tip: on Dashboard tap Sync to tip (or ask the API to rescan from your oldest unspent note height).`,
      );
    }
    throw new Error(msg);
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

  checkTransactionConfirmations: () =>
    request<CheckConfirmationsResponse>("/api/transaction/check-confirmations", {
      method: "POST",
      body: JSON.stringify({}),
    }),

  speedUpTransaction: (params: {
    originalTxid: string;
    password?: string;
    zebraUrl?: string;
  }) =>
    request<SpeedUpTransactionResponse>("/api/transaction/speed-up", {
      method: "POST",
      body: JSON.stringify({
        original_txid: params.originalTxid,
        password: params.password ?? null,
        zebra_url: params.zebraUrl ?? null,
      }),
    }),

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

  getWalletProfiles: () =>
    request<NetworkWalletStatusResponse>("/api/wallet/profiles"),

  switchWalletProfile: (profileId: string) =>
    request<{ success: boolean }>("/api/wallet/profiles/switch", {
      method: "POST",
      body: JSON.stringify({ profile_id: profileId }),
    }),

  configureNetworkWallet: (params: {
    network: "mainnet" | "testnet";
    profileId?: string;
    zebraUrl?: string;
  }) =>
    request<NetworkWalletStatusResponse>("/api/wallet/profiles/configure-network", {
      method: "POST",
      body: JSON.stringify({
        network: params.network,
        profile_id: params.profileId ?? null,
        zebra_url: params.zebraUrl ?? null,
      }),
    }),

  createOrRestoreTestnetWallet: (params: {
    name?: string;
    password?: string;
    mnemonic?: string;
    rpcUrl?: string;
  }) =>
    request<TestnetWalletResponse>("/api/wallet/profiles/testnet", {
      method: "POST",
      body: JSON.stringify({
        name: params.name ?? null,
        password: params.password ?? null,
        mnemonic: params.mnemonic ?? null,
        rpc_url: params.rpcUrl ?? null,
      }),
    }),

  changePassword: (currentPassword: string, newPassword: string) =>
    request<{ success: boolean; message: string }>("/api/wallet/change-password", {
      method: "POST",
      body: JSON.stringify({
        current_password: currentPassword,
        new_password: newPassword,
      }),
    }),

  revealMnemonic: (password?: string) =>
    request<{ value: string }>("/api/wallet/reveal-mnemonic", {
      method: "POST",
      body: JSON.stringify({ password: password ?? null }),
    }),

  revealPrivateKey: (password?: string) =>
    request<{ value: string }>("/api/wallet/reveal-private-key", {
      method: "POST",
      body: JSON.stringify({ password: password ?? null }),
    }),

  getPrivacyNetwork: () =>
    request<PrivacyNetworkResponse>("/api/config/privacy-network"),

  updatePrivacyNetwork: (params: {
    broadcastViaNymMixnet?: boolean;
    attestPrivateNetwork?: boolean;
    forceClearnet?: boolean;
    requirePrivacyNetwork?: boolean;
  }) =>
    request<PrivacyNetworkResponse>("/api/config/privacy-network", {
      method: "POST",
      body: JSON.stringify({
        broadcast_via_nym_mixnet: params.broadcastViaNymMixnet ?? null,
        attest_private_network: params.attestPrivateNetwork ?? null,
        force_clearnet: params.forceClearnet ?? null,
        require_privacy_network: params.requirePrivacyNetwork ?? null,
      }),
    }),

  getIronwoodStatus: () =>
    request<IronwoodStatusResponse>("/api/ironwood/status"),

  ironwoodPlan: () =>
    request<IronwoodPlanSaveResponse>("/api/ironwood/plan", {
      method: "POST",
      body: JSON.stringify({}),
    }),

  ironwoodSplit: (params?: { password?: string; dryRun?: boolean }) =>
    request<IronwoodSplitResponse>("/api/ironwood/split", {
      method: "POST",
      body: JSON.stringify({
        password: params?.password ?? null,
        dry_run: params?.dryRun ?? false,
      }),
    }),

  ironwoodMigrate: (password?: string) =>
    request<IronwoodMigrateResponse>("/api/ironwood/migrate", {
      method: "POST",
      body: JSON.stringify({ password: password ?? null }),
    }),

  ironwoodBroadcast: (params?: {
    password?: string;
    attestPrivateNetwork?: boolean;
    forceClearnet?: boolean;
    dryRun?: boolean;
    waitConfirm?: boolean;
  }) =>
    request<IronwoodBroadcastResponse>("/api/ironwood/broadcast", {
      method: "POST",
      body: JSON.stringify({
        password: params?.password ?? null,
        attest_private_network: params?.attestPrivateNetwork ?? null,
        force_clearnet: params?.forceClearnet ?? null,
        dry_run: params?.dryRun ?? false,
        wait_confirm: params?.waitConfirm ?? false,
      }),
    }),

  provingStatus: () => request<ProvingStatusResponse>("/api/proving/status"),

  downloadProvingParameters: () =>
    request<string>("/api/proving/download", {
      method: "POST",
      body: JSON.stringify({}),
    }),
};
