/**
 * Crosslink Network observer dashboard (independent finalizer grades).
 * Keep in sync with desktop-client/src/lib/hybridPos.ts
 * and browser-extension/wasm-core/popup/src/lib/hybridPos.ts
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

export type HybridPosStanding =
  | "reliable"
  | "uneven"
  | "unknown"
  | "provisional";

export function normalizeFinalizerHex(hex: string): string {
  return hex.trim().toLowerCase().replace(/^0x/, "");
}

export function isValidFinalizerHex(hex: string): boolean {
  return /^[0-9a-f]{64}$/.test(normalizeFinalizerHex(hex));
}

export async function fetchHybridPosScoreboard(): Promise<HybridPosFinalizer[]> {
  const res = await fetch(`${HYBRID_POS_BASE}/api/scoreboard`);
  if (!res.ok) throw new Error(`Hybrid PoS scoreboard failed (${res.status})`);
  return res.json();
}

export function indexScoreboard(
  rows: HybridPosFinalizer[],
): Map<string, HybridPosFinalizer> {
  const map = new Map<string, HybridPosFinalizer>();
  for (const row of rows) {
    map.set(normalizeFinalizerHex(row.pubkey), row);
  }
  return map;
}

export function hybridPosStanding(
  row: HybridPosFinalizer | null | undefined,
): HybridPosStanding {
  if (!row) return "unknown";
  if (row.provisional) return "provisional";
  if (row.unobserved || row.grade == null) return "unknown";
  if ((row.score ?? 0) >= RELIABLE_MIN_SCORE) return "reliable";
  return "uneven";
}

export type RetargetRiskReason = "offline" | "uneven";

export interface AtRiskBond {
  pk: string;
  latest_val: number;
}

export interface AtRiskDelegation {
  finalizer: string;
  bondPks: string[];
  bondedZat: number;
  reason: RetargetRiskReason;
  suggestion: HybridPosFinalizer | null;
}

export function finalizerNeedsRetarget(
  row: HybridPosFinalizer | null | undefined,
): RetargetRiskReason | null {
  if (!row) return null;
  if (row.live === false) return "offline";
  if (hybridPosStanding(row) === "uneven") return "uneven";
  return null;
}

export function retargetRiskLabel(reason: RetargetRiskReason): string {
  return reason === "offline" ? "offline" : "below B−";
}

export function saferSuggestionPool(
  rows: HybridPosFinalizer[],
  excludePubkeys: string[] = [],
): HybridPosFinalizer[] {
  const exclude = new Set(excludePubkeys.map(normalizeFinalizerHex));
  return rows.filter((row) => {
    const key = normalizeFinalizerHex(row.pubkey);
    if (!isValidFinalizerHex(key) || exclude.has(key)) return false;
    if (row.live === false) return false;
    if (row.in_threshold_set) return false;
    return hybridPosStanding(row) === "reliable";
  });
}

export function pickSaferFinalizer(
  pool: HybridPosFinalizer[],
  seedHex: string,
): HybridPosFinalizer | null {
  if (pool.length === 0) return null;
  const sorted = [...pool].sort((a, b) =>
    normalizeFinalizerHex(a.pubkey).localeCompare(normalizeFinalizerHex(b.pubkey)),
  );
  return sorted[seededIndex(seedHex, sorted.length)] ?? null;
}

export function atRiskDelegations(
  active: Record<string, AtRiskBond[]>,
  rows: HybridPosFinalizer[],
): AtRiskDelegation[] {
  const byKey = indexScoreboard(rows);
  const out: AtRiskDelegation[] = [];
  for (const [finalizer, bonds] of Object.entries(active)) {
    if (!bonds.length) continue;
    const reason = finalizerNeedsRetarget(
      byKey.get(normalizeFinalizerHex(finalizer)),
    );
    if (!reason) continue;
    const pool = saferSuggestionPool(rows, [finalizer]);
    out.push({
      finalizer,
      bondPks: bonds.map((b) => b.pk),
      bondedZat: bonds.reduce((sum, b) => sum + b.latest_val, 0),
      reason,
      suggestion: pickSaferFinalizer(pool, finalizer),
    });
  }
  out.sort((a, b) => {
    if (a.reason !== b.reason) return a.reason === "offline" ? -1 : 1;
    return b.bondedZat - a.bondedZat;
  });
  return out;
}

function seededIndex(seed: string, n: number): number {
  const s = normalizeFinalizerHex(seed);
  let h = 0;
  for (let i = 0; i < s.length; i++) {
    h = (Math.imul(h, 31) + s.charCodeAt(i)) >>> 0;
  }
  return n <= 0 ? 0 : h % n;
}

export type FinalizerCrowdLabel = "largest_third" | "spread";

export function finalizerCrowdLabel(
  row: HybridPosFinalizer | null | undefined,
): FinalizerCrowdLabel | null {
  if (!row) return null;
  return row.in_threshold_set ? "largest_third" : "spread";
}

export function finalizerCrowdCopy(label: FinalizerCrowdLabel): string {
  return label === "largest_third" ? "largest third" : "spread";
}

export function finalizerDisplayName(
  row: HybridPosFinalizer | null | undefined,
): string | null {
  const n = row?.name?.trim();
  return n ? n : null;
}
