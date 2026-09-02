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

const SYNC_PHASES = [
  "Connecting to Zebra…",
  "Scanning shielded notes…",
  "First scan can take several minutes…",
];

function formatElapsed(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return m > 0 ? `${m}m ${s}s` : `${s}s`;
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
  const autoSyncRan = useRef(false);
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

  const loadDashboard = useCallback(async () => {
    setError("");
    try {
      if (onDevice) {
        await loadLegacyStatus();
        return;
      }
      const balanceRes = await api.getBalance();
      setBalance(balanceRes.balance_zec);
      await Promise.all([loadWalletStatus(), loadLegacyStatus()]);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load wallet");
    } finally {
      setLoading(false);
    }
  }, [loadWalletStatus, loadLegacyStatus, onDevice]);

  const runSync = useCallback(async () => {
    if (syncing) return;
    setSyncing(true);
    setSyncPhase(0);
    setSyncElapsed(0);
    setError("");
    try {
      const result = await api.syncWallet(password || undefined);
      setBalance(result.balance_zec);
      await loadWalletStatus();
      await loadLegacyStatus();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Sync failed");
    } finally {
      setSyncing(false);
    }
  }, [password, loadWalletStatus, loadLegacyStatus, syncing]);

  useFocusEffect(
    useCallback(() => {
      setLoading(true);
      void loadDashboard();
      if (autoSync && !onDevice && !autoSyncRan.current) {
        autoSyncRan.current = true;
        void runSync();
      }
      return () => {
        autoSyncRan.current = false;
      };
    }, [autoSync, loadDashboard, runSync, onDevice]),
  );

  useEffect(() => {
    if (!syncing) return;
    const started = Date.now();
    const tick = setInterval(() => {
      setSyncElapsed(Math.floor((Date.now() - started) / 1000));
      setSyncPhase((i) => (i + 1) % SYNC_PHASES.length);
    }, 4000);
    return () => clearInterval(tick);
  }, [syncing]);

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
    ? SYNC_PHASES[syncPhase]
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
          <Text style={styles.eyebrow}>Shielded balance</Text>
          <SyncPill label={pillLabel} tone={pillTone} onPress={() => void runSync()} />
        </View>

        <Text style={styles.balance}>
          {loading && !syncing ? "—" : balance.toFixed(8)}
          <Text style={styles.zec}> ZEC</Text>
        </Text>

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
