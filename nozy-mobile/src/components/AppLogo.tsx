import { Image, StyleSheet, View } from "react-native";
import { colors, radius, spacing } from "../theme";

type Props = {
  /** Large centered logo on Welcome */
  variant?: "welcome" | "header";
};

const LOGO = require("../../assets/logo.png");

export function AppLogo({ variant = "welcome" }: Props) {
  if (variant === "header") {
    return (
      <View style={styles.headerWrap}>
        <Image
          source={LOGO}
          style={styles.header}
          resizeMode="contain"
          accessibilityLabel="NozyWallet"
        />
      </View>
    );
  }

  return (
    <Image
      source={LOGO}
      style={styles.welcome}
      resizeMode="contain"
      accessibilityLabel="NozyWallet"
    />
  );
}

const styles = StyleSheet.create({
  welcome: {
    width: 300,
    height: 200,
    marginTop: spacing.md,
    marginBottom: spacing.sm,
  },
  headerWrap: {
    alignSelf: "stretch",
    alignItems: "center",
    borderRadius: radius.lg,
    overflow: "hidden",
    marginBottom: spacing.sm,
    borderWidth: 1,
    borderColor: colors.border,
    backgroundColor: colors.surface,
  },
  header: {
    width: "100%",
    height: 120,
  },
});
