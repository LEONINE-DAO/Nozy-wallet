import { CloseCircle, Refresh } from "@solar-icons/react";

import { useWalletSyncControls } from "../hooks/useWalletSyncControls";
import { cn } from "../lib/cn";

interface SyncControlButtonProps {
  compact?: boolean;
  className?: string;
}

export function SyncControlButton({ compact = false, className }: SyncControlButtonProps) {
  const { startSync, stopSync, isSyncing, isStoppingSync } = useWalletSyncControls();

  const mode = isSyncing ? (isStoppingSync ? "stopping" : "stop") : "sync";
  const label =
    mode === "stopping" ? "Stopping" : mode === "stop" ? (compact ? "Stop" : "Stop sync") : compact ? "Sync" : "Sync now";

  return (
    <button
      type="button"
      className={cn("nw-sync-btn", `nw-sync-btn--${mode}`, compact && "nw-sync-btn--compact", className)}
      onClick={() => {
        if (mode === "stop" || mode === "stopping") {
          stopSync();
          return;
        }
        void startSync();
      }}
      aria-label={label}
    >
      <span className="nw-sync-btn-icon" aria-hidden>
        {mode === "sync" ? <Refresh size={14} weight="Bold" /> : <CloseCircle size={14} weight="Bold" />}
      </span>
      <span className="nw-sync-btn-label">{label}</span>
    </button>
  );
}
