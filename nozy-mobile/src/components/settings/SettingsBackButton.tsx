import { Pressable, StyleSheet, Text } from "react-native";
import { colors, fontSize, spacing } from "../../theme";

type Props = { onPress: () => void; label?: string };

export function SettingsBackButton({ onPress, label = "Back" }: Props) {
  return (
    <Pressable onPress={onPress} style={styles.btn} accessibilityRole="button">
      <Text style={styles.label}>‹ {label}</Text>
    </Pressable>
  );
}

const styles = StyleSheet.create({
  btn: { paddingVertical: spacing.sm, marginBottom: spacing.sm },
  label: { color: colors.primary, fontSize: fontSize.md, fontWeight: "600" },
});
