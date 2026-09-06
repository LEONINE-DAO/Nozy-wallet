import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import toast from "react-hot-toast";
import { Button } from "./Button";
import { walletApi } from "../lib/api";
import type {
  CrosslinkGuardianSnapshot,
  CrosslinkNextAction,
  CrosslinkRosterEntry,
} from "../lib/types";
import { formatErrorForDisplay } from "../utils/errors";
import {
  fetchHybridPosFinalizer,
  fetchHybridPosScoreboard,
  hybridPosGradeClass,
  hybridPosStanding,
  hybridPosStandingLabel,
  indexScoreboard,
  isValidFinalizerHex,
  normalizeFinalizerHex,
  type HybridPosFinalizer,
} from "../lib/hybridPos";

/** Status poll (~1 feature-net block). Positions live here; roster polled less often. */
const STATUS_REFRESH_MS = 90_000;
/** Finalizer roster changes slowly — skip on most background ticks. */
const ROSTER_REFRESH_MS = 180_000;
/** Approx feature-net block time for client-side Staking Day countdown between polls. */
const FEATURE_NET_BLOCK_SECS = 90;
/**
 * Module-level write lock + cooldown (survives remount/HMR).
 * Crosslink node rejects overlapping wallet_staking_action with "Another stake in progress".
 */
let crosslinkWriteLock = false;
let crosslinkWriteCooldownUntil = 0;
/** Short cooldown after retarget/unbond/withdraw. */
const WRITE_COOLDOWN_MS = 60_000;
/**
 * Stake confirms can take several minutes (busy reply → later bond).
 * Keep Stake blocked longer so a retry does not mint a second bond.
 */
const STAKE_SETTLE_COOLDOWN_MS = 180_000;

type Busy =
  | null
  | "refresh"
  | "stake"
  | "retarget"
  | "unbond"
  | "withdraw";

function zatToCtaz(zat: number): string {
  return (zat / 1e8).toFixed(8);
}

function shortHex(hex: string, head = 10, tail = 8): string {
  const h = hex.trim();
  if (h.length <= head + tail + 1) return h;
  return `${h.slice(0, head)}…${h.slice(-tail)}`;
}

function formatNextAction(action: CrosslinkNextAction): string {
  if (typeof action === "string") {
    switch (action) {
      case "unbond_to_exit":
        return "Window open with active stake — unbond to start exit, or retarget anytime.";
      case "stake_or_guardian":
        return "Window open — stake to a finalizer, or back your own identity from the monolith UI.";
      case "retarget_if_needed":
        return "Window closed — you can still retarget if a finalizer misbehaves.";
      default:
        return "Refresh status for guidance.";
    }
  }
  if ("wait_for_staking_day" in action) {
    return `Wait ~${action.wait_for_staking_day.blocks} blocks for Staking Day, then stake / unbond / withdraw.`;
  }
  if ("withdraw_ready" in action) {
    return `${action.withdraw_ready.count} bond(s) ready to withdraw while the window is open.`;
  }
  return "Refresh status for guidance.";
}

function availableToStakeZat(snap: CrosslinkGuardianSnapshot): number {
  const w = snap.wallet;
  if (!w) return 0;
  return w.user_shielded_spendable_zats + w.user_unshielded_zats;
}

function bondedZat(snap: CrosslinkGuardianSnapshot): number {
  return Object.values(snap.positions.active ?? {})
    .flat()
    .reduce((sum, b) => sum + b.latest_val, 0);
}

function stakedReportedZat(snap: CrosslinkGuardianSnapshot): number {
  if (snap.wallet && snap.wallet.staked_zats > 0) {
    return snap.wallet.staked_zats;
  }
  return bondedZat(snap);
}

function walletSynced(w: CrosslinkGuardianSnapshot["wallet"]): boolean {
  if (!w) return false;
  return w.tip_height === 0 || w.sync_height + 2 >= w.tip_height;
}

function withdrawableZat(snap: CrosslinkGuardianSnapshot): number {
  return (snap.positions.withdrawable ?? []).reduce((sum, b) => sum + b.latest_val, 0);
}

function rewardsZat(snap: CrosslinkGuardianSnapshot): number {
  const active = Object.values(snap.positions.active ?? {})
    .flat()
    .reduce((sum, b) => sum + Math.max(0, b.latest_val - b.initial_val), 0);
  const withdr = (snap.positions.withdrawable ?? []).reduce(
    (sum, b) => sum + Math.max(0, b.latest_val - b.initial_val),
    0,
  );
  return active + withdr;
}

function activeBondCount(snap: CrosslinkGuardianSnapshot): number {
  return Object.values(snap.positions.active ?? {}).reduce((n, arr) => n + arr.length, 0);
}

