import { create } from "zustand";

export type PopupView =
  | "welcome"
  | "unlock"
  | "dashboard"
  | "send"
  | "receive"
  | "scan"
  | "companion"
  | "vote"
  | "crosslink"
  | "settings"
  | "browser";

type UiState = {
  view: PopupView;
  setView: (view: PopupView) => void;
  statusMessage: string | null;
  setStatusMessage: (message: string | null) => void;
  pendingFinalizer: string | null;
  setPendingFinalizer: (hex: string | null) => void;
};

export const useUiStore = create<UiState>((set) => ({
  view: "welcome",
  setView: (view) => set({ view }),
  statusMessage: null,
  setStatusMessage: (message) => set({ statusMessage: message }),
  pendingFinalizer: null,
  setPendingFinalizer: (hex) => set({ pendingFinalizer: hex })
}));

