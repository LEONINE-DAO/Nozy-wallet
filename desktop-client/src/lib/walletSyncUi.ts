import toast from "react-hot-toast";
import { formatErrorForDisplay } from "../utils/errors";
import { walletApi } from "./api";
import { useWalletStore } from "../store/walletStore";
import {
  describeSyncStatus,
  refreshBalanceSnapshot,
  clearStopSync,
  requestStopSync,
  syncWalletToTip,
  type SyncOutcome,
  type SyncProgressUpdate,
} from "./syncHelpers";

const STOP_TOAST_ID = "wallet-sync-stop";

export function showSyncOutcomeToast(outcome: SyncOutcome, toastId: string) {
  if (outcome.kind === "success") {
    toast.success(outcome.message, { id: toastId });
    return;
  }
  if (outcome.kind === "info" || outcome.kind === "stopped") {
    toast(outcome.message, { id: toastId, duration: 6000 });
    return;
  }
  toast.error(outcome.message, { id: toastId });
}

function applyProgressToStore(update: SyncProgressUpdate) {
  useWalletStore.getState().setSyncProgress(update.percent, update.message);
}

export function finishSyncUi() {
  const store = useWalletStore.getState();
  store.setIsStoppingSync(false);
  store.clearSyncProgress();
  store.bumpSyncStatusEpoch();
}

export function requestUserStopSync() {
  const store = useWalletStore.getState();
  if (!store.isSyncing) return;
  store.setSyncPausedByUser(true);
  store.setIsStoppingSync(true);
  requestStopSync();
  toast.loading("Stopping…", { id: STOP_TOAST_ID });
}

export function notifyStoppedSync(outcome: SyncOutcome) {
  if (outcome.kind !== "stopped") return;
  toast(outcome.message, { id: STOP_TOAST_ID, duration: 6000 });
}

export async function runWalletSyncWithFeedback(options: {
  setIsSyncing: (syncing: boolean) => void;
  onBalance?: (available: number) => void;
  onComplete?: (outcome: SyncOutcome) => void;
  loadingMessage?: string;
}): Promise<SyncOutcome | null> {
  const { setIsSyncing, onBalance, onComplete, loadingMessage = "Syncing wallet…" } = options;
  const toastId = toast.loading(loadingMessage);
  clearStopSync();
  setIsSyncing(true);
  useWalletStore.getState().setIsStoppingSync(false);
  useWalletStore.getState().setSyncPausedByUser(false);

  try {
    const outcome = await syncWalletToTip(async (update) => {
      applyProgressToStore(update);
      toast.loading(update.message, { id: toastId });
    });

    const snapshot = await refreshBalanceSnapshot();
    if (snapshot && onBalance) {
      onBalance(snapshot.available);
    }

    if (outcome.kind === "stopped") {
      toast.dismiss(STOP_TOAST_ID);
    }
    showSyncOutcomeToast(outcome, toastId);
    onComplete?.(outcome);
    return outcome;
  } catch (error) {
    toast.error(formatErrorForDisplay(error, "Sync failed. Please try again."), { id: toastId });
    return null;
  } finally {
    setIsSyncing(false);
    finishSyncUi();
  }
}

/** Short label for banners while a multi-round sync runs. */
export function syncProgressLabel(outcome: SyncOutcome | null, isSyncing: boolean): string | null {
  if (!isSyncing) return null;
  if (outcome?.status) {
    return describeSyncStatus(outcome.status).message;
  }
  return "Syncing wallet with the network…";
}
