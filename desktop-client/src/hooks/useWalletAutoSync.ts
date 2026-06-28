import { useCallback, useEffect, useRef } from "react";

import { walletApi } from "../lib/api";
import {
  isWalletCaughtUp,
  needsWalletSync,
  refreshBalanceSnapshot,
  syncWalletAndRefresh,
} from "../lib/syncHelpers";
import { useWalletStore } from "../store/walletStore";

const STATUS_POLL_MS = 30_000;
const MIN_SYNC_INTERVAL_MS = 45_000;

type UseWalletAutoSyncOptions = {
  onCaughtUp?: () => void;
  onSyncComplete?: () => void;
};

/**
 * Keeps the wallet near chain tip while unlocked: sync on open, then retry when scan
 * or witness lag is detected. Manual sync in the header still works and shares `isSyncing`.
 */
export function useWalletAutoSync(options: UseWalletAutoSyncOptions = {}) {
  const { onCaughtUp, onSyncComplete } = options;
  const { isSyncing, setIsSyncing, setBalanceFromAvailable } = useWalletStore();
  const inFlightRef = useRef(false);
  const lastSyncAttemptRef = useRef(0);
  const isSyncingRef = useRef(isSyncing);
  isSyncingRef.current = isSyncing;

  const runCatchUp = useCallback(
    async (force = false) => {
      if (inFlightRef.current || isSyncingRef.current) {
        return;
      }

      const now = Date.now();
      if (!force && now - lastSyncAttemptRef.current < MIN_SYNC_INTERVAL_MS) {
        return;
      }

      try {
        const statusRes = await walletApi.getSyncStatus();
        if (!needsWalletSync(statusRes.data)) {
          if (isWalletCaughtUp(statusRes.data)) {
            onCaughtUp?.();
          }
          return;
        }
      } catch {
        return;
      }

      inFlightRef.current = true;
      setIsSyncing(true);
      lastSyncAttemptRef.current = now;

      try {
        const status = await syncWalletAndRefresh();
        const snapshot = await refreshBalanceSnapshot();
        if (snapshot) {
          setBalanceFromAvailable(snapshot.available);
        }
        onSyncComplete?.();
        if (status && isWalletCaughtUp(status)) {
          onCaughtUp?.();
        }
      } catch (error) {
        if (import.meta.env.DEV) {
          console.error("[useWalletAutoSync] background sync failed", error);
        }
      } finally {
        inFlightRef.current = false;
        setIsSyncing(false);
      }
    },
    [onCaughtUp, onSyncComplete, setBalanceFromAvailable, setIsSyncing],
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
