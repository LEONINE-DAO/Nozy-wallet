import { ScrollView, StyleSheet, Text } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { Card } from "../Card";
import { Toggle } from "../Toggle";
import { SettingsBackButton } from "./SettingsBackButton";
import { useWalletSession } from "../../context/WalletSessionContext";
import { colors, fontSize, spacing } from "../../theme";

type Props = { onBack: () => void };

export function SyncSettings({ onBack }: Props) {
  const { autoSync, setAutoSync } = useWalletSession();

  return (
    <SafeAreaView style={styles.safe} edges={["bottom"]}>
      <ScrollView contentContainerStyle={styles.container}>
        <SettingsBackButton onPress={onBack} />
        <Text style={styles.title}>Sync</Text>
        <Text style={styles.subtitle}>
          Keep the wallet near chain tip while unlocked so you stay ready to send.
          Needs a reachable companion API; Zebrad must be reachable from that API
          host (not from the phone).
        </Text>
        <Card>
          <Toggle
            title="Keep synced while unlocked"
            description="Checks every ~20s and catches up when tip moves. Syncs again when you reopen the app."
            checked={autoSync}
            onChange={(v) => void setAutoSync(v)}
          />
        </Card>
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  container: { padding: spacing.lg, gap: spacing.md },
  title: { color: colors.text, fontSize: fontSize.xl, fontWeight: "700" },
  subtitle: { color: colors.textMuted, fontSize: fontSize.sm, marginBottom: spacing.sm },
});
