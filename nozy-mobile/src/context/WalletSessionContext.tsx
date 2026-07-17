import AsyncStorage from "@react-native-async-storage/async-storage";
import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
import {
  getApiBaseUrl,
  getApiKey,
  setApiBaseUrl,
  setApiKey as setApiKeyHeader,
} from "../services/api";
import {
  isOnDeviceBackendAvailable,
  type WalletBackendMode,
} from "../lib/walletBackend";
import { enableExperimentalFeatures, isProductionBuild } from "../lib/buildProfile";
import { defaultHostedApiUrl } from "../lib/connectionPresets";
import { lockOnDeviceWallet } from "nozy-wallet";

const PASSWORD_KEY = "nozy.session.password";
const API_URL_KEY = "nozy.api.url";
const API_KEY_KEY = "nozy.api.key";
const AUTO_SYNC_KEY = "nozy.autosync.enabled";
const BACKEND_MODE_KEY = "nozy.backend.mode";

type WalletSessionContextValue = {
  password: string;
  setPassword: (value: string) => Promise<void>;
  /** Mark session active for tip-sync (password optional — passwordless wallets included). */
  unlockSession: (password?: string) => Promise<void>;
  clearPassword: () => Promise<void>;
  apiUrl: string;
  setApiUrl: (value: string) => Promise<void>;
  apiKey: string;
  setApiKey: (value: string) => Promise<void>;
  autoSync: boolean;
  setAutoSync: (value: boolean) => Promise<void>;
  /** True after unlock/create/restore — not “password non-empty” (passwordless wallets sync too). */
  isUnlocked: boolean;
  backendMode: WalletBackendMode;
  setBackendMode: (mode: WalletBackendMode) => Promise<void>;
  isOnDeviceNativeAvailable: boolean;
};

const WalletSessionContext = createContext<WalletSessionContextValue | null>(
  null,
);

