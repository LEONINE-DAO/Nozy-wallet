import { Pressable, StyleSheet, Text, View } from "react-native";
import { colors, radius, spacing } from "../theme";

type Tone = "neutral" | "syncing" | "ok" | "warn" | "offline";

type Props = {
  label: string;
  tone?: Tone;
  onPress?: () => void;
};

const toneStyles: Record<Tone, { bg: string; border: string; text: string }> = {
  neutral: {
    bg: colors.surfaceAlt,
    border: colors.border,
    text: colors.textMuted,
  },
  syncing: {
    bg: "rgba(0, 255, 140, 0.1)",
    border: "rgba(0, 255, 140, 0.35)",
    text: "#a7f3d0",
  },
  ok: {
    bg: colors.successSoft,
    border: "rgba(34, 197, 94, 0.35)",
    text: colors.success,
  },
  warn: {
    bg: colors.warnBg,
    border: "rgba(245, 158, 11, 0.35)",
    text: colors.warn,
  },
  offline: {
    bg: "rgba(239, 68, 68, 0.1)",
    border: "rgba(239, 68, 68, 0.35)",
    text: colors.error,
  },
};

export function SyncPill({ label, tone = "neutral", onPress }: Props) {
  const t = toneStyles[tone];
  const inner = (
    <Text style={[styles.label, { color: t.text }]} numberOfLines={1}>
      {label}
    </Text>
  );

  if (!onPress) {
    return (
      <View style={[styles.pill, { backgroundColor: t.bg, borderColor: t.border }]}>
        {inner}
      </View>
    );
  }

  return (
    <Pressable
      onPress={onPress}
      style={[styles.pill, { backgroundColor: t.bg, borderColor: t.border }]}
    >
      {inner}
    </Pressable>
  );
}

const styles = StyleSheet.create({
  pill: {
    borderWidth: 1,
    borderRadius: radius.sm,
    paddingHorizontal: 10,
    paddingVertical: 5,
    maxWidth: "50%",
  },
  label: {
    fontSize: 10,
    fontWeight: "700",
    letterSpacing: 0.3,
  },
});
