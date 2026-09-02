import * as Clipboard from "expo-clipboard";
import { useFocusEffect } from "@react-navigation/native";
import { useCallback, useState } from "react";
import { StyleSheet, Text, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "../components/Button";
import { Card } from "../components/Card";
import { PageHeader } from "../components/PageHeader";
import { useWalletSession } from "../context/WalletSessionContext";
import { api } from "../services/api";
import { colors, fontSize, spacing } from "../theme";

export function ReceiveScreen() {
  const { password } = useWalletSession();
  const [address, setAddress] = useState("");
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState("");

  const load = useCallback(async () => {
    setError("");
    try {
      const res = await api.generateAddress(password || undefined);
      setAddress(res.address);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not load address");
    }
  }, [password]);

  useFocusEffect(
    useCallback(() => {
      void load();
    }, [load]),
  );

  async function copyAddress() {
    if (!address) return;
    await Clipboard.setStringAsync(address);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  return (
    <SafeAreaView style={styles.safe} edges={["top"]}>
      <View style={styles.container}>
        <PageHeader
          title="Receive"
          description="Unified address for shielded ZEC."
        />

        <Card variant="elevated" padding="lg">
          {address ? (
            <Text style={styles.address} selectable>
              {address}
            </Text>
          ) : (
            <Text style={styles.empty}>
              No address yet — unlock and sync your wallet.
            </Text>
          )}
          <Button
            label={copied ? "Copied!" : "Copy address"}
            onPress={() => void copyAddress()}
            disabled={!address}
            variant={copied ? "secondary" : "primary"}
          />
        </Card>

        <Text style={styles.hint}>
          After someone pays you, run a sync from Settings so the new note shows
          in your balance.
        </Text>
        {error ? <Text style={styles.error}>{error}</Text> : null}
      </View>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  container: { flex: 1, padding: spacing.lg, gap: spacing.md },
  address: {
    color: colors.textMuted,
    fontSize: 11,
    fontFamily: "monospace",
    lineHeight: 18,
    marginBottom: spacing.md,
  },
  empty: {
    color: colors.textFaint,
    fontSize: fontSize.sm,
    marginBottom: spacing.md,
    textAlign: "center",
  },
  hint: {
    color: colors.textFaint,
    fontSize: 11,
    lineHeight: 16,
  },
  error: { color: colors.error, fontSize: fontSize.sm },
});
