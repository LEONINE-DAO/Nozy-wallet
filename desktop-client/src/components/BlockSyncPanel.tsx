import { useEffect, useRef, useState } from "react";
import { Refresh } from "@solar-icons/react";
import { walletApi } from "../lib/api";
import type { SyncStatusResponse } from "../lib/types";
import { useWalletStore } from "../store/walletStore";

interface BlockSyncPanelProps {
  refreshToken?: number;
}

function formatHeight(n: number | null | undefined): string {
  if (n == null) return "—";
  return n.toLocaleString();
}

function syncSummary(status: SyncStatusResponse, isSyncing: boolean): {
  headline: string;
  detail: string;
  tone: "ok" | "warn" | "offline" | "syncing";
  progress: number | null;
} {
  if (isSyncing) {
    const gap = status.scan_gap_blocks ?? 0;
    return {
      headline: gap > 0 ? `Syncing · ${gap.toLocaleString()} blocks behind` : "Syncing with the network…",
      detail: `Scanned ${formatHeight(status.last_scan_height)} · Tip ${formatHeight(status.zebra_tip)}`,
      tone: "syncing",
      progress: progressPercent(status),
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
    headline: "Up to date",
    detail: `Block ${formatHeight(tip)}`,
    tone: "ok",
    progress: 100,
  };
}

function progressPercent(status: SyncStatusResponse): number | null {
  const tip = status.zebra_tip;
  const last = status.last_scan_height;
  if (tip == null || last == null || tip === 0) return null;
  if (last >= tip) return 100;
  return Math.min(100, Math.round((last / tip) * 100));
}

const toneClasses = {
  ok: "border-gray-200/70 bg-gray-50/80 text-gray-600 dark:border-gray-700/50 dark:bg-gray-800/40 dark:text-gray-400",
  warn: "border-amber-200/80 bg-amber-50/60 text-amber-900/90 dark:border-amber-800/50 dark:bg-amber-900/20 dark:text-amber-100/90",
  offline: "border-red-200/80 bg-red-50/60 text-red-800/90 dark:border-red-800/50 dark:bg-red-900/20 dark:text-red-100/90",
  syncing: "border-sky-200/80 bg-sky-50/60 text-sky-900/90 dark:border-sky-800/50 dark:bg-sky-900/20 dark:text-sky-100/90",
};

const barClasses = {
  ok: "bg-emerald-500/70",
  warn: "bg-amber-500/80",
  offline: "bg-red-400/70",
  syncing: "bg-sky-500/80",
};

export function BlockSyncPanel({ refreshToken = 0 }: BlockSyncPanelProps) {
  const isSyncing = useWalletStore((s) => s.isSyncing);
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

  const { headline, detail, tone, progress } = syncSummary(status, isSyncing);
  const nodeBehindWallet =
    status.zebra_tip != null &&
    status.last_scan_height != null &&
    status.last_scan_height > status.zebra_tip;
  const tipStalled = nodeBehindWallet && tipStalePolls >= 2;

  return (
    <div
      className={`mt-3 rounded-lg border px-3 py-2 ${toneClasses[tone]}`}
      aria-live="polite"
    >
      <div className="flex items-center gap-2 min-w-0">
        {isSyncing && (
          <Refresh size={12} className="shrink-0 animate-spin opacity-70" aria-hidden />
        )}
        <div className="min-w-0 flex-1">
          <p className="text-[11px] font-semibold uppercase tracking-wide opacity-70">
            Chain sync
          </p>
          <p className="text-xs font-medium truncate">{headline}</p>
          <p className="text-[11px] opacity-80 truncate mt-0.5">{detail}</p>
          {tipStalled && (
            <p className="text-[11px] mt-1 opacity-90 leading-snug">
              Node tip is not advancing. Restart Zebrad and check network, peers, and system clock.
            </p>
          )}
        </div>
      </div>
      {progress != null && progress < 100 && (
        <div
          className="mt-2 h-1 rounded-full bg-black/5 dark:bg-white/10 overflow-hidden"
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
