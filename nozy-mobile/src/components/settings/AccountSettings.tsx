import { useEffect, useState } from "react";
import { ScrollView, StyleSheet, Text } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import * as Clipboard from "expo-clipboard";
import { Button } from "../Button";
import { SettingsBackButton } from "./SettingsBackButton";
import { onDeviceReceiveAddress } from "../../services/onDeviceWallet";
import { useWalletSession } from "../../context/WalletSessionContext";
import { colors, fontSize, spacing } from "../../theme";

type Props = { onBack: () => void };

export function AccountSettings({ onBack }: Props) {
  const { backendMode } = useWalletSession();
  const [address, setAddress] = useState("");
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    if (backendMode !== "on_device") return;
    void (async () => {
      try {
        const res = await onDeviceReceiveAddress();
        setAddress(res.address);
      } catch (e) {
        setError(e instanceof Error ? e.message : "Could not load address");
      }
    })();
  }, [backendMode]);

  async function copyAddress() {
    if (!address) return;
    await Clipboard.setStringAsync(address);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  return (
    <SafeAreaView style={styles.safe} edges={["bottom"]}>
      <SettingsBackButton onPress={onBack} />
      <ScrollView
        contentContainerStyle={styles.container}
        keyboardShouldPersistTaps="handled"
      >
        <Text style={styles.title}>Account</Text>
        <Text style={styles.body}>
          This phone holds your Nozy wallet. Keys stay here. Use Receive to
          share your unified address.
        </Text>
        {address ? (
          <>
            <Text style={styles.label}>Unified address</Text>
            <Text style={styles.address} selectable>
              {address}
            </Text>
            <Button
              label={copied ? "Copied!" : "Copy address"}
              onPress={() => void copyAddress()}
            />
          </>
        ) : (
          <Text style={styles.body}>
            {error || "Unlock the wallet to see your address."}
          </Text>
        )}
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  container: { padding: spacing.lg, gap: spacing.md },
  title: { color: colors.text, fontSize: fontSize.xl, fontWeight: "700" },
  body: { color: colors.textMuted, fontSize: fontSize.sm, lineHeight: 20 },
  label: { color: colors.textMuted, fontSize: fontSize.sm, fontWeight: "600" },
  address: {
    color: colors.text,
    fontSize: 12,
    fontFamily: "monospace",
    lineHeight: 18,
  },
});
