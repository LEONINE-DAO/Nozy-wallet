import { NativeStackScreenProps } from "@react-navigation/native-stack";
import { useState } from "react";
import { StyleSheet, Text, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "../components/Button";
import { Input } from "../components/Input";
import { useWalletSession } from "../context/WalletSessionContext";
import { api } from "../services/api";
import { colors, fontSize, spacing } from "../theme";
import type { RootStackParamList } from "../types";

type Props = NativeStackScreenProps<RootStackParamList, "Unlock">;

export function UnlockScreen({ navigation }: Props) {
  const { setPassword } = useWalletSession();
  const [password, setPasswordDraft] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  async function handleUnlock() {
    setError("");
    setLoading(true);
    try {
      await api.unlockWallet(password);
      await setPassword(password);
      navigation.replace("Dashboard");
    } catch (e) {
      setError(e instanceof Error ? e.message : "Unlock failed");
    } finally {
      setLoading(false);
    }
  }

  return (
    <SafeAreaView style={styles.safe}>
      <View style={styles.container}>
        <Text style={styles.title}>Unlock wallet</Text>
        <Text style={styles.subtitle}>Enter your wallet password to continue.</Text>

        <Input
          label="Password"
          value={password}
          onChangeText={setPasswordDraft}
          secureTextEntry
          placeholder="••••••••"
          error={error}
        />

        <Button
          label="Unlock"
          onPress={() => void handleUnlock()}
          loading={loading}
        />
      </View>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: {
    flex: 1,
    backgroundColor: colors.background,
  },
  container: {
    flex: 1,
    padding: spacing.lg,
    justifyContent: "center",
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
    marginBottom: spacing.sm,
  },
});
