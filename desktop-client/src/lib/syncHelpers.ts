import { walletApi } from "./api";
import type { BalanceResponse, SyncStatusResponse } from "./types";

export type { SyncStatusResponse };

export type SyncOutcomeKind = "success" | "info" | "warning" | "stopped";

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

let stopRequested = false;
let walletSyncToTipInFlight = false;
/** Scan height/tip when the current catch-up loop started — used for honest % (not last/tip). */
let catchUpAnchor: { last: number; tip: number } | null = null;

export function isWalletSyncToTipInFlight(): boolean {
  return walletSyncToTipInFlight;
}

export function requestStopSync() {
  stopRequested = true;
}

export function clearStopSync() {
  stopRequested = false;
}

function stoppedSyncOutcome(status: SyncStatusResponse | null): SyncOutcome {
  return {
    status,
    caughtUp: false,
    kind: "stopped",
    message: "Sync stopped.",
  };
}

/** Scan progress 0–100. 100 only at tip. While behind, this is catch-up remaining, not last/tip. */
export function progressPercent(status: SyncStatusResponse | null | undefined): number | null {
  if (!status) return null;
  const tip = status.zebra_tip;
  const last = status.last_scan_height;
  if (tip == null || tip === 0) return null;
  if (last == null) return 0;
  if (last >= tip) {
    if (status.witness_fresh_for_send) return 100;
    return null;
  }
  // last/tip is ~99.9% near tip even with hundreds of blocks left — do not use it.
  const start = catchUpAnchor?.last ?? last;
  const dest = Math.max(tip, catchUpAnchor?.tip ?? tip);
  const span = dest - start;
  if (span <= 0) return 0;
  const done = Math.max(0, last - start);
  return Math.min(99, Math.max(0, Math.floor((100 * done) / span)));
}

/** Short user-facing sync line. Never claims 99% synced while blocks remain. */
export function formatSyncProgressMessage(status: SyncStatusResponse | null): string {
  if (!status) return "Syncing wallet with the network…";

  if (status.zebra_tip == null) {
    return "Connecting to Zebra node…";
  }

  const tip = status.zebra_tip;
  const last = status.last_scan_height;
  const tipLabel = tip.toLocaleString();
  const lastLabel = last != null ? last.toLocaleString() : "—";
  const gap = status.scan_gap_blocks ?? (last != null ? Math.max(0, tip - last) : 0);

  if (last != null && last > tip) {
    return `Waiting for node catch-up · tip ${tipLabel} (wallet at ${lastLabel})`;
  }

  if (gap > 0) {
    const pct = progressPercent(status);
    const catchUp =
      catchUpAnchor != null && pct != null ? ` · ${pct}% of this catch-up` : "";
    return `${gap.toLocaleString()} blocks behind tip · scanned ${lastLabel} of ${tipLabel}${catchUp}`;
  }

  if (!status.witness_fresh_for_send) {
    return `Scan at tip · updating Orchard witnesses (${status.witness_lag_blocks.toLocaleString()} blocks behind)`;
  }

  return `Caught up · tip ${tipLabel}`;
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
  walletSyncToTipInFlight = true;
  try {
    return await syncWalletToTipInner(onProgress);
  } finally {
    walletSyncToTipInFlight = false;
    catchUpAnchor = null;
  }
}

async function syncWalletToTipInner(
  onProgress?: (update: SyncProgressUpdate) => void,
): Promise<SyncOutcome> {
  let lastStatus: SyncStatusResponse | null = null;

  try {
    const statusRes = await walletApi.getSyncStatus();
    lastStatus = statusRes.data;
  } catch {
    lastStatus = null;
  }
  if (
    lastStatus?.last_scan_height != null &&
    lastStatus.zebra_tip != null &&
    lastStatus.last_scan_height < lastStatus.zebra_tip
  ) {
    catchUpAnchor = { last: lastStatus.last_scan_height, tip: lastStatus.zebra_tip };
  }
  emitProgress(onProgress, lastStatus);

  for (let round = 0; round < MAX_SYNC_ROUNDS; round++) {
    if (stopRequested) {
      return stoppedSyncOutcome(lastStatus);
    }

    emitProgress(onProgress, lastStatus);

    const pollLive = setInterval(() => {
      void walletApi
        .getSyncStatus()
        .then((res) => {
          lastStatus = res.data;
          emitProgress(onProgress, lastStatus);
        })
        .catch(() => {});
    }, 2500);
    try {
      await walletApi.syncWallet();
    } finally {
      clearInterval(pollLive);
    }
    if (stopRequested) {
      try {
        const statusRes = await walletApi.getSyncStatus();
        lastStatus = statusRes.data;
      } catch {
        lastStatus = null;
      }
      return stoppedSyncOutcome(lastStatus);
    }
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
