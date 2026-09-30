import { create } from "zustand";

interface WalletState {
  balance: number;
  address: string | null;
  hasWallet: boolean;
  isLoading: boolean;
  isSyncing: boolean;
  /** True after the user hits Stop until they hit Sync again (blocks auto-sync). */
  syncPausedByUser: boolean;
  /** True while waiting for the current scan chunk to finish after Stop. */
  isStoppingSync: boolean;
  /** Bumped when a sync round ends so status panels refetch immediately. */
  syncStatusEpoch: number;
  /** Live scan percent 0–100 while syncing; null when unknown / idle. */
  syncProgressPercent: number | null;
  /** Short live sync label, e.g. "87% of chain · scanned …". */
  syncProgressLabel: string | null;
  setBalance: (balance: number) => void;
  setBalanceFromAvailable: (available: number) => void;
  setAddress: (address: string) => void;
  setHasWallet: (hasWallet: boolean) => void;
  setIsLoading: (isLoading: boolean) => void;
  setIsSyncing: (isSyncing: boolean) => void;
  setSyncPausedByUser: (paused: boolean) => void;
  setIsStoppingSync: (stopping: boolean) => void;
  bumpSyncStatusEpoch: () => void;
  setSyncProgress: (percent: number | null, label?: string | null) => void;
  clearSyncProgress: () => void;
}

export const useWalletStore = create<WalletState>((set) => ({
  balance: 0,
  address: null,
  hasWallet: false,
  isLoading: false,
  isSyncing: false,
  syncPausedByUser: false,
  isStoppingSync: false,
  syncStatusEpoch: 0,
  syncProgressPercent: null,
  syncProgressLabel: null,
  setBalance: (balance) => set({ balance }),
  setBalanceFromAvailable: (available) => set({ balance: available }),
  setAddress: (address) => set({ address }),
  setHasWallet: (hasWallet) => set({ hasWallet }),
  setIsLoading: (isLoading) => set({ isLoading }),
  setIsSyncing: (isSyncing) =>
    set(isSyncing ? { isSyncing } : { isSyncing, isStoppingSync: false }),
  setSyncPausedByUser: (paused) => set({ syncPausedByUser: paused }),
  setIsStoppingSync: (stopping) => set({ isStoppingSync: stopping }),
  bumpSyncStatusEpoch: () =>
    set((state) => ({ syncStatusEpoch: state.syncStatusEpoch + 1 })),
  setSyncProgress: (percent, label = null) =>
    set({ syncProgressPercent: percent, syncProgressLabel: label }),
  clearSyncProgress: () =>
    set({ syncProgressPercent: null, syncProgressLabel: null }),
}));
