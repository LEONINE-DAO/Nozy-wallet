import { Pressable, StyleSheet, Text, View } from "react-native";
import { colors, fontSize, spacing } from "../../theme";

type Props = {
  title: string;
  description?: string;
  onPress: () => void;
};

export function SettingsItem({ title, description, onPress }: Props) {
  return (
    <Pressable onPress={onPress} style={styles.row}>
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
    paddingVertical: spacing.md,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: colors.border,
  },
  text: { flex: 1, paddingRight: spacing.sm },
  title: { color: colors.text, fontSize: fontSize.md, fontWeight: "600" },
  desc: { color: colors.textMuted, fontSize: fontSize.sm, marginTop: 4 },
  chevron: { color: colors.textMuted, fontSize: 22 },
});
