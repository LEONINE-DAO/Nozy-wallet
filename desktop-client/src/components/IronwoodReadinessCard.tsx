import { useCallback, useEffect, useState } from "react";
import toast from "react-hot-toast";
import { Button } from "./Button";
import { walletApi } from "../lib/api";
import type { IronwoodDesktopStatusResponse } from "../lib/types";
import { useSettingsStore } from "../store/settingsStore";
import { formatErrorForDisplay } from "../utils/errors";

const REFRESH_MS = 60_000;
const ZAT_PER_ZEC = 100_000_000;

function formatZec(value: number | null | undefined): string {
  if (value == null) return "Unavailable";
  return `${value.toLocaleString(undefined, {
    minimumFractionDigits: 2,
    maximumFractionDigits: 8,
  })} ZEC`;
}

function formatZatAsZec(valueZat: number): string {
  return formatZec(valueZat / ZAT_PER_ZEC);
}

function statusTone(status: IronwoodDesktopStatusResponse | null): {
  label: string;
  className: string;
} {
  if (!status) {
    return {
      label: "Checking",
      className:
        "bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 border-gray-200 dark:border-gray-700",
    };
  }
  if (status.ironwood_active && status.ironwood_rpc_detected) {
    return {
      label: "Ironwood detected",
      className:
        "bg-emerald-50 dark:bg-emerald-900/30 text-emerald-700 dark:text-emerald-300 border-emerald-200 dark:border-emerald-800/50",
    };
  }
  if (status.ironwood_rpc_detected) {
    return {
      label: "RPC ready",
      className:
        "bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 border-blue-200 dark:border-blue-800/50",
    };
  }
  return {
    label: "Mainnet pending",
    className:
      "bg-amber-50 dark:bg-amber-900/30 text-amber-700 dark:text-amber-300 border-amber-200 dark:border-amber-800/50",
  };
}

function DetailStat({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-2xl border border-white/60 dark:border-gray-700/50 bg-white/55 dark:bg-gray-800/50 p-4 backdrop-blur-sm">
      <p className="text-xs font-semibold uppercase tracking-[0.18em] text-gray-500 dark:text-gray-400">
        {label}
      </p>
      <p className="mt-2 text-lg font-bold text-gray-900 dark:text-gray-100">{value}</p>
    </div>
  );
}

