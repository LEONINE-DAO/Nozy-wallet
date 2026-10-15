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
import { isOnDeviceBackendAvailable } from "../lib/walletBackend";
import { api } from "../services/api";
import { onDeviceReceiveAddress, readCachedReceiveAddress } from "../services/onDeviceWallet";
import { getActiveOnDeviceAccount } from "../services/onDeviceAccounts";
import { colors, fontSize, spacing } from "../theme";

export function ReceiveScreen() {
  const { password, backendMode } = useWalletSession();
  const [address, setAddress] = useState("");
  const [accountName, setAccountName] = useState("");
  const [copied, setCopied] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const load = useCallback(async () => {
    setError("");
    setLoading(true);
    try {
      if (backendMode === "on_device" || isOnDeviceBackendAvailable()) {
        try {
          const account = await getActiveOnDeviceAccount();
          const cached = await readCachedReceiveAddress(account.index);
          if (cached) {
            setAddress(cached);
            setAccountName(account.name);
            return;
          }
          const res = await onDeviceReceiveAddress();
          const next = (res.address ?? "").trim();
          if (next) {
            setAddress(next);
            setAccountName(account.name);
            return;
          }
          if (backendMode === "on_device") {
            throw new Error("No receive address yet. Create or restore a wallet first.");
          }
        } catch (e) {
          if (backendMode === "on_device") throw e;
        }
      }
      setAccountName("");
      const res = await Promise.race([
        api.generateAddress(password || undefined),
        new Promise<never>((_, reject) => {
          setTimeout(
            () =>
              reject(
                new Error(
                  "Could not reach the wallet. Create or restore a wallet on this phone, then reveal the address again.",
                ),
              ),
            8000,
          );
        }),
      ]);
      const next = (res.address ?? "").trim();
      if (!next) {
        throw new Error("The wallet did not return a receive address.");
      }
      setAddress(next);
    } catch (e) {
      const message = e instanceof Error ? e.message : "Could not load address";
      setAddress("");
      setError(message);
    } finally {
      setLoading(false);
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
              {loading
                ? "Revealing your unified address…"
                : "Reveal your unified address to copy it or show a QR code."}
            </Text>
          )}
          {address ? <ReceiveQr address={address} /> : null}
          <Button
            label={
              loading
                ? "Revealing…"
                : address
                  ? copied
                    ? "Copied!"
                    : "Copy address"
                  : "Reveal address"
            }
            onPress={() => {
              if (address) void copyAddress();
              else void load();
            }}
            disabled={loading}
            loading={loading}
            variant={copied ? "secondary" : "primary"}
          />
        </Card>

        <Text style={styles.hint}>
          Incoming ZEC shows up on Home on its own while the wallet is unlocked.
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
    color: colors.text,
    fontSize: fontSize.sm,
    fontFamily: "monospace",
    lineHeight: 20,
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