export function CrosslinkGuardianCard() {
  const [snap, setSnap] = useState<CrosslinkGuardianSnapshot | null>(null);
  const [roster, setRoster] = useState<CrosslinkRosterEntry[]>([]);
  const [busy, setBusy] = useState<Busy>(null);
  const [loading, setLoading] = useState(true);
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);
  const [amount, setAmount] = useState("0.01");
  const [finalizer, setFinalizer] = useState("");
  const [bondKey, setBondKey] = useState("");
  const [force, setForce] = useState(false);
  const [showKeys, setShowKeys] = useState(false);
  const [ufvk, setUfvk] = useState<string | null>(null);
  const [ufvkLoading, setUfvkLoading] = useState(false);
  const [hybridRows, setHybridRows] = useState<HybridPosFinalizer[]>([]);
  const [hybridLookup, setHybridLookup] = useState<HybridPosFinalizer | null>(null);
  const [hybridLookupLoading, setHybridLookupLoading] = useState(false);
  const [nowMs, setNowMs] = useState(() => Date.now());
  const actionInFlightRef = useRef(false);
  const delayedWatchRef = useRef<number | null>(null);
  const daySnapshotAtRef = useRef<number | null>(null);
  const bondInputRef = useRef<HTMLInputElement | null>(null);
  const [, setSettleTick] = useState(0);

  const load = useCallback(async (opts?: { manual?: boolean; roster?: boolean }) => {
    const manual = opts?.manual ?? false;
    const fetchRoster = opts?.roster ?? manual;
    if (manual) setBusy("refresh");
    try {
      const statusRes = await walletApi.crosslinkStatus();
      setSnap(statusRes.data);
      daySnapshotAtRef.current = Date.now();
      if (fetchRoster) {
        try {
          const rosterRes = await walletApi.crosslinkRoster(false);
          setRoster(rosterRes.data);
        } catch {
          if (manual) setRoster([]);
        }
      }
      setLastUpdated(new Date());
    } catch (e) {
      if (manual) {
        toast.error(formatErrorForDisplay(e, "Failed to load Crosslink status"));
      }
    } finally {
      // Never clear an in-flight stake/retarget busy flag from a Refresh poll.
      if (manual && !actionInFlightRef.current && !crosslinkWriteLock) {
        setBusy(null);
      }
      setLoading(false);
    }
  }, []);

  const loadHybridScoreboard = useCallback(async () => {
    try {
      const rows = await fetchHybridPosScoreboard();
      setHybridRows(rows);
    } catch {
      /* observer optional — roster still works without grades */
    }
  }, []);

  useEffect(() => {
    void loadHybridScoreboard();
    const timer = window.setInterval(() => void loadHybridScoreboard(), ROSTER_REFRESH_MS);
    return () => window.clearInterval(timer);
  }, [loadHybridScoreboard]);

  useEffect(() => {
    const key = normalizeFinalizerHex(finalizer);
    if (!isValidFinalizerHex(key)) {
      setHybridLookup(null);
      setHybridLookupLoading(false);
      return;
    }
    const cached = indexScoreboard(hybridRows).get(key);
    if (cached) {
      setHybridLookup(cached);
      setHybridLookupLoading(false);
      return;
    }
    let cancelled = false;
    setHybridLookupLoading(true);
    const timer = window.setTimeout(() => {
      void (async () => {
        try {
          const row = await fetchHybridPosFinalizer(key);
          if (!cancelled) setHybridLookup(row);
        } catch {
          if (!cancelled) setHybridLookup(null);
        } finally {
          if (!cancelled) setHybridLookupLoading(false);
        }
      })();
    }, 350);
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [finalizer, hybridRows]);

  useEffect(() => {
    let cancelled = false;
    let lastRosterAt = 0;

    const tick = (manual = false) => {
      if (cancelled || document.hidden) return;
      const now = Date.now();
      const fetchRoster = manual || now - lastRosterAt >= ROSTER_REFRESH_MS;
      if (fetchRoster) lastRosterAt = now;
      void load({ manual, roster: fetchRoster });
    };
    tick(true);
    const timer = window.setInterval(() => tick(false), STATUS_REFRESH_MS);
    const onVisible = () => {
      if (!document.hidden) tick(false);
    };
    document.addEventListener("visibilitychange", onVisible);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
      document.removeEventListener("visibilitychange", onVisible);
      if (delayedWatchRef.current != null) {
        window.clearInterval(delayedWatchRef.current);
        delayedWatchRef.current = null;
      }
    };
  }, [load]);

  useEffect(() => {
    const timer = window.setInterval(() => setNowMs(Date.now()), 1_000);
    return () => window.clearInterval(timer);
  }, []);

  const clearDelayedWatch = () => {
    if (delayedWatchRef.current != null) {
      window.clearInterval(delayedWatchRef.current);
      delayedWatchRef.current = null;
      setSettleTick((n) => n + 1);
    }
  };

  const loadUfvk = useCallback(async () => {
    setUfvkLoading(true);
    try {
      const res = await walletApi.crosslinkWalletUfvk();
      setUfvk(res.data.ufvk);
      toast.success("Node wallet UFVK loaded");
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Failed to load node wallet UFVK"));
    } finally {
      setUfvkLoading(false);
    }
  }, []);

  const copyUfvk = useCallback(async () => {
    if (!ufvk) return;
    try {
      await navigator.clipboard.writeText(ufvk);
      toast.success("UFVK copied");
    } catch {
      toast.error("Could not copy to clipboard");
    }
  }, [ufvk]);

  const watchDelayedBond = (beforeBonds: number) => {
    clearDelayedWatch();
    const started = Date.now();
    setSettleTick((n) => n + 1);
    delayedWatchRef.current = window.setInterval(() => {
      void (async () => {
        if (Date.now() - started > STAKE_SETTLE_COOLDOWN_MS) {
          toast(
            "Settle window ended with no new bond — check GUI pending list, then try Stake once if needed.",
            { duration: 10_000, icon: "⚠️" },
          );
          clearDelayedWatch();
          return;
        }
        try {
          const statusRes = await walletApi.crosslinkStatus();
          setSnap(statusRes.data);
          daySnapshotAtRef.current = Date.now();
          const n = activeBondCount(statusRes.data);
          if (n > beforeBonds) {
            toast.success(`Stake confirmed — ${n} active bonds`);
            clearDelayedWatch();
          }
        } catch {
          /* keep watching */
        }
      })();
    }, 15_000);
  };

  const selectBondKey = useCallback((pk: string) => {
    setBondKey(pk);
    toast.success("Bond selected for unbond / withdraw");
    window.requestAnimationFrame(() => {
      bondInputRef.current?.focus();
      bondInputRef.current?.scrollIntoView({ behavior: "smooth", block: "center" });
    });
  }, []);

  const selectFinalizer = useCallback((key: string) => {
    setFinalizer(key);
    toast.success("Finalizer pasted for stake / retarget");
  }, []);

  const stakeSettling = delayedWatchRef.current != null;

  const run = async (
    kind: Busy,
    fn: () => Promise<string | { tone: "warn" | "success"; message: string } | void>,
  ) => {
    if (!kind) return;
    const now = Date.now();
    if (busy || actionInFlightRef.current || crosslinkWriteLock) {
      toast.error("Stake already in progress — wait for the node to finish.");
      return;
    }
    if (kind === "stake" && delayedWatchRef.current != null) {
      toast.error(
        "Previous stake still settling — wait for confirmation (or ~3 min) before staking again.",
      );
      return;
    }
    if (now < crosslinkWriteCooldownUntil) {
      const secs = Math.ceil((crosslinkWriteCooldownUntil - now) / 1000);
      toast.error(
        `Wait ${secs}s — Crosslink only allows one stake at a time. Do not also stake in the GUI.`,
      );
      return;
    }
    crosslinkWriteLock = true;
    actionInFlightRef.current = true;
    setBusy(kind);
    let applyCooldownMs = 0;
    try {
      const okMsg = await fn();
      const tone =
        okMsg && typeof okMsg === "object" && okMsg.tone === "warn" ? "warn" : "success";
      const text =
        typeof okMsg === "string"
          ? okMsg
          : okMsg && typeof okMsg === "object"
            ? okMsg.message
            : "Submitted via Crosslink node wallet";
      applyCooldownMs =
        kind === "stake" ? STAKE_SETTLE_COOLDOWN_MS : WRITE_COOLDOWN_MS;
      if (tone === "warn") {
        toast(text, { duration: 10_000, icon: "⚠️" });
        if (kind === "stake") {
          const beforeBonds = snap ? activeBondCount(snap) : 0;
          watchDelayedBond(beforeBonds);
        }
      } else {
        toast.success(text);
        if (kind === "stake") {
          const beforeBonds = snap ? activeBondCount(snap) : 0;
          watchDelayedBond(beforeBonds);
        }
      }
      await load();
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Crosslink action failed"));
    } finally {
      if (applyCooldownMs > 0) {
        crosslinkWriteCooldownUntil = Date.now() + applyCooldownMs;
      }
      actionInFlightRef.current = false;
      crosslinkWriteLock = false;
      setBusy(null);
      setSettleTick((n) => n + 1);
    }
  };

  const dayLabel = useMemo(() => {
    if (!snap) return "…";
    const d = snap.staking_day;
    const blocks = d.open
      ? d.blocks_remaining_in_window ?? 0
      : d.blocks_until_next ?? 0;
    const snappedAt = daySnapshotAtRef.current ?? nowMs;
    const elapsedBlocks = Math.max(
      0,
      Math.floor((nowMs - snappedAt) / (FEATURE_NET_BLOCK_SECS * 1000)),
    );
    const remainingBlocks = Math.max(0, blocks - elapsedBlocks);
    const etaMin = Math.max(0, Math.round((remainingBlocks * FEATURE_NET_BLOCK_SECS) / 60));
    if (d.open) {
      return `~${remainingBlocks} blocks left · ~${etaMin}m`;
    }
    return `opens in ~${remainingBlocks} blocks · ~${etaMin}m`;
  }, [snap, nowMs]);

  const stakingDayOpen = Boolean(snap?.staking_day.open);

  const stakingDayProgress = useMemo(() => {
    if (!snap) return { pct: 0, mode: "closed" as const };
    const d = snap.staking_day;
    const window = d.window || 70;
    const cycle = d.cycle || 150;
    const blocks = d.open
      ? d.blocks_remaining_in_window ?? 0
      : d.blocks_until_next ?? 0;
    const snappedAt = daySnapshotAtRef.current ?? nowMs;
    const elapsedBlocks = Math.max(
      0,
      Math.floor((nowMs - snappedAt) / (FEATURE_NET_BLOCK_SECS * 1000)),
    );
    const remainingBlocks = Math.max(0, blocks - elapsedBlocks);
    if (d.open) {
      const elapsed = window - remainingBlocks;
      return {
        pct: Math.min(100, Math.max(0, (elapsed / window) * 100)),
        mode: "open" as const,
      };
    }
    const closedSpan = cycle - window;
    const elapsed = closedSpan - remainingBlocks;
    return {
      pct: Math.min(100, Math.max(0, (elapsed / closedSpan) * 100)),
      mode: "closed" as const,
    };
  }, [snap, nowMs]);

  const writeWindowOpen = !snap || snap.staking_day.open || force;

  const bondIsWithdrawable = useMemo(() => {
    if (!snap || !bondKey.trim()) return false;
    const key = bondKey.trim().toLowerCase();
    return (snap.positions.withdrawable ?? []).some((b) => b.pk.toLowerCase() === key);
  }, [snap, bondKey]);

  const bondIsActive = useMemo(() => {
    if (!snap || !bondKey.trim()) return false;
    const key = bondKey.trim().toLowerCase();
    return Object.values(snap.positions.active ?? {})
      .flat()
      .some((b) => b.pk.toLowerCase() === key);
  }, [snap, bondKey]);

  const flatBonds = useMemo(() => {
    if (!snap) {
      return [] as {
        pk: string;
        finalizer: string | null;
        initial: number;
        latest: number;
      }[];
    }
    const out: {
      pk: string;
      finalizer: string | null;
      initial: number;
      latest: number;
    }[] = [];
    for (const [fin, bonds] of Object.entries(snap.positions.active ?? {})) {
      for (const b of bonds) {
        out.push({
          pk: b.pk,
          finalizer: fin,
          initial: b.initial_val,
          latest: b.latest_val,
        });
      }
    }
    for (const b of snap.positions.withdrawable ?? []) {
      out.push({
        pk: b.pk,
        finalizer: null,
        initial: b.initial_val,
        latest: b.latest_val,
      });
    }
    return out;
  }, [snap]);

  /** Distinct finalizers from active bonds (may not appear on the TFL roster). */
  const stakedFinalizers = useMemo(() => {
    if (!snap) return [] as { finalizer: string; bonds: number; totalLatest: number }[];
    const rows = Object.entries(snap.positions.active ?? {}).map(([finalizer, bonds]) => ({
      finalizer,
      bonds: bonds.length,
      totalLatest: bonds.reduce((sum, b) => sum + b.latest_val, 0),
    }));
    rows.sort((a, b) => b.totalLatest - a.totalLatest);
    return rows;
  }, [snap]);

  const hybridByKey = useMemo(() => indexScoreboard(hybridRows), [hybridRows]);

  const selectedFinalizerInsight = useMemo(() => {
    const key = normalizeFinalizerHex(finalizer);
    if (!isValidFinalizerHex(key)) return null;
    return hybridLookup ?? hybridByKey.get(key) ?? null;
  }, [finalizer, hybridLookup, hybridByKey]);

  const inputClass =
    "mt-1.5 w-full rounded-xl border border-white/10 bg-black/25 px-3.5 py-2.5 text-sm text-gray-100 placeholder:text-gray-500 shadow-inner shadow-black/20 transition focus:border-violet-400/40 focus:outline-none focus:ring-2 focus:ring-violet-500/20";

  return (
    <section className="relative overflow-hidden rounded-2xl border border-violet-500/20 bg-gradient-to-br from-violet-950/35 via-[#0c0b09] to-cyan-950/25 p-6 shadow-2xl shadow-violet-950/25">
      <div
        aria-hidden
        className="pointer-events-none absolute -right-20 -top-20 h-56 w-56 rounded-full bg-violet-600/15 blur-3xl"
      />
      <div
        aria-hidden
        className="pointer-events-none absolute -bottom-16 -left-16 h-48 w-48 rounded-full bg-cyan-500/10 blur-3xl"
      />

      {/* Hero */}
      <div className="relative flex flex-col gap-5 lg:flex-row lg:items-start lg:justify-between">
        <div className="min-w-0 space-y-3">
          <div className="flex flex-wrap items-center gap-2">
            <span className="rounded-full border border-violet-400/30 bg-violet-500/10 px-3 py-1 text-[0.65rem] font-bold uppercase tracking-[0.22em] text-violet-200">
              Season 1
            </span>
            <StatusPill
              tone={stakingDayOpen ? "open" : "closed"}
              label={stakingDayOpen ? "Staking Day open" : "Staking Day closed"}
            />
            {!loading && lastUpdated && (
              <span className="inline-flex items-center gap-1.5 rounded-full border border-white/10 bg-white/5 px-2.5 py-1 text-[0.65rem] font-medium text-gray-400">
                <span className="relative flex h-1.5 w-1.5">
                  <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-400 opacity-60" />
                  <span className="relative inline-flex h-1.5 w-1.5 rounded-full bg-emerald-400" />
                </span>
                Live · {lastUpdated.toLocaleTimeString()}
              </span>
            )}
          </div>
          <div>
            <h3 className="text-2xl font-extrabold tracking-tight text-primary-100">Crosslink</h3>
          </div>
        </div>
        <Button
          type="button"
          variant="secondary"
          size="sm"
          className="shrink-0 self-start"
          disabled={busy === "refresh"}
          onClick={() => void load({ manual: true })}
        >
          {busy === "refresh" ? "Refreshing…" : "Refresh"}
        </Button>
      </div>

      {/* Wallet balance — what you can stake without opening the GUI */}
      <div className="relative mt-6 grid gap-3 lg:grid-cols-3">
        <div
          className={`rounded-2xl border p-5 lg:col-span-1 ${
            snap?.wallet != null
              ? "border-emerald-400/35 bg-gradient-to-br from-emerald-500/20 to-emerald-950/10"
              : "border-amber-400/35 bg-gradient-to-br from-amber-500/15 to-amber-950/10"
          }`}
        >
          <p
            className={`text-[0.65rem] font-bold uppercase tracking-[0.2em] ${
              snap?.wallet != null ? "text-emerald-200/90" : "text-amber-200/90"
            }`}
          >
            Available to stake
          </p>
          <p className="mt-2 flex items-baseline gap-2">
            <span
              className={`text-3xl font-extrabold tabular-nums tracking-tight ${
                snap?.wallet != null ? "text-emerald-50" : "text-amber-50"
              }`}
            >
              {snap?.wallet != null ? zatToCtaz(availableToStakeZat(snap)) : "—"}
            </span>
            {snap?.wallet != null && (
              <span className="text-sm font-medium text-gray-500">cTAZ</span>
            )}
          </p>
          {snap?.wallet == null && snap != null ? (
            <p className="mt-2 text-xs leading-relaxed text-amber-100/90">
              Spendable balance unavailable on this node build.
            </p>
          ) : (
            <p className="mt-2 text-xs leading-relaxed text-gray-400">
              Unstaked node wallet balance.
            </p>
          )}
          {snap?.wallet && !walletSynced(snap.wallet) && (
            <p className="mt-2 text-xs text-amber-200/90">
              Wallet still scanning ({snap.wallet.sync_height.toLocaleString()} /{" "}
              {snap.wallet.tip_height.toLocaleString()}) — balance may be low until caught up.
            </p>
          )}
        </div>
        <MetricCard
          label="Currently staked"
          value={snap ? zatToCtaz(stakedReportedZat(snap)) : "—"}
          unit="cTAZ"
          sub="Active bonds"
        />
        <MetricCard
          label="Rewards on bonds"
          value={snap ? zatToCtaz(rewardsZat(snap)) : "—"}
          unit="cTAZ"
          accent="violet"
          sub="Earned rewards"
        />
      </div>

      {/* Metrics */}
      <div className="relative mt-4 grid gap-3 sm:grid-cols-2">
        <MetricCard label="PoW height" value={snap ? snap.height.toLocaleString() : "—"} />
        <MetricCard
          label="Staking window"
          value={dayLabel}
          accent={stakingDayOpen ? "emerald" : "rose"}
          accentFill
          sub={
            stakingDayOpen
              ? "Stake · unbond · withdraw enabled"
              : "Retarget still available"
          }
        />
      </div>

      {/* Staking Day progress */}
      <div className="relative mt-4 rounded-2xl border border-white/10 bg-black/20 px-4 py-3 backdrop-blur-sm">
        <div className="mb-2 flex items-center justify-between gap-2 text-xs">
          <span
            className={`font-medium ${
              stakingDayOpen ? "text-emerald-300/90" : "text-rose-300/90"
            }`}
          >
            {stakingDayOpen ? "Window progress" : "Until next window"}
          </span>
          <span
            className={`tabular-nums ${
              stakingDayOpen ? "text-emerald-400/80" : "text-rose-400/80"
            }`}
          >
            {Math.round(stakingDayProgress.pct)}%
          </span>
        </div>
        <div className="h-1.5 overflow-hidden rounded-full bg-white/5">
          <div
            className={`h-full rounded-full transition-all duration-700 ease-out ${
              stakingDayOpen
                ? "bg-gradient-to-r from-emerald-500 to-emerald-400"
                : "bg-gradient-to-r from-rose-600 to-rose-400"
            }`}
            style={{ width: `${stakingDayProgress.pct}%` }}
          />
        </div>
      </div>

      {/* Next action */}
      <div className="relative mt-4 overflow-hidden rounded-2xl border border-violet-400/25 bg-gradient-to-r from-violet-500/10 via-transparent to-cyan-500/5 px-5 py-4 backdrop-blur-sm">
        <div className="absolute inset-y-0 left-0 w-1 bg-gradient-to-b from-violet-400 to-cyan-400" />
        <p className="text-[0.65rem] font-bold uppercase tracking-[0.2em] text-violet-300/90">
          Next action
        </p>
        <p className="mt-2 text-sm leading-relaxed text-gray-100">
          {snap ? formatNextAction(snap.next_action) : "Loading guardian status…"}
        </p>
        {snap && (
          <div className="mt-3 flex flex-wrap gap-2">
            <Chip>{activeBondCount(snap)} active bonds</Chip>
            <Chip>
              {(snap.positions.withdrawable ?? []).length} withdrawable ·{" "}
              {zatToCtaz(withdrawableZat(snap))} cTAZ
            </Chip>
            <Chip>
              TFL{" "}
              {snap.tfl_activated == null ? "?" : snap.tfl_activated ? "on" : "off"}
              {snap.finalizer_count != null ? ` · ${snap.finalizer_count} finalizers` : ""}
            </Chip>
          </div>
        )}
      </div>

      {/* UFVK export */}
      <Panel className="mt-4 border-amber-500/25 bg-gradient-to-br from-amber-500/8 to-transparent">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div className="space-y-1">
            <p className="text-[0.65rem] font-bold uppercase tracking-[0.2em] text-amber-200/90">
              UFVK export
            </p>
            <p className="text-sm leading-relaxed text-gray-300">
              Export your node wallet UFVK for payout submission.
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <Button
              type="button"
              variant="outline"
              size="sm"
              disabled={ufvkLoading}
              onClick={() => void loadUfvk()}
            >
              {ufvkLoading ? "Loading…" : ufvk ? "Reload" : "Load UFVK"}
            </Button>
            {ufvk && (
              <Button type="button" variant="primary" size="sm" onClick={() => void copyUfvk()}>
                Copy
              </Button>
            )}
          </div>
        </div>
        {ufvk && (
          <p className="mt-3 rounded-xl border border-white/5 bg-black/30 px-3 py-2 font-mono text-xs leading-relaxed text-gray-400 break-all">
            {showKeys ? ufvk : `${ufvk.slice(0, 28)}…${ufvk.slice(-20)}`}
          </p>
        )}
      </Panel>

      {/* Actions */}
      <div className="relative mt-6 grid gap-4 lg:grid-cols-2">
        <Panel title="Stake" subtitle="Create a new delegation bond">
          <label className="block text-sm text-gray-400">
            Amount (cTAZ)
            <input
              className={inputClass}
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              disabled={!!busy}
            />
          </label>
          <label className="block text-sm text-gray-400">
            Finalizer (64 hex)
            <input
              className={`${inputClass} font-mono`}
              value={finalizer}
              onChange={(e) => setFinalizer(e.target.value.trim())}
              placeholder="Paste from roster or monolith UI"
              disabled={!!busy}
            />
          </label>
          {isValidFinalizerHex(finalizer) && (
            <FinalizerInsightPanel
              row={selectedFinalizerInsight}
              loading={hybridLookupLoading && !selectedFinalizerInsight}
            />
          )}
          {!writeWindowOpen && (
            <p className="rounded-xl border border-amber-500/20 bg-amber-500/5 px-3 py-2 text-xs text-amber-100/90">
              Staking Day closed — stake / unbond / withdraw disabled. Retarget anytime, or enable
              Force below.
            </p>
          )}
          <div className="flex flex-wrap gap-2 pt-1">
            <Button
              type="button"
              disabled={!!busy || !finalizer || !writeWindowOpen || stakeSettling}
              title={
                stakeSettling
                  ? "Previous stake still settling — wait for confirmation"
                  : undefined
              }
              onClick={() =>
                void run("stake", async () => {
                  const n = Number(amount);
                  if (!Number.isFinite(n) || n <= 0) {
                    throw new Error("Enter a positive cTAZ amount");
                  }
                  const res = await walletApi.crosslinkStake({
                    amount_ctaz: n,
                    finalizer,
                    force,
                  });
                  const raw = res.data.result;
                  if (raw && typeof raw === "object") {
                    const o = raw as {
                      reconciled?: boolean;
                      pending?: boolean;
                      busy?: boolean;
                      submitted_uncertain?: boolean;
                      duplicate_recent?: boolean;
                      message?: string;
                    };
                    if (o.reconciled) {
                      return {
                        tone: "success" as const,
                        message: o.message || "Stake landed on chain",
                      };
                    }
                    if (o.duplicate_recent) {
                      return {
                        tone: "warn" as const,
                        message:
                          o.message ||
                          "Matching bond already exists recently — skipped double stake.",
                      };
                    }
                    if (o.busy || o.submitted_uncertain) {
                      return {
                        tone: "warn" as const,
                        message:
                          o.message ||
                          "Node still finishing a stake — check bonds/GUI pending, then retry if needed.",
                      };
                    }
                    if (o.pending) {
                      return {
                        tone: "success" as const,
                        message: o.message || "Stake submitted — waiting to be mined",
                      };
                    }
                  }
                })
              }
            >
              Stake
            </Button>
            <Button
              type="button"
              variant="outline"
              disabled={!!busy || !finalizer || !bondKey}
              onClick={() =>
                void run("retarget", async () => {
                  await walletApi.crosslinkRetarget({
                    bond: bondKey,
                    finalizer,
                  });
                })
              }
            >
              Retarget
            </Button>
          </div>
          {stakeSettling && (
            <p className="text-xs text-amber-200/90">
              Stake settling — locked ~3 min to prevent a double bond.
            </p>
          )}
        </Panel>

        <Panel title="Unbond / withdraw" subtitle="Exit or claim finished bonds">
          <label className="block text-sm text-gray-400">
            Bond pk (from positions)
            <input
              ref={bondInputRef}
              className={`${inputClass} font-mono`}
              value={bondKey}
              onChange={(e) => setBondKey(e.target.value.trim())}
              disabled={!!busy}
            />
          </label>
          <label className="flex items-center gap-2.5 text-sm text-gray-400">
            <input
              type="checkbox"
              className="rounded border-white/20 bg-black/30 text-violet-500 focus:ring-violet-500/30"
              checked={force}
              onChange={(e) => setForce(e.target.checked)}
              disabled={!!busy}
            />
            Force outside Staking Day
          </label>
          <div className="flex flex-wrap gap-2 pt-1">
            <Button
              type="button"
              variant="outline"
              disabled={!!busy || !bondKey || !writeWindowOpen}
              onClick={() =>
                void run("unbond", async () => {
                  if (!bondIsActive) {
                    throw new Error(
                      "Select an active bond first (click it under Your bonds). Withdrawable bonds are already past unbond.",
                    );
                  }
                  await walletApi.crosslinkUnbond({ bond: bondKey, force });
                })
              }
            >
              Unbond
            </Button>
            <Button
              type="button"
              disabled={!!busy || !bondKey || !writeWindowOpen || !bondIsWithdrawable}
              title={
                bondKey && !bondIsWithdrawable
                  ? "Unbond first — Withdraw only works on withdrawable bonds"
                  : undefined
              }
              onClick={() =>
                void run("withdraw", async () => {
                  if (!bondIsWithdrawable) {
                    throw new Error(
                      "This bond is still active. Click Unbond first, wait until it appears as withdrawable, then Withdraw.",
                    );
                  }
                  await walletApi.crosslinkWithdraw({ bond: bondKey, force });
                })
              }
            >
              Withdraw
            </Button>
          </div>
          {bondKey && !bondIsWithdrawable && bondIsActive && (
            <p className="text-xs text-amber-200/90">
              Active bond — unbond first, then withdraw when listed as withdrawable.
            </p>
          )}
        </Panel>
      </div>

      {/* Lists */}
      <div className="relative mt-6 space-y-4">
        <ListPanel
          title={`Your bonds${flatBonds.length > 0 ? ` · ${flatBonds.length}` : ""}`}
          subtitle="Tap a bond → fills Unbond / withdraw"
          action={
            <button
              type="button"
              className="text-xs font-medium text-violet-300 hover:text-violet-200 transition"
              onClick={() => setShowKeys((v) => !v)}
            >
              {showKeys ? "Hide keys" : "Show keys"}
            </button>
          }
          empty="No bonds yet — stake during Staking Day."
          items={flatBonds}
          renderItem={(b) => (
            <li
              key={b.pk}
              role="button"
              tabIndex={0}
              className={`group flex cursor-pointer flex-wrap items-center justify-between gap-2 rounded-xl border px-3 py-2.5 transition ${
                bondKey.toLowerCase() === b.pk.toLowerCase()
                  ? "border-violet-400/40 bg-violet-500/10"
                  : "border-white/5 bg-white/[0.03] hover:border-violet-400/25 hover:bg-violet-500/5"
              }`}
              onClick={() => selectBondKey(b.pk)}
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  selectBondKey(b.pk);
                }
              }}
            >
              <div className="min-w-0 font-mono text-xs">
                <p className="text-left text-gray-200 transition group-hover:text-violet-200">
                  {showKeys ? b.pk : shortHex(b.pk)}
                </p>
                {b.finalizer ? (
                  <button
                    type="button"
                    className="mt-0.5 text-left text-gray-500 hover:text-cyan-300"
                    title="Use finalizer for stake / retarget"
                    onClick={(e) => {
                      e.stopPropagation();
                      selectFinalizer(b.finalizer!);
                    }}
                  >
                    → {showKeys ? b.finalizer : shortHex(b.finalizer)} · retarget
                  </button>
                ) : (
                  <p className="mt-0.5 text-amber-300/80">Withdrawable</p>
                )}
              </div>
              <div className="text-right">
                <span className="block shrink-0 rounded-lg bg-black/30 px-2 py-1 tabular-nums text-xs text-gray-300">
                  {zatToCtaz(b.latest)} cTAZ
                </span>
                {b.latest > b.initial && (
                  <span className="mt-1 block text-[0.65rem] font-medium tabular-nums text-emerald-400/90">
                    +{zatToCtaz(b.latest - b.initial)} earned
                  </span>
                )}
              </div>
            </li>
          )}
        />

        {stakedFinalizers.length > 0 && (
          <ListPanel
            title={`Your finalizers · ${stakedFinalizers.length}`}
            subtitle="Stake / retarget only — tap Your bonds above to unbond"
            items={stakedFinalizers}
            renderItem={(e) => (
              <li
                key={e.finalizer}
                className="flex items-center justify-between gap-2 rounded-xl border border-white/5 bg-white/[0.03] px-3 py-2 font-mono text-xs transition hover:border-cyan-400/20 hover:bg-cyan-500/5"
              >
                <button
                  type="button"
                  className="min-w-0 truncate text-left text-gray-200 hover:text-cyan-200"
                  title="Paste as finalizer"
                  onClick={() => selectFinalizer(e.finalizer)}
                >
                  {showKeys ? e.finalizer : shortHex(e.finalizer)}
                </button>
                <div className="flex shrink-0 items-center gap-2">
                  <FinalizerGradeBadge row={hybridByKey.get(normalizeFinalizerHex(e.finalizer))} />
                  <span className="tabular-nums text-gray-500">
                    {e.bonds} · {zatToCtaz(e.totalLatest)} cTAZ
                  </span>
                </div>
              </li>
            )}
          />
        )}

        {roster.length > 0 && (
          <ListPanel
            title={`Network roster · ${roster.length}`}
            subtitle="Stake / retarget — fills finalizer field"
            items={roster}
            renderItem={(e, i) => (
              <li
                key={e.finalizer}
                className="flex items-center justify-between gap-2 rounded-xl border border-white/5 px-3 py-2 font-mono text-xs transition hover:bg-white/[0.04]"
              >
                <button
                  type="button"
                  className="min-w-0 truncate text-left text-gray-300 hover:text-gray-100"
                  title="Paste as finalizer"
                  onClick={() => selectFinalizer(e.finalizer)}
                >
                  <span className="text-gray-600">{i + 1}.</span>{" "}
                  {showKeys ? e.finalizer : shortHex(e.finalizer)}
                </button>
                <div className="flex shrink-0 items-center gap-2">
                  <FinalizerGradeBadge row={hybridByKey.get(normalizeFinalizerHex(e.finalizer))} />
                  <span className="tabular-nums text-gray-500">
                    {zatToCtaz(e.stake_zat)} · {(e.share * 100).toFixed(1)}%
                  </span>
                </div>
              </li>
            )}
          />
        )}
      </div>

    </section>
  );
}

