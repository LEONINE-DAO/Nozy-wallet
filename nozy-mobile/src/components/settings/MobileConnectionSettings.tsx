import { useEffect, useState } from "react";
import { Alert, ScrollView, StyleSheet, Text } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { ConnectionSetupFields } from "../ConnectionSetupFields";
import { SettingsBackButton } from "./SettingsBackButton";
import { useWalletSession } from "../../context/WalletSessionContext";
import { requireHostedApiKey } from "../../lib/buildProfile";
import { isHostedApiUrl } from "../../lib/connectionPresets";
import { api } from "../../services/api";
import { colors, fontSize, spacing } from "../../theme";

type Props = { onBack: () => void };

export function MobileConnectionSettings({ onBack }: Props) {
  const { apiUrl, setApiUrl, apiKey, setApiKey } = useWalletSession();
  const [urlDraft, setUrlDraft] = useState(apiUrl);
  const [keyDraft, setKeyDraft] = useState(apiKey);
  const [status, setStatus] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    setUrlDraft(apiUrl);
    setKeyDraft(apiKey);
  }, [apiUrl, apiKey]);

  async function save() {
    setLoading(true);
    setError("");
    setStatus("");
    const previousUrl = apiUrl;
    const previousKey = apiKey;
    try {
      if (
        requireHostedApiKey() &&
        isHostedApiUrl(urlDraft) &&
        !keyDraft.trim()
      ) {
        const msg = "API key is required for the hosted API.";
        setError(msg);
        Alert.alert("Cannot save", msg);
        return;
      }
      await setApiUrl(urlDraft);
      await setApiKey(keyDraft);
      await Promise.race([
        api.health(),
        new Promise<never>((_, reject) =>
          setTimeout(
            () =>
              reject(
                new Error(
                  "Timed out reaching API. Check the URL (real phones cannot use 10.0.2.2) and that the API is running.",
                ),
              ),
            12000,
          ),
        ),
      ]);
      const ok = "API URL and key saved — connected";
      setStatus(ok);
      Alert.alert("Connected", ok);
    } catch (e) {
      await setApiUrl(previousUrl);
      await setApiKey(previousKey);
      setUrlDraft(previousUrl);
      setKeyDraft(previousKey);
      const msg = e instanceof Error ? e.message : "Could not reach API";
      setError(msg);
      Alert.alert("Connection failed", msg);
    } finally {
      setLoading(false);
    }
  }

  return (
    <SafeAreaView style={styles.safe} edges={["bottom"]}>
      <ScrollView
        contentContainerStyle={styles.container}
        keyboardShouldPersistTaps="handled"
      >
        <SettingsBackButton onPress={onBack} />
        <Text style={styles.title}>Mobile connection</Text>
        <Text style={styles.subtitle}>
          Sync needs a working Zebrad behind the API. Use your own home PC / VPS
          node, or another operator you trust. Nozy hosted API has no Nozy Zebrad
          until funding — it will not sync by itself.
        </Text>
        <ConnectionSetupFields
          urlDraft={urlDraft}
          keyDraft={keyDraft}
          onUrlChange={setUrlDraft}
          onKeyChange={setKeyDraft}
          onSave={save}
          saving={loading}
          status={status}
          error={error}
        />
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  container: { padding: spacing.lg, gap: spacing.md },
  title: { color: colors.text, fontSize: fontSize.xl, fontWeight: "700" },
  subtitle: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
    marginBottom: spacing.sm,
  },
});
