import { useEffect, useMemo, useState } from "react";
import { extensionApi, type WalletScanProgressResult } from "../lib/extensionApi";
import { isPublicLwdUrl } from "../lib/nodeConnect";
import {
  isScanInProgress,
  scanPercentDisplay,
  scanPercentLabel,
  scanRateLabel
} from "../lib/scanFormat";
import { isFullPage, openWalletPage } from "../lib/walletPage";
import { Icon, SyncGlitchText } from "./ui";

type ChipTone = "neutral" | "accent" | "success" | "danger";

function syncChip(scan: WalletScanProgressResult | null): {
  label: string;
  tone: ChipTone;
  title: string;
} | null {
  if (isScanInProgress(scan) && scan) {
    const done = (scan.scannedBlocks ?? 0).toLocaleString();
    const total = (scan.totalBlocks ?? 0).toLocaleString();
    const rate = scanRateLabel(scan);
    return {
      label: `${scanPercentLabel(scan)}%`,
      tone: "accent",
      title: `Syncing ${done} / ${total} blocks · ${scan.discoveredNotes ?? 0} notes${
        rate ? ` · ${rate}` : ""
      }\nRuns in the background — you can close this popup.`
    };
  }
  if (scan?.status === "scanning" && scan.sessionWaitingSince) {
    return {
      label: "Unlock to resume",
      tone: "neutral",
      title: "Sync is waiting — unlock the wallet and it will continue automatically."
    };
  }
  if (scan?.status === "stopped") {
    return {
      label: `Paused ${scanPercentLabel(scan)}%`,
      tone: "neutral",
      title: "Sync stopped before reaching the chain tip."
    };
  }
  if (scan?.status === "failed") {
    return { label: "Sync failed", tone: "danger", title: scan.scanError || "Sync failed." };
  }
  return null;
}

/** Same words as the dashboard Connect pill: Local node or NozyWallet. */
export function NodeSourceChip({ onOpen }: { onOpen?: () => void }) {
  const [label, setLabel] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    const load = async () => {
      try {
        const status = await extensionApi.rpcGetStatus();
        if (cancelled) return;
        const next = !status.connected
          ? "Not connected"
          : status.mode === "public_lwd" || isPublicLwdUrl(status.endpoint || "")
            ? "NozyWallet"
            : "Local node";
        setLabel(next);
      } catch {
        if (!cancelled) setLabel(null);
      }
    };
    void load();
    const id = setInterval(() => void load(), 8000);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, []);

  if (!label) return null;
  return (
    <button
      type="button"
      className={`nw-syncchip nw-syncchip--${label === "Not connected" ? "neutral" : "success"}`}
      title="Sync source — same as the dashboard"
      onClick={onOpen}
    >
      {label}
    </button>
  );
}

/**
 * Persistent brand bar. Sync state is reduced to a tappable chip plus a hairline progress
 * line so a multi-day scan never dominates the top of the popup.
 */
export function AppHeader({
  scan,
  unlocked,
  onOpenSync,
  onLock,
  showBrand = true
}: {
  scan: WalletScanProgressResult | null;
  unlocked: boolean;
  onOpenSync: () => void;
  onLock: () => void;
  showBrand?: boolean;
}) {
  const chip = useMemo(() => syncChip(scan), [scan]);
  const scanning = isScanInProgress(scan);
  const percent = scanPercentDisplay(scan);

  return (
    <header className="nw-appbar">
      {showBrand ? (
        <div className="nw-brand">
          <img className="nw-brand__mark" src="./logo.jpg" alt="Nozy Wallet" />
        </div>
      ) : (
        <div />
      )}

      {unlocked && (
        <div className="flex items-center gap-1.5">
          <NodeSourceChip onOpen={onOpenSync} />
          {chip && (
            <button
              type="button"
              className={`nw-syncchip nw-syncchip--${scanning ? "cyber" : chip.tone}`}
              title={chip.title}
              onClick={onOpenSync}
            >
              <span className={`nw-syncdot${scanning ? " nw-syncdot--live" : ""}`} />
              {scanning ? (
                <SyncGlitchText>{chip.label}</SyncGlitchText>
              ) : (
                chip.label
              )}
            </button>
          )}
          {!isFullPage() && (
            <button
              type="button"
              className="nw-iconbtn"
              title="Open full wallet in a tab"
              aria-label="Open full wallet in a tab"
              onClick={() => void openWalletPage({ view: "dashboard" })}
            >
              <Icon name="expand" size={14} />
            </button>
          )}
          <button
            type="button"
            className="nw-iconbtn"
            title="Lock wallet"
            aria-label="Lock wallet"
            onClick={onLock}
          >
            <Icon name="lock" size={14} />
          </button>
        </div>
      )}

      {scanning && (
        <div className="nw-appbar__progress">
          <div
            className="nw-appbar__progress-fill"
            style={{ width: `${Math.max(0.8, percent)}%` }}
          />
        </div>
      )}
    </header>
  );
}
