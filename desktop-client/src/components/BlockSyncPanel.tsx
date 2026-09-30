import { useEffect, useRef, useState } from "react";
import { Refresh } from "@solar-icons/react";
import { walletApi } from "../lib/api";
import { progressPercent } from "../lib/syncHelpers";
import type { SyncStatusResponse } from "../lib/types";
import { useWalletStore } from "../store/walletStore";
import { SyncControlButton } from "./SyncControlButton";

interface BlockSyncPanelProps {
  refreshToken?: number;
}

function formatHeight(n: number | null | undefined): string {
  if (n == null) return "—";
  return n.toLocaleString();
}

function syncSummary(
  status: SyncStatusResponse,
  isSyncing: boolean,
  isStopping: boolean,
  paused: boolean,
  livePercent: number | null,
): {
  headline: string;
  detail: string;
  tone: "ok" | "warn" | "offline" | "syncing";
  progress: number | null;
  gap: number;
} {
  const tip = status.zebra_tip;
  const last = status.last_scan_height;
  const gap = status.scan_gap_blocks ?? 0;

  if (isStopping) {
    return {
      headline: "Stopping…",
      detail: `Scanned ${formatHeight(last)} · Tip ${formatHeight(tip)}`,
      tone: "warn",
      progress: livePercent ?? progressPercent(status),
      gap,
    };
  }

  if (isSyncing) {
    if (gap > 0) {
      return {
        headline: `Syncing · ${gap.toLocaleString()} blocks left`,
        detail: `${gap.toLocaleString()} behind · scanned ${formatHeight(last)} of tip ${formatHeight(tip)}`,
        tone: "syncing",
        progress: progressPercent(status) ?? livePercent,
        gap,
      };
    }
    if (!status.witness_fresh_for_send) {
      return {
        headline: `Scan at tip · witnesses ${status.witness_lag_blocks.toLocaleString()} behind`,
        detail: `Need ≤ ${status.max_send_witness_lag_blocks} to send`,
        tone: "syncing",
        progress: null,
        gap,
      };
    }
    return {
      headline: "Syncing with the network…",
      detail: `Scanned ${formatHeight(last)} · Tip ${formatHeight(tip)}`,
      tone: "syncing",
      progress: progressPercent(status) ?? livePercent,
      gap,
    };
  }

  if (status.zebra_tip == null) {
    return {
      headline: "Node unreachable",
      detail: "Check Network settings and that Zebrad is running",
      tone: "offline",
      progress: null,
      gap,
    };
  }

  if (last != null && last > tip!) {
    const behind = (last - tip!).toLocaleString();
    return {
      headline: `Node ${behind} blocks behind wallet`,
      detail: `Node tip ${formatHeight(tip)} · Wallet scanned ${formatHeight(last)}`,
      tone: "warn",
      progress: null,
      gap,
    };
  }

  if (last == null) {
    return {
      headline: "Not scanned yet",
      detail: `Chain tip ${formatHeight(tip)} · tap Sync to scan notes`,
      tone: "warn",
      progress: 0,
      gap,
    };
  }

  if (gap > 0) {
    return {
      headline: paused
        ? `Sync paused · ${gap.toLocaleString()} blocks behind`
        : `${gap.toLocaleString()} block${gap === 1 ? "" : "s"} behind · not syncing`,
      detail: `Scanned ${formatHeight(last)} of tip ${formatHeight(tip)}`,
      tone: "warn",
      progress: null,
      gap,
    };
  }

  if (!status.witness_fresh_for_send && status.witness_lag_blocks > 0) {
    return {
      headline: "Scan at tip · witness updating",
      detail: `Witness ${status.witness_lag_blocks.toLocaleString()} blocks behind (max ${status.max_send_witness_lag_blocks})`,
      tone: "warn",
      progress: null,
      gap,
    };
  }

  return {
    headline: "Caught up",
    detail: `Tip ${formatHeight(tip)}`,
    tone: "ok",
    progress: 100,
    gap: 0,
  };
}

const tonePanelClass = {
  ok: "nw-sync-panel",
  warn: "nw-sync-panel nw-sync-panel--warn",
  offline: "nw-sync-panel nw-sync-panel--offline",
  syncing: "nw-sync-panel",
};

