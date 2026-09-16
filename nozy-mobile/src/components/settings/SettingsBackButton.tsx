import { Pressable, StyleSheet, Text, View } from "react-native";
import { useSafeAreaInsets } from "react-native-safe-area-context";
import { colors, fontSize, spacing } from "../../theme";

type Props = { onPress: () => void; label?: string };

export function SettingsBackButton({
  onPress,
  label = "Back to Settings",
}: Props) {
  const insets = useSafeAreaInsets();

  return (
    <View style={[styles.wrap, { paddingTop: Math.max(insets.top, spacing.sm) }]}>
      <Pressable
        onPress={onPress}
        accessibilityRole="button"
        accessibilityLabel={label}
        hitSlop={12}
        android_ripple={{ color: "rgba(200,204,212,0.18)" }}
        style={({ pressed }) => [styles.btn, pressed && styles.pressed]}
      >
        <Text style={styles.chevron}>‹</Text>
        <Text style={styles.label}>{label}</Text>
      </Pressable>
    </View>
  );
}

const styles = StyleSheet.create({
  wrap: {
    paddingHorizontal: spacing.lg,
    paddingBottom: spacing.xs,
    backgroundColor: colors.background,
  },
  btn: {
    minHeight: 48,
    flexDirection: "row",
    alignItems: "center",
    alignSelf: "flex-start",
    paddingRight: spacing.md,
    gap: spacing.xs,
  },
  pressed: { opacity: 0.7 },
  chevron: {
    color: colors.primary,
    fontSize: 28,
    fontWeight: "500",
    lineHeight: 32,
    marginTop: -2,
  },
  label: {
    color: colors.primary,
    fontSize: fontSize.md,
    fontWeight: "700",
  },
});
