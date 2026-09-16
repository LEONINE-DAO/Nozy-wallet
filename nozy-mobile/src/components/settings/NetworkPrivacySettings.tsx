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
          Compact sync talks to lightwalletd. Default is
          https://lwd.nozywallet.org:443. That server sees your IP and that you
          are syncing — not your seed or keys.
        </Text>
        <Text style={styles.body}>
          For stronger network privacy, set your own lightwalletd URL in
          On-device wallet or Network & node, on a machine you control.
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
