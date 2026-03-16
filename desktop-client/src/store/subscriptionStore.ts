import { create } from "zustand";

interface SubscriptionState {
  hasNymSubscription: boolean;
  setHasNymSubscription: (v: boolean) => void;
}

export const useSubscriptionStore = create<SubscriptionState>((set) => ({
  hasNymSubscription: false,
  setHasNymSubscription: (v) => set({ hasNymSubscription: v }),
}));
