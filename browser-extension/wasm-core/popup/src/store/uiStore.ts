import { create } from "zustand";

export type PopupView = "welcome" | "unlock" | "dashboard" | "send" | "receive" | "settings";

type UiState = {
  view: PopupView;
  setView: (view: PopupView) => void;
  statusMessage: string | null;
  setStatusMessage: (message: string | null) => void;
};

export const useUiStore = create<UiState>((set) => ({
  view: "welcome",
  setView: (view) => set({ view }),
  statusMessage: null,
  setStatusMessage: (message) => set({ statusMessage: message })
}));

