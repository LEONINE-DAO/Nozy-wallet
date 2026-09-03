import { useCallback, useEffect, useMemo, useState, type CSSProperties } from "react";
import { extensionApi, getCompanionPrefs } from "../lib/extensionApi";
import {
  fetchHybridPosFinalizer,
  fetchHybridPosScoreboard,
  hybridPosStanding,
  hybridPosStandingLabel,
  indexScoreboard,
  isValidFinalizerHex,
  normalizeFinalizerHex,
  type HybridPosFinalizer,
  type HybridPosStanding
} from "../lib/hybridPos";
import { useUiStore } from "../store/uiStore";
import { finalizerFromUrl, isFullPage } from "../lib/walletPage";
import {
  Button,
  Callout,
  Card,
  EmptyState,
  Eyebrow,
  Hint,
  Icon,
  Input,
  LogBlock,
  PageHeader,
  Pill,
  Screen,
  SectionTitle,
  StatRow
} from "./ui";

/** Status poll (~1 feature-net block). Positions live here; roster polled less often. */
const STATUS_REFRESH_MS = 90_000;
/** Finalizer roster changes slowly — skip on most background ticks. */
const ROSTER_REFRESH_MS = 180_000;

type CrosslinkStatus = Awaited<
  ReturnType<typeof extensionApi.companionCrosslinkStatus>
>;
type RosterEntry = Awaited<
  ReturnType<typeof extensionApi.companionCrosslinkRoster>
>[number];

function zatToCtaz(zat: number): string {
  return (zat / 1e8).toFixed(4);
}

function availableToStakeZat(
  wallet: NonNullable<CrosslinkStatus["wallet"]>
): number {
  return wallet.user_shielded_spendable_zats + wallet.user_unshielded_zats;
}

function walletSynced(wallet: NonNullable<CrosslinkStatus["wallet"]>): boolean {
  return wallet.tip_height === 0 || wallet.sync_height + 2 >= wallet.tip_height;
}

function shortHex(hex: string): string {
  const h = hex.trim();
  if (h.length <= 20) return h;
  return `${h.slice(0, 10)}…${h.slice(-8)}`;
}

function formatNextAction(action: CrosslinkStatus["next_action"]): string {
  if (typeof action === "string") {
    switch (action) {
      case "unbond_to_exit":
        return "Unbond to exit, or retarget anytime.";
      case "stake_or_guardian":
        return "Window open — stake to a finalizer.";
      case "retarget_if_needed":
        return "Window closed — retarget still allowed.";
      default:
        return "Refresh for guidance.";
    }
  }
  if ("wait_for_staking_day" in action) {
    return `Wait ~${action.wait_for_staking_day.blocks} blocks for Staking Day.`;
  }
  if ("withdraw_ready" in action) {
    return `${action.withdraw_ready.count} bond(s) ready to withdraw.`;
  }
  return "Refresh for guidance.";
}

