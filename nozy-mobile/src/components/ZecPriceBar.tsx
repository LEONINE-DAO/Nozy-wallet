import { Image, StyleSheet, Text, View } from "react-native";
import { formatUsd } from "../lib/zecPrice";
import { colors, fontSize, radius, spacing } from "../theme";

const ZEC_LOGO = require("../../assets/zec.png");

export function ZecPriceBar({
  zecAmount,
  usdPerZec,
  hideBalances,
}: {
  zecAmount: number;
  usdPerZec: number | null;
  hideBalances: boolean;
}) {
  const unit =
    usdPerZec != null && usdPerZec > 0
      ? `1 ZEC ≈ ${formatUsd(usdPerZec)}`
      : "ZEC · price unavailable";
  const walletUsd =
    usdPerZec != null && usdPerZec > 0
      ? `≈ ${formatUsd(zecAmount * usdPerZec)}`
      : "Price unavailable";

  return (
    <View style={styles.bar}>
      <View style={styles.left}>
        <Image
          source={ZEC_LOGO}
          style={styles.logo}
          accessibilityLabel="Zcash logo"
        />
        <View style={styles.copy}>
          <Text style={styles.name}>Zcash</Text>
          <Text style={styles.unit} numberOfLines={1}>
            {unit}
          </Text>
        </View>
      </View>
      <View style={styles.right}>
        <Text style={styles.amount}>
          {hideBalances ? "••••••" : `${zecAmount.toFixed(8)} ZEC`}
        </Text>
        <Text style={styles.usd}>
          {hideBalances ? "••••" : walletUsd}
        </Text>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  bar: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
    gap: spacing.md,
    padding: spacing.md,
    borderRadius: radius.xl,
    borderWidth: 1,
    borderColor: "rgba(57, 255, 159, 0.22)",
    backgroundColor: "rgba(0, 0, 0, 0.35)",
  },
  left: {
    flexDirection: "row",
    alignItems: "center",
    gap: spacing.md,
    flex: 1,
    minWidth: 0,
  },
  logo: {
    width: 48,
    height: 48,
    borderRadius: 24,
    borderWidth: 1,
    borderColor: "rgba(244, 183, 40, 0.35)",
  },
  copy: { flex: 1, minWidth: 0, gap: 2 },
  name: {
    color: colors.text,
    fontSize: fontSize.md,
    fontWeight: "800",
    letterSpacing: 0.4,
    textTransform: "uppercase",
  },
  unit: {
    color: "rgba(57, 255, 159, 0.72)",
    fontSize: fontSize.sm,
    fontWeight: "600",
  },
  right: { alignItems: "flex-end", gap: 2 },
  amount: {
    color: colors.text,
    fontSize: fontSize.sm,
    fontWeight: "700",
    fontVariant: ["tabular-nums"],
  },
  usd: {
    color: "rgba(57, 255, 159, 0.72)",
    fontSize: fontSize.sm,
    fontWeight: "700",
    fontVariant: ["tabular-nums"],
  },
});
