import { useCallback, useEffect, useState, type ReactNode } from "react";
import toast from "react-hot-toast";
import { Button } from "./Button";
import { walletApi } from "../lib/api";
import {
  ironwoodActionDisableReason,
  ironwoodNextStep,
} from "../lib/ironwoodNextStep";
import type { IronwoodDesktopStatusResponse } from "../lib/types";
import { useSettingsStore } from "../store/settingsStore";
import { formatErrorForDisplay } from "../utils/errors";
import { NYMVPN_IRONWOOD_STOPGAP_SHORT } from "../lib/nymIronwoodStopgap";
import { SendEgressCard } from "./SendEgressCard";

const REFRESH_MS = 60_000;
const ZAT_PER_ZEC = 100_000_000;

function formatZec(value: number | null | undefined): string {
  if (value == null) return "—";
  return value.toLocaleString(undefined, {
    minimumFractionDigits: 2,
    maximumFractionDigits: 8,
  });
}

function formatZatAsZec(valueZat: number): string {
  return formatZec(valueZat / ZAT_PER_ZEC);
}

function statusTone(status: IronwoodDesktopStatusResponse | null): {
  label: string;
  tone: "ok" | "warn" | "info" | "neutral";
} {
  if (!status) return { label: "Checking", tone: "neutral" };
  if (status.orchard_wallet_zat <= 0 && status.ironwood_active) {
    return { label: "Migration complete", tone: "ok" };
  }
  if (status.ironwood_active && status.ironwood_rpc_detected) {
    return { label: "Ironwood active", tone: "ok" };
  }
  if (status.ironwood_rpc_detected) return { label: "RPC ready", tone: "info" };
  if (status.ironwood_active) return { label: "Need Ironwood RPC", tone: "warn" };
  return { label: "Waiting on chain", tone: "warn" };
}

const pillTone: Record<string, string> = {
  ok: "border-emerald-400/35 bg-emerald-500/15 text-emerald-200",
  warn: "border-amber-400/35 bg-amber-500/15 text-amber-200",
  info: "border-sky-400/30 bg-sky-500/10 text-sky-200",
  neutral: "border-white/10 bg-white/5 text-gray-300",
};

function MetricTile({
  label,
  value,
  unit,
  hint,
  accent = "default",
}: {
  label: string;
  value: string;
  unit?: string;
  hint?: string;
  accent?: "default" | "amber" | "emerald";
}) {
  const accentCls =
    accent === "amber"
      ? "border-amber-400/30 bg-gradient-to-br from-amber-500/15 to-amber-950/20"
      : accent === "emerald"
        ? "border-emerald-400/30 bg-gradient-to-br from-emerald-500/15 to-emerald-950/20"
        : "border-white/10 bg-black/25";
  const labelCls =
    accent === "amber"
      ? "text-amber-200/90"
      : accent === "emerald"
        ? "text-emerald-200/90"
        : "text-gray-400";
  return (
    <div className={`rounded-2xl border p-4 backdrop-blur-sm ${accentCls}`}>
      <p className={`text-[0.65rem] font-bold uppercase tracking-[0.2em] ${labelCls}`}>{label}</p>
      <p className="mt-2 flex items-baseline gap-2">
        <span className="text-2xl font-extrabold tabular-nums tracking-tight text-primary-100">
          {value}
        </span>
        {unit ? <span className="text-sm font-medium text-gray-500">{unit}</span> : null}
      </p>
      {hint ? <p className="mt-1.5 text-xs leading-relaxed text-gray-400">{hint}</p> : null}
    </div>
  );
}

function Panel({
  title,
  children,
  className = "",
}: {
  title: string;
  children: ReactNode;
  className?: string;
}) {
  return (
    <div
      className={`rounded-2xl border border-white/10 bg-black/20 p-4 backdrop-blur-sm ${className}`}
    >
      <p className="text-[0.65rem] font-bold uppercase tracking-[0.18em] text-gray-400">{title}</p>
      <div className="mt-3">{children}</div>
    </div>
  );
}

