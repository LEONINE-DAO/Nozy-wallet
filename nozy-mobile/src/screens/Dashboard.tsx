import * as Clipboard from "expo-clipboard";
import { useFocusEffect } from "@react-navigation/native";
import { NativeStackScreenProps } from "@react-navigation/native-stack";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  ActivityIndicator,
  RefreshControl,
  ScrollView,
  StyleSheet,
  Text,
  View,
} from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "../components/Button";
import { useWalletSession } from "../context/WalletSessionContext";
import { api } from "../services/api";
import { onDeviceMoveLegacy, onDeviceSaplingStatus } from "../services/onDeviceSapling";
import { colors, fontSize, spacing } from "../theme";
import type { RootStackParamList, SaplingStatusResponse, SyncResponse, WalletStatusResponse } from "../types";

type Props = NativeStackScreenProps<RootStackParamList, "Dashboard">;

const SYNC_PHASES = [
  "Connecting to Zebra…",
  "Fetching chain tip…",
  "Scanning shielded notes…",
  "First full scan can take several minutes…",
  "Still working — keep the API window open…",
];

function formatElapsed(seconds: number): string {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return m > 0 ? `${m}m ${s}s` : `${s}s`;
}

export function DashboardScreen({ navigation }: Props) {
  const { password, clearPassword, apiUrl, autoSync, backendMode } =
    useWalletSession();
  const [balance, setBalance] = useState(0);
  const [address, setAddress] = useState("");
  const [walletStatus, setWalletStatus] = useState<WalletStatusResponse | null>(null);
  const [lastSyncResult, setLastSyncResult] = useState<SyncResponse | null>(null);
  const [syncMessage, setSyncMessage] = useState("");
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
      // Status is optional if API is mid-sync
    }
  }, [onDevice]);

  const loadDashboard = useCallback(async () => {
    setError("");
    try {
      if (onDevice) {
        await loadLegacyStatus();
        return;
      }
      const [balanceRes, addressRes] = await Promise.all([
        api.getBalance(),
        api.generateAddress(password || undefined),
        loadWalletStatus(),
        loadLegacyStatus(),
      ]);
      setBalance(balanceRes.balance_zec);
      setAddress(addressRes.address);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load wallet");
    } finally {
      setLoading(false);
    }
  }, [password, loadWalletStatus, loadLegacyStatus, onDevice]);

  const handleMoveLegacy = useCallback(async () => {
    if (legacyBusy) return;
    setLegacyBusy(true);
    setError("");
    try {
      if (onDevice) {
        const message = await onDeviceMoveLegacy();
        setSyncMessage(message);
        await loadDashboard();
        return;
      }
      await api.scanSapling({ password: password || undefined });
      const res = await api.shieldSapling({ password: password || undefined });
      setSyncMessage(res.message);
      await loadDashboard();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not move legacy funds");
    } finally {
      setLegacyBusy(false);
    }
  }, [legacyBusy, password, loadDashboard, onDevice]);
  const runSync = useCallback(async () => {
    setSyncing(true);
    setSyncMessage("");
    setSyncPhase(0);
    setSyncElapsed(0);
    setError("");
    try {
      const result = await api.syncWallet(password || undefined);
      setLastSyncResult(result);
      setBalance(result.balance_zec);
      setSyncMessage(result.message);
      await loadWalletStatus();
      await loadLegacyStatus();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Sync failed");
    } finally {
      setSyncing(false);
    }
  }, [password, loadWalletStatus, loadLegacyStatus]);

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

  async function copyAddress() {
    if (address) await Clipboard.setStringAsync(address);
  }

  async function handleLogout() {
    await clearPassword();
    navigation.replace("Welcome");
  }

  const scannedHeight =
    lastSyncResult?.last_scan_height ?? walletStatus?.last_sync_height ?? null;
  const chainTip =
    lastSyncResult?.chain_tip ?? walletStatus?.current_block_height ?? null;
  const blocksBehind = walletStatus?.blocks_behind ?? null;
  const isSynced = blocksBehind === 0;

  return (
    <SafeAreaView style={styles.safe}>
      <ScrollView
        contentContainerStyle={styles.container}
        refreshControl={
          <RefreshControl
            refreshing={loading && !syncing}
            onRefresh={() => {
              setLoading(true);
              void loadDashboard();
            }}
            tintColor={colors.primary}
          />
        }
      >
        <Text style={styles.eyebrow}>Shielded balance</Text>
        <Text style={styles.balance}>
          {loading && !syncing ? "..." : balance.toFixed(8)} ZEC
        </Text>

        <View style={styles.card}>
          <Text style={styles.cardLabel}>Chain sync status</Text>
          <View style={styles.statRow}>
            <Text style={styles.statLabel}>Scanned height</Text>
            <Text style={styles.statValue}>
              {scannedHeight ?? "—"}
            </Text>
          </View>
          <View style={styles.statRow}>
            <Text style={styles.statLabel}>Chain tip</Text>
            <Text style={styles.statValue}>{chainTip ?? "—"}</Text>
          </View>
          <View style={styles.statRow}>
            <Text style={styles.statLabel}>Blocks behind</Text>
            <Text
              style={[
                styles.statValue,
                blocksBehind === null
                  ? styles.statMuted
                  : isSynced
                    ? styles.statOk
                    : styles.statWarn,
              ]}
            >
              {blocksBehind === null
                ? "—"
                : isSynced
                  ? "Synced ✓"
                  : `${blocksBehind} behind`}
            </Text>
          </View>
          {autoSync ? (
            <Text style={styles.autoSyncHint}>Auto-sync on open is enabled</Text>
          ) : null}
        </View>

        {syncing ? (
          <View style={styles.syncProgress}>
            <ActivityIndicator color={colors.primary} size="large" />
            <Text style={styles.syncProgressTitle}>Syncing wallet…</Text>
            <Text style={styles.syncProgressPhase}>{SYNC_PHASES[syncPhase]}</Text>
            <Text style={styles.syncProgressTime}>
              Elapsed: {formatElapsed(syncElapsed)}
            </Text>
            <Text style={styles.syncProgressNote}>
              Do not close the API server window. First sync scans full history and
              can take 5–30+ minutes depending on your node.
            </Text>
          </View>
        ) : null}

        {lastSyncResult && !syncing ? (
          <View style={styles.card}>
            <Text style={styles.cardLabel}>Last sync result</Text>
            <Text style={styles.syncDetail}>
              Notes in cache: {lastSyncResult.total_notes} · New this scan:{" "}
              {lastSyncResult.new_notes_in_scan}
            </Text>
            <Text style={styles.syncDetail}>
              Height {lastSyncResult.last_scan_height} → tip{" "}
              {lastSyncResult.chain_tip}
              {lastSyncResult.already_synced ? " (already at tip)" : ""}
            </Text>
          </View>
        ) : null}

        {legacyStatus ? (
          <View style={[styles.card, styles.legacyCard]}>
            <Text style={styles.cardLabel}>Legacy funds</Text>
            <Text style={styles.syncDetail}>
              {legacyStatus.unspent_zec.toFixed(8)} ZEC available to move into your
              shielded balance (fee ~{legacyStatus.fee_zec.toFixed(8)} ZEC).
            </Text>
            <Button
              label={legacyBusy ? "Moving…" : "Move to shielded"}
              onPress={() => void handleMoveLegacy()}
              loading={legacyBusy}
              disabled={legacyBusy || syncing}
            />
          </View>
        ) : null}

        <View style={styles.card}>
          <Text style={styles.cardLabel}>Receive address</Text>
          <Text style={styles.address} selectable>
            {address || "Generate an address after unlock"}
          </Text>
          <Button
            label="Copy address"
            variant="secondary"
            onPress={() => void copyAddress()}
            disabled={!address}
          />
        </View>

        {syncMessage && !syncing ? (
          <Text style={styles.syncOk}>{syncMessage}</Text>
        ) : null}
        {error ? <Text style={styles.error}>{error}</Text> : null}

        <View style={styles.actions}>
          <Button
            label={syncing ? "Syncing…" : "Sync wallet"}
            onPress={() => void runSync()}
            loading={syncing}
            disabled={syncing}
          />
          <Button
            label="Send ZEC"
            onPress={() => navigation.navigate("Send")}
            disabled={syncing}
          />
          <Button
            label="Sell mode"
            onPress={() => navigation.navigate("Sell")}
            disabled={syncing}
          />
          <Button
            label="Transaction history"
            variant="secondary"
            onPress={() => navigation.navigate("TransactionHistory")}
          />
          <Button
            label="Address book"
            variant="secondary"
            onPress={() => navigation.navigate("AddressBook")}
          />
          <Button
            label="Keystone wallet"
            variant="secondary"
            onPress={() => navigation.navigate("Keystone")}
          />
          <Button
            label="Settings"
            variant="secondary"
            onPress={() => navigation.navigate("Settings")}
          />
          <Button
            label="Refresh balance"
            variant="secondary"
            onPress={() => {
              setLoading(true);
              void loadDashboard();
            }}
            disabled={syncing}
          />
          <Button label="Log out" variant="ghost" onPress={() => void handleLogout()} />
        </View>

        <Text style={styles.footer}>API: {apiUrl}</Text>
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  container: { padding: spacing.lg, gap: spacing.md },
  eyebrow: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
    fontWeight: "600",
    textTransform: "uppercase",
    letterSpacing: 1,
  },
  balance: { color: colors.primary, fontSize: 40, fontWeight: "800" },
  card: {
    backgroundColor: colors.surface,
    borderRadius: 16,
    padding: spacing.lg,
    gap: spacing.sm,
    borderWidth: 1,
    borderColor: colors.border,
  },
  legacyCard: {
    borderColor: colors.primary,
  },
  cardLabel: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
    fontWeight: "600",
    marginBottom: spacing.xs,
  },
  statRow: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
  },
  statLabel: { color: colors.textMuted, fontSize: fontSize.sm },
  statValue: { color: colors.text, fontSize: fontSize.sm, fontWeight: "600" },
  statOk: { color: colors.success },
  statWarn: { color: colors.primary },
  statMuted: { color: colors.textMuted },
  autoSyncHint: { color: colors.textMuted, fontSize: 12, marginTop: spacing.xs },
  syncProgress: {
    backgroundColor: colors.surfaceLight,
    borderRadius: 16,
    padding: spacing.lg,
    alignItems: "center",
    gap: spacing.sm,
    borderWidth: 1,
    borderColor: colors.border,
  },
  syncProgressTitle: {
    color: colors.text,
    fontSize: fontSize.lg,
    fontWeight: "700",
  },
  syncProgressPhase: {
    color: colors.primary,
    fontSize: fontSize.md,
    textAlign: "center",
  },
  syncProgressTime: { color: colors.textMuted, fontSize: fontSize.sm },
  syncProgressNote: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
    textAlign: "center",
    lineHeight: 20,
    marginTop: spacing.sm,
  },
  syncDetail: { color: colors.textMuted, fontSize: fontSize.sm, lineHeight: 20 },
  address: { color: colors.text, fontSize: fontSize.sm, lineHeight: 20 },
  syncOk: { color: colors.success, fontSize: fontSize.sm },
  error: { color: colors.error, fontSize: fontSize.sm },
  actions: { gap: spacing.sm, marginTop: spacing.sm },
  footer: { color: colors.textMuted, fontSize: 12, marginTop: spacing.lg },
});
