import { useEffect, useState } from "react";

import { walletApi } from "../lib/api";
import {
  formatSyncProgressMessage,
  progressPercent,
} from "../lib/syncHelpers";
import type { SyncStatusResponse } from "../lib/types";
import { useWalletStore } from "../store/walletStore";
import { Button } from "./Button";

interface SyncStatusBannerProps {
  onSync?: () => void;
  isSyncing?: boolean;
  refreshToken?: number;
}

function bannerTone(status: SyncStatusResponse): "offline" | "warn" | "info" {
  if (status.zebra_tip == null) return "offline";
  const gap = status.scan_gap_blocks ?? 0;
  if (gap > 0 || !status.witness_fresh_for_send) return "warn";
  return "info";
}

const toneClasses = {
  offline:
    "bg-red-100 dark:bg-red-950/50 border-red-300 dark:border-red-700 text-red-950 dark:text-red-50",
  warn:
    "bg-amber-100 dark:bg-amber-950/50 border-amber-300 dark:border-amber-700 text-amber-950 dark:text-amber-50",
  info:
    "bg-sky-100 dark:bg-sky-950/50 border-sky-300 dark:border-sky-700 text-sky-950 dark:text-sky-50",
  syncing:
    "bg-sky-100 dark:bg-sky-950/50 border-sky-300 dark:border-sky-700 text-sky-950 dark:text-sky-50",
};

export function SyncStatusBanner({ onSync, isSyncing, refreshToken = 0 }: SyncStatusBannerProps) {
  const [status, setStatus] = useState<SyncStatusResponse | null>(null);
  const syncProgressPercent = useWalletStore((s) => s.syncProgressPercent);
  const syncProgressLabel = useWalletStore((s) => s.syncProgressLabel);

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
  }, [refreshToken, isSyncing]);

  if (!status && !isSyncing) return null;

  const needsSync =
    isSyncing ||
    !status ||
    status.zebra_tip == null ||
    (status.scan_gap_blocks ?? 0) > 0 ||
    !status.witness_fresh_for_send;

  if (!needsSync) return null;

  const tone = status
    ? isSyncing
      ? "syncing"
      : bannerTone(status)
    : "syncing";

  const percent =
    syncProgressPercent ?? (status ? progressPercent(status) : null);
  const message = isSyncing
    ? syncProgressLabel ||
      (status ? formatSyncProgressMessage(status) : "Syncing wallet with the network…")
    : status?.message || "Wallet needs to sync with the network.";

  return (
    <div
      className={`shrink-0 px-4 py-3 border-b-2 flex flex-col gap-2 text-sm ${toneClasses[tone]}`}
      aria-live="polite"
    >
      <div className="flex items-center justify-between gap-3 flex-wrap">
        <p className="font-semibold text-base min-w-0 leading-snug">{message}</p>
        {onSync && !isSyncing && (
          <Button size="sm" onClick={onSync} className="shrink-0 font-semibold">
            Sync to tip
          </Button>
        )}
      </div>
      {isSyncing && percent != null && (
        <div
          className="h-2.5 rounded-full bg-black/20 dark:bg-white/20 overflow-hidden"
          role="progressbar"
          aria-valuenow={percent}
          aria-valuemin={0}
          aria-valuemax={100}
          aria-label={`Wallet sync ${percent} percent`}
        >
          <div
            className="h-full rounded-full bg-sky-600 dark:bg-sky-400 transition-all duration-500"
            style={{ width: `${percent}%` }}
          />
        </div>
      )}
    </div>
  );
}
