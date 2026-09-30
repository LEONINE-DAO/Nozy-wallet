import { useCallback, useEffect, useState } from "react";
import {
  extensionApi,
  getCompanionPrefs,
  setCompanionPrefs,
  type NymDvpnSyncStatus,
  type NymMixnetReadiness,
  type PrivacyNetworkSnapshot,
  type SendEgressKind,
  type SendEgressSnapshot
} from "../lib/extensionApi";
import {
  DEFAULT_LOCAL_LWD_URL,
  isMixnetLwdUrl,
  MIXNET_LWD_GRPC_URL,
  MIXNET_LWD_METRICS_URL
} from "../lib/lwdMixnet";
import { Button, Callout, Card, Hint, Pill, SectionTitle } from "./ui";

function badgeTone(kind: SendEgressKind): "success" | "accent" | "warn" | "danger" | "neutral" {
  switch (kind) {
    case "local":
    case "trusted":
      return "success";
    case "mixnet":
    case "tor":
    case "i2p":
      return "accent";
    case "direct_remote":
      return "warn";
    case "blocked":
      return "danger";
    default:
      return "neutral";
  }
}

export function SendEgressBadge({ compact = false }: { compact?: boolean }) {
  const [egress, setEgress] = useState<SendEgressSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const prefs = await getCompanionPrefs();
        const snap = await extensionApi.companionSendEgress(prefs.baseUrl);
        if (!cancelled) {
          setEgress(snap);
          setError(null);
        }
      } catch (e) {
        if (!cancelled) {
          setEgress(null);
          setError(e instanceof Error ? e.message : String(e));
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  if (error && !egress) {
    return (
      <Hint>
        Next send: companion offline — start nozywallet-api to use the same Nym path as desktop.
      </Hint>
    );
  }
  if (!egress) return null;

  return (
    <div className="space-y-1">
      <div className="flex items-center gap-2">
        <span className="text-[12px] font-semibold">Next send</span>
        <Pill tone={badgeTone(egress.kind)}>{egress.label}</Pill>
      </div>
      <Hint>{egress.summary}</Hint>
      {!compact && egress.detail ? <Hint>{egress.detail}</Hint> : null}
      {egress.show_stopgap ? (
        <Hint>
          Stopgap:{" "}
          <a href={egress.stopgap_url} target="_blank" rel="noopener noreferrer">
            {egress.stopgap_url}
          </a>
        </Hint>
      ) : null}
    </div>
  );
}

export function NetworkPrivacyPanel() {
  const [privacy, setPrivacy] = useState<PrivacyNetworkSnapshot | null>(null);
  const [dvpn, setDvpn] = useState<NymDvpnSyncStatus | null>(null);
  const [mixnet, setMixnet] = useState<NymMixnetReadiness | null>(null);
  const [lwdUrl, setLwdUrl] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [okMsg, setOkMsg] = useState<string | null>(null);
  const [probeLog, setProbeLog] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    const prefs = await getCompanionPrefs();
    setLwdUrl(prefs.lightwalletdUrl);
    const [p, d, m] = await Promise.all([
      extensionApi.companionPrivacyNetwork(prefs.baseUrl),
      extensionApi.companionNymDvpn(prefs.baseUrl, prefs.lightwalletdUrl),
      extensionApi.companionNymMixnet(prefs.baseUrl)
    ]);
    setPrivacy(p);
    setDvpn(d);
    setMixnet(m);
  }, []);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        await refresh();
        if (!cancelled) setError(null);
      } catch (e) {
        if (!cancelled) setError(e instanceof Error ? e.message : String(e));
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [refresh]);

  const patch = async (next: Partial<PrivacyNetworkSnapshot>) => {
    setBusy(true);
    setError(null);
    setOkMsg(null);
    try {
      const prefs = await getCompanionPrefs();
      const p = await extensionApi.companionSetPrivacyNetwork({
        baseUrl: prefs.baseUrl,
        patch: next
      });
      setPrivacy(p);
      setOkMsg("Saved to the companion config (same file as desktop/CLI).");
      await refresh();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const onDvpn = async (enabled: boolean) => {
    setBusy(true);
    setError(null);
    setOkMsg(null);
    try {
      const prefs = await getCompanionPrefs();
      const d = await extensionApi.companionSetNymDvpn({
        baseUrl: prefs.baseUrl,
        enabled
      });
      setDvpn(d);
      setOkMsg(enabled ? "dVPN sync helper enabled." : "dVPN sync helper off.");
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const useMixnetLwd = async () => {
    setBusy(true);
    setError(null);
    setOkMsg(null);
    try {
      const prefs = await getCompanionPrefs();
      await setCompanionPrefs({
        baseUrl: prefs.baseUrl,
        lightwalletdUrl: MIXNET_LWD_GRPC_URL,
        apiKey: prefs.apiKey
      });
      setLwdUrl(MIXNET_LWD_GRPC_URL);
      setOkMsg(
        `Companion LWD override → ${MIXNET_LWD_GRPC_URL}. Start lwd-mixnet-client first; not a product default.`
      );
      await refresh();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const useDirectLwd = async () => {
    setBusy(true);
    setError(null);
    setOkMsg(null);
    try {
      const prefs = await getCompanionPrefs();
      await setCompanionPrefs({
        baseUrl: prefs.baseUrl,
        lightwalletdUrl: DEFAULT_LOCAL_LWD_URL,
        apiKey: prefs.apiKey
      });
      setLwdUrl(DEFAULT_LOCAL_LWD_URL);
      setOkMsg(`Companion LWD override → ${DEFAULT_LOCAL_LWD_URL}.`);
      await refresh();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const probeMixnetLwd = async () => {
    setBusy(true);
    setError(null);
    setOkMsg(null);
    setProbeLog(null);
    try {
      const prefs = await getCompanionPrefs();
      const snap = await extensionApi.probeLwdMixnetProxy({
        baseUrl: prefs.baseUrl,
        grpc_url: MIXNET_LWD_GRPC_URL,
        metrics_url: MIXNET_LWD_METRICS_URL
      });
      setProbeLog(JSON.stringify(snap, null, 2));
      if (snap.ok) {
        setOkMsg("Mixnet LWD proxy answering — health + GetLightdInfo OK.");
      } else {
        setError(
          snap.health_error ||
            snap.lightd_error ||
            "Mixnet LWD proxy not ready (is lwd-mixnet-client healthy on :9068/:9070?)."
        );
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const probeDvpn = async () => {
    setBusy(true);
    setError(null);
    setOkMsg(null);
    setProbeLog(null);
    try {
      const prefs = await getCompanionPrefs();
      const res = await extensionApi.companionNymDvpnProbe({
        baseUrl: prefs.baseUrl,
        lightwalletd_url: prefs.lightwalletdUrl || undefined,
        blocks: 2
      });
      setProbeLog(JSON.stringify(res, null, 2));
      setOkMsg(res.ok ? "dVPN probe OK." : "dVPN probe finished with errors — see log.");
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Card className="space-y-2">
      <SectionTitle>Network privacy (Nym)</SectionTitle>
      <Hint>
        Same hybrid as desktop/CLI: mixnet helper for remote sendraw, dVPN for remote compact sync,
        local Zebrad stays direct. Chrome does not embed the Nym SDK — the extension drives the
        local companion.
      </Hint>

      <SendEgressBadge />

      {error ? <Callout tone="danger">{error}</Callout> : null}
      {okMsg ? <Callout tone="success">{okMsg}</Callout> : null}

      {mixnet ? (
        <div className="space-y-1 rounded-lg border border-[var(--nw-border)] p-2">
          <div className="flex flex-wrap items-center gap-2">
            <span className="text-[13px] font-semibold">Mixnet send helper</span>
            <Pill tone={mixnet.helper_ok ? "success" : "warn"}>
              {mixnet.helper_ok ? "helper found" : "helper missing"}
            </Pill>
            <Pill tone={mixnet.would_use_mixnet ? "accent" : "neutral"}>
              {mixnet.would_use_mixnet ? "would use mixnet" : "would not use"}
            </Pill>
          </div>
          <Hint>
            Requested: {mixnet.requested ? "yes" : "no"}. Local Zebrad:{" "}
            {mixnet.zebra_url_local ? "yes (direct)" : "no"}.
            {mixnet.helper_error ? ` ${mixnet.helper_error}` : ""}
          </Hint>
        </div>
      ) : null}

      {privacy ? (
        <label className="flex cursor-pointer items-start gap-2 pt-1">
          <input
            type="checkbox"
            className="mt-0.5"
            checked={privacy.broadcast_via_nym_mixnet}
            disabled={busy}
            onChange={(e) => void patch({ broadcast_via_nym_mixnet: e.target.checked })}
          />
          <span>
            <span className="block text-[13px] font-semibold">Nym mixnet for remote send</span>
            <Hint>
              Writes privacy_network.broadcast_via_nym_mixnet. Local/LAN RPC stays Case A1
              (direct). Needs the smolmix helper on this PC.
            </Hint>
          </span>
        </label>
      ) : null}

      {dvpn ? (
        <label className="flex cursor-pointer items-start gap-2">
          <input
            type="checkbox"
            className="mt-0.5"
            checked={dvpn.requested}
            disabled={busy}
            onChange={(e) => void onDvpn(e.target.checked)}
          />
          <span>
            <span className="block text-[13px] font-semibold">Nym dVPN for remote compact sync</span>
            <Hint>
              Local :9067 stays direct. Helper{" "}
              {dvpn.helper_ok ? "found" : dvpn.helper_error ?? "missing"}. Would use dVPN:{" "}
              {dvpn.would_use_dvpn ? "yes" : "no"}.
            </Hint>
          </span>
        </label>
      ) : null}

      <div className="space-y-1.5 rounded-lg border border-[var(--nw-border)] p-2">
        <span className="block text-[13px] font-semibold">LWD over mixnet (operator)</span>
        <Hint>
          Optional: point companion compact-sync at the local{" "}
          <span className="nw-mono">lwd-mixnet-client</span> (:9068). Requires that Docker/client
          half and a reachable serving address. Never the store default.
        </Hint>
        <Hint>
          Current LWD override:{" "}
          <span className="nw-mono">{lwdUrl || "(unset → companion default)"}</span>
          {isMixnetLwdUrl(lwdUrl) ? " · mixnet proxy" : ""}
        </Hint>
        <div className="flex flex-wrap gap-2">
          <Button size="sm" disabled={busy} onClick={() => void useMixnetLwd()}>
            Use :9068 proxy
          </Button>
          <Button size="sm" variant="ghost" disabled={busy} onClick={() => void useDirectLwd()}>
            Use :9067 direct
          </Button>
          <Button size="sm" variant="secondary" disabled={busy} onClick={() => void probeMixnetLwd()}>
            Probe :9068
          </Button>
          <Button size="sm" variant="ghost" disabled={busy} onClick={() => void probeDvpn()}>
            Probe dVPN
          </Button>
        </div>
      </div>

      {probeLog ? (
        <pre className="max-h-40 overflow-auto rounded-lg bg-[var(--nw-surface-2)] p-2 text-[10px] leading-snug">
          {probeLog}
        </pre>
      ) : null}

      <Hint>
        Consumer stopgap:{" "}
        <a href="https://zcash.nym.com" target="_blank" rel="noopener noreferrer">
          zcash.nym.com
        </a>{" "}
        — Fast mode for sync, Mixnet + new exit for Ironwood send. Not the in-app helpers.
      </Hint>

      <Button variant="ghost" size="sm" disabled={busy} onClick={() => void refresh()}>
        Refresh Nym status
      </Button>
    </Card>
  );
}