export function IronwoodReadinessCard() {
  const [status, setStatus] = useState<IronwoodDesktopStatusResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState<"plan" | "split" | "migrate" | "broadcast" | null>(null);
  const attestPrivateNetworkForMigration = useSettingsStore(
    (s) => s.attestPrivateNetworkForMigration
  );

  const load = useCallback(async () => {
    try {
      const res = await walletApi.getIronwoodStatus({
        attest_private_network: attestPrivateNetworkForMigration,
      });
      setStatus(res.data);
      setError(null);
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
  const safer = status?.safer_migration;
  const blockers =
    status?.blockers.length && status.blockers.length > 0
      ? status.blockers
      : ["No migration blockers reported."];

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

  const onSplit = async () => {
    setBusy("split");
    try {
      const res = await walletApi.ironwoodSplit({});
      toast.success(res.data.message);
      await load();
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Note split failed"));
    } finally {
      setBusy(null);
    }
  };

  const onMigrate = async () => {
    setBusy("migrate");
    try {
      const res = await walletApi.ironwoodMigrate({});
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
    setBusy("broadcast");
    try {
      const res = await walletApi.ironwoodBroadcast({
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

  return (
    <section className="overflow-hidden rounded-2xl border border-amber-200/70 dark:border-amber-800/40 bg-gradient-to-br from-amber-50 via-white to-emerald-50 dark:from-amber-950/30 dark:via-gray-900 dark:to-emerald-950/20 p-6 shadow-xl shadow-amber-900/5">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <div>
          <div className="flex flex-wrap items-center gap-3">
            <p className="text-xs font-bold uppercase tracking-[0.24em] text-amber-700 dark:text-amber-400">
              Ironwood / NU6.3 Readiness
            </p>
            <span className={`rounded-full border px-3 py-1 text-xs font-semibold ${tone.className}`}>
              {loading ? "Checking" : tone.label}
            </span>
            {status?.readiness_state && (
              <span className="rounded-full border border-gray-200 dark:border-gray-700 px-3 py-1 text-xs font-semibold text-gray-600 dark:text-gray-300">
                {status.readiness_state}
              </span>
            )}
          </div>
          <h3 className="mt-3 text-2xl font-extrabold text-gray-950 dark:text-gray-100">
            Safer Orchard → Ironwood migration
          </h3>
          <p className="mt-2 max-w-3xl text-sm leading-6 text-gray-600 dark:text-gray-400">
            Prefer a <strong>local Zebrad</strong>. If ZIP 318 requires it, Split first; then Plan
            saves the schedule, Migrate prebuilds the next turnstile, Broadcast submits it
            in-window.
            {status?.zip318_note_split_required
              ? " Note split is required before Migrate — use Split below."
              : null}
          </p>
        </div>

        <div className="flex flex-wrap gap-2">
          <Button
            variant="outline"
            disabled={!canSplit}
            onClick={() => void onSplit()}
          >
            {busy === "split" ? "Splitting…" : "Split notes"}
          </Button>
          <Button variant="outline" disabled={!canPlan} onClick={() => void onPlan()}>
            {busy === "plan" ? "Planning…" : "Plan migration"}
          </Button>
          <Button disabled={!canMigrate} onClick={() => void onMigrate()}>
            {busy === "migrate" ? "Migrating…" : "Start migration"}
          </Button>
          <Button
            variant="outline"
            disabled={!canBroadcast}
            onClick={() => void onBroadcast()}
          >
            {busy === "broadcast" ? "Broadcasting…" : "Broadcast"}
          </Button>
        </div>
      </div>

      {error && (
        <div className="mt-5 rounded-2xl border border-red-200 dark:border-red-800/50 bg-red-50 dark:bg-red-900/20 p-4 text-sm text-red-700 dark:text-red-300">
          {error}
        </div>
      )}

      <div className="mt-6 grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        <DetailStat
          label="Activation"
          value={
            status?.activation_height
              ? `Height ${status.activation_height.toLocaleString()}`
              : status
                ? `Target ${status.activation_target_date}`
                : "Checking"
          }
        />
        <DetailStat
          label="Zebra RPC"
          value={status?.ironwood_rpc_detected ? "Ironwood pool detected" : "Not detected yet"}
        />
        <DetailStat
          label="Orchard wallet"
          value={status ? formatZatAsZec(status.orchard_wallet_zat) : "Checking"}
        />
        <DetailStat
          label="Ironwood wallet"
          value={status ? formatZatAsZec(status.ironwood_wallet_zat) : "Checking"}
        />
      </div>

      {safer && !safer.network_privacy_allowed && (
        <div className="mt-4 rounded-2xl border border-amber-200/70 dark:border-amber-800/40 bg-amber-50/70 dark:bg-amber-950/20 p-4 text-sm text-amber-900 dark:text-amber-200">
          Remote clearnet Zebrad is blocked for safer migration. Point Network &amp; Node at a
          local Zebrad. Nym/Tor attestation is under Settings → Network privacy → Advanced.
        </div>
      )}

      <div className="mt-5 grid gap-4 lg:grid-cols-[1.1fr_0.9fr]">
        <div className="rounded-2xl border border-white/60 dark:border-gray-700/50 bg-white/60 dark:bg-gray-800/50 p-4">
          <p className="text-sm font-bold text-gray-900 dark:text-gray-100">Migration plan</p>
          <div className="mt-3 grid gap-3 text-sm text-gray-600 dark:text-gray-400 sm:grid-cols-3">
            <div>
              <p className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400">Notes</p>
              <p className="mt-1 font-semibold text-gray-900 dark:text-gray-100">
                {status ? status.migration_note_count.toLocaleString() : "Checking"}
              </p>
            </div>
            <div>
              <p className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400">Amount</p>
              <p className="mt-1 font-semibold text-gray-900 dark:text-gray-100">
                {status ? formatZatAsZec(status.migration_zat) : "Checking"}
              </p>
            </div>
            <div>
              <p className="text-xs uppercase tracking-wide text-gray-500 dark:text-gray-400">
                Next bucket
              </p>
              <p className="mt-1 font-semibold text-gray-900 dark:text-gray-100">
                {status?.next_anchor_bucket_height
                  ? status.next_anchor_bucket_height.toLocaleString()
                  : status
                    ? "No transfers"
                    : "Checking"}
              </p>
            </div>
          </div>
          <p className="mt-3 text-xs text-gray-500 dark:text-gray-400">
            ZIP 318 transfers: {status ? status.zip318_transfer_count.toLocaleString() : "checking"}
            . Note splitting:{" "}
            {status?.zip318_note_split_required ? "required (use Split)" : "not required or no plan yet"}
            . Readiness: {status?.readiness_state ?? "checking"}.
          </p>
        </div>

        <div className="rounded-2xl border border-white/60 dark:border-gray-700/50 bg-white/60 dark:bg-gray-800/50 p-4">
          <p className="text-sm font-bold text-gray-900 dark:text-gray-100">Safety gates</p>
          <ul className="mt-3 space-y-2 text-sm text-gray-600 dark:text-gray-400">
            {blockers.map((blocker) => (
              <li key={blocker} className="flex gap-2">
                <span className="mt-1 h-2 w-2 shrink-0 rounded-full bg-amber-500" />
                <span>{blocker}</span>
              </li>
            ))}
          </ul>
        </div>
      </div>

      <div className="mt-5 flex flex-wrap items-center gap-x-4 gap-y-2 text-xs font-medium uppercase tracking-[0.16em] text-gray-500 dark:text-gray-400">
        <span>Network: {status?.network ?? "unknown"}</span>
        <span>Tip: {status?.chain_tip?.toLocaleString() ?? "unavailable"}</span>
        <span>Chain Orchard: {formatZec(status?.orchard_chain_value_zec)}</span>
        <span>Chain Ironwood: {formatZec(status?.ironwood_chain_value_zec)}</span>
        <span>
          IP gate:{" "}
          {safer?.network_privacy_allowed
            ? safer.network_privacy_mode ?? "allowed"
            : "blocked — use local node"}
        </span>
      </div>
    </section>
  );
}
