/**
 * Crosslink Network observer dashboard (independent finalizer grades).
 * https://zcash-hybrid-pos.vercel.app/
 */

export const HYBRID_POS_BASE = "https://zcash-hybrid-pos.vercel.app";

/** Published delegation bar on the observer dashboard (B− and up). */
export const RELIABLE_MIN_SCORE = 80;

export interface HybridPosFinalizer {
  rank: number;
  pubkey: string;
  stake_ctaz: number;
  share_pct: number;
  cumulative_pct: number;
  in_threshold_set: boolean;
  name: string | null;
  website: string | null;
  score: number | null;
  grade: string | null;
  provisional: boolean;
  unobserved: boolean;
  score_note: string | null;
  voted: number | null;
  of: number | null;
  pct: number | null;
  first_seen: number | null;
  live: boolean;
}

export type HybridPosStanding = "reliable" | "uneven" | "unknown" | "provisional";

export function normalizeFinalizerHex(hex: string): string {
  return hex.trim().toLowerCase().replace(/^0x/, "");
}

export function isValidFinalizerHex(hex: string): boolean {
  return /^[0-9a-f]{64}$/.test(normalizeFinalizerHex(hex));
}

export async function fetchHybridPosScoreboard(): Promise<HybridPosFinalizer[]> {
  const res = await fetch(`${HYBRID_POS_BASE}/api/scoreboard`, { cache: "no-store" });
  if (!res.ok) throw new Error(`Hybrid PoS scoreboard failed (${res.status})`);
  return res.json();
}

export async function fetchHybridPosFinalizer(
  pubkey: string,
  hours = 24
): Promise<HybridPosFinalizer | null> {
  const key = normalizeFinalizerHex(pubkey);
  if (!isValidFinalizerHex(key)) return null;
  const res = await fetch(
    `${HYBRID_POS_BASE}/api/finalizer?pubkey=${encodeURIComponent(key)}&hours=${hours}`,
    { cache: "no-store" }
  );
  if (!res.ok) return null;
  return res.json();
}

export function indexScoreboard(rows: HybridPosFinalizer[]): Map<string, HybridPosFinalizer> {
  const map = new Map<string, HybridPosFinalizer>();
  for (const row of rows) {
    map.set(normalizeFinalizerHex(row.pubkey), row);
  }
  return map;
}

export function hybridPosStanding(row: HybridPosFinalizer | null | undefined): HybridPosStanding {
  if (!row) return "unknown";
  if (row.provisional) return "provisional";
  if (row.unobserved || row.grade == null) return "unknown";
  if ((row.score ?? 0) >= RELIABLE_MIN_SCORE) return "reliable";
  return "uneven";
}

export function hybridPosStandingLabel(standing: HybridPosStanding): string {
  switch (standing) {
    case "reliable":
      return "Good pick — B− or better finality participation";
    case "uneven":
      return "Caution — below the B− reliability bar";
    case "provisional":
      return "Provisional — limited observation window";
    default:
      return "No scoreboard data — finalizer not observed recently";
  }
}
