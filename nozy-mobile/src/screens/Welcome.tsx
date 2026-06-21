import { NativeStackScreenProps } from "@react-navigation/native-stack";
import { useEffect, useState } from "react";
import {
  ActivityIndicator,
  Platform,
  StyleSheet,
  Text,
  View,
} from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "../components/Button";
import { Input } from "../components/Input";
import { useWalletSession } from "../context/WalletSessionContext";
import { api } from "../services/api";
import { colors, fontSize, spacing } from "../theme";
import type { RootStackParamList } from "../types";

type Props = NativeStackScreenProps<RootStackParamList, "Welcome">;

export function WelcomeScreen({ navigation }: Props) {
  const { apiUrl, setApiUrl, apiKey, setApiKey } = useWalletSession();
  const [checking, setChecking] = useState(true);
  const [apiOk, setApiOk] = useState(false);
  const [error, setError] = useState("");
  const [urlDraft, setUrlDraft] = useState(apiUrl);
  const [keyDraft, setKeyDraft] = useState(apiKey);

  useEffect(() => {
    setKeyDraft(apiKey);
  }, [apiKey]);

  useEffect(() => {
    void checkWallet();
  }, [apiUrl, apiKey]);

  async function checkWallet() {
    setChecking(true);
    setError("");
    try {
      await api.health();
      setApiOk(true);
      const info = await api.walletExists();
      if (info.exists) {
        navigation.replace(info.has_password ? "Unlock" : "Dashboard");
        return;
      }
    } catch (e) {
      setApiOk(false);
      setError(e instanceof Error ? e.message : "Could not reach API");
    } finally {
      setChecking(false);
    }
  }

  async function saveConnection() {
    await setApiUrl(urlDraft);
    await setApiKey(keyDraft);
    await checkWallet();
  }

  return (
    <SafeAreaView style={styles.safe}>
      <View style={styles.container}>
        <Text style={styles.badge}>LEONINE DAO</Text>
        <Text style={styles.title}>NozyWallet</Text>
        <Text style={styles.subtitle}>
          Private Zcash wallet — mobile companion to your local API.
        </Text>

        <View style={styles.card}>
          <Input
            label="API server URL"
            value={urlDraft}
            onChangeText={setUrlDraft}
            autoCapitalize="none"
            autoCorrect={false}
            placeholder={
              Platform.OS === "android"
                ? "http://10.0.2.2:3000"
                : "http://localhost:3000"
            }
          />
          <Input
            label="API key (optional — required for public VPS)"
            value={keyDraft}
            onChangeText={setKeyDraft}
            autoCapitalize="none"
            autoCorrect={false}
            secureTextEntry
            placeholder="Leave blank for local API"
          />
          <Button label="Save & connect" onPress={() => void saveConnection()} />
          <View style={styles.statusRow}>
            {checking ? (
              <ActivityIndicator color={colors.primary} />
            ) : (
              <Text style={[styles.status, apiOk ? styles.ok : styles.bad]}>
                {apiOk ? "API connected" : error || "API offline"}
              </Text>
            )}
          </View>
        </View>

        <View style={styles.actions}>
          <Button
            label="Create new wallet"
            onPress={() => navigation.navigate("CreateWallet")}
            disabled={!apiOk || checking}
          />
          <Button
            label="Restore from seed"
            variant="secondary"
            onPress={() => navigation.navigate("RestoreWallet")}
            disabled={!apiOk || checking}
          />
        </View>
      </View>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: {
    flex: 1,
    backgroundColor: colors.background,
  },
  container: {
    flex: 1,
    padding: spacing.lg,
    justifyContent: "center",
    gap: spacing.lg,
  },
  badge: {
    color: colors.primary,
    fontSize: fontSize.sm,
    fontWeight: "700",
    letterSpacing: 2,
  },
  title: {
    color: colors.text,
    fontSize: fontSize.xxl,
    fontWeight: "800",
  },
  subtitle: {
    color: colors.textMuted,
    fontSize: fontSize.md,
    lineHeight: 24,
  },
  card: {
    backgroundColor: colors.surface,
    borderRadius: 16,
    padding: spacing.lg,
    gap: spacing.md,
    borderWidth: 1,
    borderColor: colors.border,
  },
  statusRow: {
    minHeight: 24,
    alignItems: "center",
  },
  status: {
    fontSize: fontSize.sm,
    fontWeight: "600",
  },
  ok: {
    color: colors.success,
  },
  bad: {
    color: colors.error,
  },
  actions: {
    gap: spacing.md,
  },
});
