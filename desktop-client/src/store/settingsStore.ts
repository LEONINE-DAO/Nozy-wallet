import { create } from "zustand";
import { persist } from "zustand/middleware";
import {
  DEFAULT_FIAT_CURRENCY,
  type FiatCurrency,
  isFiatCurrency,
} from "../lib/fiatCurrencies";

export type { FiatCurrency } from "../lib/fiatCurrencies";

interface SettingsState {
  showNavigationLabels: boolean;
  hideBalance: boolean;

  // Fiat display (balance, history)
  fiatCurrency: FiatCurrency;
  useLiveFiatPrice: boolean;
  customFiatPerZec: number | null;

  // Security Settings
  autoLockEnabled: boolean;
  autoLockMinutes: string;
  biometricsEnabled: boolean;
  screenshotProtection: boolean;

  /** Advanced: attest NymVPN/Tor when using remote Zebrad for safer migration Priority 1. Default off — prefer local node. */
  attestPrivateNetworkForMigration: boolean;

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
  setFiatCurrency: (currency: FiatCurrency) => void;
  setUseLiveFiatPrice: (use: boolean) => void;
  setCustomFiatPerZec: (rate: number | null) => void;
  setAutoLockEnabled: (enabled: boolean) => void;
  setAutoLockMinutes: (minutes: string) => void;
  setBiometricsEnabled: (enabled: boolean) => void;
  setScreenshotProtection: (enabled: boolean) => void;
  setAttestPrivateNetworkForMigration: (enabled: boolean) => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      // Default values
      showNavigationLabels: true,
      hideBalance: false,
      fiatCurrency: "USD",
      useLiveFiatPrice: true,
      customFiatPerZec: null,
      autoLockEnabled: true,
      autoLockMinutes: "15",
      biometricsEnabled: false,
      screenshotProtection: true,
      attestPrivateNetworkForMigration: false,
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
      setFiatCurrency: (currency) => set({ fiatCurrency: currency }),
      setUseLiveFiatPrice: (use) => set({ useLiveFiatPrice: use }),
      setCustomFiatPerZec: (rate) => set({ customFiatPerZec: rate }),
      setAutoLockEnabled: (enabled) => set({ autoLockEnabled: enabled }),
      setAutoLockMinutes: (minutes) => set({ autoLockMinutes: minutes }),
      setBiometricsEnabled: (enabled) => set({ biometricsEnabled: enabled }),
      setScreenshotProtection: (enabled) =>
        set({ screenshotProtection: enabled }),
      setAttestPrivateNetworkForMigration: (enabled) =>
        set({ attestPrivateNetworkForMigration: enabled }),
    }),
    {
      name: "nozy-settings-storage",
      migrate: (persisted) => {
        const state = { ...(persisted as Record<string, unknown> | undefined) };
        // Drop legacy dark-mode preference — desktop is light-only.
        delete state.darkMode;
        delete state.setDarkMode;
        if (!isFiatCurrency(state.fiatCurrency as FiatCurrency | undefined)) {
          state.fiatCurrency = DEFAULT_FIAT_CURRENCY;
        }
        return state;
      },
    }
  )
);
