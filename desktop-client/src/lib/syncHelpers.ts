import { walletApi } from "./api";
import type { BalanceResponse, SyncStatusResponse } from "./types";

export type BalanceSnapshot = {
  available: number;
  confirmed: number;
  pending: number;
};

export function balanceFromResponse(data: BalanceResponse): BalanceSnapshot {
  return {
    available: data.available_zec ?? data.balance ?? 0,
    confirmed: data.confirmed_zec ?? data.verified_balance ?? 0,
    pending: data.pending_zec ?? 0,
  };
}

export function isWalletReadyForSend(status: SyncStatusResponse): {
  ready: boolean;
  reason?: string;
} {
  if (status.zebra_tip == null) {
    return {
      ready: false,
      reason: "Zebra node is unreachable. Check Network settings, then sync again.",
    };
  }

  const lastScan = status.last_scan_height;
  if (lastScan != null && lastScan > status.zebra_tip) {
    return {
      ready: false,
      reason: `Zebra node is still syncing (tip ${status.zebra_tip.toLocaleString()}, wallet scanned to ${lastScan.toLocaleString()}). Wait for the node to catch up before sending.`,
    };
  }

  const gap = status.scan_gap_blocks ?? 0;
  if (gap > 0) {
    return {
      ready: false,
      reason: `Wallet scan is ${gap} blocks behind chain tip. Sync to tip before sending.`,
    };
  }

  if (!status.witness_fresh_for_send) {
    return {
      ready: false,
      reason: `Orchard witness is ${status.witness_lag_blocks} blocks behind (max ${status.max_send_witness_lag_blocks}). Sync to tip before sending.`,
    };
  }

  return { ready: true };
}

export function isWalletCaughtUp(status: SyncStatusResponse): boolean {
  const gap = status.scan_gap_blocks ?? 0;
  return gap === 0 && status.witness_fresh_for_send && status.zebra_tip != null;
}

export function needsWalletSync(status: SyncStatusResponse): boolean {
  return !isWalletCaughtUp(status);
}

export async function refreshWalletAddress(): Promise<string | null> {
  try {
    const statusRes = await walletApi.getWalletStatus();
    if (statusRes.data.unlocked && statusRes.data.address) {
      return statusRes.data.address;
    }
  } catch {
    // Best-effort: fall through to generate_address.
  }

  try {
    const addressRes = await walletApi.generateAddress();
    return addressRes.data?.address ?? null;
  } catch {
    return null;
  }
}

export async function refreshBalanceSnapshot(): Promise<BalanceSnapshot | null> {
  try {
    const balanceRes = await walletApi.getBalance();
    const data = balanceRes?.data;
    if (data && typeof data === "object") {
      return balanceFromResponse(data);
    }
  } catch {
    // Best-effort refresh.
  }
  return null;
}

export async function syncWalletAndRefresh(): Promise<SyncStatusResponse | null> {
  await walletApi.syncWallet();
  await walletApi.checkTransactionConfirmations().catch(() => undefined);
  await refreshBalanceSnapshot();
  try {
    const statusRes = await walletApi.getSyncStatus();
    return statusRes.data;
  } catch {
    return null;
  }
}
