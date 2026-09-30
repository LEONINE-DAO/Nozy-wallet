import { useEffect, useState } from "react";

import { walletApi } from "../lib/api";
import {
  formatSyncProgressMessage,
  progressPercent,
} from "../lib/syncHelpers";
import type { SyncStatusResponse } from "../lib/types";
import { useWalletStore } from "../store/walletStore";
import { SyncControlButton } from "./SyncControlButton";

interface SyncStatusBannerProps {
  isSyncing?: boolean;
  refreshToken?: number;
}

function bannerTone(status: SyncStatusResponse): "offline" | "warn" | "ok" {
  if (status.zebra_tip == null) return "offline";
  const gap = status.scan_gap_blocks ?? 0;
  if (gap > 0 || !status.witness_fresh_for_send) return "warn";
  return "ok";
}

const tonePanelClass = {
  offline: "nw-sync-panel nw-sync-panel--offline",
  warn: "nw-sync-panel nw-sync-panel--warn",
  ok: "nw-sync-panel",
  syncing: "nw-sync-panel",
};

export function SyncStatusBanner({ isSyncing, refreshToken = 0 }: SyncStatusBannerProps) {
  const [status, setStatus] = useState<SyncStatusResponse | null>(null);
  const syncProgressPercent = useWalletStore((s) => s.syncProgressPercent);
  const syncProgressLabel = useWalletStore((s) => s.syncProgressLabel);
  const isStoppingSync = useWalletStore((s) => s.isStoppingSync);
  const syncPausedByUser = useWalletStore((s) => s.syncPausedByUser);
  const syncStatusEpoch = useWalletStore((s) => s.syncStatusEpoch);

  useEffect(() => {
    let cancelled = false;
    const load = async () => {
      try {
        const res = await walletApi.getSyncStatus();
        if (!cancelled) setStatus(res.data);
      } catch {
        if (!cancelled) setStatus(null);
      }
    };
    load();
    const pollMs = isSyncing ? 2_500 : 30_000;
    const id = setInterval(load, pollMs);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, [refreshToken, isSyncing, syncStatusEpoch]);

  if (!status && !isSyncing) return null;

  const gap = status?.scan_gap_blocks ?? 0;
  const needsSync =
    isSyncing ||
    !status ||
    status.zebra_tip == null ||
    gap > 0 ||
    !status.witness_fresh_for_send;

  if (!needsSync) return null;

  const tone = status
    ? isSyncing
      ? "syncing"
      : bannerTone(status)
    : "syncing";

  const percent = status ? progressPercent(status) : syncProgressPercent;
  const stopping = Boolean(isSyncing && isStoppingSync);
  const message = stopping
    ? "Stopping…"
    : isSyncing
      ? status
        ? formatSyncProgressMessage(status)
        : syncProgressLabel || "Syncing wallet with the network…"
      : syncPausedByUser && gap > 0
        ? `Sync paused · ${gap.toLocaleString()} blocks behind tip.`
        : status?.message || "Wallet is behind the network and not currently syncing.";

  return (
    <div
      className={`shrink-0 border-b px-4 py-3 ${tonePanelClass[tone]}`}
      aria-live="polite"
    >
      <div className="flex flex-wrap items-center justify-between gap-3">
        <p
          className="nw-sync-glitch min-w-0 text-sm font-semibold leading-snug text-emerald-100"
          data-text={message}
        >
          {message}
        </p>
        <SyncControlButton compact={false} />
      </div>
      {isSyncing && percent != null && (
        <div
          className="nw-sync-bar mt-2.5 h-2.5 overflow-hidden rounded-full"
          role="progressbar"
          aria-valuenow={percent}
          aria-valuemin={0}
          aria-valuemax={100}
          aria-label={
            percent != null && (status?.scan_gap_blocks ?? 0) > 0
              ? `Catch-up ${percent} percent`
              : `Wallet scan ${percent} percent`
          }
        >
          <div
            className="nw-sync-bar-fill h-full rounded-full transition-all duration-500"
            style={{ width: `${percent}%` }}
          />
        </div>
      )}
    </div>
  );
}
