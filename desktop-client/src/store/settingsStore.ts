import { create } from "zustand";
import { persist } from "zustand/middleware";

interface SettingsState {
  // UI Settings
  showNavigationLabels: boolean;
  hideBalance: boolean;

  // Notification Settings
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

  // Setters
  setShowNavigationLabels: (show: boolean) => void;
  setHideBalance: (hide: boolean) => void;
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
      transactionNotifs: true,
      soundEnabled: true,
      dndEnabled: false,
      dndFrom: "22:00",
      dndTo: "08:00",
      autoLockEnabled: true,
      autoLockMinutes: "15",
      biometricsEnabled: false,
      screenshotProtection: true,

      // Setters
      setShowNavigationLabels: (show) => set({ showNavigationLabels: show }),
      setHideBalance: (hide) => set({ hideBalance: hide }),
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
