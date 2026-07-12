import toast from "react-hot-toast";
import { formatErrorForDisplay } from "../utils/errors";
import { useWalletStore } from "../store/walletStore";
import {
  describeSyncStatus,
  refreshBalanceSnapshot,
  syncWalletToTip,
  type SyncOutcome,
  type SyncProgressUpdate,
} from "./syncHelpers";

export function showSyncOutcomeToast(outcome: SyncOutcome, toastId: string) {
  if (outcome.kind === "success") {
    toast.success(outcome.message, { id: toastId });
    return;
  }
  if (outcome.kind === "info") {
    toast(outcome.message, { id: toastId, duration: 6000 });
    return;
  }
  toast.error(outcome.message, { id: toastId });
}

function applyProgressToStore(update: SyncProgressUpdate) {
  useWalletStore.getState().setSyncProgress(update.percent, update.message);
}

export async function runWalletSyncWithFeedback(options: {
  setIsSyncing: (syncing: boolean) => void;
  onBalance?: (available: number) => void;
  onComplete?: (outcome: SyncOutcome) => void;
  loadingMessage?: string;
}): Promise<SyncOutcome | null> {
  const { setIsSyncing, onBalance, onComplete, loadingMessage = "Syncing wallet…" } = options;
  const toastId = toast.loading(loadingMessage);
  setIsSyncing(true);

  try {
    const outcome = await syncWalletToTip((update) => {
      applyProgressToStore(update);
      toast.loading(update.message, { id: toastId });
    });

    const snapshot = await refreshBalanceSnapshot();
    if (snapshot && onBalance) {
      onBalance(snapshot.available);
    }

    showSyncOutcomeToast(outcome, toastId);
    onComplete?.(outcome);
    return outcome;
  } catch (error) {
    toast.error(formatErrorForDisplay(error, "Sync failed. Please try again."), { id: toastId });
    return null;
  } finally {
    setIsSyncing(false);
    useWalletStore.getState().clearSyncProgress();
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