function StatusPill({
  tone,
  label,
}: {
  tone: "open" | "closed";
  label: string;
}) {
  const styles =
    tone === "open"
      ? "border-emerald-400/35 bg-emerald-500/15 text-emerald-200"
      : "border-rose-400/35 bg-rose-500/15 text-rose-200";
  return (
    <span className={`rounded-full border px-3 py-1 text-[0.65rem] font-semibold ${styles}`}>
      {label}
    </span>
  );
}

function Chip({ children }: { children: ReactNode }) {
  return (
    <span className="rounded-lg border border-white/10 bg-white/5 px-2.5 py-1 text-[0.65rem] font-medium text-gray-400">
      {children}
    </span>
  );
}

function FinalizerGradeBadge({ row }: { row: HybridPosFinalizer | null | undefined }) {
  if (!row?.grade) {
    return (
      <span className="rounded-md border border-white/10 bg-white/5 px-1.5 py-0.5 text-[0.6rem] font-semibold text-gray-500">
        —
      </span>
    );
  }
  const standing = hybridPosStanding(row);
  return (
    <span
      className={`rounded-md border px-1.5 py-0.5 text-[0.6rem] font-bold tabular-nums ${hybridPosGradeClass(standing)}`}
      title={hybridPosStandingLabel(standing)}
    >
      {row.grade}
    </span>
  );
}

