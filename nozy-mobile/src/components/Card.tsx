import { StyleProp, StyleSheet, View, ViewStyle } from "react-native";
import { colors, radius, shadows, spacing } from "../theme";

type Props = {
  children: React.ReactNode;
  variant?: "glass" | "solid" | "elevated";
  padding?: "none" | "sm" | "md" | "lg";
  style?: StyleProp<ViewStyle>;
};

const paddingMap = {
  none: 0,
  sm: spacing.sm,
  md: 14,
  lg: spacing.lg,
} as const;

export function Card({
  children,
  variant = "solid",
  padding = "md",
  style,
}: Props) {
  return (
    <View
      style={[
        styles.base,
        variant === "elevated" && styles.elevated,
        variant === "glass" && styles.glass,
        { padding: paddingMap[padding] },
        style,
      ]}
    >
      {children}
    </View>
  );
}

const styles = StyleSheet.create({
  base: {
    borderRadius: radius.lg,
    borderWidth: 1,
    borderColor: colors.border,
    backgroundColor: colors.surface,
    ...shadows.card,
  },
  glass: {
    backgroundColor: "rgba(12, 18, 16, 0.88)",
  },
  elevated: {
    borderColor: colors.platinumLine,
    backgroundColor: colors.surface,
  },
});
