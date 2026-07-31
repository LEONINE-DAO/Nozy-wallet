import AsyncStorage from "@react-native-async-storage/async-storage";
import { useEffect, useState } from "react";
import { Alert, ScrollView, StyleSheet, Text, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "../Button";
import { Input } from "../Input";
import { SettingsBackButton } from "./SettingsBackButton";
import { useWalletSession } from "../../context/WalletSessionContext";
import { isOnDeviceBackendAvailable } from "../../lib/walletBackend";
import { colors, fontSize, spacing } from "../../theme";

export const ONDEVICE_MNEMONIC_KEY = "nozy.ondevice.mnemonic";
export const ONDEVICE_DATA_DIR_KEY = "nozy.ondevice.dataDir";
export const ONDEVICE_COMPACT_DB_KEY = "nozy.ondevice.compactDb";
export const ONDEVICE_ZEBRA_URL_KEY = "nozy.ondevice.zebraUrl";
export const ONDEVICE_LWD_URL_KEY = "nozy.ondevice.lwdUrl";

type Props = { onBack: () => void };

export function OnDeviceWalletSettings({ onBack }: Props) {
  const { backendMode, setBackendMode, isOnDeviceNativeAvailable } =
    useWalletSession();
  const [mnemonic, setMnemonic] = useState("");
  const [dataDir, setDataDir] = useState("nozy-wallet-data");
  const [compactDb, setCompactDb] = useState("nozy-wallet-data/lwd_compact.sqlite");
  const [zebraUrl, setZebraUrl] = useState("http://127.0.0.1:8232");
  const [lwdUrl, setLwdUrl] = useState("http://127.0.0.1:9067");
  const [status, setStatus] = useState("");
  const [error, setError] = useState("");
  const available = isOnDeviceNativeAvailable || isOnDeviceBackendAvailable();

  useEffect(() => {
    void (async () => {
      const [m, d, c, z, l] = await Promise.all([
        AsyncStorage.getItem(ONDEVICE_MNEMONIC_KEY),
        AsyncStorage.getItem(ONDEVICE_DATA_DIR_KEY),
        AsyncStorage.getItem(ONDEVICE_COMPACT_DB_KEY),
        AsyncStorage.getItem(ONDEVICE_ZEBRA_URL_KEY),
        AsyncStorage.getItem(ONDEVICE_LWD_URL_KEY),
      ]);
      if (m) setMnemonic(m);
      if (d) setDataDir(d);
      if (c) setCompactDb(c);
      if (z) setZebraUrl(z);
      if (l) setLwdUrl(l);
    })();
  }, []);

  async function savePaths() {
    setError("");
    await AsyncStorage.multiSet([
      [ONDEVICE_DATA_DIR_KEY, dataDir.trim()],
      [ONDEVICE_COMPACT_DB_KEY, compactDb.trim()],
      [ONDEVICE_ZEBRA_URL_KEY, zebraUrl.trim()],
      [ONDEVICE_LWD_URL_KEY, lwdUrl.trim()],
    ]);
    if (mnemonic.trim()) {
      await AsyncStorage.setItem(ONDEVICE_MNEMONIC_KEY, mnemonic.trim());
    }
    setStatus("On-device Sapling paths saved.");
  }

  async function enableOnDevice() {
    setError("");
    try {
      await savePaths();
      await setBackendMode("on_device");
      setStatus("Backend mode: on-device (experimental).");
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
      Alert.alert("On-device unavailable", msg);
    }
  }

  async function useCompanion() {
    setError("");
    await setBackendMode("companion");
    setStatus("Backend mode: companion API.");
  }

  return (
    <SafeAreaView style={styles.safe} edges={["bottom"]}>
      <ScrollView
        contentContainerStyle={styles.container}
        keyboardShouldPersistTaps="handled"
      >
        <SettingsBackButton onPress={onBack} />
        <Text style={styles.title}>On-device wallet</Text>
        <Text style={styles.body}>
          Experimental Sapling shield-to-self via libnozy_ffi (#208). Still needs
          reachable Zebrad JSON-RPC and lightwalletd. Companion mode is unchanged
          for store builds.
        </Text>
        <Text style={styles.meta}>
          Native FFI: {available ? "available" : "not loaded"} · Mode:{" "}
          {backendMode}
        </Text>

        <Input
          label="BIP-39 mnemonic (on-device only)"
          value={mnemonic}
          onChangeText={setMnemonic}
          multiline
          autoCapitalize="none"
          secureTextEntry
        />
        <Input
          label="Wallet data dir (absolute on device)"
          value={dataDir}
          onChangeText={setDataDir}
          autoCapitalize="none"
        />
        <Input
          label="Compact DB path"
          value={compactDb}
          onChangeText={setCompactDb}
          autoCapitalize="none"
        />
        <Input
          label="Zebrad JSON-RPC URL"
          value={zebraUrl}
          onChangeText={setZebraUrl}
          autoCapitalize="none"
        />
        <Input
          label="lightwalletd gRPC URL"
          value={lwdUrl}
          onChangeText={setLwdUrl}
          autoCapitalize="none"
        />

        <View style={styles.actions}>
          <Button label="Save paths" onPress={() => void savePaths()} />
          <Button
            label="Use on-device backend"
            onPress={() => void enableOnDevice()}
            disabled={!available}
          />
          <Button
            label="Use companion API"
            variant="secondary"
            onPress={() => void useCompanion()}
          />
        </View>
        {status ? <Text style={styles.ok}>{status}</Text> : null}
        {error ? <Text style={styles.err}>{error}</Text> : null}
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  container: { padding: spacing.lg, gap: spacing.md },
  title: {
    color: colors.text,
    fontSize: fontSize.xl,
    fontWeight: "700",
  },
  body: { color: colors.textMuted, fontSize: fontSize.sm, lineHeight: 20 },
  meta: { color: colors.textMuted, fontSize: fontSize.sm },
  actions: { gap: spacing.sm, marginTop: spacing.sm },
  ok: { color: colors.primary, fontSize: fontSize.sm },
  err: { color: "#c0392b", fontSize: fontSize.sm },
});
