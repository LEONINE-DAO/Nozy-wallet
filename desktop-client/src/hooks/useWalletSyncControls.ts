import { useCallback } from "react";

import { useWalletStore } from "../store/walletStore";
import { requestUserStopSync, runWalletSyncWithFeedback } from "../lib/walletSyncUi";

export function useWalletSyncControls() {
  const isSyncing = useWalletStore((s) => s.isSyncing);
  const isStoppingSync = useWalletStore((s) => s.isStoppingSync);
  const syncPausedByUser = useWalletStore((s) => s.syncPausedByUser);
  const setIsSyncing = useWalletStore((s) => s.setIsSyncing);
  const setBalanceFromAvailable = useWalletStore((s) => s.setBalanceFromAvailable);

  const startSync = useCallback(async () => {
    if (useWalletStore.getState().isSyncing) return;
    await runWalletSyncWithFeedback({
      setIsSyncing,
      onBalance: setBalanceFromAvailable,
    });
  }, [setIsSyncing, setBalanceFromAvailable]);

  const stopSync = useCallback(() => {
    requestUserStopSync();
  }, []);

  return { startSync, stopSync, isSyncing, isStoppingSync, syncPausedByUser };
}
