import { CompositeScreenProps } from "@react-navigation/native";
import type { BottomTabScreenProps } from "@react-navigation/bottom-tabs";
import type { NativeStackScreenProps } from "@react-navigation/native-stack";
import { useEffect, useState } from "react";
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
import { PageHeader } from "../components/PageHeader";
import { useWalletSession } from "../context/WalletSessionContext";
import { api } from "../services/api";
import {
  resolveSendRecipient,
  type ZnsRegistration,
} from "../lib/zns";
import { colors, fontSize, spacing } from "../theme";
import type { MainTabParamList, RootStackParamList } from "../types";

type Props = CompositeScreenProps<
  BottomTabScreenProps<MainTabParamList, "Send">,
  NativeStackScreenProps<RootStackParamList>
>;

async function resolveRecipientOrThrow(raw: string): Promise<string> {
  const result = await resolveSendRecipient(raw, async (name, network) => {
    const res = await api.resolveZnsName({ name, network });
    if (!res.found || !res.registration?.address) return null;
    return res.registration as ZnsRegistration;
  });
  if (result.kind === "address") return result.address;
  if (result.kind === "name") return result.registration.address;
  if (result.kind === "unresolved") {
    throw new Error(`No Zcash name registered for “${result.name}”.`);
  }
  throw new Error(result.message);
}

export function SendScreen({ navigation, route }: Props) {
  const { password } = useWalletSession();
  const [recipient, setRecipient] = useState(route.params?.recipient ?? "");
  const [amount, setAmount] = useState("");
  const [memo, setMemo] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [result, setResult] = useState("");
  const [resolvedHint, setResolvedHint] = useState("");

  useEffect(() => {
    if (route.params?.recipient) {
      setRecipient(route.params.recipient);
    }
  }, [route.params?.recipient]);

  function openStackScreen(name: "AddressBook" | "Keystone") {
    navigation.getParent()?.navigate(name);
  }

  async function handleSend() {
    setError("");
    setResult("");
    setResolvedHint("");
    const parsed = parseFloat(amount);
    if (!recipient.trim() || Number.isNaN(parsed) || parsed <= 0) {
      setError("Enter a valid recipient and amount");
      return;
    }

    setLoading(true);
    try {
      const resolved = await resolveRecipientOrThrow(recipient);
      if (resolved !== recipient.trim()) {
        setResolvedHint(`Resolved → ${resolved.slice(0, 16)}…`);
      }
      const res = await api.sendTransaction({
        recipient: resolved,
        amount: parsed,
        memo: memo.trim() || undefined,
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
    <SafeAreaView style={styles.safe} edges={["top"]}>
      <KeyboardAvoidingView
        style={styles.flex}
        behavior={Platform.OS === "ios" ? "padding" : undefined}
      >
        <ScrollView contentContainerStyle={styles.container}>
          <PageHeader
            title="Send"
            description="Shielded ZEC to a u1… address or Zcash name."
          />

          <Input
            label="Recipient"
            value={recipient}
            onChangeText={(t) => {
              setRecipient(t);
              setResolvedHint("");
            }}
            autoCapitalize="none"
            placeholder="u1… or zoie"
          />
          {resolvedHint ? <Text style={styles.ok}>{resolvedHint}</Text> : null}
          <Button
            label="Pick from address book"
            variant="ghost"
            size="sm"
            onPress={() => openStackScreen("AddressBook")}
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

          <Text style={styles.fee}>Network fee (ZIP-317 × 4)</Text>

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
            onPress={() => openStackScreen("Keystone")}
          />
        </ScrollView>
      </KeyboardAvoidingView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  flex: { flex: 1 },
  container: { padding: spacing.lg, gap: spacing.md, paddingBottom: spacing.xl },
  fee: { color: colors.textFaint, fontSize: fontSize.sm },
  error: { color: colors.error, fontSize: fontSize.sm },
  ok: { color: colors.success, fontSize: fontSize.sm },
});
