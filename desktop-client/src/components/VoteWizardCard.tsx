import { useCallback, useEffect, useState, type ReactNode } from "react";
import toast from "react-hot-toast";
import { Button } from "./Button";
import { FeatureShell, MetricTile, Panel } from "./FeatureSurface";
import { walletApi } from "../lib/api";
import type { VoteStatusResponse } from "../lib/types";
import { formatErrorForDisplay } from "../utils/errors";

type Busy =
  | null
  | "refresh"
  | "export"
  | "prepare"
  | "delegate"
  | "sign"
  | "finish";

interface VoteWizardCardProps {
  onNavigateIronwood?: () => void;
}

function StepRow({
  step,
  title,
  detail,
  action,
}: {
  step: number;
  title: string;
  detail?: string;
  action: ReactNode;
}) {
  return (
    <div className="flex flex-col gap-3 rounded-xl border border-white/8 bg-black/20 px-4 py-3 sm:flex-row sm:items-center sm:justify-between">
      <div className="min-w-0 flex gap-3">
        <span className="flex h-7 w-7 shrink-0 items-center justify-center rounded-full border border-primary/30 bg-primary/10 text-xs font-bold text-primary">
          {step}
        </span>
        <div className="min-w-0">
          <p className="text-sm font-semibold text-gray-100">{title}</p>
          {detail ? <p className="mt-0.5 text-xs leading-relaxed text-gray-500">{detail}</p> : null}
        </div>
      </div>
      <div className="shrink-0 sm:pl-4">{action}</div>
    </div>
  );
}