export function IronwoodReadinessCard() {
  const [status, setStatus] = useState<IronwoodDesktopStatusResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState<"plan" | "split" | "migrate" | "broadcast" | "refresh" | null>(
    null
  );
  const [walletPassword, setWalletPassword] = useState("");
  const [walletHasPassword, setWalletHasPassword] = useState(false);
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);
  const attestPrivateNetworkForMigration = useSettingsStore(
    (s) => s.attestPrivateNetworkForMigration
  );

  const load = useCallback(async () => {
    try {
      const [res, walletStatus] = await Promise.all([
        walletApi.getIronwoodStatus({
          attest_private_network: attestPrivateNetworkForMigration,
        }),
        walletApi.getWalletStatus().catch(() => null),
      ]);
      setStatus(res.data);
      setError(null);
      setLastUpdated(new Date());
      if (walletStatus?.data) {
        setWalletHasPassword(Boolean(walletStatus.data.has_password));
      }
    } catch (e) {
      setError(
        formatErrorForDisplay(e, "Ironwood status unavailable. Check desktop backend and Zebra RPC.")
      );
      setStatus(null);
    } finally {
      setLoading(false);
    }
  }, [attestPrivateNetworkForMigration]);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      if (cancelled) return;
      await load();
    })();
    const timer = window.setInterval(() => {
      if (!cancelled) void load();
    }, REFRESH_MS);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, [load]);

  const tone = statusTone(status);
  const next = ironwoodNextStep(status);
  const safer = status?.safer_migration;
  const blockers =
    status?.blockers.length && status.blockers.length > 0
      ? status.blockers
      : ["No migration blockers reported."];
  const migrationDone =
    Boolean(status?.ironwood_active) && (status?.orchard_wallet_zat ?? 0) <= 0;
  const residualOrchard = (status?.orchard_wallet_zat ?? 0) > 0;

  const canPlan = Boolean(status?.migration_enabled) && busy == null;
  const canSplit =
    Boolean(status?.migration_enabled && status.ironwood_active && status.zip318_note_split_required) &&
    busy == null;
  const canMigrate =
    Boolean(status?.migration_enabled && status.ready_to_prebuild && !status.zip318_note_split_required) &&
    busy == null;
  const canBroadcast =
    Boolean(status?.migration_enabled && status.ready_to_broadcast && safer?.network_privacy_allowed) &&
    busy == null;

  const primary = next?.primaryAction;
  const disableSplit = ironwoodActionDisableReason("split", status, safer?.network_privacy_allowed);
  const disableMigrate = ironwoodActionDisableReason(
    "migrate",
    status,
    safer?.network_privacy_allowed
  );
  const disableBroadcast = ironwoodActionDisableReason(
    "broadcast",
    status,
    safer?.network_privacy_allowed
  );

  const onPlan = async () => {
    setBusy("plan");
    try {
      const res = await walletApi.ironwoodPlanSave();
      toast.success(res.data.message);
      await load();
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Failed to save migration plan"));
    } finally {
      setBusy(null);
    }
  };

  const stepUpPassword = (): string | undefined => {
    const trimmed = walletPassword.trim();
    if (walletHasPassword && !trimmed) {
      toast.error("Enter your wallet password to split, migrate, or broadcast.");
      return undefined;
    }
    return trimmed || undefined;
  };

  const onSplit = async () => {
    const password = stepUpPassword();
    if (walletHasPassword && password === undefined) return;
    setBusy("split");
    try {
      const res = await walletApi.ironwoodSplit({ password });
      toast.success(res.data.message);
      await load();
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Note split failed"));
    } finally {
      setBusy(null);
    }
  };

  const onMigrate = async () => {
    const password = stepUpPassword();
    if (walletHasPassword && password === undefined) return;
    setBusy("migrate");
    try {
      const res = await walletApi.ironwoodMigrate({ password });
      if (res.data.prepared_txid) {
        toast.success(res.data.message);
      } else if (res.data.blockers.length > 0) {
        toast.error(res.data.blockers[0] ?? res.data.message);
      } else {
        toast.success(res.data.message);
      }
      await load();
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Migration prebuild failed"));
    } finally {
      setBusy(null);
    }
  };

  const onBroadcast = async () => {
    const password = stepUpPassword();
    if (walletHasPassword && password === undefined) return;
    setBusy("broadcast");
    try {
      const res = await walletApi.ironwoodBroadcast({
        password,
        attest_private_network: attestPrivateNetworkForMigration,
        wait_confirm: false,
      });
      if (res.data.blockers.length > 0 && !res.data.txid) {
        toast.error(res.data.blockers[0] ?? res.data.message);
      } else if (res.data.blockers.length > 0) {
        toast(res.data.message, { icon: "⚠️" });
      } else {
        toast.success(res.data.message);
      }
      await load();
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Migration broadcast failed"));
    } finally {
      setBusy(null);
    }
  };

  const onRefresh = async () => {
    setBusy("refresh");
    try {
      await load();
    } finally {
      setBusy(null);
    }
  };

  const inputClass =
    "w-full min-w-[16rem] rounded-xl border border-white/10 bg-black/30 px-3.5 py-2.5 text-sm text-gray-100 placeholder:text-gray-500 shadow-inner shadow-black/20 transition focus:border-emerald-400/40 focus:outline-none focus:ring-2 focus:ring-emerald-500/20";

  return (
    <section className="relative overflow-hidden rounded-2xl border border-emerald-500/20 bg-gradient-to-br from-emerald-950/40 via-[#0c0b09] to-amber-950/30 p-6 shadow-2xl shadow-emerald-950/20">
      <div
        aria-hidden
        className="pointer-events-none absolute -right-24 -top-24 h-64 w-64 rounded-full bg-emerald-600/15 blur-3xl"
      />
      <div
        aria-hidden
        className="pointer-events-none absolute -bottom-20 -left-16 h-52 w-52 rounded-full bg-amber-500/10 blur-3xl"
      />

      {/* Hero */}
      <div className="relative flex flex-col gap-5 lg:flex-row lg:items-start lg:justify-between">
        <div className="min-w-0 space-y-3">
          <div className="flex flex-wrap items-center gap-2">
            <span className="rounded-full border border-emerald-400/30 bg-emerald-500/10 px-3 py-1 text-[0.65rem] font-bold uppercase tracking-[0.22em] text-emerald-200">
              NU6.3
            </span>
            <span
              className={`rounded-full border px-3 py-1 text-[0.65rem] font-bold uppercase tracking-[0.16em] ${pillTone[tone.tone]}`}
            >
              {loading ? "Checking" : tone.label}
            </span>
            {status?.readiness_state ? (
              <span className="rounded-full border border-white/10 bg-white/5 px-2.5 py-1 text-[0.65rem] font-medium text-gray-400">
                {status.readiness_state}
              </span>
            ) : null}
            {!loading && lastUpdated ? (
              <span className="inline-flex items-center gap-1.5 rounded-full border border-white/10 bg-white/5 px-2.5 py-1 text-[0.65rem] font-medium text-gray-400">
                <span className="relative flex h-1.5 w-1.5">
                  <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-400 opacity-60" />
                  <span className="relative inline-flex h-1.5 w-1.5 rounded-full bg-emerald-400" />
                </span>
                Live · {lastUpdated.toLocaleTimeString()}
              </span>
            ) : null}
          </div>
          <div>
            <h3 className="text-2xl font-extrabold tracking-tight text-primary-100">
              {migrationDone ? "Ironwood pool ready" : "Orchard → Ironwood"}
            </h3>
            <p className="mt-2 max-w-2xl text-sm leading-relaxed text-gray-400">
              {migrationDone
                ? "No residual Orchard in this wallet. Prefer a local Zebrad — new shielded outputs go to Ironwood."
                : "ZIP 318 migration: Plan → Split (if needed) → Migrate → Broadcast. Prefer local Zebrad."}
            </p>
          </div>
        </div>
        <Button
          type="button"
          variant="secondary"
          size="sm"
          className="shrink-0 self-start"
          disabled={busy === "refresh"}
          onClick={() => void onRefresh()}
        >
          {busy === "refresh" ? "Refreshing…" : "Refresh"}
        </Button>
      </div>

      {/* Pool balances */}
      <div className="relative mt-6 grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        <MetricTile
          label="Orchard wallet"
          value={status ? formatZatAsZec(status.orchard_wallet_zat) : "—"}
          unit="ZEC"
          accent={residualOrchard ? "amber" : "default"}
          hint={residualOrchard ? "migrate via turnstile" : "Cleared"}
        />
        <MetricTile
          label="Ironwood wallet"
          value={status ? formatZatAsZec(status.ironwood_wallet_zat) : "—"}
          unit="ZEC"
          accent={status && status.ironwood_wallet_zat > 0 ? "emerald" : "default"}
          hint={
            status && status.ironwood_wallet_zat > 0
              ? status.ironwood_note_count && status.ironwood_note_count > 1
                ? `${status.ironwood_note_count} notes · largest ${formatZatAsZec(status.ironwood_max_note_zat ?? 0)} · max send ${formatZatAsZec(status.ironwood_max_send_zat ?? 0)}`
                : "One note — spendable after sync"
              : "Sync if you expect a receive"
          }
        />
        <MetricTile
          label="Activation"
          value={
            status?.activation_height
              ? status.activation_height.toLocaleString()
              : status
                ? status.activation_target_date
                : "—"
          }
          hint={status?.activation_height ? status.activation_target_date : "Target date"}
        />
        <MetricTile
          label="Zebra RPC"
          value={status?.ironwood_rpc_detected ? "Detected" : "Missing"}
          hint={status?.ironwood_rpc_detected ? "Ironwood pool on node" : "Upgrade / point at NU6.3 node"}
        />
      </div>

      {/* Next step */}
      {next ? (
        <div
          className={`relative mt-4 overflow-hidden rounded-2xl border px-5 py-4 backdrop-blur-sm ${
            migrationDone
              ? "border-emerald-400/25 bg-gradient-to-r from-emerald-500/15 via-transparent to-emerald-500/5"
              : "border-amber-400/25 bg-gradient-to-r from-amber-500/12 via-transparent to-emerald-500/5"
          }`}
        >
          <div
            className={`absolute inset-y-0 left-0 w-1 ${
              migrationDone
                ? "bg-gradient-to-b from-emerald-400 to-teal-400"
                : "bg-gradient-to-b from-amber-400 to-emerald-400"
            }`}
          />
          <p className="text-[0.65rem] font-bold uppercase tracking-[0.2em] text-gray-400">
            {migrationDone ? "Status" : "Next step"}
          </p>
          <p className="mt-1 text-lg font-bold text-primary-100">{next.title}</p>
          <p className="mt-1.5 text-sm leading-relaxed text-gray-300">{next.detail}</p>
          {residualOrchard && status ? (
            <p className="mt-3 text-[0.65rem] font-medium uppercase tracking-[0.14em] text-amber-200/80">
              Residual {formatZatAsZec(status.orchard_wallet_zat)} ZEC ·{" "}
              {status.zip318_transfer_count.toLocaleString()} transfer(s) ·{" "}
              {formatZatAsZec(status.migration_zat)} ZEC planned
            </p>
          ) : null}
        </div>
      ) : null}

      {error ? (
        <div className="relative mt-4 rounded-2xl border border-red-400/30 bg-red-500/10 px-4 py-3 text-sm text-red-200">
          {error}
        </div>
      ) : null}

      {/* Actions */}
      {!migrationDone ? (
        <div className="relative mt-4 rounded-2xl border border-white/10 bg-black/25 p-4 backdrop-blur-sm">
          <div className="flex flex-col gap-3 lg:flex-row lg:items-end lg:justify-between">
            <div className="min-w-0">
              <p className="text-[0.65rem] font-bold uppercase tracking-[0.18em] text-gray-400">
                Migration actions
              </p>
              <p className="mt-1 text-xs text-gray-500">
                Greyed steps unlock after Plan (and when the ZIP 318 window is open).
              </p>
            </div>
            {walletHasPassword ? (
              <input
                type="password"
                autoComplete="current-password"
                value={walletPassword}
                onChange={(e) => setWalletPassword(e.target.value)}
                placeholder="Wallet password"
                className={inputClass}
              />
            ) : null}
          </div>
          <div className="mt-4 flex flex-wrap gap-2">
            <Button
              variant={primary === "split" ? "primary" : "outline"}
              disabled={!canSplit}
              title={disableSplit ?? undefined}
              onClick={() => void onSplit()}
            >
              {busy === "split" ? "Splitting…" : "Split"}
            </Button>
            <Button
              variant={primary === "plan" ? "primary" : "outline"}
              disabled={!canPlan}
              onClick={() => void onPlan()}
            >
              {busy === "plan" ? "Planning…" : "Plan"}
            </Button>
            <Button
              variant={primary === "migrate" ? "primary" : "outline"}
              disabled={!canMigrate}
              title={disableMigrate ?? undefined}
              onClick={() => void onMigrate()}
            >
              {busy === "migrate" ? "Migrating…" : "Migrate"}
            </Button>
            <Button
              variant={primary === "broadcast" ? "primary" : "outline"}
              disabled={!canBroadcast}
              title={disableBroadcast ?? undefined}
              onClick={() => void onBroadcast()}
            >
              {busy === "broadcast" ? "Broadcasting…" : "Broadcast"}
            </Button>
          </div>
          {(disableMigrate || disableBroadcast || disableSplit) ? (
            <p className="mt-3 text-xs leading-5 text-amber-200/85">
              {primary === "plan" || status?.readiness_state === "needs-plan"
                ? "Greyed buttons unlock after Plan saves the ZIP 318 schedule."
                : disableMigrate || disableBroadcast || disableSplit}
            </p>
          ) : null}
        </div>
      ) : null}

      {!migrationDone && (status?.activation_notice || status?.orchard_funds_at_risk) ? (
        <div className="relative mt-4 rounded-2xl border border-amber-400/20 bg-amber-500/8 px-4 py-3 text-sm text-amber-100/90">
          <p className="text-[0.65rem] font-bold uppercase tracking-[0.18em] text-amber-200/90">
            Why migrate
          </p>
          <p className="mt-2 leading-relaxed">{status.activation_notice}</p>
          {status.migration_privacy_warnings?.length ? (
            <ul className="mt-2 list-disc space-y-1 pl-5 text-amber-100/80">
              {status.migration_privacy_warnings.map((warning) => (
                <li key={warning}>{warning}</li>
              ))}
            </ul>
          ) : null}
        </div>
      ) : null}

      <div className="relative mt-4">
        <SendEgressCard compact />
      </div>

      {safer?.baseline_hygiene_notes && safer.baseline_hygiene_notes.length > 0 ? (
        <div className="relative mt-4 rounded-2xl border border-emerald-400/20 bg-emerald-500/8 px-4 py-3 text-sm text-emerald-100/90">
          <p className="text-[0.65rem] font-bold uppercase tracking-[0.18em] text-emerald-200/90">
            Baseline hygiene
          </p>
          <p className="mt-1 text-xs leading-relaxed text-emerald-100/70">
            Timing and start-height disciplines Nym transport cannot provide.
          </p>
          <ul className="mt-2 list-disc space-y-1 pl-5 text-emerald-100/80">
            {safer.baseline_hygiene_notes.map((note) => (
              <li key={note}>{note}</li>
            ))}
          </ul>
        </div>
      ) : null}

      {safer && !safer.network_privacy_allowed ? (
        <div className="relative mt-4 rounded-2xl border border-amber-400/25 bg-amber-500/10 px-4 py-3 text-sm text-amber-100/90">
          Remote clearnet Zebrad is blocked for safer migration. Point Network &amp; Node at a
          local Zebrad. {NYMVPN_IRONWOOD_STOPGAP_SHORT} Attestation: Settings → Network privacy →
          Advanced.
        </div>
      ) : null}

      <div className="relative mt-4 grid gap-3 lg:grid-cols-2">
        <Panel title="Migration plan">
          <div className="grid gap-3 sm:grid-cols-3">
            <div>
              <p className="text-[0.65rem] uppercase tracking-wide text-gray-500">Notes</p>
              <p className="mt-1 text-lg font-bold tabular-nums text-gray-100">
                {status ? status.migration_note_count.toLocaleString() : "—"}
              </p>
            </div>
            <div>
              <p className="text-[0.65rem] uppercase tracking-wide text-gray-500">Amount</p>
              <p className="mt-1 text-lg font-bold tabular-nums text-gray-100">
                {status ? `${formatZatAsZec(status.migration_zat)} ZEC` : "—"}
              </p>
            </div>
            <div>
              <p className="text-[0.65rem] uppercase tracking-wide text-gray-500">Next bucket</p>
              <p className="mt-1 text-lg font-bold tabular-nums text-gray-100">
                {status?.next_anchor_bucket_height
                  ? status.next_anchor_bucket_height.toLocaleString()
                  : status
                    ? "None"
                    : "—"}
              </p>
            </div>
          </div>
          <p className="mt-3 text-xs leading-relaxed text-gray-500">
            ZIP 318 · {status ? status.zip318_transfer_count.toLocaleString() : "—"} transfers ·
            split{" "}
            {status?.zip318_note_split_required ? "required" : "not required"} · readiness{" "}
            {status?.readiness_state ?? "—"}
          </p>
        </Panel>

        <Panel title="Safety gates">
          <ul className="space-y-2.5 text-sm text-gray-300">
            {blockers.map((blocker) => (
              <li key={blocker} className="flex gap-2.5">
                <span className="mt-1.5 h-1.5 w-1.5 shrink-0 rounded-full bg-amber-400" />
                <span className="leading-snug">{blocker}</span>
              </li>
            ))}
          </ul>
        </Panel>
      </div>

      <div className="relative mt-5 flex flex-wrap items-center gap-x-4 gap-y-2 border-t border-white/5 pt-4 text-[0.65rem] font-medium uppercase tracking-[0.14em] text-gray-500">
        <span>Network {status?.network ?? "—"}</span>
        <span>Tip {status?.chain_tip?.toLocaleString() ?? "—"}</span>
        <span>Chain Orchard {formatZec(status?.orchard_chain_value_zec)}</span>
        <span>Chain Ironwood {formatZec(status?.ironwood_chain_value_zec)}</span>
        <span>
          IP{" "}
          {safer?.network_privacy_allowed
            ? safer.network_privacy_mode ?? "allowed"
            : "blocked"}
        </span>
      </div>
    </section>
  );
}