function FinalizerInsightPanel({
  row,
  loading,
}: {
  row: HybridPosFinalizer | null;
  loading: boolean;
}) {
  if (loading) {
    return (
      <div className="rounded-xl border border-white/10 bg-black/20 px-3 py-2.5 text-xs text-gray-400">
        Loading finalizer grade from Crosslink Network observer…
      </div>
    );
  }
  if (!row) {
    return (
      <div className="rounded-xl border border-white/10 bg-black/20 px-3 py-2.5 text-xs leading-relaxed text-gray-400">
        No observer data for this finalizer yet. Grades come from{" "}
        <a
          href="https://zcash-hybrid-pos.vercel.app/"
          target="_blank"
          rel="noopener noreferrer"
          className="text-violet-300 hover:text-violet-200"
        >
          Crosslink Network
        </a>{" "}
        — an independent node measuring finality participation.
      </div>
    );
  }

  const standing = hybridPosStanding(row);
  const votePct = row.pct ?? (row.of ? ((row.voted ?? 0) / row.of) * 100 : null);

  return (
    <div
      className={`rounded-xl border px-3 py-3 text-xs leading-relaxed ${hybridPosGradeClass(standing)}`}
    >
      <div className="flex flex-wrap items-center gap-2">
        <span className="text-sm font-bold">{row.grade ?? "—"}</span>
        {row.score != null && (
          <span className="rounded-md border border-current/20 bg-black/20 px-1.5 py-0.5 font-semibold tabular-nums">
            {row.score.toFixed(1)} score
          </span>
        )}
        {row.live === false && (
          <span className="rounded-md border border-rose-400/30 bg-rose-500/10 px-1.5 py-0.5 text-rose-200">
            offline
          </span>
        )}
        {row.in_threshold_set && (
          <span className="rounded-md border border-amber-400/30 bg-amber-500/10 px-1.5 py-0.5 text-amber-100">
            top ⅓ stake
          </span>
        )}
      </div>
      <p className="mt-2 font-medium">{hybridPosStandingLabel(standing)}</p>
      <div className="mt-2 flex flex-wrap gap-x-4 gap-y-1 tabular-nums text-[0.7rem] opacity-90">
        <span>Rank #{row.rank ?? "—"}</span>
        {row.share_pct != null && <span>{row.share_pct.toFixed(2)}% stake share</span>}
        {votePct != null && (
          <span>
            {votePct.toFixed(1)}% finality votes ({row.voted?.toLocaleString()} /{" "}
            {row.of?.toLocaleString()})
          </span>
        )}
        {row.stake_ctaz != null && (
          <span>{row.stake_ctaz.toLocaleString(undefined, { maximumFractionDigits: 2 })} cTAZ bonded</span>
        )}
      </div>
      {row.in_threshold_set && (
        <p className="mt-2 text-[0.7rem] opacity-90">
          Holds enough stake to stall finality if misbehaving — spreading stake outside the largest
          finalizers is safer for the network.
        </p>
      )}
      {row.score_note && <p className="mt-2 text-[0.7rem] opacity-90">{row.score_note}</p>}
      <p className="mt-2 text-[0.65rem] opacity-75">
        Data from{" "}
        <a
          href="https://zcash-hybrid-pos.vercel.app/"
          target="_blank"
          rel="noopener noreferrer"
          className="underline underline-offset-2"
        >
          Crosslink Network observer
        </a>
        . Grades reflect finality participation, not guaranteed future rewards.
      </p>
    </div>
  );
}

