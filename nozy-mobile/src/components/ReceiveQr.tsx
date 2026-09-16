import { useMemo } from "react";
import { Image, StyleSheet, Text, View } from "react-native";
import { addressQrDataUrl } from "../lib/qrPng";
import { colors, radius, spacing } from "../theme";

export function ReceiveQr({ address }: { address: string }) {
  const uri = useMemo(() => {
    try {
      return addressQrDataUrl(address);
    } catch {
      return null;
    }
  }, [address]);

  if (!uri) {
    return (
      <Text style={styles.fallback}>Could not draw a QR for this address.</Text>
    );
  }

  return (
    <View style={styles.frame}>
      <Image
        source={{ uri }}
        style={styles.image}
        accessibilityLabel="Wallet receive address QR code"
      />
    </View>
  );
}

const styles = StyleSheet.create({
  frame: {
    alignSelf: "center",
    marginTop: spacing.md,
    backgroundColor: "#ffffff",
    borderRadius: radius.lg,
    padding: spacing.sm,
    borderWidth: 1,
    borderColor: colors.platinumLine,
  },
  image: {
    width: 196,
    height: 196,
  },
  fallback: {
    marginTop: spacing.md,
    color: colors.textFaint,
    fontSize: 12,
    textAlign: "center",
  },
});
