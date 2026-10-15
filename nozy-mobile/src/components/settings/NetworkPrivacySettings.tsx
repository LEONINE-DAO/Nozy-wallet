import { ScrollView, StyleSheet, Text } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { SettingsBackButton } from "./SettingsBackButton";
import { colors, fontSize, spacing } from "../../theme";

type Props = { onBack: () => void };

export function NetworkPrivacySettings({ onBack }: Props) {
  return (
    <SafeAreaView style={styles.safe} edges={["bottom"]}>
      <SettingsBackButton onPress={onBack} />
      <ScrollView
        contentContainerStyle={styles.container}
        keyboardShouldPersistTaps="handled"
      >
        <Text style={styles.title}>Network privacy</Text>
        <Text style={styles.body}>
          Compact sync uses the Nozy server at
          https://lwd.nozywallet.org:443, or a node on your home network. That
          server sees your IP and that you are syncing — not your seed or keys.
        </Text>
        <Text style={styles.body}>
          Other servers, including zec.rocks, are not used. A node you run
          yourself on this phone, this computer, or your home network is allowed.
        </Text>
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  container: { padding: spacing.lg, gap: spacing.md },
  title: { color: colors.text, fontSize: fontSize.xl, fontWeight: "700" },
  body: { color: colors.textMuted, fontSize: fontSize.sm, lineHeight: 20 },
});
