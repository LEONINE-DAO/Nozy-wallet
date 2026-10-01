import { useCallback, useEffect, useState } from "react";
import { extensionApi } from "../lib/extensionApi";
import {
  DEFAULT_RPC,
  NODE_SETUP_MODES,
  PUBLIC_LWD_URL,
  connectFailureHint,
  setupHelp,
  type NodeSetupMode
} from "../lib/nodeConnect";
import { Button, Callout, Card, CopyButton, Hint, Pill, SectionTitle } from "./ui";

export type NodeConnectState = {
  connected: boolean;
  endpoint: string;
  blockCount: number | null;
  checking: boolean;
  message: string | null;
  source: string | null;
  mode?: "zebrad" | "public_lwd" | "none";
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
  const [mode, setMode] = useState<NodeSetupMode>("local");
  const [customUrl, setCustomUrl] = useState(initialEndpoint);
  const [connected, setConnected] = useState(false);
  const [endpoint, setEndpoint] = useState("");
  const [blockCount, setBlockCount] = useState<number | null>(null);
  const [checking, setChecking] = useState(true);
  const [message, setMessage] = useState<string | null>(null);
  const [source, setSource] = useState<string | null>(null);
  const [connectMode, setConnectMode] = useState<"zebrad" | "public_lwd" | "none">("none");
  const [showHelp, setShowHelp] = useState(variant === "welcome");
  const [showPublicOffer, setShowPublicOffer] = useState(false);

  const publish = useCallback(
    (partial: Partial<NodeConnectState>) => {
      onStateChange?.({
        connected,
        endpoint,
        blockCount,
        checking,
        message,
        source,
        mode: connectMode,
        ...partial
      });
    },
    [onStateChange, connected, endpoint, blockCount, checking, message, source, connectMode]
  );

  const applySuccess = useCallback(
    (
      rpcEndpoint: string,
      blocks: number | null,
      msg: string,
      src: string,
      nextMode: "zebrad" | "public_lwd" = src === "public_lwd" ? "public_lwd" : "zebrad"
    ) => {
      setConnected(true);
      setEndpoint(rpcEndpoint);
      setBlockCount(blocks);
      setMessage(msg);
      setSource(src);
      setConnectMode(nextMode);
      setCustomUrl(rpcEndpoint);
      setShowPublicOffer(false);
      onConnected?.(rpcEndpoint, blocks);
      publish({
        connected: true,
        endpoint: rpcEndpoint,
        blockCount: blocks,
        message: msg,
        source: src,
        mode: nextMode,
        checking: false
      });
    },
    [onConnected, publish]
  );

  const applyFailure = useCallback(
    (msg: string, offerPublic = true) => {
      setConnected(false);
      setMessage(msg);
      setSource(null);
      setConnectMode("none");
      if (offerPublic) setShowPublicOffer(true);
      publish({
        connected: false,
        message: msg,
        checking: false,
        source: null,
        mode: "none"
      });
    },
    [publish]
  );

  const refreshStatus = useCallback(async () => {
    setChecking(true);
    publish({ checking: true });
    try {
      const status = await extensionApi.rpcGetStatus();
      if (status.connected) {
        const isPublic = status.mode === "public_lwd";
        applySuccess(
          status.endpoint,
          status.blockCount ?? null,
          status.blockCount != null
            ? `Connected — ${status.blockCount.toLocaleString()} blocks`
            : `Connected to ${status.endpoint}`,
          isPublic ? "public_lwd" : "saved",
          isPublic ? "public_lwd" : "zebrad"
        );
        return;
      }
      applyFailure(
        "No node connected yet. Start a local node, or choose NozyWallet."
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
      const withTimeout = <T,>(p: Promise<T>, ms: number, label: string) => {
        // The raced call can reject later. Mark that rejection handled so Chrome
        // does not surface it after the timeout already won.
        p.catch(() => {});
        return Promise.race([
          p,
          new Promise<T>((_, reject) =>
            setTimeout(() => reject(new Error(`${label} timed out after ${ms}ms`)), ms)
          )
        ]);
      };
      try {
        const status = await withTimeout(extensionApi.rpcGetStatus(), 8000, "rpcGetStatus");
        if (cancelled) return;
        if (status.connected) {
          const isPublic = status.mode === "public_lwd";
          applySuccess(
            status.endpoint,
            status.blockCount ?? null,
            status.blockCount != null
              ? `Connected — ${status.blockCount.toLocaleString()} blocks`
              : `Connected to ${status.endpoint}`,
            isPublic ? "public_lwd" : "saved",
            isPublic ? "public_lwd" : "zebrad"
          );
          return;
        }
        let found = false;
        for (const url of [DEFAULT_RPC]) {
          try {
            const res = await withTimeout(
              extensionApi.rpcConnect({ url, tryCompanion: false }),
              6000,
              "rpcConnect"
            );
            if (cancelled) return;
            applySuccess(
              res.rpcEndpoint,
              res.blockCount,
              formatSuccessMessage(res.rpcEndpoint, res.blockCount, res.source),
              res.source,
              "zebrad"
            );
            found = true;
            break;
          } catch {
            /* Zebrad and Zakura both use these local JSON-RPC ports. */
          }
        }
        if (!cancelled && !found) {
          applyFailure(
            "No Zebrad or Zakura node was found on this computer."
          );
        }
      } catch (e) {
        if (!cancelled) {
          applyFailure(
            (e as Error).message ||
              "No Zebrad or Zakura node was found on this computer."
          );
        }
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

  const connectPublic = async () => {
    setChecking(true);
    setMessage(null);
    publish({ checking: true, message: null });
    try {
      const res = await extensionApi.connectPublicLwd();
      applySuccess(
        res.endpoint,
        res.blockCount,
        res.message,
        "public_lwd",
        "public_lwd"
      );
    } catch (e) {
      applyFailure((e as Error).message, true);
    } finally {
      setChecking(false);
    }
  };

  const connect = async () => {
    setChecking(true);
    setMessage(null);
    publish({ checking: true, message: null });
    try {
      if (mode === "public") {
        await connectPublic();
        return;
      }
      // Local node: discover Zebrad / Zakura / Crosslink on this machine.
      const res = await extensionApi.rpcConnect({ tryCompanion: true });
      applySuccess(
        res.rpcEndpoint,
        res.blockCount,
        formatSuccessMessage(res.rpcEndpoint, res.blockCount, res.source),
        res.source,
        "zebrad"
      );
    } catch (e) {
      applyFailure((e as Error).message);
    } finally {
      setChecking(false);
    }
  };

  const primaryLabel =
    mode === "public"
      ? checking
        ? "Connecting…"
        : "Connect NozyWallet"
      : checking
        ? "Finding node…"
        : "Find node";

  return (
    <Card className="space-y-3">
      <div className="flex items-center justify-between gap-2">
        <SectionTitle>Connect</SectionTitle>
        <Pill tone={checking ? "neutral" : connected ? "success" : "danger"}>
          {checking
            ? "Checking…"
            : connected
              ? connectMode === "public_lwd"
                ? "NozyWallet"
                : "Local node"
              : "Not connected"}
        </Pill>
      </div>

      <Hint>
        Use a <strong>local node</strong> on this computer, or <strong>NozyWallet</strong> sync
        (keys stay in this extension).
      </Hint>

      <div className="flex flex-wrap gap-1.5">
        {NODE_SETUP_MODES.map((m) => (
          <button
            key={m.id}
            type="button"
            disabled={disabled}
            onClick={() => {
              setMode(m.id);
              if (m.id === "local") setCustomUrl(DEFAULT_RPC);
              if (m.id === "public") setCustomUrl(PUBLIC_LWD_URL);
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

      {connected && endpoint && (
        <div
          className="flex items-center gap-2 rounded-lg px-2 py-1.5 text-xs"
          style={{ background: "var(--nw-surface-alt)" }}
        >
          <span className="nw-mono flex-1 truncate">{endpoint}</span>
          <CopyButton value={endpoint} label="Copy URL" />
        </div>
      )}

      {message && <Callout tone={connected ? "success" : "warn"}>{message}</Callout>}

      {!connected && showPublicOffer && !checking && (
        <Callout tone="info">
          <p className="text-xs leading-relaxed">
            No local node was found. Connect with <strong>NozyWallet</strong> sync instead? Your
            keys stay in this extension.
          </p>
          <div className="mt-2 flex flex-wrap gap-2">
            <Button
              variant="primary"
              size="sm"
              disabled={disabled || checking}
              onClick={() => {
                setMode("public");
                void connectPublic();
              }}
            >
              Yes — NozyWallet
            </Button>
            <Button
              size="sm"
              disabled={disabled || checking}
              onClick={() => setShowPublicOffer(false)}
            >
              No — I’ll use a local node
            </Button>
          </div>
        </Callout>
      )}

      {!connected && message && !checking && !showPublicOffer && (
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
        <Button size="sm" disabled={disabled || checking} onClick={() => void refreshStatus()}>
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
        : source === "public_lwd"
          ? " (public LWD)"
          : "";
  return `${blocks}Connected at ${endpoint}${via}`;
}
