import { useCallback, useEffect, useState } from "react";
import {
  extensionApi,
  getCompanionPrefs,
  type NymVpnAppStatus
} from "../lib/extensionApi";
import { isNymVpnOnrampUrl, NYM_VPN_APP_URL, ZCASH_NYM_FREE_URL } from "../lib/nymVpn";
import { isFullPage, openWalletPage } from "../lib/walletPage";
import { Button, Callout, Card, Hint, Input, Pill, Screen, SectionTitle } from "./ui";

const SUGGESTED: Array<{ label: string; url: string; onramp: boolean }> = [
  { label: "Get NymVPN (free month)", url: ZCASH_NYM_FREE_URL, onramp: true },
  { label: "NymVPN app", url: NYM_VPN_APP_URL, onramp: true },
  { label: "Zcash", url: "https://z.cash", onramp: false },
  { label: "Forum", url: "https://forum.zcashcommunity.com", onramp: false }
];

function normalizeBrowseUrl(raw: string): string | null {
  const t = raw.trim();
  if (!t) return null;
  const lower = t.toLowerCase();
  if (
    lower.startsWith("javascript:") ||
    lower.startsWith("data:") ||
    lower.startsWith("file:") ||
    lower.startsWith("blob:")
  ) {
    return null;
  }
  if (/^https?:\/\//i.test(t)) return t;
  return `https://${t}`;
}

function openExternal(url: string): void {
  if (typeof chrome !== "undefined" && chrome.tabs?.create) {
    void chrome.tabs.create({ url });
    return;
  }
  window.open(url, "_blank", "noopener,noreferrer");
}

function statusPill(status: NymVpnAppStatus | null, companionError: string | null) {
  if (companionError && !status) {
    return <Pill tone="danger">Companion offline</Pill>;
  }
  if (!status) {
    return <Pill tone="neutral">Checking NymVPN…</Pill>;
  }
  if (status.browse_allowed) {
    const mode =
      status.mode === "fast" ? "Fast" : status.mode === "mixnet" ? "Mixnet" : "Connected";
    return <Pill tone="success">NymVPN {mode}</Pill>;
  }
  if (status.daemon_present) {
    return <Pill tone="warn">NymVPN disconnected</Pill>;
  }
  return <Pill tone="danger">NymVPN not detected</Pill>;
}

export function NymVpnPromoCard({
  onOpenBrowser
}: {
  onOpenBrowser?: () => void;
}) {
  return (
    <Card className="space-y-2">
      <SectionTitle>Browse with NymVPN</SectionTitle>
      <Hint>
        Sites you open from this wallet use your computer’s internet. Connect the{" "}
        <strong>NymVPN</strong> app in <strong>Fast mode</strong> first. Nozy does not start the VPN
        — the companion checks that it is actually up, then unlocks the viewer.
      </Hint>
      <div className="flex flex-wrap gap-2">
        <Button size="sm" variant="primary" onClick={() => openExternal(ZCASH_NYM_FREE_URL)}>
          Get a free NymVPN month
        </Button>
        {onOpenBrowser ? (
          <Button size="sm" variant="secondary" onClick={onOpenBrowser}>
            Open browser
          </Button>
        ) : null}
      </div>
      <Hint>
        Claim at{" "}
        <a href={ZCASH_NYM_FREE_URL} target="_blank" rel="noopener noreferrer">
          zcash.nym.com
        </a>
        . Fast mode for browsing and sync; Mixnet mode + a new exit only when you send.
      </Hint>
    </Card>
  );
}

export function BrowserView() {
  const fullPage = isFullPage();
  const [urlInput, setUrlInput] = useState("https://zcash.nym.com");
  const [frameUrl, setFrameUrl] = useState(ZCASH_NYM_FREE_URL);
  const [status, setStatus] = useState<NymVpnAppStatus | null>(null);
  const [companionError, setCompanionError] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refreshStatus = useCallback(async () => {
    try {
      const prefs = await getCompanionPrefs();
      const snap = await extensionApi.companionNymVpnApp(prefs.baseUrl);
      setStatus(snap);
      setCompanionError(null);
    } catch (e) {
      setStatus(null);
      setCompanionError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  useEffect(() => {
    void refreshStatus();
    const id = window.setInterval(() => {
      void refreshStatus();
    }, 5000);
    return () => window.clearInterval(id);
  }, [refreshStatus]);

  const browseAllowed = Boolean(status?.browse_allowed);
  const canShowFrame = isNymVpnOnrampUrl(frameUrl) || (browseAllowed && frameUrl !== "about:blank");

  useEffect(() => {
    if (!browseAllowed && frameUrl !== "about:blank" && !isNymVpnOnrampUrl(frameUrl)) {
      setFrameUrl("about:blank");
    }
  }, [browseAllowed, frameUrl]);

  const go = (raw: string, mode: "frame" | "tab") => {
    const url = normalizeBrowseUrl(raw);
    if (!url) {
      setError("Enter an https URL (javascript/data URLs are blocked).");
      return;
    }
    const onramp = isNymVpnOnrampUrl(url);
    if (!onramp && !browseAllowed) {
      setError(
        companionError
          ? "Start nozywallet-api (companion). Nozy uses it to see whether NymVPN is connected."
          : "Connect NymVPN Fast mode in the NymVPN app, then Refresh. Sites stay locked until the tunnel is up."
      );
      return;
    }
    setError(null);
    if (mode === "tab" || !fullPage) {
      openExternal(url);
      return;
    }
    setFrameUrl(url);
  };

  if (!fullPage) {
    return (
      <Screen>
        <NymVpnPromoCard
          onOpenBrowser={() => {
            void openWalletPage({ view: "browser" });
          }}
        />
        <Hint>The site viewer lives in Full wallet so it has room, like Keplr’s dashboard.</Hint>
      </Screen>
    );
  }

  return (
    <div className="nw-browser">
      <div className="nw-browser__promo">
        <div className="flex flex-wrap items-center gap-2">
          <p className="text-[13px] font-semibold">NymVPN required for browsing</p>
          {statusPill(status, companionError)}
        </div>
        <p className="nw-hint mt-1">
          Chrome cannot send this viewer through Nozy’s mixnet helper, and it cannot start NymVPN.
          Connect <strong>Fast mode</strong> in the NymVPN app. Claim/install links stay open;
          other sites stay locked until the companion sees the tunnel.
        </p>
        {status?.detail ? <p className="nw-hint mt-1">{status.detail}</p> : null}
        {companionError && !status ? (
          <Callout tone="warn">
            Companion offline — start nozywallet-api so Nozy can check NymVPN. You can still open
            zcash.nym.com to install it.
          </Callout>
        ) : null}
        <div className="mt-2 flex flex-wrap gap-2">
          <Button size="sm" variant="primary" onClick={() => openExternal(ZCASH_NYM_FREE_URL)}>
            Get NymVPN — zcash.nym.com
          </Button>
          <Button size="sm" variant="secondary" onClick={() => openExternal(NYM_VPN_APP_URL)}>
            NymVPN app
          </Button>
          <Button size="sm" variant="secondary" onClick={() => void refreshStatus()}>
            Refresh
          </Button>
        </div>
      </div>

      <form
        className="nw-browser__bar"
        onSubmit={(e) => {
          e.preventDefault();
          go(urlInput, "frame");
        }}
      >
        <Input
          value={urlInput}
          onChange={(e) => setUrlInput(e.target.value)}
          placeholder="https://"
          aria-label="Site URL"
        />
        <Button
          type="submit"
          size="sm"
          disabled={!browseAllowed && !isNymVpnOnrampUrl(normalizeBrowseUrl(urlInput) ?? "")}
        >
          View
        </Button>
        <Button
          type="button"
          size="sm"
          variant="secondary"
          disabled={!browseAllowed && !isNymVpnOnrampUrl(normalizeBrowseUrl(urlInput) ?? "")}
          onClick={() => go(urlInput, "tab")}
        >
          Open tab
        </Button>
      </form>

      {error ? <Callout tone="warn">{error}</Callout> : null}

      <div className="nw-browser__chips">
        {SUGGESTED.map((s) => {
          const locked = !s.onramp && !browseAllowed;
          return (
            <button
              key={s.url}
              type="button"
              className={`nw-browser__chip${locked ? " nw-browser__chip--locked" : ""}`}
              disabled={locked}
              onClick={() => {
                setUrlInput(s.url);
                go(s.url, "frame");
              }}
            >
              {s.label}
            </button>
          );
        })}
      </div>

      <div className="nw-browser__frame">
        <iframe title="Site viewer" src={canShowFrame ? frameUrl : "about:blank"} />
        {!canShowFrame ? (
          <div className="nw-browser__blocked">
            Connect NymVPN Fast mode, keep nozywallet-api running, then View. zcash.nym.com stays
            available so you can claim and install. Many sites block in-wallet frames — use Open tab
            after the tunnel is up.
          </div>
        ) : null}
      </div>
    </div>
  );
}
