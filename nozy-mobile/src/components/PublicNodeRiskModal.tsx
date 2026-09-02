import { StyleSheet, Text, View } from "react-native";
import { Button } from "./Button";
import { Modal } from "./Modal";
import { colors, fontSize, spacing } from "../theme";

type Props = {
  visible: boolean;
  onCancel: () => void;
  onConfirm: () => void;
  context: "api" | "zebra";
};

const copy = {
  api: {
    title: "Use hosted API?",
    body:
      "A third-party API can see your IP, request timing, and wallet operations it proxies. Prefer your own API on a PC or VPS you control when possible.",
  },
  zebra: {
    title: "Use a public node?",
    body:
      "Connecting through a public Zebra node can leak your IP and which blocks or transactions you query. A local or self-hosted node is more private.",
  },
} as const;

export function PublicNodeRiskModal({
  visible,
  onCancel,
  onConfirm,
  context,
}: Props) {
  const { title, body } = copy[context];

  return (
    <Modal visible={visible} onClose={onCancel} title={title}>
      <Text style={styles.body}>{body}</Text>
      <View style={styles.actions}>
        <Button label="Cancel" variant="secondary" onPress={onCancel} />
        <Button label="I understand" onPress={onConfirm} />
      </View>
    </Modal>
  );
}

const styles = StyleSheet.create({
  body: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
    lineHeight: 20,
  },
  actions: {
    flexDirection: "row",
    gap: spacing.sm,
    justifyContent: "flex-end",
    marginTop: spacing.md,
  },
});
