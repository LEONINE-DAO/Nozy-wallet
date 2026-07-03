import { useEffect, useState } from "react";
import { walletApi } from "../lib/api";
import type { OrchardPoolStatsResponse } from "../lib/types";

const REFRESH_MS = 60_000;

function formatPoolZec(value: number): string {
  return value.toLocaleString(undefined, {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });
}

export function OrchardPoolBanner() {
  const [stats, setStats] = useState<OrchardPoolStatsResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    const load = async () => {
      try {
        const res = await walletApi.getOrchardPoolStats();
        if (!cancelled) {
          setStats(res.data);
          setError(null);
        }
      } catch {
        if (!cancelled) {
          setError("Zebra node unreachable");
          setStats(null);
        }
      }
    };

    void load();
    const timer = window.setInterval(() => void load(), REFRESH_MS);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, []);

  const poolLabel =
    stats != null
      ? `${formatPoolZec(stats.chain_value_zec)} ZEC`
      : error ?? "ORCHARD POOL :: SYNCING…";

  return (
    <div
      className="relative shrink-0 w-full overflow-hidden border-b border-[#00ff41]/25 bg-black px-6 py-4 font-mono text-sm text-[#00ff41] shadow-[0_0_28px_rgba(0,255,65,0.12)]"
      role="status"
      aria-live="polite"
      aria-label={`Orchard shielded pool total: ${poolLabel}`}
    >
      <div
        className="pointer-events-none absolute inset-0 opacity-[0.14] bg-[linear-gradient(transparent_50%,rgba(0,255,65,0.08)_50%)] bg-size-[100%_4px]"
        aria-hidden="true"
      />
      <div
        className="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_at_center,rgba(0,255,65,0.08),transparent_70%)]"
        aria-hidden="true"
      />

      <div className="relative z-10 flex w-full flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
        <div className="w-full min-w-0">
          <p className="text-[10px] font-bold tracking-[0.25em] text-[#00ff41]/70">
            ORCHARD POOL
          </p>
          <p className="mt-1 w-full break-words text-xl font-bold leading-tight tracking-wide text-[#39ff14] drop-shadow-[0_0_8px_rgba(57,255,20,0.45)] sm:text-2xl tabular-nums">
            {poolLabel}
          </p>
        </div>

        {stats != null && (
          <div className="flex shrink-0 flex-wrap items-center gap-x-4 gap-y-1 text-[10px] uppercase tracking-[0.15em] text-[#00ff41]/60 sm:text-right">
            <span>BLK {stats.block_height.toLocaleString()}</span>
            <span>{stats.monitored ? "MONITORED" : "UNMONITORED"}</span>
          </div>
        )}
      </div>
    </div>
  );
}
