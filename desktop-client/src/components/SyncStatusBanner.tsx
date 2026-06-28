import { useEffect, useState } from "react";

import { walletApi } from "../lib/api";
import type { SyncStatusResponse } from "../lib/types";
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
    "bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800 text-red-900 dark:text-red-100",
  warn:
    "bg-amber-50 dark:bg-amber-900/20 border-amber-200 dark:border-amber-800 text-amber-900 dark:text-amber-100",
  info:
    "bg-sky-50 dark:bg-sky-900/20 border-sky-200 dark:border-sky-800 text-sky-900 dark:text-sky-100",
};

export function SyncStatusBanner({ onSync, isSyncing, refreshToken = 0 }: SyncStatusBannerProps) {
  const [status, setStatus] = useState<SyncStatusResponse | null>(null);

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
    const id = setInterval(load, 30_000);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, [refreshToken]);

  if (!status) return null;

  const needsSync =
    status.zebra_tip == null ||
    (status.scan_gap_blocks ?? 0) > 0 ||
    !status.witness_fresh_for_send;

  if (!needsSync) return null;

  const tone = bannerTone(status);

  return (
    <div
      className={`shrink-0 px-4 py-2.5 border-b flex items-center justify-between gap-3 flex-wrap text-sm ${toneClasses[tone]}`}
    >
      <p>{isSyncing ? "Catching up with the network…" : status.message}</p>
      {onSync && !isSyncing && (
        <Button size="sm" onClick={onSync} disabled={isSyncing} className="shrink-0">
          Sync to tip
        </Button>
      )}
    </div>
  );
}
