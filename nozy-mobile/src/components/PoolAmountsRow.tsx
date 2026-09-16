import { StyleSheet, Text, View } from "react-native";
import { colors, fontSize, radius, spacing } from "../theme";

function formatPool(amount: number, hide: boolean): string {
  if (hide) return "••••";
  return `${amount.toLocaleString(undefined, {
    minimumFractionDigits: 2,
    maximumFractionDigits: 8,
  })} ZEC`;
}

function PoolTile({
  label,
  amount,
  hideBalances,
}: {
  label: string;
  amount: number;
  hideBalances: boolean;
}) {
  const lit = !hideBalances && amount > 0;
  return (
    <View style={[styles.tile, lit && styles.tileLit]}>
      <Text style={styles.label}>{label}</Text>
      <Text style={styles.value} numberOfLines={1}>
        {formatPool(amount, hideBalances)}
      </Text>
    </View>
  );
}

export function PoolAmountsRow({
  orchardZec,
  ironwoodZec,
  hideBalances,
}: {
  orchardZec: number;
  ironwoodZec: number;
  hideBalances: boolean;
}) {
  return (
    <View style={styles.row}>
      <PoolTile label="Orchard" amount={orchardZec} hideBalances={hideBalances} />
      <PoolTile
        label="Ironwood"
        amount={ironwoodZec}
        hideBalances={hideBalances}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  row: {
    flexDirection: "row",
    gap: spacing.sm,
  },
  tile: {
    flex: 1,
    minWidth: 0,
    paddingHorizontal: spacing.md,
    paddingVertical: spacing.sm + 2,
    borderRadius: radius.lg,
    borderWidth: 1,
    borderColor: "rgba(57, 255, 159, 0.22)",
    backgroundColor: "rgba(0, 0, 0, 0.35)",
    gap: 4,
  },
  tileLit: {
    borderColor: "rgba(57, 255, 159, 0.45)",
    backgroundColor: "rgba(16, 185, 129, 0.12)",
  },
  label: {
    color: "rgba(57, 255, 159, 0.85)",
    fontSize: 10,
    fontWeight: "700",
    letterSpacing: 1.6,
    textTransform: "uppercase",
  },
  value: {
    color: colors.text,
    fontSize: fontSize.sm,
    fontWeight: "800",
    fontVariant: ["tabular-nums"],
  },
});