function MetricCard({
  label,
  value,
  unit,
  sub,
  accent,
  accentFill,
  highlight,
}: {
  label: string;
  value: string;
  unit?: string;
  sub?: string;
  accent?: "emerald" | "amber" | "violet" | "rose";
  accentFill?: boolean;
  highlight?: boolean;
}) {
  const accentBorder =
    accent === "emerald"
      ? "border-emerald-500/30"
      : accent === "rose"
        ? "border-rose-500/30"
        : accent === "amber"
          ? "border-amber-500/25"
          : accent === "violet"
            ? "border-violet-400/30"
            : "border-white/10";
  const accentBg =
    accentFill && accent === "emerald"
      ? "bg-gradient-to-br from-emerald-500/15 to-emerald-950/10"
      : accentFill && accent === "rose"
        ? "bg-gradient-to-br from-rose-500/15 to-rose-950/10"
        : highlight
          ? "bg-gradient-to-br from-violet-500/15 to-cyan-500/5 shadow-lg shadow-violet-950/20"
          : "bg-black/25";
  const labelClass =
    accentFill && accent === "emerald"
      ? "text-emerald-200/90"
      : accentFill && accent === "rose"
        ? "text-rose-200/90"
        : "text-gray-500";
  const valueClass =
    accentFill && accent === "emerald"
      ? "text-emerald-50"
      : accentFill && accent === "rose"
        ? "text-rose-50"
        : "text-primary-100";
  const subClass =
    accentFill && accent === "emerald"
      ? "text-emerald-200/75"
      : accentFill && accent === "rose"
        ? "text-rose-200/75"
        : "text-gray-500";
  const unitClass =
    accentFill && accent === "emerald"
      ? "text-emerald-300/70"
      : accentFill && accent === "rose"
        ? "text-rose-300/70"
        : "text-gray-500";
  return (
    <div className={`rounded-2xl border ${accentBorder} ${accentBg} p-4 backdrop-blur-sm`}>
      <p className={`text-[0.65rem] font-bold uppercase tracking-[0.18em] ${labelClass}`}>
        {label}
      </p>
      <p className="mt-2 flex items-baseline gap-1.5">
        <span className={`text-xl font-bold tabular-nums tracking-tight ${valueClass}`}>
          {value}
        </span>
        {unit && <span className={`text-xs font-medium ${unitClass}`}>{unit}</span>}
      </p>
      {sub && <p className={`mt-1 text-[0.65rem] ${subClass}`}>{sub}</p>}
    </div>
  );
}

