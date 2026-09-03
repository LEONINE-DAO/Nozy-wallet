import { useCallback, useEffect, useState } from "react";
import toast from "react-hot-toast";
import { Button } from "./Button";
import { walletApi } from "../lib/api";
import type {
  VoteActiveRound,
  VoteStatusResponse,
} from "../lib/types";
import { formatErrorForDisplay } from "../utils/errors";

type Busy =
  | null
  | "refresh"
  | "export"
  | "prepare"
  | "delegate"
  | "sign"
  | "finish"
  | "cast";

interface VoteWizardCardProps {
  onNavigateIronwood?: () => void;
}

export function VoteWizardCard({ onNavigateIronwood }: VoteWizardCardProps) {
  const [env, setEnv] = useState<"prod" | "stage">("prod");
  const [status, setStatus] = useState<VoteStatusResponse | null>(null);
  const [active, setActive] = useState<VoteActiveRound | null>(null);
  const [activeError, setActiveError] = useState<string | null>(null);
  const [choices, setChoices] = useState<Record<string, number>>({});
  const [busy, setBusy] = useState<Busy>(null);
  const [password, setPassword] = useState("");
  const [lastDelegationTx, setLastDelegationTx] = useState<string | null>(null);

  const load = useCallback(async () => {
    setBusy("refresh");
    try {
      const statusRes = await walletApi.voteStatus(env);
      setStatus(statusRes.data);
      try {
        const activeRes = await walletApi.voteActive(env);
        setActive(activeRes.data);
        setActiveError(null);
        setChoices((prev) => {
          const next = { ...prev };
          for (const p of activeRes.data.proposals ?? []) {
            const key = String(p.id);
            if (next[key] == null && p.options.length > 0) {
              next[key] = p.options[0].index ?? 0;
            }
          }
          return next;
        });
      } catch (e) {
        setActive(null);
        setActiveError(formatErrorForDisplay(e, "Failed to load active round"));
      }
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

  return (
    <div className="rounded-2xl border border-gray-700/60 bg-gray-900/40 p-6 flex flex-col gap-6">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div className="min-w-0">
          <p className="text-xs font-semibold uppercase tracking-[0.18em] text-gray-500">
            Calendar
          </p>
          <p className="mt-1 text-sm text-gray-300">
            Snapshot {status?.snapshot_utc ?? "…"} · Vote{" "}
            {status?.vote_start_utc ?? "…"} → {status?.vote_end_utc ?? "…"}
          </p>
          {status && (
            <p className="mt-2 text-sm text-gray-200">{status.phase_message}</p>
          )}
        </div>
        <div className="flex items-center gap-2">
          <select
            className="rounded-xl border border-gray-600 bg-gray-800 px-3 py-2 text-sm text-gray-100"
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
            disabled={busy === "refresh"}
            onClick={() => void load()}
          >
            Refresh
          </Button>
        </div>
      </div>

      <div className="grid gap-3 sm:grid-cols-3">
        <Stat
          label="Notes exported"
          value={
            status?.notes_exported
              ? String(status.notes_count ?? "yes")
              : "No"
          }
        />
        <Stat label="Hotkey" value={status?.hotkey_ready ? "Ready" : "—"} />
        <Stat
          label="Phase"
          value={status?.phase?.replace(/_/g, " ") ?? "—"}
        />
      </div>

      {status?.phase === "pre_snapshot" && (
        <div className="rounded-xl border border-amber-700/50 bg-amber-950/30 px-4 py-3 text-sm text-amber-100">
          Migrate Orchard → Ironwood before the snapshot so notes count toward
          voting weight.{" "}
          {onNavigateIronwood && (
            <button
              type="button"
              className="underline font-semibold"
              onClick={onNavigateIronwood}
            >
              Open Ironwood
            </button>
          )}
        </div>
      )}

      <ol className="flex flex-col gap-4 list-decimal list-inside text-sm text-gray-200">
        <li className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
          <span>
            Export Ironwood notes at snapshot (skips rebuild if already valid; otherwise can take a long time)
          </span>
          <Button
            type="button"
            disabled={!!busy}
            onClick={() =>
              void run("export", async () => {
                toast("Checking / rebuilding snapshot witnesses…", {
                  icon: "⏳",
                  duration: 8000,
                });
                const res = await walletApi.voteExportNotes(
                  password.trim() || undefined
                );
                toast.success(res.data.message);
              })
            }
          >
            {busy === "export" ? "Exporting…" : "1. Export"}
          </Button>
        </li>
        {status?.notes_exported && (
          <p className="text-xs text-amber-200/90 -mt-2 ml-5">
            Protocol minimum is 0.125 ZEC (12 500 000 zat) per ballot. Smaller Ironwood
            balances at the snapshot are exported but cannot be prepared/delegated.
          </p>
        )}
        <li className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
          <span>Prepare hotkey, round, and import notes</span>
          <Button
            type="button"
            disabled={!!busy || !status?.notes_exported}
            onClick={() =>
              void run("prepare", async () => {
                const res = await walletApi.votePrepare(env);
                toast.success(res.data.message);
              })
            }
          >
            {busy === "prepare" ? "Preparing…" : "2. Prepare"}
          </Button>
        </li>
        <li className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
          <span>Build delegation (signing request)</span>
          <Button
            type="button"
            disabled={!!busy || !status?.notes_exported}
            onClick={() =>
              void run("delegate", async () => {
                const res = await walletApi.voteDelegate(env);
                toast.success(res.data.message);
              })
            }
          >
            {busy === "delegate" ? "Building…" : "3. Delegate"}
          </Button>
        </li>
        <li className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
          <span>Sign with this wallet (seed never leaves Nozy)</span>
          <Button
            type="button"
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
            {busy === "sign" ? "Signing…" : "4. Sign"}
          </Button>
        </li>
        <li className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
          <span>Prove + submit delegation (can take several minutes)</span>
          <Button
            type="button"
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
                    { icon: "⏳", duration: 8000 },
                  );
                } else {
                  toast.error("Delegation submit finished without a tx hash — check logs");
                }
              })
            }
          >
            {busy === "finish" ? "Proving…" : "5. Submit delegation"}
          </Button>
        </li>
      </ol>

      <div className="border-t border-gray-700 pt-4 flex flex-col gap-4">
        <div>
          <p className="text-xs font-semibold uppercase tracking-[0.18em] text-gray-500">
            Ballot
          </p>
          {activeError && (
            <p className="mt-2 text-sm text-amber-200">
              No active round yet: {activeError}
            </p>
          )}
          {active && (
            <div className="mt-3 flex flex-col gap-4">
              {active.title && (
                <p className="font-semibold text-gray-100">{active.title}</p>
              )}
              {(active.proposals ?? []).map((p) => (
                <fieldset key={p.id} className="flex flex-col gap-2">
                  <legend className="text-sm font-medium text-gray-200">
                    Q{p.id}: {p.title}
                  </legend>
                  <div className="flex flex-col gap-1.5 pl-1">
                    {p.options.map((o, optPos) => {
                      const optIndex = o.index ?? optPos;
                      return (
                      <label
                        key={optIndex}
                        className="flex items-center gap-2 text-sm text-gray-300 cursor-pointer"
                      >
                        <input
                          type="radio"
                          name={`proposal-${p.id}`}
                          checked={choices[String(p.id)] === optIndex}
                          onChange={() =>
                            setChoices((c) => ({
                              ...c,
                              [String(p.id)]: optIndex,
                            }))
                          }
                        />
                        {o.label}
                      </label>
                      );
                    })}
                  </div>
                </fieldset>
              ))}
              <Button
                type="button"
                disabled={!!busy || !active.proposals?.length}
                title={
                  lastDelegationTx
                    ? undefined
                    : "Complete step 5 (Submit delegation) before casting"
                }
                onClick={() =>
                  void run("cast", async () => {
                    const res = await walletApi.voteCast({
                      env,
                      choices,
                      delegation_tx: lastDelegationTx ?? undefined,
                      single_share: true,
                    });
                    toast.success(`Cast ${res.data.proposal_count} choice(s)`);
                  })
                }
              >
                {busy === "cast" ? "Casting…" : "6. Cast ballot"}
              </Button>
              {!lastDelegationTx && (
                <p className="text-xs text-amber-200/90">
                  Cast needs a confirmed delegation. Finish step 5 first (even if you already
                  cast before — restart clears the in-memory tx until step 5 runs again).
                </p>
              )}
            </div>
          )}
        </div>

        <label className="flex flex-col gap-1 text-sm text-gray-400 max-w-sm">
          Wallet password (if prompted)
          <input
            type="password"
            autoComplete="current-password"
            className="rounded-xl border border-gray-600 bg-gray-800 px-3 py-2 text-gray-100"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
          />
        </label>

        {status?.forum_url && (
          <a
            className="text-sm text-primary underline"
            href={status.forum_url}
            target="_blank"
            rel="noreferrer"
          >
            Forum thread
          </a>
        )}
      </div>
    </div>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-xl border border-gray-700/50 bg-gray-800/40 px-4 py-3">
      <p className="text-xs uppercase tracking-wide text-gray-500">{label}</p>
      <p className="mt-1 font-semibold text-gray-100">{value}</p>
    </div>
  );
}
