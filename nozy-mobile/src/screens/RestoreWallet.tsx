import { NativeStackScreenProps } from "@react-navigation/native-stack";
import { useState } from "react";
import {
  KeyboardAvoidingView,
  Platform,
  ScrollView,
  StyleSheet,
  Text,
} from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "../components/Button";
import { Input } from "../components/Input";
import { useWalletSession } from "../context/WalletSessionContext";
import { api } from "../services/api";
import { colors, fontSize, spacing } from "../theme";
import type { RootStackParamList } from "../types";

type Props = NativeStackScreenProps<RootStackParamList, "RestoreWallet">;

export function RestoreWalletScreen({ navigation }: Props) {
  const { setPassword } = useWalletSession();
  const [mnemonic, setMnemonic] = useState("");
  const [password, setPasswordDraft] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  async function handleRestore() {
    setError("");
    setLoading(true);
    try {
      await api.restoreWallet(mnemonic.trim(), password);
      await setPassword(password);
      navigation.replace("Dashboard");
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to restore wallet");
    } finally {
      setLoading(false);
    }
  }

  return (
    <SafeAreaView style={styles.safe}>
      <KeyboardAvoidingView
        style={styles.flex}
        behavior={Platform.OS === "ios" ? "padding" : undefined}
      >
        <ScrollView contentContainerStyle={styles.container}>
          <Text style={styles.title}>Restore wallet</Text>
          <Text style={styles.subtitle}>
            Enter your 12 or 24 word seed phrase and wallet password.
          </Text>

          <Input
            label="Seed phrase"
            value={mnemonic}
            onChangeText={setMnemonic}
            multiline
            numberOfLines={4}
            placeholder="word1 word2 word3 ..."
            autoCapitalize="none"
            style={styles.multiline}
          />
          <Input
            label="Wallet password"
            value={password}
            onChangeText={setPasswordDraft}
            secureTextEntry
            placeholder="••••••••"
            error={error}
          />

          <Button
            label="Restore wallet"
            onPress={() => void handleRestore()}
            loading={loading}
          />
          <Button
            label="Back"
            variant="ghost"
            onPress={() => navigation.goBack()}
          />
        </ScrollView>
      </KeyboardAvoidingView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: {
    flex: 1,
    backgroundColor: colors.background,
  },
  flex: {
    flex: 1,
  },
  container: {
    padding: spacing.lg,
    gap: spacing.md,
  },
  title: {
    color: colors.text,
    fontSize: fontSize.xl,
    fontWeight: "800",
  },
  subtitle: {
    color: colors.textMuted,
    fontSize: fontSize.md,
    lineHeight: 22,
    marginBottom: spacing.sm,
  },
  multiline: {
    minHeight: 120,
    textAlignVertical: "top",
  },
});
