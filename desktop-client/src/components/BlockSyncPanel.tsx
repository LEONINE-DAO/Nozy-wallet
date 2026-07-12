import { useEffect, useRef, useState } from "react";
import { Refresh } from "@solar-icons/react";
import { walletApi } from "../lib/api";
import { progressPercent } from "../lib/syncHelpers";
import type { SyncStatusResponse } from "../lib/types";
import { useWalletStore } from "../store/walletStore";

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
  livePercent: number | null,
): {
  headline: string;
  detail: string;
  tone: "ok" | "warn" | "offline" | "syncing";
  progress: number | null;
} {
  if (isSyncing) {
    const gap = status.scan_gap_blocks ?? 0;
    const pct = livePercent ?? progressPercent(status);
    return {
      headline:
        pct != null
          ? `${pct}% synced${gap > 0 ? ` · ${gap.toLocaleString()} blocks behind` : ""}`
          : gap > 0
            ? `Syncing · ${gap.toLocaleString()} blocks behind`
            : "Syncing with the network…",
      detail: `Scanned ${formatHeight(status.last_scan_height)} · Tip ${formatHeight(status.zebra_tip)}`,
      tone: "syncing",
      progress: pct,
    };
  }

  if (status.zebra_tip == null) {
    return {
      headline: "Node unreachable",
      detail: "Check Network settings and that Zebrad is running",
      tone: "offline",
      progress: null,
    };
  }

  const tip = status.zebra_tip;
  const last = status.last_scan_height;
  const gap = status.scan_gap_blocks ?? 0;

  if (last != null && last > tip) {
    const behind = (last - tip).toLocaleString();
    return {
      headline: `Node ${behind} blocks behind wallet`,
      detail: `Node tip ${formatHeight(tip)} · Wallet scanned ${formatHeight(last)}`,
      tone: "warn",
      progress: null,
    };
  }

  if (last == null) {
    return {
      headline: "Not scanned yet",
      detail: `Chain tip ${formatHeight(tip)} · run sync to scan notes`,
      tone: "warn",
      progress: 0,
    };
  }

  if (gap > 0) {
    return {
      headline: `${gap.toLocaleString()} block${gap === 1 ? "" : "s"} behind`,
      detail: `Scanned ${formatHeight(last)} · Tip ${formatHeight(tip)}`,
      tone: "warn",
      progress: progressPercent(status),
    };
  }

  if (!status.witness_fresh_for_send && status.witness_lag_blocks > 0) {
    return {
      headline: "Scan at tip · witness updating",
      detail: `Witness ${status.witness_lag_blocks.toLocaleString()} blocks behind (max ${status.max_send_witness_lag_blocks})`,
      tone: "warn",
      progress: null,
    };
  }

  return {
    headline: "100% synced",
    detail: `Tip ${formatHeight(tip)}`,
    tone: "ok",
    progress: 100,
  };
}

const toneClasses = {
  ok: "border-emerald-300 bg-emerald-50 text-emerald-950 dark:border-emerald-700/60 dark:bg-emerald-950/40 dark:text-emerald-50",
  warn: "border-amber-300 bg-amber-50 text-amber-950 dark:border-amber-700/60 dark:bg-amber-950/40 dark:text-amber-50",
  offline: "border-red-300 bg-red-50 text-red-950 dark:border-red-700/60 dark:bg-red-950/40 dark:text-red-50",
  syncing: "border-sky-300 bg-sky-50 text-sky-950 dark:border-sky-700/60 dark:bg-sky-950/40 dark:text-sky-50",
};

const barClasses = {
  ok: "bg-emerald-500/70",
  warn: "bg-amber-500/80",
  offline: "bg-red-400/70",
  syncing: "bg-sky-500/80",
};

export function BlockSyncPanel({ refreshToken = 0 }: BlockSyncPanelProps) {
  const isSyncing = useWalletStore((s) => s.isSyncing);
  const syncProgressPercent = useWalletStore((s) => s.syncProgressPercent);
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
  }, [refreshToken, isSyncing]);

  if (loading && !status) {
    return (
      <div className="mt-3 rounded-lg border border-gray-200/60 bg-white/40 px-3 py-2 text-xs text-gray-400">
        Chain sync · loading…
      </div>
    );
  }

  if (!status) {
    return (
      <div className="mt-3 rounded-lg border border-gray-200/60 bg-white/40 px-3 py-2 text-xs text-gray-500">
        Chain sync · status unavailable
      </div>
    );
  }

  const { headline, detail, tone, progress } = syncSummary(
    status,
    isSyncing,
    syncProgressPercent,
  );
  const nodeBehindWallet =
    status.zebra_tip != null &&
    status.last_scan_height != null &&
    status.last_scan_height > status.zebra_tip;
  const tipStalled = nodeBehindWallet && tipStalePolls >= 2;

  return (
    <div
      className={`mt-3 mb-2 rounded-xl border-2 px-4 py-3 shadow-sm ${toneClasses[tone]}`}
      aria-live="polite"
    >
      <div className="flex items-center gap-3 min-w-0">
        {isSyncing && (
          <Refresh size={16} className="shrink-0 animate-spin" aria-hidden />
        )}
        <div className="min-w-0 flex-1">
          <p className="text-xs font-bold uppercase tracking-wider text-current/70">
            Chain sync
          </p>
          <p className="text-base font-bold truncate mt-0.5">{headline}</p>
          <p className="text-sm font-medium truncate mt-0.5 text-current/80">{detail}</p>
          {tipStalled && (
            <p className="text-sm mt-1.5 font-medium leading-snug">
              Node tip is not advancing. Restart Zebrad and check network, peers, and system clock.
            </p>
          )}
        </div>
        {progress != null && (
          <span className="shrink-0 text-lg font-extrabold tabular-nums">{progress}%</span>
        )}
      </div>
      {progress != null && (
        <div
          className="mt-3 h-2.5 rounded-full bg-black/15 dark:bg-white/20 overflow-hidden"
          role="progressbar"
          aria-valuenow={progress}
          aria-valuemin={0}
          aria-valuemax={100}
        >
          <div
            className={`h-full rounded-full transition-all duration-500 ${barClasses[tone]}`}
            style={{ width: `${progress}%` }}
          />
        </div>
      )}
    </div>
  );
}
