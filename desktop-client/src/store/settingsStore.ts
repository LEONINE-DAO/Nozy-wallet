import { create } from "zustand";
import { persist } from "zustand/middleware";

export type FiatCurrency = "USD" | "EUR";

interface SettingsState {
  showNavigationLabels: boolean;
  hideBalance: boolean;
  darkMode: boolean;

  // Fiat display ( in History)
  showFiatEquivalent: boolean;
  fiatCurrency: FiatCurrency;
  useLiveFiatPrice: boolean;
  customFiatPerZec: number | null;

  // Notification Settings so we know when people want to be nozy
  transactionNotifs: boolean;
  soundEnabled: boolean;
  dndEnabled: boolean;
  dndFrom: string;
  dndTo: string;

  // Security Settings
  autoLockEnabled: boolean;
  autoLockMinutes: string;
  biometricsEnabled: boolean;
  screenshotProtection: boolean;

  onboardingFirstSyncDismissed: boolean;
  setOnboardingFirstSyncDismissed: (dismissed: boolean) => void;

  // Multi-account (labels and active; account list is backend-driven later)
  accountLabels: Record<string, string>;
  activeAccountId: string;
  setAccountLabel: (accountId: string, label: string) => void;
  setActiveAccountId: (accountId: string) => void;

  // Setters
  setShowNavigationLabels: (show: boolean) => void;
  setHideBalance: (hide: boolean) => void;
  setDarkMode: (enabled: boolean) => void;
  setShowFiatEquivalent: (show: boolean) => void;
  setFiatCurrency: (currency: FiatCurrency) => void;
  setUseLiveFiatPrice: (use: boolean) => void;
  setCustomFiatPerZec: (rate: number | null) => void;
  setTransactionNotifs: (show: boolean) => void;
  setSoundEnabled: (enabled: boolean) => void;
  setDndEnabled: (enabled: boolean) => void;
  setDndFrom: (time: string) => void;
  setDndTo: (time: string) => void;
  setAutoLockEnabled: (enabled: boolean) => void;
  setAutoLockMinutes: (minutes: string) => void;
  setBiometricsEnabled: (enabled: boolean) => void;
  setScreenshotProtection: (enabled: boolean) => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      // Default values
      showNavigationLabels: false,
      hideBalance: false,
      darkMode: false,
      showFiatEquivalent: false,
      fiatCurrency: "USD",
      useLiveFiatPrice: true,
      customFiatPerZec: null,
      transactionNotifs: true,
      soundEnabled: true,
      dndEnabled: false,
      dndFrom: "22:00",
      dndTo: "08:00",
      autoLockEnabled: true,
      autoLockMinutes: "15",
      biometricsEnabled: false,
      screenshotProtection: true,
      onboardingFirstSyncDismissed: false,
      setOnboardingFirstSyncDismissed: (dismissed) =>
        set({ onboardingFirstSyncDismissed: dismissed }),

      accountLabels: { "0": "Default" },
      activeAccountId: "0",
      setAccountLabel: (accountId, label) =>
        set((s) => ({
          accountLabels: { ...s.accountLabels, [accountId]: label.trim() || s.accountLabels[accountId] || accountId },
        })),
      setActiveAccountId: (accountId) => set({ activeAccountId: accountId }),

      // Setters
      setShowNavigationLabels: (show) => set({ showNavigationLabels: show }),
      setHideBalance: (hide) => set({ hideBalance: hide }),
      setDarkMode: (enabled) => set({ darkMode: enabled }),
      setShowFiatEquivalent: (show) => set({ showFiatEquivalent: show }),
      setFiatCurrency: (currency) => set({ fiatCurrency: currency }),
      setUseLiveFiatPrice: (use) => set({ useLiveFiatPrice: use }),
      setCustomFiatPerZec: (rate) => set({ customFiatPerZec: rate }),
      setTransactionNotifs: (show) => set({ transactionNotifs: show }),
      setSoundEnabled: (enabled) => set({ soundEnabled: enabled }),
      setDndEnabled: (enabled) => set({ dndEnabled: enabled }),
      setDndFrom: (time) => set({ dndFrom: time }),
      setDndTo: (time) => set({ dndTo: time }),
      setAutoLockEnabled: (enabled) => set({ autoLockEnabled: enabled }),
      setAutoLockMinutes: (minutes) => set({ autoLockMinutes: minutes }),
      setBiometricsEnabled: (enabled) => set({ biometricsEnabled: enabled }),
      setScreenshotProtection: (enabled) =>
        set({ screenshotProtection: enabled }),
    }),
    {
      name: "nozy-settings-storage",
    }
  )
);
