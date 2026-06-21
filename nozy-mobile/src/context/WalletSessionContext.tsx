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

const PASSWORD_KEY = "nozy.session.password";
const API_URL_KEY = "nozy.api.url";
const API_KEY_KEY = "nozy.api.key";
const AUTO_SYNC_KEY = "nozy.autosync.enabled";

type WalletSessionContextValue = {
  password: string;
  setPassword: (value: string) => Promise<void>;
  clearPassword: () => Promise<void>;
  apiUrl: string;
  setApiUrl: (value: string) => Promise<void>;
  apiKey: string;
  setApiKey: (value: string) => Promise<void>;
  autoSync: boolean;
  setAutoSync: (value: boolean) => Promise<void>;
  isUnlocked: boolean;
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
  const [apiUrl, setApiUrlState] = useState(getApiBaseUrl());
  const [apiKey, setApiKeyState] = useState(getApiKey() ?? "");
  const [autoSync, setAutoSyncState] = useState(false);

  useEffect(() => {
    void (async () => {
      const [storedPassword, storedApiUrl, storedApiKey, storedAutoSync] =
        await Promise.all([
          AsyncStorage.getItem(PASSWORD_KEY),
          AsyncStorage.getItem(API_URL_KEY),
          AsyncStorage.getItem(API_KEY_KEY),
          AsyncStorage.getItem(AUTO_SYNC_KEY),
        ]);
      if (storedPassword) setPasswordState(storedPassword);
      if (storedApiUrl) {
        setApiBaseUrl(storedApiUrl);
        setApiUrlState(storedApiUrl);
      }
      if (storedApiKey) {
        setApiKeyHeader(storedApiKey);
        setApiKeyState(storedApiKey);
      }
      if (storedAutoSync === "true") setAutoSyncState(true);
    })();
  }, []);

  const setPassword = useCallback(async (value: string) => {
    setPasswordState(value);
    await AsyncStorage.setItem(PASSWORD_KEY, value);
  }, []);

  const clearPassword = useCallback(async () => {
    setPasswordState("");
    await AsyncStorage.removeItem(PASSWORD_KEY);
  }, []);

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

  const value = useMemo(
    () => ({
      password,
      setPassword,
      clearPassword,
      apiUrl,
      setApiUrl,
      apiKey,
      setApiKey,
      autoSync,
      setAutoSync,
      isUnlocked: password.length > 0,
    }),
    [
      password,
      setPassword,
      clearPassword,
      apiUrl,
      setApiUrl,
      apiKey,
      setApiKey,
      autoSync,
      setAutoSync,
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
