import * as Clipboard from "expo-clipboard";
import { useFocusEffect } from "@react-navigation/native";
import { useCallback, useState } from "react";
import { ScrollView, StyleSheet, Text } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "../components/Button";
import { Card } from "../components/Card";
import { PageHeader } from "../components/PageHeader";
import { ReceiveQr } from "../components/ReceiveQr";
import { useWalletSession } from "../context/WalletSessionContext";
import { api } from "../services/api";
import { onDeviceReceiveAddress } from "../services/onDeviceWallet";
import { getActiveOnDeviceAccount } from "../services/onDeviceAccounts";
import { colors, fontSize, spacing } from "../theme";

export function ReceiveScreen() {
  const { password, backendMode } = useWalletSession();
  const [address, setAddress] = useState("");
  const [accountName, setAccountName] = useState("");
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState("");

  const load = useCallback(async () => {
    setError("");
    try {
      if (backendMode === "on_device") {
        const [res, account] = await Promise.all([
          onDeviceReceiveAddress(),
          getActiveOnDeviceAccount(),
        ]);
        setAddress(res.address);
        setAccountName(account.name);
        return;
      }
      setAccountName("");
      const res = await api.generateAddress(password || undefined);
      setAddress(res.address);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not load address");
    }
  }, [password, backendMode]);

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
      <ScrollView
        contentContainerStyle={styles.container}
        keyboardShouldPersistTaps="handled"
      >
        <PageHeader
          title="Receive"
          description={
            accountName
              ? `Unified address for ${accountName}.`
              : "Unified address for shielded ZEC."
          }
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
          {address ? <ReceiveQr address={address} /> : null}
        </Card>

        <Text style={styles.hint}>
          After someone pays you, run a sync from Settings so the new note shows
          in your balance.
        </Text>
        {error ? <Text style={styles.error}>{error}</Text> : null}
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  container: { padding: spacing.lg, gap: spacing.md, paddingBottom: spacing.xl },
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
