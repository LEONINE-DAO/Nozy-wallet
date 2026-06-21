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
  const { apiUrl } = useWalletSession();
  const expoVersion =
    Constants.expoConfig?.version ?? APP_VERSION;

  return (
    <SafeAreaView style={styles.safe} edges={["bottom"]}>
      <ScrollView contentContainerStyle={styles.container}>
        <Text style={styles.badge}>LEONINE DAO</Text>
        <Text style={styles.title}>NozyWallet Mobile</Text>
        <Text style={styles.version}>Version {expoVersion}</Text>

        <Text style={styles.section}>How this app works</Text>
        <Text style={styles.body}>
          NozyWallet Mobile is a companion app. It connects to a NozyWallet API
          server you configure — on your PC at home or on a hosted VPS — which
          syncs with a Zebra node and holds wallet scan data.
        </Text>
        <Text style={styles.body}>
          Your phone sends requests to the API URL in Settings. The API talks to
          Zcash through Zebra. First sync can take several minutes.
        </Text>

        <View style={styles.card}>
          <Text style={styles.cardLabel}>Current API URL</Text>
          <Text style={styles.cardValue} selectable>
            {apiUrl || "Not set"}
          </Text>
        </View>

        <Text style={styles.section}>Privacy & data</Text>
        <Text style={styles.body}>
          Wallet passwords and API keys are stored on this device only
          (AsyncStorage). Seed phrases shown during setup are handled by the API
          server according to your deployment — self-hosted or VPS.
        </Text>
        <Text style={styles.body}>
          If you use a public hosted API, the operator of that server can access
          wallet data stored there. Use your own server when you need full
          control.
        </Text>
        <Text style={styles.body}>
          NozyWallet enforces shielded (Orchard) privacy on-chain. Network
          traffic to your API uses HTTPS when configured on a public host.
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