function Panel({
  title,
  subtitle,
  className = "",
  children,
}: {
  title?: string;
  subtitle?: string;
  className?: string;
  children: ReactNode;
}) {
  return (
    <div
      className={`rounded-2xl border border-white/10 bg-black/20 p-4 backdrop-blur-sm ${className}`}
    >
      {title && (
        <div className="mb-4">
          <p className="text-sm font-semibold text-gray-100">{title}</p>
          {subtitle && <p className="mt-0.5 text-xs text-gray-500">{subtitle}</p>}
        </div>
      )}
      <div className="flex flex-col gap-3">{children}</div>
    </div>
  );
}

function ListPanel<T>({
  title,
  subtitle,
  action,
  empty,
  items,
  renderItem,
}: {
  title: string;
  subtitle?: string;
  action?: ReactNode;
  empty?: string;
  items: T[];
  renderItem: (item: T, index: number) => ReactNode;
}) {
  return (
    <Panel>
      <div className="-mt-1 mb-3 flex items-start justify-between gap-2">
        <div>
          <p className="text-sm font-semibold text-gray-100">{title}</p>
          {subtitle && <p className="mt-0.5 text-xs text-gray-500">{subtitle}</p>}
        </div>
        {action}
      </div>
      {items.length === 0 ? (
        <p className="text-sm text-gray-500">{empty}</p>
      ) : (
        <ul className="max-h-48 space-y-1.5 overflow-y-auto pr-1 text-sm">{items.map(renderItem)}</ul>
      )}
    </Panel>
  );
}
