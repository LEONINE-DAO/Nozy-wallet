import { create } from "zustand";

interface WalletState {
  balance: number;
  address: string | null;
  hasWallet: boolean;
  isLoading: boolean;
  isSyncing: boolean;
  setBalance: (balance: number) => void;
  setAddress: (address: string) => void;
  setHasWallet: (hasWallet: boolean) => void;
  setIsLoading: (isLoading: boolean) => void;
  setIsSyncing: (isSyncing: boolean) => void;
}

export const useWalletStore = create<WalletState>((set) => ({
  balance: 0,
  address: null,
  hasWallet: false,
  isLoading: false,
  isSyncing: false,
  setBalance: (balance) => set({ balance }),
  setAddress: (address) => set({ address }),
  setHasWallet: (hasWallet) => set({ hasWallet }),
  setIsLoading: (isLoading) => set({ isLoading }),
  setIsSyncing: (isSyncing) => set({ isSyncing }),
}));
