import { useState } from "react";
import { Modal, Pressable, StyleSheet, Text, View } from "react-native";
import { colors, fontSize, radius, spacing } from "../theme";

type Option = { label: string; value: string };

type Props = {
  label?: string;
  value: string;
  options: Option[];
  onChange: (value: string) => void;
  error?: string;
};

export function Select({ label, value, options, onChange, error }: Props) {
  const [open, setOpen] = useState(false);
  const selected = options.find((o) => o.value === value);

  return (
    <View style={styles.wrap}>
      {label ? <Text style={styles.label}>{label}</Text> : null}
      <Pressable
        style={[styles.trigger, error ? styles.triggerError : null]}
        onPress={() => setOpen(true)}
      >
        <Text style={styles.triggerText}>{selected?.label ?? value}</Text>
        <Text style={styles.chevron}>▾</Text>
      </Pressable>
      {error ? <Text style={styles.error}>{error}</Text> : null}

      <Modal visible={open} transparent animationType="fade" onRequestClose={() => setOpen(false)}>
        <View style={styles.overlay}>
          <Pressable
            style={StyleSheet.absoluteFillObject}
            onPress={() => setOpen(false)}
          />
          <View style={styles.sheet}>
            {options.map((opt) => (
              <Pressable
                key={opt.value}
                style={[styles.option, opt.value === value && styles.optionActive]}
                onPress={() => {
                  onChange(opt.value);
                  setOpen(false);
                }}
              >
                <Text
                  style={[
                    styles.optionText,
                    opt.value === value && styles.optionTextActive,
                  ]}
                >
                  {opt.label}
                </Text>
              </Pressable>
            ))}
          </View>
        </View>
      </Modal>
    </View>
  );
}

const styles = StyleSheet.create({
  wrap: { gap: spacing.sm },
  label: {
    color: colors.text,
    fontSize: fontSize.sm,
    fontWeight: "600",
  },
  trigger: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
    backgroundColor: colors.surfaceLight,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radius.md,
    paddingHorizontal: spacing.md,
    minHeight: 44,
  },
  triggerError: { borderColor: colors.error },
  triggerText: { color: colors.text, fontSize: fontSize.md },
  chevron: { color: colors.textMuted, fontSize: fontSize.md },
  error: { color: colors.error, fontSize: fontSize.sm },
  overlay: {
    flex: 1,
    backgroundColor: "rgba(0,0,0,0.5)",
    justifyContent: "flex-end",
  },
  sheet: {
    backgroundColor: colors.surface,
    borderTopLeftRadius: radius.xl,
    borderTopRightRadius: radius.xl,
    padding: spacing.md,
    gap: spacing.xs,
    zIndex: 1,
  },
  option: {
    padding: spacing.md,
    borderRadius: radius.md,
  },
  optionActive: {
    backgroundColor: colors.primary + "22",
  },
  optionText: {
    color: colors.text,
    fontSize: fontSize.md,
  },
  optionTextActive: {
    color: colors.primary,
    fontWeight: "700",
  },
});