export function VoteWizardCard({ onNavigateIronwood }: VoteWizardCardProps) {
  const [env, setEnv] = useState<"prod" | "stage">("prod");
  const [status, setStatus] = useState<VoteStatusResponse | null>(null);
  const [busy, setBusy] = useState<Busy>(null);
  const [password, setPassword] = useState("");
  const [lastDelegationTx, setLastDelegationTx] = useState<string | null>(null);
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);

  const load = useCallback(async () => {
    setBusy("refresh");
    try {
      const statusRes = await walletApi.voteStatus(env);
      setStatus(statusRes.data);
      setLastUpdated(new Date());
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Failed to load vote status"));
    } finally {
      setBusy(null);
    }
  }, [env]);

  useEffect(() => {
    void load();
  }, [load]);

  const run = async (kind: Busy, fn: () => Promise<void>) => {
    if (busy) return;
    setBusy(kind);
    try {
      await fn();
      await load();
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Vote action failed"));
    } finally {
      setBusy(null);
    }
  };

  const phaseLabel = status?.phase?.replace(/_/g, " ") ?? "—";
  const phaseTone =
    status?.phase === "voting_open"
      ? "border-emerald-400/35 bg-emerald-500/15 text-emerald-200"
      : status?.phase === "pre_snapshot"
        ? "border-amber-400/35 bg-amber-500/15 text-amber-200"
        : "border-white/10 bg-white/5 text-gray-300";

  const inputClass =
    "mt-1.5 w-full rounded-xl border border-white/10 bg-black/30 px-3.5 py-2.5 text-sm text-gray-100 placeholder:text-gray-500 shadow-inner shadow-black/20 transition focus:border-primary/40 focus:outline-none focus:ring-2 focus:ring-primary/20";

  return (
    <FeatureShell>
      {/* Hero */}
      <div className="flex flex-col gap-5 lg:flex-row lg:items-start lg:justify-between">
        <div className="min-w-0 space-y-3">
          <div className="flex flex-wrap items-center gap-2">
            <span className="rounded-full border border-primary/35 bg-primary/10 px-3 py-1 text-[0.65rem] font-bold uppercase tracking-[0.22em] text-primary">
              NU7
            </span>
            <span
              className={`rounded-full border px-3 py-1 text-[0.65rem] font-bold uppercase tracking-[0.16em] ${phaseTone}`}
            >
              {phaseLabel}
            </span>
            {lastUpdated ? (
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
              Coinholder ballot
            </h3>
            <p className="mt-2 max-w-2xl text-sm leading-relaxed text-gray-400">
              Weight = spendable Ironwood at snapshot. Seed stays in Nozy; voting hotkey is
              separate.
            </p>
          </div>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <select
            className="rounded-xl border border-white/10 bg-black/30 px-3 py-2 text-sm text-gray-100 focus:border-primary/40 focus:outline-none focus:ring-2 focus:ring-primary/20"
            value={env}
            onChange={(e) => setEnv(e.target.value as "prod" | "stage")}
            disabled={!!busy}
          >
            <option value="prod">prod</option>
            <option value="stage">stage</option>
          </select>
          <Button
            type="button"
            variant="secondary"
            size="sm"
            disabled={busy === "refresh"}
            onClick={() => void load()}
          >
            {busy === "refresh" ? "Refreshing…" : "Refresh"}
          </Button>
        </div>
      </div>

      {/* Calendar strip */}
      <div className="relative mt-4 overflow-hidden rounded-2xl border border-primary/20 bg-gradient-to-r from-primary/12 via-transparent to-emerald-500/5 px-5 py-4 backdrop-blur-sm">
        <div className="absolute inset-y-0 left-0 w-1 bg-gradient-to-b from-primary to-emerald-400" />
        <p className="text-[0.65rem] font-bold uppercase tracking-[0.2em] text-gray-400">
          Calendar
        </p>
        <p className="mt-1 text-sm text-gray-200">
          Snapshot {status?.snapshot_utc ?? "…"} · Vote {status?.vote_start_utc ?? "…"} →{" "}
          {status?.vote_end_utc ?? "…"}
        </p>
        {status?.phase_message ? (
          <p className="mt-2 text-sm leading-relaxed text-gray-300">{status.phase_message}</p>
        ) : null}
      </div>

      {/* Metrics */}
      <div className="relative mt-4 grid gap-3 sm:grid-cols-3">
        <MetricTile
          label="Notes exported"
          value={status?.notes_exported ? String(status.notes_count ?? "Yes") : "No"}
          accent={status?.notes_exported ? "emerald" : "default"}
          hint={status?.notes_exported ? "Snapshot witnesses ready" : "Run step 1"}
        />
        <MetricTile
          label="Hotkey"
          value={status?.hotkey_ready ? "Ready" : "—"}
          accent={status?.hotkey_ready ? "gold" : "default"}
          hint="Separate from wallet seed"
        />
        <MetricTile
          label="Phase"
          value={phaseLabel}
          accent={status?.phase === "pre_snapshot" ? "amber" : "default"}
        />
      </div>

      {status?.phase === "pre_snapshot" ? (
        <div className="relative mt-4 rounded-2xl border border-amber-400/25 bg-amber-500/10 px-4 py-3 text-sm text-amber-100/90">
          Migrate Orchard → Ironwood before the snapshot so notes count toward voting weight.{" "}
          {onNavigateIronwood ? (
            <button
              type="button"
              className="font-semibold text-primary underline decoration-primary/40 underline-offset-2 hover:decoration-primary"
              onClick={onNavigateIronwood}
            >
              Open Ironwood
            </button>
          ) : null}
        </div>
      ) : null}

      {/* Wizard steps */}
      <div className="relative mt-4 space-y-2">
        <p className="mb-3 text-[0.65rem] font-bold uppercase tracking-[0.18em] text-gray-400">
          Delegation wizard
        </p>
        <StepRow
          step={1}
          title="Export Ironwood notes"
          detail="At snapshot — skips rebuild if valid; otherwise can take a long time"
          action={
            <Button
              type="button"
              size="sm"
              disabled={!!busy}
              onClick={() =>
                void run("export", async () => {
                  toast("Checking / rebuilding snapshot witnesses…", {
                    icon: "⏳",
                    duration: 8000,
                  });
                  const res = await walletApi.voteExportNotes(password.trim() || undefined);
                  toast.success(res.data.message);
                })
              }
            >
              {busy === "export" ? "Exporting…" : "Export"}
            </Button>
          }
        />
        {status?.notes_exported ? (
          <p className="px-1 text-xs text-amber-200/85">
            Protocol minimum is 0.125 ZEC (12 500 000 zat) per ballot. Smaller balances export but
            cannot prepare/delegate.
          </p>
        ) : null}
        <StepRow
          step={2}
          title="Prepare hotkey & round"
          detail="Import notes into the vote environment"
          action={
            <Button
              type="button"
              size="sm"
              variant="outline"
              disabled={!!busy || !status?.notes_exported}
              onClick={() =>
                void run("prepare", async () => {
                  const res = await walletApi.votePrepare(env);
                  toast.success(res.data.message);
                })
              }
            >
              {busy === "prepare" ? "Preparing…" : "Prepare"}
            </Button>
          }
        />
        <StepRow
          step={3}
          title="Build delegation"
          detail="Creates the signing request"
          action={
            <Button
              type="button"
              size="sm"
              variant="outline"
              disabled={!!busy || !status?.notes_exported}
              onClick={() =>
                void run("delegate", async () => {
                  const res = await walletApi.voteDelegate(env);
                  toast.success(res.data.message);
                })
              }
            >
              {busy === "delegate" ? "Building…" : "Delegate"}
            </Button>
          }
        />
        <StepRow
          step={4}
          title="Sign with this wallet"
          detail="Seed never leaves Nozy"
          action={
            <Button
              type="button"
              size="sm"
              variant="outline"
              disabled={!!busy || !status?.signing_request_present}
              onClick={() =>
                void run("sign", async () => {
                  const res = await walletApi.voteSignDelegation(
                    password.trim() || undefined,
                    env
                  );
                  toast.success(res.data.message);
                })
              }
            >
              {busy === "sign" ? "Signing…" : "Sign"}
            </Button>
          }
        />
        <StepRow
          step={5}
          title="Prove + submit delegation"
          detail="PIR / ZKP can take several minutes"
          action={
            <Button
              type="button"
              size="sm"
              variant="outline"
              disabled={!!busy || !status?.sig_present}
              onClick={() =>
                void run("finish", async () => {
                  toast("PIR + ZKP1 proving started…", { icon: "⏳" });
                  const res = await walletApi.voteDelegateFinish(env, true);
                  setLastDelegationTx(res.data.tx_hash || null);
                  const short = res.data.tx_hash
                    ? `${res.data.tx_hash.slice(0, 16)}…`
                    : "see log";
                  if (res.data.confirmed) {
                    toast.success(`Delegation confirmed · ${short}`);
                  } else if (res.data.tx_hash) {
                    toast(
                      `Submitted ${short} — wait a minute, then cast (confirmation still pending)`,
                      { icon: "⏳", duration: 8000 }
                    );
                  } else {
                    toast.error("Delegation submit finished without a tx hash — check logs");
                  }
                })
              }
            >
              {busy === "finish" ? "Proving…" : "Submit"}
            </Button>
          }
        />
      </div>

      {/* Unlock + links */}
      <div className="relative mt-4 grid gap-3 lg:grid-cols-2">
        <Panel title="Wallet unlock" subtitle="Only when export / sign prompts for it">
          <label className="block text-sm text-gray-400">
            Password
            <input
              type="password"
              autoComplete="current-password"
              className={inputClass}
              value={password}
              onChange={(e) => setPassword(e.target.value)}
            />
          </label>
        </Panel>
        <Panel title="After delegation" subtitle="Ballot choices live on the Valar vote servers">
          {lastDelegationTx ? (
            <p className="mb-3 break-all font-mono text-xs text-emerald-200/90">
              Delegation tx: {lastDelegationTx}
            </p>
          ) : (
            <p className="mb-3 text-sm text-gray-400">
              Finish steps 1–5 here. Cast your ballot with the official Valar tools or{" "}
              <span className="font-mono text-gray-300">nozy-vote cast</span> once you know your
              choices.
            </p>
          )}
          <div className="flex flex-wrap gap-3">
            {status?.forum_url ? (
              <a
                className="text-sm font-medium text-primary underline decoration-primary/40 underline-offset-2 hover:decoration-primary"
                href={status.forum_url}
                target="_blank"
                rel="noreferrer"
              >
                Forum thread →
              </a>
            ) : null}
            <a
              className="text-sm font-medium text-primary underline decoration-primary/40 underline-offset-2 hover:decoration-primary"
              href="https://tally.valargroup.org"
              target="_blank"
              rel="noreferrer"
            >
              Tallies →
            </a>
          </div>
        </Panel>
      </div>
    </FeatureShell>
  );
}
