import { CompositeScreenProps, useFocusEffect } from "@react-navigation/native";
import type { BottomTabScreenProps } from "@react-navigation/bottom-tabs";
import type { NativeStackScreenProps } from "@react-navigation/native-stack";
import { useCallback, useEffect, useRef, useState } from "react";
import { Pressable, StyleSheet, Text, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "../components/Button";
import { AppLogo } from "../components/AppLogo";
import { CyberpunkSyncPanel } from "../components/CyberpunkSyncPanel";
import { SyncPill } from "../components/SyncPill";
import { useWalletSession } from "../context/WalletSessionContext";
import { api } from "../services/api";
import { onDeviceMoveLegacy, onDeviceSaplingStatus } from "../services/onDeviceSapling";
import { onDeviceCompactSync, onDeviceShieldedSnapshot } from "../services/onDeviceWallet";
import { loadHideBalances } from "../lib/displayPrefs";
import { formatUsd, getZecUsdPrice } from "../lib/zecPrice";
import { getActiveOnDeviceAccount } from "../services/onDeviceAccounts";
import { PoolAmountsRow } from "../components/PoolAmountsRow";
import { ZecPriceBar } from "../components/ZecPriceBar";
import { colors, fontSize, spacing } from "../theme";
import type {
  MainTabParamList,
  RootStackParamList,
  SaplingStatusResponse,
  WalletStatusResponse,
} from "../types";

type Props = CompositeScreenProps<
  BottomTabScreenProps<MainTabParamList, "Home">,
  NativeStackScreenProps<RootStackParamList>
>;

const SYNC_PHASES_LWD = [
  "Connecting to lightwalletd…",
  "Downloading compact blocks…",
  "First scan can take several minutes…",
];

const SYNC_PHASES_API = [
  "Connecting to Zebra…",
  "Scanning shielded notes…",
  "First scan can take several minutes…",
];

function formatElapsed(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return m > 0 ? `${m}m ${s}s` : `${s}s`;
}

const ZAT = 100_000_000;

function splitPools(
  available: number,
  orchard?: number,
  ironwood?: number,
): { orchard: number; ironwood: number } {
  let orchardZec = orchard ?? 0;
  let ironwoodZec = ironwood ?? 0;
  if (orchardZec + ironwoodZec === 0 && available > 0) {
    orchardZec = available;
  }
  return { orchard: orchardZec, ironwood: ironwoodZec };
}

export function DashboardScreen({ navigation }: Props) {
  const { password, autoSync, backendMode } = useWalletSession();
  const [balance, setBalance] = useState(0);
  const [walletStatus, setWalletStatus] = useState<WalletStatusResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [syncing, setSyncing] = useState(false);
  const [syncPhase, setSyncPhase] = useState(0);
  const [syncElapsed, setSyncElapsed] = useState(0);
  const [error, setError] = useState("");
  const [legacyStatus, setLegacyStatus] = useState<SaplingStatusResponse | null>(null);
  const [legacyBusy, setLegacyBusy] = useState(false);
  const [hideBalances, setHideBalances] = useState(false);
  const [usdPerZec, setUsdPerZec] = useState<number | null>(null);
  const [accountName, setAccountName] = useState("");
  const [orchardZec, setOrchardZec] = useState(0);
  const [ironwoodZec, setIronwoodZec] = useState(0);
  const autoSyncRan = useRef(false);
  const syncingRef = useRef(false);
  const onDevice = backendMode === "on_device";

  const loadLegacyStatus = useCallback(async () => {
    try {
      if (onDevice) {
        const status = await onDeviceSaplingStatus();
        setLegacyStatus(status.has_legacy_balance ? status : null);
        return;
      }
      const status = await api.getSaplingStatus();
      setLegacyStatus(status.has_legacy_balance ? status : null);
    } catch {
      setLegacyStatus(null);
    }
  }, [onDevice]);

  const loadWalletStatus = useCallback(async () => {
    if (onDevice) return;
    try {
      const status = await api.walletStatus();
      setWalletStatus(status);
      setBalance(status.balance_zec);
    } catch {
      // optional during sync
    }
  }, [onDevice]);

  const applyPools = useCallback(
    (available: number, orchard?: number, ironwood?: number) => {
      const split = splitPools(available, orchard, ironwood);
      setOrchardZec(split.orchard);
      setIronwoodZec(split.ironwood);
    },
    [],
  );

  const loadDashboard = useCallback(async () => {
    setError("");
    try {
      if (onDevice) {
        await loadLegacyStatus();
        const snap = await onDeviceShieldedSnapshot();
        setBalance(snap.unspentZec);
        applyPools(snap.unspentZec, snap.orchardZec, snap.ironwoodZec);
        return;
      }
      const balanceRes = await api.getBalance();
      const available = balanceRes.available_zec ?? balanceRes.balance_zec;
      setBalance(available);
      let orchard = balanceRes.orchard_zec ?? 0;
      let ironwood = balanceRes.ironwood_zec ?? 0;
      try {
        const iw = await api.getIronwoodStatus();
        if (orchard + ironwood === 0) {
          orchard = iw.orchard_wallet_zat / ZAT;
          ironwood = iw.ironwood_wallet_zat / ZAT;
        }
      } catch {
        // companion ironwood status is optional
      }
      applyPools(available, orchard, ironwood);
      await Promise.all([loadWalletStatus(), loadLegacyStatus()]);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load wallet");
    } finally {
      setLoading(false);
    }
  }, [loadWalletStatus, loadLegacyStatus, onDevice, applyPools]);

  const runSync = useCallback(async () => {
    if (syncingRef.current) return;
    syncingRef.current = true;
    setSyncing(true);
    setSyncPhase(0);
    setSyncElapsed(0);
    setError("");
    try {
      if (onDevice) {
        const result = await onDeviceCompactSync();
        setBalance(result.unspentZec);
        applyPools(result.unspentZec, result.orchardZec, result.ironwoodZec);
        setWalletStatus({
          balance_zec: result.unspentZec,
          pending_transactions: 0,
          total_transactions: 0,
          last_sync_height: result.chainTip,
          current_block_height: result.chainTip,
          blocks_behind: 0,
          witness_lag_blocks: 0,
          witness_fresh_for_send: false,
          max_send_witness_lag_blocks: 0,
          ready_for_send: false,
        });
        await loadLegacyStatus();
        return;
      }
      const result = await api.syncWallet(password || undefined);
      setBalance(result.balance_zec);
      await loadWalletStatus();
      await loadLegacyStatus();
      await loadDashboard();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Sync failed");
    } finally {
      syncingRef.current = false;
      setSyncing(false);
    }
  }, [password, loadWalletStatus, loadLegacyStatus, onDevice, applyPools, loadDashboard]);

  useFocusEffect(
    useCallback(() => {
      void loadHideBalances().then(setHideBalances);
      if (onDevice) {
        void getActiveOnDeviceAccount()
          .then((a) => setAccountName(a.name))
          .catch(() => setAccountName(""));
      } else {
        setAccountName("");
      }
      setLoading(true);
      void loadDashboard();
      if (autoSync && !autoSyncRan.current) {
        autoSyncRan.current = true;
        void runSync();
      }
      return () => {
        autoSyncRan.current = false;
      };
    }, [autoSync, loadDashboard, runSync, onDevice]),
  );

  useEffect(() => {
    void (async () => {
      const rate = await getZecUsdPrice();
      setUsdPerZec(rate);
    })();
  }, []);

  useEffect(() => {
    if (!syncing) return;
    const started = Date.now();
    const tick = setInterval(() => {
      setSyncElapsed(Math.floor((Date.now() - started) / 1000));
      setSyncPhase((i) => (i + 1) % (onDevice ? SYNC_PHASES_LWD : SYNC_PHASES_API).length);
    }, 4000);
    return () => clearInterval(tick);
  }, [syncing, onDevice]);

  useEffect(() => {
    if (!onDevice) return;
    applyPools(balance);
  }, [onDevice, balance, applyPools]);

  const blocksBehind = walletStatus?.blocks_behind ?? null;
  const isSynced = blocksBehind === 0;
  const scannedHeight = walletStatus?.last_sync_height ?? null;
  const chainTip = walletStatus?.current_block_height ?? null;
  const syncPercent =
    scannedHeight != null && chainTip != null && chainTip > 0
      ? Math.min(100, Math.floor((scannedHeight / chainTip) * 100))
      : null;

  const pillLabel = syncing
    ? `Syncing ${formatElapsed(syncElapsed)}`
    : blocksBehind === null
      ? "Tap to sync"
      : isSynced
        ? "Synced"
        : syncPercent != null
          ? `${syncPercent}% synced`
          : `${blocksBehind} behind`;

  const pillTone = syncing
    ? "syncing"
    : blocksBehind === null
      ? "offline"
      : isSynced
        ? "ok"
        : "warn";

  const syncHeadline = syncing
    ? `Syncing · ${formatElapsed(syncElapsed)}`
    : isSynced
      ? "Wallet synced"
      : "Tap to sync wallet";

  const syncDetail = syncing
    ? (onDevice ? SYNC_PHASES_LWD : SYNC_PHASES_API)[syncPhase]
    : scannedHeight != null && chainTip != null
      ? `Height ${scannedHeight.toLocaleString()} / ${chainTip.toLocaleString()}`
      : "Use Send and Receive tabs below";

  /** Cyberpunk panel only while syncing or catching up — hidden at 100% synced. */
  const showSyncPanel =
    syncing || (blocksBehind != null && blocksBehind > 0);

  return (
    <SafeAreaView style={styles.safe} edges={["top"]}>
      <View style={styles.body}>
        <AppLogo variant="header" />

        <View style={styles.balanceRow}>
          <Text style={styles.eyebrow}>
            {accountName ? `${accountName} · shielded` : "Shielded balance"}
          </Text>
          <SyncPill label={pillLabel} tone={pillTone} onPress={() => void runSync()} />
        </View>

        <Text style={styles.balance}>
          {loading && !syncing ? "—" : hideBalances ? "••••••••" : balance.toFixed(8)}
          <Text style={styles.zec}> ZEC</Text>
        </Text>
        {hideBalances ? null : usdPerZec != null && usdPerZec > 0 ? (
          <Text style={styles.fiat}>≈ {formatUsd(balance * usdPerZec)}</Text>
        ) : null}

        <PoolAmountsRow
          orchardZec={orchardZec}
          ironwoodZec={ironwoodZec}
          hideBalances={hideBalances}
        />

        <ZecPriceBar
          zecAmount={balance}
          usdPerZec={usdPerZec}
          hideBalances={hideBalances}
        />

        <View style={styles.actions}>
          <Button
            label="Send"
            onPress={() => navigation.navigate("Send")}
            disabled={syncing}
            style={styles.actionBtn}
          />
          <Button
            label="Receive"
            variant="secondary"
            onPress={() => navigation.navigate("Receive")}
            disabled={syncing}
            style={styles.actionBtn}
          />
        </View>

        {showSyncPanel ? (
          <Pressable onPress={() => void runSync()} disabled={syncing}>
            <CyberpunkSyncPanel
              headline={syncHeadline}
              detail={syncDetail}
              percent={syncing ? null : syncPercent}
              tone={syncing ? "syncing" : isSynced ? "ok" : "warn"}
              indeterminate={syncing}
              showSpinner={syncing}
            />
          </Pressable>
        ) : null}

        {legacyStatus ? (
          <View style={styles.legacyBanner}>
            <Text style={styles.legacyText}>
              Legacy: {legacyStatus.unspent_zec.toFixed(4)} ZEC
            </Text>
            <Button
              label={legacyBusy ? "…" : "Shield"}
              size="sm"
              variant="secondary"
              onPress={() => void (async () => {
                setLegacyBusy(true);
                try {
                  if (onDevice) await onDeviceMoveLegacy();
                  else {
                    await api.scanSapling({ password: password || undefined });
                    await api.shieldSapling({ password: password || undefined });
                  }
                  await loadDashboard();
                } finally {
                  setLegacyBusy(false);
                }
              })()}
              loading={legacyBusy}
              disabled={legacyBusy || syncing}
            />
          </View>
        ) : null}

        {error ? <Text style={styles.error}>{error}</Text> : null}
      </View>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  body: {
    flex: 1,
    paddingHorizontal: spacing.lg,
    paddingTop: spacing.md,
    paddingBottom: spacing.sm,
    gap: spacing.md,
  },
  balanceRow: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
    gap: spacing.sm,
  },
  eyebrow: {
    color: colors.primary,
    fontSize: 10,
    fontWeight: "700",
    textTransform: "uppercase",
    letterSpacing: 2,
    flex: 1,
  },
  balance: {
    color: colors.primary,
    fontSize: 34,
    fontWeight: "800",
    letterSpacing: -1,
    marginTop: -4,
  },
  zec: {
    fontSize: fontSize.sm,
    fontWeight: "600",
    color: colors.textFaint,
  },
  fiat: {
    color: "rgba(57, 255, 159, 0.7)",
    fontSize: fontSize.sm,
    fontWeight: "600",
    marginTop: -8,
  },
  actions: {
    flexDirection: "row",
    gap: spacing.sm,
  },
  actionBtn: {
    flex: 1,
  },
  legacyBanner: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
    gap: spacing.sm,
    paddingHorizontal: spacing.md,
    paddingVertical: spacing.sm,
    borderRadius: 12,
    borderWidth: 1,
    borderColor: colors.platinumLine,
    backgroundColor: colors.surface,
  },
  legacyText: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
    flex: 1,
  },
  error: { color: colors.error, fontSize: fontSize.sm },
});
