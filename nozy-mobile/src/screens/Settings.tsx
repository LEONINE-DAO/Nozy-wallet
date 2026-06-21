import { NativeStackScreenProps } from "@react-navigation/native-stack";
import { useEffect, useState } from "react";
import { ScrollView, StyleSheet, Text, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "../components/Button";
import { Input } from "../components/Input";
import { useWalletSession } from "../context/WalletSessionContext";
import { api } from "../services/api";
import { colors, fontSize, spacing } from "../theme";
import type { RootStackParamList } from "../types";

type Props = NativeStackScreenProps<RootStackParamList, "Settings">;

export function SettingsScreen({ navigation }: Props) {
  const { apiUrl, setApiUrl, apiKey, setApiKey, autoSync, setAutoSync } =
    useWalletSession();
  const [zebraUrl, setZebraUrl] = useState("");
  const [network, setNetwork] = useState("");
  const [theme, setTheme] = useState("dark");
  const [mobileApiUrl, setMobileApiUrl] = useState(apiUrl);
  const [mobileApiKey, setMobileApiKey] = useState(apiKey);
  const [status, setStatus] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    setMobileApiKey(apiKey);
  }, [apiKey]);

  useEffect(() => {
    void api
      .getConfig()
      .then((c) => {
        setZebraUrl(c.zebra_url);
        setNetwork(c.network);
        setTheme(c.theme);
      })
      .catch(() => {});
  }, []);

  async function saveZebra() {
    setError("");
    setLoading(true);
    try {
      await api.setZebraUrl(zebraUrl.trim());
      setStatus("Zebra URL saved");
    } catch (e) {
      setError(e instanceof Error ? e.message : "Save failed");
    } finally {
      setLoading(false);
    }
  }

  async function testZebra() {
    setError("");
    setStatus("");
    setLoading(true);
    try {
      const res = await api.testZebra(zebraUrl.trim() || undefined);
      setStatus(res.message);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Connection test failed");
    } finally {
      setLoading(false);
    }
  }

  async function toggleTheme() {
    const next = theme === "dark" ? "light" : "dark";
    try {
      await api.setTheme(next);
      setTheme(next);
      setStatus(`Theme set to ${next}`);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Theme update failed");
    }
  }

  async function saveMobileApi() {
    await setApiUrl(mobileApiUrl);
    await setApiKey(mobileApiKey);
    setStatus("API URL and key saved");
  }

  return (
    <SafeAreaView style={styles.safe} edges={["bottom"]}>
      <ScrollView contentContainerStyle={styles.container}>
        <Text style={styles.section}>Mobile app</Text>
        <Input
          label="API server URL (this app)"
          value={mobileApiUrl}
          onChangeText={setMobileApiUrl}
          autoCapitalize="none"
        />
        <Input
          label="API key (for VPS with NOZY_API_KEY)"
          value={mobileApiKey}
          onChangeText={setMobileApiKey}
          autoCapitalize="none"
          secureTextEntry
          placeholder="Leave blank for local API"
        />
        <Button
          label="Save API URL & key"
          variant="secondary"
          onPress={() => void saveMobileApi()}
        />

        <Text style={styles.section}>Wallet / node</Text>
        <Text style={styles.meta}>Network: {network || "..."}</Text>
        <Input
          label="Zebra RPC URL"
          value={zebraUrl}
          onChangeText={setZebraUrl}
          autoCapitalize="none"
          placeholder="http://127.0.0.1:8232"
        />
        <View style={styles.row}>
          <Button
            label="Save Zebra URL"
            onPress={() => void saveZebra()}
            loading={loading}
            style={styles.half}
          />
          <Button
            label="Test connection"
            variant="secondary"
            onPress={() => void testZebra()}
            loading={loading}
            style={styles.half}
          />
        </View>

        <Text style={styles.section}>Sync</Text>
        <Button
          label={autoSync ? "Auto-sync on dashboard: ON" : "Auto-sync on dashboard: OFF"}
          variant="secondary"
          onPress={() => void setAutoSync(!autoSync)}
        />
        <Text style={styles.meta}>
          When ON, wallet syncs automatically each time you open the Dashboard.
          First sync can take a long time — keep the API running.
        </Text>

        <Text style={styles.section}>Appearance</Text>
        <Button
          label={`Theme: ${theme} (tap to switch)`}
          variant="secondary"
          onPress={() => void toggleTheme()}
        />

        <Text style={styles.section}>About</Text>
        <Button
          label="About & privacy"
          variant="secondary"
          onPress={() => navigation.navigate("About")}
        />

        {status ? <Text style={styles.ok}>{status}</Text> : null}
        {error ? <Text style={styles.error}>{error}</Text> : null}

        <Button label="Back" variant="ghost" onPress={() => navigation.goBack()} />
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  container: { padding: spacing.lg, gap: spacing.md },
  section: {
    color: colors.primary,
    fontSize: fontSize.sm,
    fontWeight: "700",
    letterSpacing: 1,
    textTransform: "uppercase",
    marginTop: spacing.md,
  },
  meta: { color: colors.textMuted, fontSize: fontSize.sm },
  row: { flexDirection: "row", gap: spacing.sm },
  half: { flex: 1 },
  ok: { color: colors.success, fontSize: fontSize.sm },
  error: { color: colors.error, fontSize: fontSize.sm },
});
