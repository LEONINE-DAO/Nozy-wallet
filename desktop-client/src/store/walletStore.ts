import { create } from "zustand";

interface WalletState {
  balance: number;
  address: string | null;
  hasWallet: boolean;
  isLoading: boolean;
  setBalance: (balance: number) => void;
  setAddress: (address: string) => void;
  setHasWallet: (hasWallet: boolean) => void;
  setIsLoading: (isLoading: boolean) => void;
}

export const useWalletStore = create<WalletState>((set) => ({
  balance: 0,
  address: null,
  hasWallet: false,
  isLoading: false,
  setBalance: (balance) => set({ balance }),
  setAddress: (address) => set({ address }),
  setHasWallet: (hasWallet) => set({ hasWallet }),
  setIsLoading: (isLoading) => set({ isLoading }),
}));
