import { Pressable, StyleSheet, Text, View } from "react-native";
import { colors, fontSize, radius, spacing } from "../../theme";

type Props = {
  title: string;
  description?: string;
  onPress: () => void;
};

export function SettingsItem({ title, description, onPress }: Props) {
  return (
    <Pressable
      onPress={onPress}
      style={({ pressed }) => [styles.row, pressed && styles.pressed]}
    >
      <View style={styles.text}>
        <Text style={styles.title}>{title}</Text>
        {description ? <Text style={styles.desc}>{description}</Text> : null}
      </View>
      <Text style={styles.chevron}>›</Text>
    </Pressable>
  );
}

const styles = StyleSheet.create({
  row: {
    flexDirection: "row",
    alignItems: "center",
    paddingVertical: 12,
    paddingHorizontal: spacing.md,
    borderRadius: radius.md,
    borderWidth: 1,
    borderColor: colors.border,
    backgroundColor: colors.surface,
    marginBottom: spacing.xs,
  },
  pressed: {
    borderColor: colors.platinumLine,
    backgroundColor: colors.primarySoft,
  },
  text: { flex: 1, paddingRight: spacing.sm },
  title: { color: colors.text, fontSize: fontSize.sm, fontWeight: "600" },
  desc: {
    color: colors.textFaint,
    fontSize: 11,
    marginTop: 2,
    lineHeight: 15,
  },
  chevron: { color: colors.textMuted, fontSize: 20 },
});
