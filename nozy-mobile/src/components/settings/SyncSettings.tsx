import { ScrollView, StyleSheet, Switch, Text, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { CyberpunkSyncPanel } from "../CyberpunkSyncPanel";
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
        {autoSync ? (
          <CyberpunkSyncPanel
            headline="Auto-sync enabled"
            detail="Syncs when you open the app and while unlocked"
            tone="ok"
          />
        ) : null}
        <View style={styles.card}>
          <View style={styles.toggleRow}>
            <View style={styles.toggleCopy}>
              <Text style={styles.toggleTitle}>Keep synced while unlocked</Text>
              <Text style={styles.toggleDesc}>
                Checks every ~20s and catches up when tip moves. Syncs again when you
                reopen the app.
              </Text>
            </View>
            <Switch
              value={autoSync}
              onValueChange={(v) => void setAutoSync(v)}
              trackColor={{ false: colors.border, true: "rgba(0, 255, 140, 0.35)" }}
              thumbColor={autoSync ? "#39ff14" : "#f4f4f5"}
            />
          </View>
        </View>
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  container: { padding: spacing.lg, gap: spacing.md },
  title: { color: colors.text, fontSize: fontSize.xl, fontWeight: "700" },
  subtitle: { color: colors.textMuted, fontSize: fontSize.sm, marginBottom: spacing.sm },
  card: {
    backgroundColor: colors.surface,
    borderRadius: 16,
    padding: spacing.lg,
    borderWidth: 1,
    borderColor: colors.border,
  },
  toggleRow: {
    flexDirection: "row",
    alignItems: "center",
    gap: spacing.md,
  },
  toggleCopy: { flex: 1, gap: spacing.xs },
  toggleTitle: { color: colors.text, fontSize: fontSize.md, fontWeight: "600" },
  toggleDesc: { color: colors.textMuted, fontSize: fontSize.sm, lineHeight: 20 },
});
//Lowo do this in his sleep