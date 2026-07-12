import { create } from "zustand";

interface WalletState {
  balance: number;
  address: string | null;
  hasWallet: boolean;
  isLoading: boolean;
  isSyncing: boolean;
  /** Live scan percent 0–100 while syncing; null when unknown / idle. */
  syncProgressPercent: number | null;
  /** Short live sync label, e.g. "87% synced · scanned …". */
  syncProgressLabel: string | null;
  setBalance: (balance: number) => void;
  setBalanceFromAvailable: (available: number) => void;
  setAddress: (address: string) => void;
  setHasWallet: (hasWallet: boolean) => void;
  setIsLoading: (isLoading: boolean) => void;
  setIsSyncing: (isSyncing: boolean) => void;
  setSyncProgress: (percent: number | null, label?: string | null) => void;
  clearSyncProgress: () => void;
}

export const useWalletStore = create<WalletState>((set) => ({
  balance: 0,
  address: null,
  hasWallet: false,
  isLoading: false,
  isSyncing: false,
  syncProgressPercent: null,
  syncProgressLabel: null,
  setBalance: (balance) => set({ balance }),
  setBalanceFromAvailable: (available) => set({ balance: available }),
  setAddress: (address) => set({ address }),
  setHasWallet: (hasWallet) => set({ hasWallet }),
  setIsLoading: (isLoading) => set({ isLoading }),
  setIsSyncing: (isSyncing) => set({ isSyncing }),
  setSyncProgress: (percent, label = null) =>
    set({ syncProgressPercent: percent, syncProgressLabel: label }),
  clearSyncProgress: () =>
    set({ syncProgressPercent: null, syncProgressLabel: null }),
}));