export function BlockSyncPanel({ refreshToken = 0 }: BlockSyncPanelProps) {
  const isSyncing = useWalletStore((s) => s.isSyncing);
  const isStoppingSync = useWalletStore((s) => s.isStoppingSync);
  const syncPausedByUser = useWalletStore((s) => s.syncPausedByUser);
  const syncProgressPercent = useWalletStore((s) => s.syncProgressPercent);
  const syncStatusEpoch = useWalletStore((s) => s.syncStatusEpoch);
  const [status, setStatus] = useState<SyncStatusResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [tipStalePolls, setTipStalePolls] = useState(0);
  const prevTipRef = useRef<number | null>(null);

  useEffect(() => {
    let cancelled = false;

    const load = async () => {
      try {
        const res = await walletApi.getSyncStatus();
        if (!cancelled) {
          const tip = res.data.zebra_tip;
          if (tip != null) {
            if (prevTipRef.current === tip) {
              setTipStalePolls((n) => n + 1);
            } else {
              setTipStalePolls(0);
            }
            prevTipRef.current = tip;
          }
          setStatus(res.data);
          setLoading(false);
        }
      } catch {
        if (!cancelled) {
          setStatus(null);
          setLoading(false);
        }
      }
    };

    load();
    const pollMs = isSyncing ? 5_000 : 10_000;
    const id = setInterval(load, pollMs);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, [refreshToken, isSyncing, syncStatusEpoch]);

  if (loading && !status) {
    return (
      <div className="nw-sync-panel mt-0 rounded-xl border px-3 py-2 text-xs">
        <span className="nw-sync-glitch" data-text="Chain sync · loading…">
          Chain sync · loading…
        </span>
      </div>
    );
  }

  if (!status) {
    return (
      <div className="nw-sync-panel nw-sync-panel--offline mt-0 rounded-xl border px-3 py-2.5 text-xs">
        <p>Chain sync · status unavailable</p>
        <div className="mt-2.5 flex justify-end">
          <SyncControlButton compact />
        </div>
      </div>
    );
  }

  const { headline, detail, tone, progress, gap } = syncSummary(
    status,
    isSyncing,
    isSyncing && isStoppingSync,
    syncPausedByUser,
    syncProgressPercent,
  );
  const nodeBehindWallet =
    status.zebra_tip != null &&
    status.last_scan_height != null &&
    status.last_scan_height > status.zebra_tip;
  const tipStalled = nodeBehindWallet && tipStalePolls >= 2;

  return (
    <div
      className={`mt-0 mb-0 rounded-xl border px-3 py-2.5 ${tonePanelClass[tone]}`}
      aria-live="polite"
    >
      <div className="flex min-w-0 items-center gap-3">
        {isSyncing && !isStoppingSync && (
          <Refresh size={16} className="shrink-0 animate-spin text-emerald-300" aria-hidden />
        )}
        <div className="min-w-0 flex-1">
          <p className="text-[0.65rem] font-bold uppercase tracking-[0.18em] text-emerald-300/70">
            Chain sync
          </p>
          <p
            className="nw-sync-glitch mt-0.5 truncate text-sm font-bold text-emerald-100"
            data-text={headline}
          >
            {headline}
          </p>
          <p
            className="nw-sync-glitch mt-0.5 truncate text-xs font-medium text-emerald-200/80"
            data-text={detail}
          >
            {detail}
          </p>
          {tipStalled && (
            <p className="mt-1.5 text-xs font-medium leading-snug text-amber-200/90">
              Node tip is not advancing. Restart Zebrad and check network, peers, and system clock.
            </p>
          )}
        </div>
        {isSyncing && progress != null && (
          <span
            className="nw-sync-glitch shrink-0 text-lg font-extrabold tabular-nums text-emerald-200"
            data-text={`${progress}%`}
          >
            {progress}%
          </span>
        )}
      </div>
      {progress != null && (isSyncing || gap > 0) && (
        <div
          className="nw-sync-bar mt-2.5 h-2 overflow-hidden rounded-full"
          role="progressbar"
          aria-valuenow={progress}
          aria-valuemin={0}
          aria-valuemax={100}
        >
          <div
            className="nw-sync-bar-fill h-full rounded-full transition-all duration-500"
            style={{ width: `${progress}%` }}
          />
        </div>
      )}
      <div className="mt-2.5 flex justify-end">
        <SyncControlButton compact />
      </div>
    </div>
  );
}
