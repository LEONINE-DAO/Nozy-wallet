import { ScrollView, StyleSheet, Text } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { SettingsBackButton } from "./SettingsBackButton";
import { colors, fontSize, spacing } from "../../theme";

type Props = { onBack: () => void; title: string; body: string };

function StubSettings({ onBack, title, body }: Props) {
  return (
    <SafeAreaView style={styles.safe} edges={["bottom"]}>
      <ScrollView contentContainerStyle={styles.container}>
        <SettingsBackButton onPress={onBack} />
        <Text style={styles.title}>{title}</Text>
        <Text style={styles.body}>{body}</Text>
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  container: { padding: spacing.lg, gap: spacing.md },
  title: { color: colors.text, fontSize: fontSize.xl, fontWeight: "700" },
  body: { color: colors.textMuted, fontSize: fontSize.sm, lineHeight: 20 },
});

export function AccountSettings({ onBack }: { onBack: () => void }) {
  return (
    <StubSettings
      onBack={onBack}
      title="Account"
      body="Manage account details via the companion API Account flows when connected."
    />
  );
}

export function DisplaySettings({ onBack }: { onBack: () => void }) {
  return (
    <StubSettings
      onBack={onBack}
      title="Display"
      body="Display preferences will live here."
    />
  );
}

export function LightClientSettings({ onBack }: { onBack: () => void }) {
  return (
    <StubSettings
      onBack={onBack}
      title="Light client"
      body="Experimental on-device LWD sync via zeaking-ffi. Compact sync only — Sapling shield uses nozy-ffi (#208)."
    />
  );
}

export function NetworkPrivacySettings({ onBack }: { onBack: () => void }) {
  return (
    <StubSettings
      onBack={onBack}
      title="Network privacy"
      body="Configure privacy network preferences when available."
    />
  );
}

export function WalletsAccountsSettings({ onBack }: { onBack: () => void }) {
  return (
    <StubSettings
      onBack={onBack}
      title="Wallets & accounts"
      body="Multi-wallet management via companion API."
    />
  );
}
