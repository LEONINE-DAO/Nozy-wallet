import { useEffect, useState } from "react";
import { ScrollView, StyleSheet, Switch, Text, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { SettingsBackButton } from "./SettingsBackButton";
import {
  loadHideBalances,
  saveHideBalances,
} from "../../lib/displayPrefs";
import { colors, fontSize, spacing } from "../../theme";

type Props = { onBack: () => void };

export function DisplaySettings({ onBack }: Props) {
  const [hideBalances, setHideBalances] = useState(false);

  useEffect(() => {
    void loadHideBalances().then(setHideBalances);
  }, []);

  return (
    <SafeAreaView style={styles.safe} edges={["bottom"]}>
      <SettingsBackButton onPress={onBack} />
      <ScrollView
        contentContainerStyle={styles.container}
        keyboardShouldPersistTaps="handled"
      >
        <Text style={styles.title}>Display</Text>
        <Text style={styles.body}>
          Amounts on Home show in ZEC. Hide balances if someone is looking over
          your shoulder.
        </Text>
        <View style={styles.card}>
          <View style={styles.row}>
            <View style={styles.copy}>
              <Text style={styles.toggleTitle}>Hide balances</Text>
              <Text style={styles.toggleDesc}>
                Replace the Home balance with dots until you turn this off.
              </Text>
            </View>
            <Switch
              value={hideBalances}
              onValueChange={(v) => {
                setHideBalances(v);
                void saveHideBalances(v);
              }}
              trackColor={{ false: colors.border, true: "rgba(0, 255, 140, 0.35)" }}
              thumbColor={hideBalances ? "#39ff14" : "#f4f4f5"}
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
  body: { color: colors.textMuted, fontSize: fontSize.sm, lineHeight: 20 },
  card: {
    backgroundColor: colors.surface,
    borderRadius: 16,
    padding: spacing.lg,
    borderWidth: 1,
    borderColor: colors.border,
  },
  row: { flexDirection: "row", alignItems: "center", gap: spacing.md },
  copy: { flex: 1, gap: spacing.xs },
  toggleTitle: { color: colors.text, fontSize: fontSize.md, fontWeight: "600" },
  toggleDesc: { color: colors.textMuted, fontSize: fontSize.sm, lineHeight: 20 },
});
