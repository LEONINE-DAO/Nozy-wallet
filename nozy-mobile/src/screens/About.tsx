import { NativeStackScreenProps } from "@react-navigation/native-stack";
import Constants from "expo-constants";
import {
  Linking,
  ScrollView,
  StyleSheet,
  Text,
  View,
} from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "../components/Button";
import { APP_VERSION, LINKS } from "../constants/links";
import { useWalletSession } from "../context/WalletSessionContext";
import { colors, fontSize, spacing } from "../theme";
import type { RootStackParamList } from "../types";

type Props = NativeStackScreenProps<RootStackParamList, "About">;

function openUrl(url: string) {
  void Linking.openURL(url).catch(() => {});
}

export function AboutScreen({ navigation }: Props) {
  const { apiUrl, backendMode } = useWalletSession();
  const expoVersion =
    Constants.expoConfig?.version ?? APP_VERSION;
  const onDevice = backendMode === "on_device";

  return (
    <SafeAreaView style={styles.safe} edges={["bottom"]}>
      <ScrollView contentContainerStyle={styles.container}>
        <Text style={styles.badge}>LEONINE DAO</Text>
        <Text style={styles.title}>NozyWallet Mobile</Text>
        <Text style={styles.version}>Version {expoVersion}</Text>

        <Text style={styles.section}>How this app works</Text>
        <Text style={styles.body}>
          {onDevice
            ? "Keys stay on this phone. Compact sync uses lightwalletd (default lwd.nozywallet.org). Set your own node in Settings if you run one."
            : "This build talks to a nozywallet-api you run. The API syncs with Zebra. Your phone sends requests to the API URL in Settings."}
        </Text>

        {onDevice ? (
          <View style={styles.card}>
            <Text style={styles.cardLabel}>Lightwalletd</Text>
            <Text style={styles.cardValue} selectable>
              lwd.nozywallet.org
            </Text>
          </View>
        ) : (
          <View style={styles.card}>
            <Text style={styles.cardLabel}>Current API URL</Text>
            <Text style={styles.cardValue} selectable>
              {apiUrl || "Not set"}
            </Text>
          </View>
        )}

        <Text style={styles.section}>Privacy & data</Text>
        <Text style={styles.body}>
          Seed, keys, and password stay on this device. Compact sync does not
          send your seed to the server.
        </Text>
        <Text style={styles.body}>
          NozyWallet uses shielded (Orchard) addresses on-chain.
        </Text>

        <Text style={styles.section}>Links</Text>
        <Button
          label="Privacy policy"
          variant="secondary"
          onPress={() => openUrl(LINKS.privacyPolicy)}
        />
        <Button
          label="Documentation"
          variant="secondary"
          onPress={() => openUrl(LINKS.documentation)}
        />
        <Button
          label="GitHub"
          variant="secondary"
          onPress={() => openUrl(LINKS.github)}
        />
        <Button
          label="Contact support"
          variant="secondary"
          onPress={() => openUrl(LINKS.supportEmail)}
        />

        <Text style={styles.footer}>
          © LEONINE DAO · Private Zcash wallet
        </Text>

        <Button label="Back" variant="ghost" onPress={() => navigation.goBack()} />
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  container: { padding: spacing.lg, gap: spacing.md },
  badge: {
    color: colors.primary,
    fontSize: fontSize.sm,
    fontWeight: "700",
    letterSpacing: 2,
  },
  title: {
    color: colors.text,
    fontSize: fontSize.xl,
    fontWeight: "800",
  },
  version: { color: colors.textMuted, fontSize: fontSize.sm },
  section: {
    color: colors.primary,
    fontSize: fontSize.sm,
    fontWeight: "700",
    letterSpacing: 1,
    textTransform: "uppercase",
    marginTop: spacing.md,
  },
  body: {
    color: colors.text,
    fontSize: fontSize.md,
    lineHeight: 24,
  },
  card: {
    backgroundColor: colors.surface,
    borderRadius: 12,
    padding: spacing.md,
    borderWidth: 1,
    borderColor: colors.border,
    gap: spacing.xs,
  },
  cardLabel: { color: colors.textMuted, fontSize: fontSize.sm },
  cardValue: { color: colors.text, fontSize: fontSize.sm },
  footer: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
    textAlign: "center",
    marginTop: spacing.md,
  },
});
