import { NativeStackScreenProps } from "@react-navigation/native-stack";
import { useEffect, useState } from "react";
import {
  KeyboardAvoidingView,
  Platform,
  Pressable,
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

type Props = NativeStackScreenProps<RootStackParamList, "Send">;

export function SendScreen({ navigation, route }: Props) {
  const { password } = useWalletSession();
  const [recipient, setRecipient] = useState(route.params?.recipient ?? "");
  const [amount, setAmount] = useState("");
  const [memo, setMemo] = useState("");
  const [priority, setPriority] = useState(false);
  const [feeZec, setFeeZec] = useState<number | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [result, setResult] = useState("");

  useEffect(() => {
    void api.estimateFee(priority).then((f) => setFeeZec(f.fee_zec)).catch(() => {});
  }, [priority]);

  useEffect(() => {
    if (route.params?.recipient) {
      setRecipient(route.params.recipient);
    }
  }, [route.params?.recipient]);

  async function handleSend() {
    setError("");
    setResult("");
    const parsed = parseFloat(amount);
    if (!recipient.trim() || Number.isNaN(parsed) || parsed <= 0) {
      setError("Enter a valid recipient and amount");
      return;
    }

    setLoading(true);
    try {
      const res = await api.sendTransaction({
        recipient: recipient.trim(),
        amount: parsed,
        memo: memo.trim() || undefined,
        priority,
        password: password || undefined,
      });
      if (res.success) {
        setResult(res.message);
        setAmount("");
        setMemo("");
      } else {
        setError(res.message);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : "Send failed");
    } finally {
      setLoading(false);
    }
  }

  return (
    <SafeAreaView style={styles.safe} edges={["bottom"]}>
      <KeyboardAvoidingView
        style={styles.flex}
        behavior={Platform.OS === "ios" ? "padding" : undefined}
      >
        <ScrollView contentContainerStyle={styles.container}>
          <Text style={styles.subtitle}>
            Send shielded ZEC (u1 addresses). Transparent t1 addresses are not supported.
          </Text>

          <Input
            label="Recipient address"
            value={recipient}
            onChangeText={setRecipient}
            autoCapitalize="none"
            placeholder="u1..."
          />
          <Button
            label="Pick from address book"
            variant="ghost"
            onPress={() => navigation.navigate("AddressBook")}
          />

          <Input
            label="Amount (ZEC)"
            value={amount}
            onChangeText={setAmount}
            keyboardType="decimal-pad"
            placeholder="0.001"
          />

          <Input
            label="Memo (optional)"
            value={memo}
            onChangeText={setMemo}
            placeholder="Private memo"
          />

          <Pressable
            style={styles.toggleRow}
            onPress={() => setPriority((p) => !p)}
          >
            <View style={[styles.checkbox, priority && styles.checkboxOn]} />
            <Text style={styles.toggleLabel}>Priority fee (faster confirmation)</Text>
          </Pressable>

          {feeZec !== null ? (
            <Text style={styles.fee}>Estimated fee: {feeZec.toFixed(8)} ZEC</Text>
          ) : null}

          {error ? <Text style={styles.error}>{error}</Text> : null}
          {result ? <Text style={styles.ok}>{result}</Text> : null}

          <Button
            label="Send ZEC"
            onPress={() => void handleSend()}
            loading={loading}
          />
          <Button
            label="Send with Keystone"
            variant="secondary"
            onPress={() => navigation.navigate("Keystone")}
          />
        </ScrollView>
      </KeyboardAvoidingView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  flex: { flex: 1 },
  container: { padding: spacing.lg, gap: spacing.md },
  subtitle: { color: colors.textMuted, fontSize: fontSize.md, lineHeight: 22 },
  toggleRow: { flexDirection: "row", alignItems: "center", gap: spacing.md },
  checkbox: {
    width: 22,
    height: 22,
    borderRadius: 6,
    borderWidth: 2,
    borderColor: colors.border,
  },
  checkboxOn: { backgroundColor: colors.primary, borderColor: colors.primary },
  toggleLabel: { color: colors.text, fontSize: fontSize.md },
  fee: { color: colors.textMuted, fontSize: fontSize.sm },
  error: { color: colors.error, fontSize: fontSize.sm },
  ok: { color: colors.success, fontSize: fontSize.sm },
});