/** Keplr-style Home → Staked: pick a finalizer from the roster, then stake. */
export function HomeStakedPanel({
  onManage
}: {
  onManage: (finalizer?: string) => void;
}) {
  const [status, setStatus] = useState<CrosslinkStatus | null>(null);
  const [roster, setRoster] = useState<RosterEntry[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [hybridRows, setHybridRows] = useState<HybridPosFinalizer[]>([]);
  const [selected, setSelected] = useState("");
  const [amount, setAmount] = useState("0.01");
  const [query, setQuery] = useState("");
  const [busy, setBusy] = useState(false);
  const [log, setLog] = useState("");

  const reload = useCallback(async () => {
    const prefs = await getCompanionPrefs();
    const s = await extensionApi.companionCrosslinkStatus({
      baseUrl: prefs.baseUrl
    });
    setStatus(s);
    try {
      const r = await extensionApi.companionCrosslinkRoster({
        baseUrl: prefs.baseUrl
      });
      setRoster(r);
    } catch {
      setRoster([]);
    }
  }, []);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        await reload();
        if (!cancelled) setError(null);
      } catch (e) {
        if (!cancelled) {
          setStatus(null);
          setError(e instanceof Error ? e.message : String(e));
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
      try {
        const rows = await fetchHybridPosScoreboard();
        if (!cancelled) setHybridRows(rows);
      } catch {
        /* observer optional */
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [reload]);

  const hybridByKey = useMemo(() => indexScoreboard(hybridRows), [hybridRows]);

  const stakedRows = useMemo(() => {
    if (!status) return [];
    const rows = Object.entries(status.positions.active).map(([finalizer, bonds]) => ({
      finalizer,
      bonds: bonds.length,
      totalLatest: bonds.reduce((sum, b) => sum + b.latest_val, 0)
    }));
    rows.sort((a, b) => b.totalLatest - a.totalLatest);
    return rows;
  }, [status]);

  const filteredRoster = useMemo(() => {
    const q = query.trim().toLowerCase();
    const list = q
      ? roster.filter((e) => e.finalizer.toLowerCase().includes(q))
      : roster;
    return list.slice(0, 80);
  }, [roster, query]);

  const bonded = stakedRows.reduce((sum, r) => sum + r.totalLatest, 0);
  const liquid = status?.wallet != null ? availableToStakeZat(status.wallet) : 0;
  const day = status?.staking_day;
  const selectedInsight = hybridByKey.get(normalizeFinalizerHex(selected));

  const pick = (hex: string) => {
    setSelected(hex);
    setLog("");
  };

  const openManage = (hex?: string) => {
    onManage(hex ?? (selected || undefined));
  };

  const stakeSelected = async () => {
    if (!selected) {
      setLog("Pick a finalizer from the list first.");
      return;
    }
    const n = Number(amount);
    if (!Number.isFinite(n) || n <= 0) {
      setLog("Enter a positive cTAZ amount.");
      return;
    }
    setBusy(true);
    setLog("");
    try {
      const prefs = await getCompanionPrefs();
      const res = await extensionApi.companionCrosslinkStake({
        baseUrl: prefs.baseUrl,
        amount_ctaz: n,
        finalizer: selected
      });
      setLog(`Stake ${res.action} ok`);
      await reload();
    } catch (e) {
      setLog(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  if (loading) {
    return <Hint>Loading Crosslink roster…</Hint>;
  }

  if (error || !status) {
    return (
      <Card className="space-y-2.5">
        <SectionTitle>Staked · Crosslink</SectionTitle>
        <Hint>
          Staking is cTAZ on the Crosslink node wallet. Start nozywallet-api and the
          Crosslink GUI to load the finalizer list.
        </Hint>
        {error ? <Callout tone="warn">{error}</Callout> : null}
        <Button variant="primary" size="sm" onClick={() => onManage()}>
          Open staking
        </Button>
      </Card>
    );
  }

  return (
    <>
      <div>
        <div className="flex items-center justify-between gap-2">
          <Eyebrow>Staked · feature-net</Eyebrow>
          <Pill tone={day?.open ? "success" : "danger"}>
            {day?.open
              ? `OPEN · ${day.blocks_remaining_in_window ?? 0} left`
              : `CLOSED · ${day?.blocks_until_next ?? 0} until next`}
          </Pill>
        </div>
        <div className="nw-dash-fiat mt-1.5">{zatToCtaz(bonded)} cTAZ</div>
        <Hint className="mt-1">
          {liquid > 0
            ? `${zatToCtaz(liquid)} cTAZ left to stake in the node wallet`
            : "Bonded to finalizers · rewards stay in the node wallet"}
        </Hint>
      </div>

      <Card className="space-y-2.5">
        <SectionTitle>Stake to a finalizer</SectionTitle>
        <Hint>
          Tap a finalizer below (Keplr-style validator pick), then stake. Window must
          be OPEN unless you use Manage → Force.
        </Hint>
        {selected ? (
          <StatRow
            label="Selected"
            value={
              selectedInsight?.grade
                ? `${shortHex(selected)} · ${selectedInsight.grade}`
                : shortHex(selected)
            }
            tone="accent"
          />
        ) : (
          <Hint>No finalizer selected yet.</Hint>
        )}
        <Input
          label="Amount (cTAZ)"
          mono
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          placeholder="0.01"
          disabled={busy}
        />
        <div className="grid grid-cols-2 gap-2">
          <Button
            variant="primary"
            icon="shield"
            disabled={busy || !selected || !day?.open}
            onClick={() => void stakeSelected()}
          >
            Stake
          </Button>
          <Button variant="secondary" disabled={busy} onClick={() => openManage()}>
            Manage
          </Button>
        </div>
        {!day?.open ? (
          <Hint>Staking Day is closed — pick a finalizer and Retarget from Manage.</Hint>
        ) : null}
        {log ? <Hint>{log}</Hint> : null}
      </Card>

      <Card flush>
        <div className="space-y-2 px-3.5 py-2.5">
          <SectionTitle>Finalizers ({roster.length})</SectionTitle>
          <Input
            placeholder="Search hex or paste identity"
            mono
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
        </div>
        {filteredRoster.length === 0 ? (
          <div className="px-3.5 pb-3.5">
            <EmptyState>
              {roster.length === 0
                ? "Roster empty — node TFL recency may still be warming up."
                : "No finalizer matches that search."}
            </EmptyState>
          </div>
        ) : (
          <div className="max-h-52 overflow-y-auto">
            {filteredRoster.map((e, i) => {
              const row = hybridByKey.get(normalizeFinalizerHex(e.finalizer));
              const rank = row?.rank ?? i + 1;
              const active = normalizeFinalizerHex(selected) === normalizeFinalizerHex(e.finalizer);
              return (
                <button
                  key={e.finalizer}
                  type="button"
                  className="nw-asset-row w-full text-left"
                  style={{
                    ...(i === 0 ? { borderTop: "1px solid var(--nw-border)" } : {}),
                    background: active ? "var(--nw-platinum-soft)" : "transparent"
                  }}
                  onClick={() => pick(e.finalizer)}
                >
                  <span style={{ color: "var(--nw-platinum)" }}>
                    <Icon name="shield" size={18} />
                  </span>
                  <div className="min-w-0 flex-1">
                    <p className="text-[12px] font-semibold leading-tight">
                      #{rank} {shortHex(e.finalizer)}
                    </p>
                    <p className="nw-hint mt-0.5 truncate">
                      {zatToCtaz(e.stake_zat)} cTAZ · {(e.share * 100).toFixed(1)}%
                      {row?.live ? " · live" : ""}
                    </p>
                  </div>
                  <FinalizerGradeBadge row={row} />
                </button>
              );
            })}
          </div>
        )}
      </Card>

      {stakedRows.length > 0 ? (
        <Card flush>
          <div className="px-3.5 py-2.5">
            <SectionTitle>Your delegations</SectionTitle>
          </div>
          {stakedRows.map((row, i) => {
            const insight = hybridByKey.get(normalizeFinalizerHex(row.finalizer));
            return (
              <button
                key={row.finalizer}
                type="button"
                className="nw-asset-row w-full text-left"
                style={i === 0 ? { borderTop: "1px solid var(--nw-border)" } : undefined}
                onClick={() => pick(row.finalizer)}
              >
                <span style={{ color: "var(--nw-platinum)" }}>
                  <Icon name="shield" size={18} />
                </span>
                <div className="min-w-0 flex-1">
                  <p className="text-[12px] font-semibold leading-tight">
                    {shortHex(row.finalizer)}
                    {insight?.grade ? ` · ${insight.grade}` : ""}
                  </p>
                  <p className="nw-hint mt-0.5 truncate">
                    {row.bonds} bond{row.bonds === 1 ? "" : "s"}
                    {insight?.rank != null ? ` · #${insight.rank}` : ""}
                  </p>
                </div>
                <p className="shrink-0 text-[12px] font-semibold tabular-nums">
                  {zatToCtaz(row.totalLatest)} cTAZ
                </p>
              </button>
            );
          })}
        </Card>
      ) : null}
    </>
  );
}

/**
 * Crosslink Protocol Guardian via companion api-server.
 * Feature-net cTAZ; node wallet signs stake actions (not WASM).
 */
export function CrosslinkView() {
  const pendingFinalizer = useUiStore((s) => s.pendingFinalizer);
  const setPendingFinalizer = useUiStore((s) => s.setPendingFinalizer);
  const [status, setStatus] = useState<CrosslinkStatus | null>(null);
  const [roster, setRoster] = useState<RosterEntry[]>([]);
  const [log, setLog] = useState("");
  const [busy, setBusy] = useState(false);
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);
  const [amount, setAmount] = useState("0.01");
  const [finalizer, setFinalizer] = useState("");
  const [bondKey, setBondKey] = useState("");
  const [ufvk, setUfvk] = useState("");
  const [force, setForce] = useState(false);
  const [hybridRows, setHybridRows] = useState<HybridPosFinalizer[]>([]);
  const [hybridLookup, setHybridLookup] = useState<HybridPosFinalizer | null>(null);
  const [hybridLookupLoading, setHybridLookupLoading] = useState(false);

  useEffect(() => {
    const fromUrl = finalizerFromUrl();
    if (fromUrl) {
      setFinalizer(fromUrl);
      return;
    }
    if (!pendingFinalizer) return;
    setFinalizer(pendingFinalizer);
    setPendingFinalizer(null);
  }, [pendingFinalizer, setPendingFinalizer]);

  const stakedFinalizers = useMemo(() => {
    if (!status) return [] as { finalizer: string; bonds: number; totalLatest: number }[];
    const rows = Object.entries(status.positions.active).map(([finalizer, bonds]) => ({
      finalizer,
      bonds: bonds.length,
      totalLatest: bonds.reduce((sum, b) => sum + b.latest_val, 0)
    }));
    rows.sort((a, b) => b.totalLatest - a.totalLatest);
    return rows;
  }, [status]);

  const load = useCallback(async (opts?: { manual?: boolean; roster?: boolean }) => {
    const manual = opts?.manual ?? false;
    const fetchRoster = opts?.roster ?? manual;
    if (manual) setBusy(true);
    const prefs = await getCompanionPrefs();
    try {
      const s = await extensionApi.companionCrosslinkStatus({
        baseUrl: prefs.baseUrl
      });
      setStatus(s);
      if (manual) setLog("");
      setLastUpdated(new Date());
    } catch (e) {
      if (manual) {
        setStatus(null);
        setLog(e instanceof Error ? e.message : String(e));
      }
    }
    if (fetchRoster) {
      try {
        const r = await extensionApi.companionCrosslinkRoster({
          baseUrl: prefs.baseUrl
        });
        setRoster(r);
      } catch {
        if (manual) setRoster([]);
      }
    }
    if (manual) setBusy(false);
  }, []);

  const loadHybridScoreboard = useCallback(async () => {
    try {
      const rows = await fetchHybridPosScoreboard();
      setHybridRows(rows);
    } catch {
      /* observer optional — roster still works without grades */
    }
  }, []);

  const hybridByKey = useMemo(() => indexScoreboard(hybridRows), [hybridRows]);
  const selectedFinalizerInsight = useMemo(() => {
    const key = normalizeFinalizerHex(finalizer);
    if (!isValidFinalizerHex(key)) return null;
    return hybridLookup ?? hybridByKey.get(key) ?? null;
  }, [finalizer, hybridLookup, hybridByKey]);

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
    };
  }, [load]);

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
    const cached = hybridByKey.get(key);
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
  }, [finalizer, hybridByKey]);

  const run = async (fn: () => Promise<void>) => {
    if (busy) return;
    setBusy(true);
    try {
      await fn();
      await load();
    } catch (e) {
      setLog(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const bonded = useMemo(() => {
    if (!status) return 0;
    return Object.values(status.positions.active)
      .flat()
      .reduce((sum, b) => sum + b.latest_val, 0);
  }, [status]);

  const dayLabel = useMemo(() => {
    if (!status) return "…";
    const d = status.staking_day;
    return d.open
      ? `OPEN · ${d.blocks_remaining_in_window ?? 0} left`
      : `CLOSED · ${d.blocks_until_next ?? 0} until next`;
  }, [status]);

  const pageLayout = isFullPage();
  const refreshCtl = (
    <div className="flex flex-col items-end gap-1">
      <Button size="sm" icon="refresh" disabled={busy} onClick={() => void load({ manual: true })}>
        Refresh
      </Button>
      {lastUpdated && (
        <span className="nw-hint text-[10px] tabular-nums">
          Live · {lastUpdated.toLocaleTimeString()}
        </span>
      )}
    </div>
  );

  return (
    <Screen className={pageLayout ? "max-w-5xl px-6 pt-5" : undefined}>
      {pageLayout ? (
        <div className="flex items-start justify-between gap-3">
          <div>
            <p className="nw-hint">Zcash · Crosslink feature-net (cTAZ)</p>
            <p className="mt-1 text-[13px] font-semibold">Pick a finalizer and stake from the node wallet.</p>
          </div>
          {refreshCtl}
        </div>
      ) : (
        <PageHeader title="Crosslink" description="Staking dashboard" trailing={refreshCtl} />
      )}

      {status && (
        <Card className="space-y-1.5">
          <div className="flex items-center justify-between gap-2">
            <Eyebrow>Staking day</Eyebrow>
            <Pill tone={status.staking_day.open ? "success" : "danger"}>{dayLabel}</Pill>
          </div>
          <StatRow label="Height" value={status.height} />
          <StatRow
            label="Available to stake"
            value={
              status.wallet != null
                ? `${zatToCtaz(availableToStakeZat(status.wallet))} cTAZ`
                : "—"
            }
            tone={status.wallet != null ? "accent" : undefined}
          />
          <StatRow label="Bonded" value={`${zatToCtaz(bonded)} cTAZ`} />
          <StatRow label="Withdrawable" value={status.positions.withdrawable.length} />
          {status.wallet && !walletSynced(status.wallet) && (
            <Hint>
              Wallet scanning ({status.wallet.sync_height.toLocaleString()} /{" "}
              {status.wallet.tip_height.toLocaleString()}) — balance may be low until caught up.
            </Hint>
          )}
          {status.wallet == null && (
            <Callout tone="warn">Spendable balance unavailable on this node build.</Callout>
          )}
          <StatRow
            label="TFL"
            value={status.tfl_activated == null ? "?" : status.tfl_activated ? "on" : "off"}
          />
          <Callout tone="accent">Next: {formatNextAction(status.next_action)}</Callout>
        </Card>
      )}

      <Card className="space-y-2.5">
        <SectionTitle>Stake / retarget</SectionTitle>
        <Input
          label="Amount (cTAZ)"
          mono
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          placeholder="0.01"
          disabled={busy}
        />
        <Input
          label="Finalizer"
          mono
          value={finalizer}
          onChange={(e) => setFinalizer(e.target.value.trim())}
          placeholder="64-hex finalizer key"
          disabled={busy}
        />
        {isValidFinalizerHex(finalizer) && (
          <FinalizerInsightPanel
            row={selectedFinalizerInsight}
            loading={hybridLookupLoading && !selectedFinalizerInsight}
          />
        )}
        <div className="flex flex-wrap gap-2">
          <Button
            variant="primary"
            size="sm"
            disabled={busy || !finalizer}
            onClick={() =>
              run(async () => {
                const n = Number(amount);
                if (!Number.isFinite(n) || n <= 0) {
                  throw new Error("Enter a positive cTAZ amount");
                }
                const prefs = await getCompanionPrefs();
                const res = await extensionApi.companionCrosslinkStake({
                  baseUrl: prefs.baseUrl,
                  amount_ctaz: n,
                  finalizer,
                  force
                });
                setLog(`Stake ${res.action} ok`);
              })
            }
          >
            Stake
          </Button>
          <Button
            size="sm"
            disabled={busy || !finalizer || !bondKey}
            onClick={() =>
              run(async () => {
                const prefs = await getCompanionPrefs();
                const res = await extensionApi.companionCrosslinkRetarget({
                  baseUrl: prefs.baseUrl,
                  bond: bondKey,
                  finalizer
                });
                setLog(`Retarget ${res.action} ok`);
              })
            }
          >
            Retarget
          </Button>
        </div>
      </Card>

      <Card className="space-y-2.5">
        <SectionTitle>Unbond / withdraw</SectionTitle>
        <Input
          label="Bond public key"
          mono
          value={bondKey}
          onChange={(e) => setBondKey(e.target.value.trim())}
          placeholder="Bond pk"
          disabled={busy}
        />
        <label className="nw-hint flex items-center gap-2">
          <input
            type="checkbox"
            checked={force}
            onChange={(e) => setForce(e.target.checked)}
            disabled={busy}
          />
          Force outside Staking Day
        </label>
        <div className="flex flex-wrap gap-2">
          <Button
            size="sm"
            disabled={busy || !bondKey}
            onClick={() =>
              run(async () => {
                const prefs = await getCompanionPrefs();
                const res = await extensionApi.companionCrosslinkUnbond({
                  baseUrl: prefs.baseUrl,
                  bond: bondKey,
                  force
                });
                setLog(`Unbond ${res.action} ok`);
              })
            }
          >
            Unbond
          </Button>
          <Button
            variant="primary"
            size="sm"
            disabled={busy || !bondKey}
            onClick={() =>
              run(async () => {
                const prefs = await getCompanionPrefs();
                const res = await extensionApi.companionCrosslinkWithdraw({
                  baseUrl: prefs.baseUrl,
                  bond: bondKey,
                  force
                });
                setLog(`Withdraw ${res.action} ok`);
              })
            }
          >
            Withdraw
          </Button>
        </div>
      </Card>

      <Card className="space-y-2">
        <SectionTitle>UFVK export</SectionTitle>
        <Hint>Export UFVK for payout submission.</Hint>
        {ufvk ? (
          <LogBlock>{ufvk}</LogBlock>
        ) : (
          <EmptyState>UFVK not loaded yet.</EmptyState>
        )}
        <Button
          size="sm"
          disabled={busy}
          onClick={() =>
            run(async () => {
              const prefs = await getCompanionPrefs();
              const res = await extensionApi.companionCrosslinkWalletUfvk({
                baseUrl: prefs.baseUrl
              });
              setUfvk(res.ufvk);
              setLog("UFVK loaded");
            })
          }
        >
          Load UFVK
        </Button>
      </Card>

      {status && (
        <Card className="space-y-1.5">
          <SectionTitle>
            Bonds (
            {Object.values(status.positions.active).reduce((n, a) => n + a.length, 0) +
              status.positions.withdrawable.length}
            )
          </SectionTitle>
          {(Object.values(status.positions.active).reduce((n, a) => n + a.length, 0) +
            status.positions.withdrawable.length) >
            5 && (
            <Hint>Scroll for more bonds.</Hint>
          )}
          <div className="max-h-32 space-y-0.5 overflow-y-auto">
          {Object.entries(status.positions.active).flatMap(([fin, bonds]) =>
            bonds.map((b) => (
              <button
                key={b.pk}
                type="button"
                className="nw-mono block w-full rounded-lg px-2 py-1.5 text-left text-[10px]"
                style={{
                  background: "var(--nw-surface-alt)",
                  border: "1px solid var(--nw-border)",
                  color: "var(--nw-muted)"
                }}
                onClick={() => {
                  setBondKey(b.pk);
                  setFinalizer(fin);
                  setLog("Bond + finalizer selected");
                }}
              >
                {shortHex(b.pk)} → {shortHex(fin)} · {zatToCtaz(b.latest_val)}
              </button>
            ))
          )}
          {status.positions.withdrawable.map((b) => (
            <button
              key={b.pk}
              type="button"
              className="nw-mono block w-full rounded-lg px-2 py-1.5 text-left text-[10px]"
              style={{
                background: "var(--nw-platinum-soft)",
                border: "1px solid var(--nw-platinum-line)",
                color: "var(--nw-platinum)"
              }}
              onClick={() => {
                setBondKey(b.pk);
                setLog("Withdrawable bond selected");
              }}
            >
              withdraw {shortHex(b.pk)} · {zatToCtaz(b.latest_val)}
            </button>
          ))}
          </div>
          {Object.keys(status.positions.active).length === 0 &&
            status.positions.withdrawable.length === 0 && <EmptyState>No bonds yet.</EmptyState>}
        </Card>
      )}

      {stakedFinalizers.length > 0 && (
        <Card className="space-y-1">
          <SectionTitle>You stake to ({stakedFinalizers.length})</SectionTitle>
          <Hint>From your bonds — tap to paste for stake / retarget.</Hint>
          <div className="max-h-32 space-y-0.5 overflow-y-auto">
            {stakedFinalizers.map((e) => {
              const row = hybridByKey.get(normalizeFinalizerHex(e.finalizer));
              return (
                <button
                  key={e.finalizer}
                  type="button"
                  className="nw-mono flex w-full items-center justify-between gap-1 rounded px-1.5 py-1 text-left text-[10px]"
                  style={{ color: "var(--nw-muted)" }}
                  onClick={() => {
                    setFinalizer(e.finalizer);
                    setLog("Finalizer pasted");
                  }}
                >
                  <span className="min-w-0 truncate">
                    {row?.rank != null ? `#${row.rank} · ` : ""}
                    {shortHex(e.finalizer)} · {e.bonds}b · {zatToCtaz(e.totalLatest)}
                  </span>
                  <FinalizerGradeBadge row={row} />
                </button>
              );
            })}
          </div>
        </Card>
      )}

      {roster.length > 0 && (
        <Card className="space-y-1">
          <SectionTitle>Roster ({roster.length})</SectionTitle>
          <Hint>
            Tap to paste. Observer rank + node grade (same as Desktop). Highest stake first.
          </Hint>
          <div className="max-h-40 space-y-0.5 overflow-y-auto">
            {roster.map((e, i) => {
              const row = hybridByKey.get(normalizeFinalizerHex(e.finalizer));
              const rank = row?.rank ?? i + 1;
              return (
                <button
                  key={e.finalizer}
                  type="button"
                  className="nw-mono flex w-full items-center justify-between gap-1 rounded px-1.5 py-1 text-left text-[10px]"
                  style={{ color: "var(--nw-faint)" }}
                  onClick={() => {
                    setFinalizer(e.finalizer);
                    setLog("Finalizer pasted");
                  }}
                >
                  <span className="min-w-0 truncate">
                    #{rank} {shortHex(e.finalizer)} · {zatToCtaz(e.stake_zat)} (
                    {(e.share * 100).toFixed(1)}%)
                  </span>
                  <FinalizerGradeBadge row={row} />
                </button>
              );
            })}
          </div>
        </Card>
      )}

      {log && <LogBlock>{log}</LogBlock>}
    </Screen>
  );
}

function gradeTone(standing: HybridPosStanding): CSSProperties {
  switch (standing) {
    case "reliable":
      return {
        borderColor: "var(--nw-success)",
        background: "var(--nw-success-soft)",
        color: "var(--nw-success)"
      };
    case "uneven":
      return {
        borderColor: "var(--nw-warn)",
        background: "var(--nw-warn-soft)",
        color: "var(--nw-warn)"
      };
    case "provisional":
      return {
        borderColor: "var(--nw-platinum-line)",
        background: "var(--nw-platinum-soft)",
        color: "var(--nw-platinum)"
      };
    default:
      return {
        borderColor: "var(--nw-border)",
        background: "var(--nw-surface-alt)",
        color: "var(--nw-faint)"
      };
  }
}

function FinalizerGradeBadge({ row }: { row: HybridPosFinalizer | null | undefined }) {
  if (!row?.grade) {
    return (
      <span
        className="shrink-0 rounded px-1 py-0.5 text-[9px] font-semibold"
        style={{
          border: "1px solid var(--nw-border)",
          color: "var(--nw-faint)"
        }}
      >
        —
      </span>
    );
  }
  const standing = hybridPosStanding(row);
  return (
    <span
      className="shrink-0 rounded px-1 py-0.5 text-[9px] font-bold tabular-nums"
      style={{ border: "1px solid", ...gradeTone(standing) }}
      title={hybridPosStandingLabel(standing)}
    >
      {row.grade}
    </span>
  );
}

function FinalizerInsightPanel({
  row,
  loading
}: {
  row: HybridPosFinalizer | null;
  loading: boolean;
}) {
  if (loading) {
    return <Hint>Loading finalizer grade from Crosslink Network observer…</Hint>;
  }
  if (!row) {
    return (
      <Hint>
        No observer data for this finalizer yet. Grades come from the independent Crosslink
        Network scoreboard.
      </Hint>
    );
  }

  const standing = hybridPosStanding(row);
  const votePct = row.pct ?? (row.of ? ((row.voted ?? 0) / row.of) * 100 : null);

  return (
    <div
      className="rounded-xl px-2.5 py-2 text-[10px] leading-relaxed"
      style={{ border: "1px solid", ...gradeTone(standing) }}
    >
      <div className="flex flex-wrap items-center gap-1.5">
        <span className="text-xs font-bold">{row.grade ?? "—"}</span>
        {row.score != null && (
          <span className="tabular-nums opacity-90">{row.score.toFixed(1)} score</span>
        )}
        {row.live === false && <span>offline</span>}
        {row.in_threshold_set && <span>top ⅓ stake</span>}
      </div>
      <p className="mt-1 font-medium">{hybridPosStandingLabel(standing)}</p>
      <div className="mt-1 flex flex-wrap gap-x-3 gap-y-0.5 tabular-nums opacity-90">
        <span>Rank #{row.rank ?? "—"}</span>
        {row.share_pct != null && <span>{row.share_pct.toFixed(2)}% share</span>}
        {votePct != null && (
          <span>
            {votePct.toFixed(1)}% finality votes
            {row.voted != null && row.of != null
              ? ` (${row.voted.toLocaleString()} / ${row.of.toLocaleString()})`
              : ""}
          </span>
        )}
      </div>
    </div>
  );
}
