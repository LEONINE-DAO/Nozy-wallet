import { useCallback, useEffect, useRef } from "react";

import { walletApi } from "../lib/api";
import {
  isWalletCaughtUp,
  isWalletSyncToTipInFlight,
  needsWalletSync,
  refreshBalanceSnapshot,
  clearStopSync,
  syncWalletToTip,
} from "../lib/syncHelpers";
import { finishSyncUi, notifyStoppedSync } from "../lib/walletSyncUi";
import { useWalletStore } from "../store/walletStore";

const STATUS_POLL_MS = 15_000;
const MIN_SYNC_INTERVAL_MS = 20_000;

type UseWalletAutoSyncOptions = {
  onCaughtUp?: () => void;
  onSyncComplete?: () => void;
};

/**
 * Keeps the wallet near chain tip while unlocked: sync on open, then retry when scan
 * or witness lag is detected. Stop still cancels the current scan; auto-sync continues
 * on the next interval so catch-up does not stay paused.
 */
export function useWalletAutoSync(options: UseWalletAutoSyncOptions = {}) {
  const { onCaughtUp, onSyncComplete } = options;
  const { isSyncing, setIsSyncing, setBalanceFromAvailable, setSyncProgress } =
    useWalletStore();
  const inFlightRef = useRef(false);
  const lastSyncAttemptRef = useRef(0);
  const isSyncingRef = useRef(isSyncing);
  isSyncingRef.current = isSyncing;

  const runCatchUp = useCallback(
    async (force = false) => {
      const loopInFlight = isWalletSyncToTipInFlight();
      if (inFlightRef.current || loopInFlight) {
        return;
      }
      // HMR/remount can leave zustand isSyncing true with no JS loop. A hung
      // backend still holds the Tauri mutex; a new loop may wait, but must not skip forever.
      if (isSyncingRef.current && !useWalletStore.getState().isStoppingSync) {
        setIsSyncing(false);
      }

      const now = Date.now();
      if (!force && now - lastSyncAttemptRef.current < MIN_SYNC_INTERVAL_MS) {
        return;
      }

      try {
        const statusRes = await walletApi.getSyncStatus();
        const s = statusRes.data;
        const needs = needsWalletSync(s);
        if (!needs) {
          if (isWalletCaughtUp(s)) {
            onCaughtUp?.();
          }
          return;
        }
      } catch {
        return;
      }

      inFlightRef.current = true;
      clearStopSync();
      setIsSyncing(true);
      useWalletStore.getState().setIsStoppingSync(false);
      lastSyncAttemptRef.current = now;

      try {
        const outcome = await syncWalletToTip(async (update) => {
          setSyncProgress(update.percent, update.message);
        });
        const snapshot = await refreshBalanceSnapshot();
        if (snapshot) {
          setBalanceFromAvailable(snapshot.available);
        }
        if (outcome.kind === "stopped") {
          notifyStoppedSync(outcome);
        } else {
          onSyncComplete?.();
          if (outcome.status && isWalletCaughtUp(outcome.status)) {
            onCaughtUp?.();
          }
        }
      } catch (error) {
        if (import.meta.env.DEV) {
          console.error("[useWalletAutoSync] background sync failed", error);
        }
      } finally {
        inFlightRef.current = false;
        setIsSyncing(false);
        finishSyncUi();
      }
    },
    [
      onCaughtUp,
      onSyncComplete,
      setBalanceFromAvailable,
      setIsSyncing,
      setSyncProgress,
    ],
  );

  useEffect(() => {
    let cancelled = false;

    const tick = async (force: boolean) => {
      if (cancelled) return;
      const statusRes = await walletApi.getWalletStatus().catch(() => null);
      if (!statusRes?.data?.unlocked) {
        return;
      }
      await runCatchUp(force);
    };

    void tick(true);
    const interval = setInterval(() => {
      void tick(false);
    }, STATUS_POLL_MS);

    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [runCatchUp]);
}
