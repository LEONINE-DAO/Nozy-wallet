import { useCallback, useEffect, useState } from "react";
import { extensionApi, getCompanionPrefs } from "../lib/extensionApi";
import {
  Button,
  Card,
  Eyebrow,
  Hint,
  LogBlock,
  PageHeader,
  Pill,
  Screen,
  SectionTitle,
  Select,
  StatRow
} from "./ui";

type VoteStatus = Awaited<ReturnType<typeof extensionApi.companionVoteStatus>>;
type Busy = null | "refresh" | "export" | "prepare" | "delegate" | "sign" | "finish";

/**
 * NU7 coinholder vote using this extension wallet (same split as Desktop):
 * export/sign in WASM; prepare/PIR/cast via local nozywallet-api + nozy-vote.
 */
export function VoteView() {
  const [env, setEnv] = useState<"prod" | "stage">("prod");
  const [status, setStatus] = useState<VoteStatus | null>(null);
  const [log, setLog] = useState("");
  const [busy, setBusy] = useState<Busy>(null);
  const [delegationTx, setDelegationTx] = useState<string | null>(null);

  const load = useCallback(async () => {
    const prefs = await getCompanionPrefs();
    try {
      const s = await extensionApi.companionVoteStatus({
        baseUrl: prefs.baseUrl,
        env
      });
      setStatus(s);
    } catch (e) {
      setStatus(null);
      setLog(e instanceof Error ? e.message : String(e));
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
      setLog(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(null);
    }
  };

  return (
    <Screen>
      <PageHeader
        title="NU7 vote"
        description="Shielded Vote · Ironwood weight at snapshot. Seed stays in this extension."
      />

      <div className="flex items-end gap-2">
        <div className="flex-1">
          <Select
            label="Environment"
            value={env}
            disabled={!!busy}
            onChange={(e) => setEnv(e.target.value as "prod" | "stage")}
          >
            <option value="prod">prod</option>
            <option value="stage">stage</option>
          </Select>
        </div>
        <Button
          size="sm"
          icon="refresh"
          disabled={busy === "refresh"}
          onClick={() =>
            void run("refresh", async () => {
              /* load() runs in finally via run() */
            })
          }
        >
          {busy === "refresh" ? "Refreshing…" : "Refresh"}
        </Button>
      </div>

      {status && (
        <Card className="space-y-1.5">
          <div className="flex items-center justify-between gap-2">
            <Eyebrow>Round status</Eyebrow>
            <Pill tone="accent">{status.phase.replace(/_/g, " ")}</Pill>
          </div>
          <Hint>{status.phase_message}</Hint>
          <StatRow label="Snapshot" value={status.snapshot_utc} />
          <StatRow label="Vote window" value={`${status.vote_start_utc} → ${status.vote_end_utc}`} />
          <StatRow
            label="Notes exported"
            value={status.notes_exported ? String(status.notes_count ?? "yes") : "no"}
          />
          <StatRow label="Hotkey" value={status.hotkey_ready ? "ready" : "—"} />
          <StatRow label="Signature" value={status.sig_present ? "present" : "—"} />
        </Card>
      )}

      <Card className="space-y-2">
        <SectionTitle>Delegation wizard</SectionTitle>
        <Hint>Uses Ironwood notes from this extension wallet. Local API finishes PIR/prove.</Hint>

        <Button
          fullWidth
          disabled={!!busy}
          onClick={() =>
            void run("export", async () => {
              const prefs = await getCompanionPrefs();
              const exported = await extensionApi.walletVoteExportNotes();
              const imported = await extensionApi.companionVoteImportNotes({
                baseUrl: prefs.baseUrl,
                notes_json: exported.notes_json
              });
              setLog(imported.message);
            })
          }
        >
          {busy === "export" ? "Exporting…" : "1. Export Ironwood notes"}
        </Button>

        <Button
          fullWidth
          disabled={!!busy || !status?.notes_exported}
          onClick={() =>
            void run("prepare", async () => {
              const prefs = await getCompanionPrefs();
              const res = await extensionApi.companionVotePrepare({
                baseUrl: prefs.baseUrl,
                env
              });
              setLog(res.message);
            })
          }
        >
          {busy === "prepare" ? "Preparing…" : "2. Prepare hotkey & round"}
        </Button>

        <Button
          fullWidth
          disabled={!!busy || !status?.notes_exported}
          onClick={() =>
            void run("delegate", async () => {
              const prefs = await getCompanionPrefs();
              const res = await extensionApi.companionVoteDelegate({
                baseUrl: prefs.baseUrl,
                env
              });
              setLog(res.message);
            })
          }
        >
          {busy === "delegate" ? "Building…" : "3. Build delegation"}
        </Button>

        <Button
          fullWidth
          disabled={!!busy || !status?.signing_request_present}
          onClick={() =>
            void run("sign", async () => {
              const prefs = await getCompanionPrefs();
              const request = await extensionApi.companionVoteSigningRequest({
                baseUrl: prefs.baseUrl
              });
              const signed = await extensionApi.walletVoteSignDelegation(JSON.stringify(request));
              const res = await extensionApi.companionVoteSubmitDelegationSig({
                baseUrl: prefs.baseUrl,
                sig_json: signed.sig_json,
                env
              });
              setLog(res.message);
            })
          }
        >
          {busy === "sign" ? "Signing…" : "4. Sign with this wallet"}
        </Button>

        <Button
          fullWidth
          disabled={!!busy || !status?.sig_present}
          onClick={() =>
            void run("finish", async () => {
              const prefs = await getCompanionPrefs();
              setLog("PIR + prove started (may take minutes)…");
              const res = await extensionApi.companionVoteDelegateFinish({
                baseUrl: prefs.baseUrl,
                env,
                wait: true
              });
              setDelegationTx(res.tx_hash || null);
              setLog(res.confirmed ? `Confirmed ${res.tx_hash}` : `Submitted ${res.tx_hash}`);
            })
          }
        >
          {busy === "finish" ? "Submitting…" : "5. Submit delegation"}
        </Button>
      </Card>

      <Card className="space-y-2">
        <SectionTitle>After delegation</SectionTitle>
        <Hint>Cast the ballot from Valar or Desktop once delegation confirms.</Hint>
        {delegationTx ? (
          <StatRow label="Delegation tx" value={delegationTx.slice(0, 20) + "…"} />
        ) : null}
        {status?.forum_url ? (
          <a
            className="text-xs font-medium"
            style={{ color: "var(--nw-platinum)" }}
            href={status.forum_url}
            target="_blank"
            rel="noreferrer"
          >
            Forum thread →
          </a>
        ) : null}
      </Card>

      {log && <LogBlock>{log}</LogBlock>}
    </Screen>
  );
}
