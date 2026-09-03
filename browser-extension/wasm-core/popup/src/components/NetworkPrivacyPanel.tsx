import { useCallback, useEffect, useState } from "react";
import {
  extensionApi,
  getCompanionPrefs,
  type NymDvpnSyncStatus,
  type PrivacyNetworkSnapshot,
  type SendEgressKind,
  type SendEgressSnapshot
} from "../lib/extensionApi";
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
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [okMsg, setOkMsg] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    const prefs = await getCompanionPrefs();
    const [p, d] = await Promise.all([
      extensionApi.companionPrivacyNetwork(prefs.baseUrl),
      extensionApi.companionNymDvpn(prefs.baseUrl, prefs.lightwalletdUrl)
    ]);
    setPrivacy(p);
    setDvpn(d);
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

  return (
    <Card className="space-y-2">
      <SectionTitle>Network privacy (Nym)</SectionTitle>
      <Hint>
        Same hybrid as desktop/CLI: mixnet for remote sendraw, dVPN for remote compact sync,
        local Zebrad stays direct. The extension talks to the companion — Chrome does not embed
        Nym.
      </Hint>

      <SendEgressBadge />

      {error ? <Callout tone="danger">{error}</Callout> : null}
      {okMsg ? <Callout tone="success">{okMsg}</Callout> : null}

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
