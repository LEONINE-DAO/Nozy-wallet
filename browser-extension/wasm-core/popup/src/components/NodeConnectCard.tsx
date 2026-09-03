import { useCallback, useEffect, useState } from "react";
import { extensionApi } from "../lib/extensionApi";
import {
  DEFAULT_RPC,
  DEFAULT_TESTNET_RPC,
  NODE_SETUP_MODES,
  connectFailureHint,
  setupHelp,
  type NodeSetupMode
} from "../lib/nodeConnect";
import { Button, Callout, Card, CopyButton, Hint, Input, Pill, SectionTitle } from "./ui";

export type NodeConnectState = {
  connected: boolean;
  endpoint: string;
  blockCount: number | null;
  checking: boolean;
  message: string | null;
  source: string | null;
};

type NodeConnectCardProps = {
  /** welcome = larger primary CTA; settings = compact */
  variant?: "welcome" | "settings";
  initialEndpoint?: string;
  disabled?: boolean;
  /** Called after a successful connect (endpoint persisted in extension storage). */
  onConnected?: (endpoint: string, blockCount: number | null) => void;
  onStateChange?: (state: NodeConnectState) => void;
};

export function NodeConnectCard({
  variant = "welcome",
  initialEndpoint = DEFAULT_RPC,
  disabled = false,
  onConnected,
  onStateChange
}: NodeConnectCardProps) {
  const [mode, setMode] = useState<NodeSetupMode>("auto");
  const [customUrl, setCustomUrl] = useState(initialEndpoint);
  const [connected, setConnected] = useState(false);
  const [endpoint, setEndpoint] = useState("");
  const [blockCount, setBlockCount] = useState<number | null>(null);
  const [checking, setChecking] = useState(true);
  const [message, setMessage] = useState<string | null>(null);
  const [source, setSource] = useState<string | null>(null);
  const [showHelp, setShowHelp] = useState(variant === "welcome");

  const publish = useCallback(
    (partial: Partial<NodeConnectState>) => {
      onStateChange?.({
        connected,
        endpoint,
        blockCount,
        checking,
        message,
        source,
        ...partial
      });
    },
    [onStateChange, connected, endpoint, blockCount, checking, message, source]
  );

  const applySuccess = useCallback(
    (
      rpcEndpoint: string,
      blocks: number | null,
      msg: string,
      src: string
    ) => {
      setConnected(true);
      setEndpoint(rpcEndpoint);
      setBlockCount(blocks);
      setMessage(msg);
      setSource(src);
      setCustomUrl(rpcEndpoint);
      onConnected?.(rpcEndpoint, blocks);
      publish({
        connected: true,
        endpoint: rpcEndpoint,
        blockCount: blocks,
        message: msg,
        source: src,
        checking: false
      });
    },
    [onConnected, publish]
  );

  const applyFailure = useCallback(
    (msg: string) => {
      setConnected(false);
      setMessage(msg);
      setSource(null);
      publish({ connected: false, message: msg, checking: false, source: null });
    },
    [publish]
  );

  const refreshStatus = useCallback(async () => {
    setChecking(true);
    publish({ checking: true });
    try {
      const status = await extensionApi.rpcGetStatus();
      if (status.connected) {
        applySuccess(
          status.endpoint,
          status.blockCount ?? null,
          status.blockCount != null
            ? `Connected — ${status.blockCount.toLocaleString()} blocks`
            : `Connected to ${status.endpoint}`,
          "saved"
        );
        return;
      }
      applyFailure(
        "No node connected yet. Start Zebrad, then click Find my node."
      );
    } catch (e) {
      applyFailure((e as Error).message);
    } finally {
      setChecking(false);
    }
  }, [applyFailure, applySuccess, publish]);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      setChecking(true);
      try {
        const status = await extensionApi.rpcGetStatus();
        if (cancelled) return;
        if (status.connected) {
          applySuccess(
            status.endpoint,
            status.blockCount ?? null,
            status.blockCount != null
              ? `Connected — ${status.blockCount.toLocaleString()} blocks`
              : `Connected to ${status.endpoint}`,
            "saved"
          );
          return;
        }
        const res = await extensionApi.rpcConnect({ tryCompanion: true });
        if (cancelled) return;
        applySuccess(
          res.rpcEndpoint,
          res.blockCount,
          formatSuccessMessage(res.rpcEndpoint, res.blockCount, res.source),
          res.source
        );
      } catch (e) {
        if (!cancelled) applyFailure((e as Error).message);
      } finally {
        if (!cancelled) setChecking(false);
      }
    })();
    return () => {
      cancelled = true;
    };
    // Mount-only: auto-connect once when popup opens.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (initialEndpoint && initialEndpoint !== DEFAULT_RPC) {
      setCustomUrl(initialEndpoint);
    }
  }, [initialEndpoint]);

  const connect = async () => {
    setChecking(true);
    setMessage(null);
    publish({ checking: true, message: null });
    try {
      let res;
      if (mode === "local") {
        res = await extensionApi.rpcConnect({ url: DEFAULT_RPC });
      } else if (mode === "remote") {
        const url = customUrl.trim();
        if (!url) throw new Error("Paste your node URL first.");
        res = await extensionApi.rpcConnect({ url, tryCompanion: false });
      } else {
        // auto + wsl: full discovery (WSL IP, localhost, companion config)
        res = await extensionApi.rpcConnect({ tryCompanion: true });
      }
      applySuccess(
        res.rpcEndpoint,
        res.blockCount,
        formatSuccessMessage(res.rpcEndpoint, res.blockCount, res.source),
        res.source
      );
    } catch (e) {
      applyFailure((e as Error).message);
    } finally {
      setChecking(false);
    }
  };

  const primaryLabel =
    mode === "remote"
      ? checking
        ? "Connecting…"
        : "Connect"
      : checking
        ? "Finding node…"
        : "Find my node";

  return (
    <Card className="space-y-3">
      <div className="flex items-center justify-between gap-2">
        <SectionTitle>Connect your node</SectionTitle>
        <Pill tone={checking ? "neutral" : connected ? "success" : "danger"}>
          {checking ? "Checking…" : connected ? "Connected" : "Not connected"}
        </Pill>
      </div>

      <Hint>
        Nozy needs a running <strong>Zebrad</strong> node before create wallet, scan, or send.
        One-time setup — we remember the URL.
      </Hint>

      <div className="flex flex-wrap gap-1.5">
        {NODE_SETUP_MODES.map((m) => (
          <button
            key={m.id}
            type="button"
            disabled={disabled || checking}
            onClick={() => {
              setMode(m.id);
              if (m.id === "local") setCustomUrl(DEFAULT_RPC);
              if (m.id === "remote" && !customUrl.trim()) {
                setCustomUrl("https://");
              }
            }}
            className="rounded-lg px-2.5 py-1 text-xs font-semibold transition-colors"
            style={
              mode === m.id
                ? { background: "var(--nw-platinum)", color: "#18181b" }
                : {
                    background: "var(--nw-surface-alt)",
                    color: "var(--nw-muted)",
                    border: "1px solid var(--nw-border)"
                  }
            }
          >
            {m.label}
          </button>
        ))}
      </div>

      {showHelp && (
        <Callout tone="info">
          <ol className="list-decimal space-y-1 pl-4 text-left text-xs leading-relaxed">
            {setupHelp(mode).map((line) => (
              <li key={line}>{line}</li>
            ))}
          </ol>
        </Callout>
      )}

      {(mode === "remote" || mode === "wsl") && (
        <Input
          label={mode === "remote" ? "Node URL" : "Or paste WSL / custom URL"}
          hint={
            mode === "remote"
              ? "Full JSON-RPC URL — https://your-server.com:443 or http://172.x.x.x:8232"
              : "Optional if auto-detect fails"
          }
          mono
          value={customUrl}
          onChange={(e) => setCustomUrl(e.target.value)}
          placeholder={
            mode === "remote" ? "https://your-node.example.com:443" : "http://172.20.199.206:8232"
          }
          disabled={disabled || checking}
        />
      )}

      {connected && endpoint && (
        <div className="flex items-center gap-2 rounded-lg px-2 py-1.5 text-xs" style={{ background: "var(--nw-surface-alt)" }}>
          <span className="nw-mono flex-1 truncate">{endpoint}</span>
          <CopyButton value={endpoint} label="Copy URL" />
        </div>
      )}

      {message && (
        <Callout tone={connected ? "success" : "warn"}>{message}</Callout>
      )}

      {!connected && message && !checking && (
        <Hint>{connectFailureHint(mode)}</Hint>
      )}

      <div className="flex flex-wrap gap-2">
        <Button
          variant="primary"
          size={variant === "welcome" ? "md" : "sm"}
          fullWidth={variant === "welcome"}
          disabled={disabled || checking}
          onClick={() => void connect()}
        >
          {primaryLabel}
        </Button>
        {mode === "local" && (
          <Button
            size="sm"
            disabled={disabled || checking}
            onClick={() =>
              void extensionApi
                .rpcConnect({ url: DEFAULT_TESTNET_RPC })
                .then((res) =>
                  applySuccess(
                    res.rpcEndpoint,
                    res.blockCount,
                    `Testnet connected — ${res.blockCount.toLocaleString()} blocks`,
                    res.source
                  )
                )
                .catch((e) => applyFailure((e as Error).message))
            }
          >
            Try testnet
          </Button>
        )}
        <Button
          size="sm"
          disabled={disabled || checking}
          onClick={() => void refreshStatus()}
        >
          Recheck
        </Button>
        <Button
          size="sm"
          variant="ghost"
          disabled={disabled}
          onClick={() => setShowHelp((v) => !v)}
        >
          {showHelp ? "Hide steps" : "Show steps"}
        </Button>
      </div>
    </Card>
  );
}

function formatSuccessMessage(
  endpoint: string,
  blockCount: number,
  source: string
): string {
  const blocks =
    Number.isFinite(blockCount) && blockCount >= 0
      ? `${blockCount.toLocaleString()} blocks — `
      : "";
  const via =
    source === "companion"
      ? " (from Nozy Desktop config)"
      : source === "autodetect"
        ? " (auto-detected)"
        : "";
  return `${blocks}Connected at ${endpoint}${via}`;
}
