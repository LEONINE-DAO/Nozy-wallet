import { walletApi } from "./api";
import type { BalanceResponse, SyncStatusResponse } from "./types";

export type { SyncStatusResponse };

export type SyncOutcomeKind = "success" | "info" | "warning";

export interface SyncOutcome {
  status: SyncStatusResponse | null;
  caughtUp: boolean;
  kind: SyncOutcomeKind;
  message: string;
}

/** Live progress callback while a multi-round sync is running. */
export interface SyncProgressUpdate {
  message: string;
  percent: number | null;
  status: SyncStatusResponse | null;
}

const MAX_SYNC_ROUNDS = 50;

/** Scan progress 0–100 from last scanned height vs chain tip. */
export function progressPercent(status: SyncStatusResponse | null | undefined): number | null {
  if (!status) return null;
  const tip = status.zebra_tip;
  const last = status.last_scan_height;
  if (tip == null || tip === 0) return null;
  if (last == null) return 0;
  if (last >= tip) return 100;
  return Math.min(100, Math.max(0, Math.round((last / tip) * 100)));
}

/** Short user-facing sync line, e.g. "87% synced · 2.1M / 2.4M". */
export function formatSyncProgressMessage(status: SyncStatusResponse | null): string {
  if (!status) return "Syncing wallet with the network…";

  if (status.zebra_tip == null) {
    return "Connecting to Zebra node…";
  }

  const tip = status.zebra_tip;
  const last = status.last_scan_height;
  const percent = progressPercent(status);
  const tipLabel = tip.toLocaleString();
  const lastLabel = last != null ? last.toLocaleString() : "—";

  if (last != null && last > tip) {
    return `Waiting for node catch-up · tip ${tipLabel} (wallet at ${lastLabel})`;
  }

  if (percent != null && percent < 100) {
    return `${percent}% synced · scanned ${lastLabel} of tip ${tipLabel}`;
  }

  if (!status.witness_fresh_for_send) {
    return `Almost done · updating Orchard witnesses (${status.witness_lag_blocks.toLocaleString()} blocks behind)`;
  }

  return `100% synced · tip ${tipLabel}`;
}

function isNodeBehindWalletScan(status: SyncStatusResponse): boolean {
  const last = status.last_scan_height;
  const tip = status.zebra_tip;
  return last != null && tip != null && last > tip;
}

/** User-facing summary after sync — never claims success while still behind. */
export function describeSyncStatus(status: SyncStatusResponse | null): SyncOutcome {
  if (!status) {
    return {
      status: null,
      caughtUp: false,
      kind: "warning",
      message: "Could not read sync status after sync.",
    };
  }

  if (status.zebra_tip == null) {
    return {
      status,
      caughtUp: false,
      kind: "warning",
      message: "Zebra node unreachable. Check Network settings, then sync again.",
    };
  }

  const tip = status.zebra_tip;
  const last = status.last_scan_height;

  if (last != null && last > tip) {
    const blocks = (last - tip).toLocaleString();
    return {
      status,
      caughtUp: false,
      kind: "info",
      message: `Zebra node is still catching up (${blocks} blocks behind your wallet). Wait for the node to reach block ${last.toLocaleString()}, then sync again.`,
    };
  }

  const gap = status.scan_gap_blocks ?? 0;
  if (gap > 0) {
    return {
      status,
      caughtUp: false,
      kind: "info",
      message: `Wallet scan is ${gap.toLocaleString()} blocks behind tip (scanned ${last?.toLocaleString() ?? "—"}, tip ${tip.toLocaleString()}).`,
    };
  }

  if (!status.witness_fresh_for_send) {
    return {
      status,
      caughtUp: false,
      kind: "info",
      message: `Orchard witness is ${status.witness_lag_blocks.toLocaleString()} blocks behind (need ≤ ${status.max_send_witness_lag_blocks}). Sync again to continue.`,
    };
  }

  return {
    status,
    caughtUp: true,
    kind: "success",
    message: `Wallet synced to block ${tip.toLocaleString()}.`,
  };
}

function emitProgress(
  onProgress: ((update: SyncProgressUpdate) => void) | undefined,
  status: SyncStatusResponse | null,
) {
  if (!onProgress) return;
  onProgress({
    message: formatSyncProgressMessage(status),
    percent: progressPercent(status),
    status,
  });
}

/**
 * Run sync until caught up, the node blocks further progress, or no forward movement.
 * Witness catch-up and large scan gaps may require multiple backend rounds.
 */
export async function syncWalletToTip(
  onProgress?: (update: SyncProgressUpdate) => void,
): Promise<SyncOutcome> {
  let lastStatus: SyncStatusResponse | null = null;
  let prevGap: number | null = null;
  let staleRounds = 0;

  try {
    const statusRes = await walletApi.getSyncStatus();
    lastStatus = statusRes.data;
  } catch {
    lastStatus = null;
  }
  emitProgress(onProgress, lastStatus);

  for (let round = 0; round < MAX_SYNC_ROUNDS; round++) {
    emitProgress(onProgress, lastStatus);

    await walletApi.syncWallet();
    await walletApi.checkTransactionConfirmations().catch(() => undefined);

    try {
      const statusRes = await walletApi.getSyncStatus();
      lastStatus = statusRes.data;
    } catch {
      lastStatus = null;
    }

    emitProgress(onProgress, lastStatus);

    const outcome = describeSyncStatus(lastStatus);
    if (outcome.caughtUp) {
      return outcome;
    }

    if (!lastStatus || isNodeBehindWalletScan(lastStatus)) {
      return outcome;
    }

    const gap = lastStatus.scan_gap_blocks ?? 0;
    if (prevGap !== null && gap === prevGap && gap > 0) {
      staleRounds += 1;
      if (staleRounds >= 2) {
        return outcome;
      }
    } else {
      staleRounds = 0;
    }
    prevGap = gap;

    if (gap > 0 || !lastStatus.witness_fresh_for_send) {
      continue;
    }

    return outcome;
  }

  return describeSyncStatus(lastStatus);
}

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
  const outcome = await syncWalletToTip();
  await refreshBalanceSnapshot();
  return outcome.status;
}
