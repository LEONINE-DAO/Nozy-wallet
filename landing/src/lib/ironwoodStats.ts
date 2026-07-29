/** NU6.3 / Ironwood mainnet activation (ecosystem PSA). */
export const NU6_3_MAINNET_ACTIVATION_HEIGHT = 3_428_143;

/** ZIP 318 shared anchor bucket interval (blocks). */
export const ZIP318_ANCHOR_BUCKET_INTERVAL_BLOCKS = 256;

/** Rough mainnet block time for ETA copy. */
export const APPROX_SECONDS_PER_BLOCK = 75;

export type IronwoodNetworkStats = {
  tip: number | null;
  activationHeight: number;
  orchardZec: number | null;
  ironwoodZec: number | null;
  migratedPct: number | null;
  blocksSinceActivation: number | null;
  ironwoodActive: boolean;
  fetchedAt: string | null;
  available: boolean;
  error?: string;
};

export type Zip318Window = {
  currentBucket: number;
  nextBucket: number;
  blocksUntilNext: number;
  etaSeconds: number;
};

export function emptyIronwoodStats(error?: string): IronwoodNetworkStats {
  return {
    tip: null,
    activationHeight: NU6_3_MAINNET_ACTIVATION_HEIGHT,
    orchardZec: null,
    ironwoodZec: null,
    migratedPct: null,
    blocksSinceActivation: null,
    ironwoodActive: false,
    fetchedAt: null,
    available: false,
    error,
  };
}

export function computeMigratedPct(
  orchardZec: number,
  ironwoodZec: number
): number | null {
  const total = orchardZec + ironwoodZec;
  if (!(total > 0) || !Number.isFinite(total)) return null;
  return (ironwoodZec / total) * 100;
}

export function zip318WindowAtTip(tip: number): Zip318Window {
  const interval = ZIP318_ANCHOR_BUCKET_INTERVAL_BLOCKS;
  const currentBucket = Math.floor(tip / interval) * interval;
  const nextBucket = currentBucket + interval;
  const blocksUntilNext = Math.max(0, nextBucket - tip);
  return {
    currentBucket,
    nextBucket,
    blocksUntilNext,
    etaSeconds: blocksUntilNext * APPROX_SECONDS_PER_BLOCK,
  };
}

export function formatZec(value: number | null | undefined, digits = 0): string {
  if (value == null || !Number.isFinite(value)) return "—";
  return value.toLocaleString(undefined, {
    maximumFractionDigits: digits,
    minimumFractionDigits: digits,
  });
}

export function formatPct(value: number | null | undefined): string {
  if (value == null || !Number.isFinite(value)) return "—";
  return `${value.toFixed(1)}%`;
}

export function formatDuration(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds <= 0) return "now";
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  if (hours >= 24) {
    const days = Math.floor(hours / 24);
    const remH = hours % 24;
    return remH > 0 ? `~${days}d ${remH}h` : `~${days}d`;
  }
  if (hours > 0) return minutes > 0 ? `~${hours}h ${minutes}m` : `~${hours}h`;
  return `~${Math.max(1, minutes)}m`;
}

/** Resolve stats API URL for Vite base (GitHub Pages) or Vercel root. */
export function ironwoodStatsApiUrl(): string {
  const base = (import.meta.env.BASE_URL || "/").replace(/\/$/, "");
  // Vercel serverless lives at site root `/api/...` even when Vite base is set.
  if (import.meta.env.VERCEL || base === "") {
    return "/api/ironwood-stats";
  }
  // Local Vite / GitHub Pages: try root API first (Vercel), else relative under base.
  return "/api/ironwood-stats";
}

export async function fetchIronwoodNetworkStats(): Promise<IronwoodNetworkStats> {
  try {
    const res = await fetch(ironwoodStatsApiUrl(), {
      headers: { Accept: "application/json" },
    });
    if (!res.ok) {
      return emptyIronwoodStats(`Stats unavailable (${res.status})`);
    }
    const data = (await res.json()) as Partial<IronwoodNetworkStats> & {
      message?: string;
    };
    const orchard = typeof data.orchardZec === "number" ? data.orchardZec : null;
    const ironwood =
      typeof data.ironwoodZec === "number" ? data.ironwoodZec : null;
    const tip = typeof data.tip === "number" ? data.tip : null;
    const activationHeight =
      typeof data.activationHeight === "number"
        ? data.activationHeight
        : NU6_3_MAINNET_ACTIVATION_HEIGHT;
    const migratedPct =
      typeof data.migratedPct === "number"
        ? data.migratedPct
        : orchard != null && ironwood != null
          ? computeMigratedPct(orchard, ironwood)
          : null;
    const blocksSinceActivation =
      tip != null ? Math.max(0, tip - activationHeight) : null;
    return {
      tip,
      activationHeight,
      orchardZec: orchard,
      ironwoodZec: ironwood,
      migratedPct,
      blocksSinceActivation,
      ironwoodActive: tip != null && tip >= activationHeight,
      fetchedAt:
        typeof data.fetchedAt === "string" ? data.fetchedAt : new Date().toISOString(),
      available: tip != null && (orchard != null || ironwood != null),
      error: typeof data.error === "string" ? data.error : data.message,
    };
  } catch {
    return emptyIronwoodStats("Could not reach live chain stats");
  }
}
