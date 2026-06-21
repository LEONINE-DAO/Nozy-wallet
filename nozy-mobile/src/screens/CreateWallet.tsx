import { NativeStackScreenProps } from "@react-navigation/native-stack";
import { useState } from "react";
import {
  KeyboardAvoidingView,
  Platform,
  ScrollView,
  StyleSheet,
  Text,
  View,
} from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "../components/Button";
import { Input } from "../components/Input";
import { useWalletSession } from "../context/WalletSessionContext";
import { api } from "../services/api";
import { colors, fontSize, spacing } from "../theme";
import type { RootStackParamList } from "../types";

type Props = NativeStackScreenProps<RootStackParamList, "CreateWallet">;

export function CreateWalletScreen({ navigation }: Props) {
  const { setPassword } = useWalletSession();
  const [password, setPasswordDraft] = useState("");
  const [confirm, setConfirm] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  async function handleCreate() {
    setError("");
    if (password && password !== confirm) {
      setError("Passwords do not match");
      return;
    }

    setLoading(true);
    try {
      const result = await api.createWallet(password || undefined);
      await setPassword(password);
      navigation.replace("MnemonicBackup", { mnemonic: result.mnemonic });
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to create wallet");
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
          <Text style={styles.title}>Create wallet</Text>
          <Text style={styles.subtitle}>
            Optional password encrypts your wallet on the API server. Leave blank
            for no password.
          </Text>

          <Input
            label="Password (optional)"
            value={password}
            onChangeText={setPasswordDraft}
            secureTextEntry
            placeholder="••••••••"
          />
          <Input
            label="Confirm password"
            value={confirm}
            onChangeText={setConfirm}
            secureTextEntry
            placeholder="••••••••"
            error={error}
          />

          <Button
            label="Generate wallet"
            onPress={() => void handleCreate()}
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
});