export function WalletSessionProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  const [password, setPasswordState] = useState("");
  const [unlocked, setUnlocked] = useState(false);
  const [apiUrl, setApiUrlState] = useState(getApiBaseUrl());
  const [apiKey, setApiKeyState] = useState(getApiKey() ?? "");
  const [autoSync, setAutoSyncState] = useState(true);
  const [backendMode, setBackendModeState] =
    useState<WalletBackendMode>("companion");
  const [onDeviceNative, setOnDeviceNative] = useState(false);

  useEffect(() => {
    setOnDeviceNative(isOnDeviceBackendAvailable());
  }, []);

  useEffect(() => {
    void (async () => {
      const [
        storedPassword,
        storedApiUrl,
        storedApiKey,
        storedAutoSync,
        storedBackend,
      ] = await Promise.all([
        AsyncStorage.getItem(PASSWORD_KEY),
        AsyncStorage.getItem(API_URL_KEY),
        AsyncStorage.getItem(API_KEY_KEY),
        AsyncStorage.getItem(AUTO_SYNC_KEY),
        AsyncStorage.getItem(BACKEND_MODE_KEY),
      ]);
      if (storedPassword) {
        // Keep password for unlock form only — do not mark session unlocked
        // until the user (or Welcome) successfully calls unlock on the API.
        setPasswordState(storedPassword);
      }
      if (storedApiUrl) {
        setApiBaseUrl(storedApiUrl);
        setApiUrlState(storedApiUrl);
      } else {
        // Preview/production bake hosted URL into extra.defaultApiUrl; keep
        // session state aligned when nothing was saved yet.
        const initial = getApiBaseUrl();
        if (
          isProductionBuild() ||
          initial.includes("nozywallet.leoninedao.org")
        ) {
          const hosted = defaultHostedApiUrl();
          setApiBaseUrl(hosted);
          setApiUrlState(hosted);
        }
      }
      if (storedApiKey) {
        setApiKeyHeader(storedApiKey);
        setApiKeyState(storedApiKey);
      } else if (getApiKey()) {
        setApiKeyState(getApiKey() ?? "");
      }
      if (storedAutoSync === "false") setAutoSyncState(false);
      else setAutoSyncState(true);
      // Stale on_device mode leaves Dashboard on empty FFI wallet after companion Unlock.
      if (
        storedBackend === "on_device" &&
        enableExperimentalFeatures() &&
        isProductionBuild()
      ) {
        setBackendModeState("on_device");
      } else {
        setBackendModeState("companion");
        if (storedBackend === "on_device") {
          await AsyncStorage.setItem(BACKEND_MODE_KEY, "companion");
        }
      }
    })();
  }, []);

  const setPassword = useCallback(async (value: string) => {
    setPasswordState(value);
    setUnlocked(true);
    if (value) {
      await AsyncStorage.setItem(PASSWORD_KEY, value);
    } else {
      await AsyncStorage.removeItem(PASSWORD_KEY);
    }
  }, []);

  const unlockSession = useCallback(
    async (value = "") => {
      await setPassword(value);
    },
    [setPassword],
  );

  const clearPassword = useCallback(async () => {
    setPasswordState("");
    setUnlocked(false);
    await AsyncStorage.removeItem(PASSWORD_KEY);
    if (backendMode === "on_device" && isOnDeviceBackendAvailable()) {
      try {
        lockOnDeviceWallet();
      } catch {
        // native module may be unavailable in Expo Go
      }
    }
  }, [backendMode]);

  const setApiUrl = useCallback(async (value: string) => {
    const trimmed = value.trim().replace(/\/$/, "");
    setApiBaseUrl(trimmed);
    setApiUrlState(trimmed);
    await AsyncStorage.setItem(API_URL_KEY, trimmed);
  }, []);

  const setApiKey = useCallback(async (value: string) => {
    const trimmed = value.trim();
    setApiKeyHeader(trimmed || null);
    setApiKeyState(trimmed);
    if (trimmed) {
      await AsyncStorage.setItem(API_KEY_KEY, trimmed);
    } else {
      await AsyncStorage.removeItem(API_KEY_KEY);
    }
  }, []);

  const setAutoSync = useCallback(async (value: boolean) => {
    setAutoSyncState(value);
    await AsyncStorage.setItem(AUTO_SYNC_KEY, value ? "true" : "false");
  }, []);

  const setBackendMode = useCallback(async (mode: WalletBackendMode) => {
    if (mode === "on_device") {
      if (!enableExperimentalFeatures()) {
        throw new Error("On-device wallet is not available in this release.");
      }
      if (!isOnDeviceBackendAvailable()) {
        throw new Error(
          "On-device wallet requires a dev client build with libnozy_ffi.so",
        );
      }
    }
    setBackendModeState(mode);
    await AsyncStorage.setItem(BACKEND_MODE_KEY, mode);
  }, []);

  const value = useMemo(
    () => ({
      password,
      setPassword,
      unlockSession,
      clearPassword,
      apiUrl,
      setApiUrl,
      apiKey,
      setApiKey,
      autoSync,
      setAutoSync,
      isUnlocked: unlocked,
      backendMode,
      setBackendMode,
      isOnDeviceNativeAvailable: onDeviceNative,
    }),
    [
      password,
      setPassword,
      unlockSession,
      clearPassword,
      apiUrl,
      setApiUrl,
      apiKey,
      setApiKey,
      autoSync,
      setAutoSync,
      unlocked,
      backendMode,
      setBackendMode,
      onDeviceNative,
    ],
  );

  return (
    <WalletSessionContext.Provider value={value}>
      {children}
    </WalletSessionContext.Provider>
  );
}

export function useWalletSession(): WalletSessionContextValue {
  const ctx = useContext(WalletSessionContext);
  if (!ctx) {
    throw new Error("useWalletSession must be used within WalletSessionProvider");
  }
  return ctx;
}

export type { WalletBackendMode };
