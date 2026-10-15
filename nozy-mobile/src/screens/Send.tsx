import { CompositeScreenProps, useFocusEffect } from "@react-navigation/native";
import type { BottomTabScreenProps } from "@react-navigation/bottom-tabs";
import type { NativeStackScreenProps } from "@react-navigation/native-stack";
import { useCallback, useEffect, useState } from "react";
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
  onDeviceSendPayment,
  onDeviceShieldedSnapshot,
  onDeviceSpendableZec,
} from "../services/onDeviceWallet";
import { subscribeWalletSync } from "../services/walletAutoSync";
import { loadHideBalances } from "../lib/displayPrefs";
import { formatUsd, getZecUsdPrice } from "../lib/zecPrice";
import {
  resolveSendRecipient,
  type ZnsRegistration,
} from "../lib/zns";
import {
  feeExceedsSourceMessage,
  formatZats,
  maxSendableZats,
  normalizeZecAmountInput,
  parseZecAmount,
  phoneSendFeeZats,
  zecToZats,
} from "../lib/sendFee";
import { tryParsePaymentInput } from "../lib/zip321";
import { takePendingPaymentUri } from "../lib/pendingPaymentUri";
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

function usdForZec(zec: number, usdPerZec: number | null): string | null {
  if (usdPerZec == null || usdPerZec <= 0 || !Number.isFinite(zec)) return null;
  return formatUsd(zec * usdPerZec);
}

export function SendScreen({ navigation, route }: Props) {
  const { password, backendMode } = useWalletSession();
  const [recipient, setRecipient] = useState(route.params?.recipient ?? "");
  const [amount, setAmount] = useState("");
  const [memo, setMemo] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [result, setResult] = useState("");
  const [resolvedHint, setResolvedHint] = useState("");
  const [spendable, setSpendable] = useState<number | null>(null);
  const [maxNoteZats, setMaxNoteZats] = useState<number | null>(null);
  const [unspentNoteCount, setUnspentNoteCount] = useState(0);
  const [usdPerZec, setUsdPerZec] = useState<number | null>(null);
  const [priceReady, setPriceReady] = useState(false);
  const [hideBalances, setHideBalances] = useState(false);
  const onDevice = backendMode === "on_device";

  const loadSpendable = useCallback(async () => {
    try {
      if (onDevice) {
        const snap = await onDeviceShieldedSnapshot();
        const unspent = snap.notes.filter((note) => !note.spent);
        const largest = unspent.reduce((max, note) => Math.max(max, zecToZats(note.value_zec)), 0);
        setSpendable(snap.unspentZec);
        setMaxNoteZats(largest);
        setUnspentNoteCount(unspent.length);
        return;
      }
      setMaxNoteZats(null);
      setUnspentNoteCount(0);
      const balanceRes = await api.getBalance();
      setSpendable(balanceRes.available_zec ?? balanceRes.balance_zec);
    } catch {
      setSpendable(null);
    }
  }, [onDevice]);

  useFocusEffect(
    useCallback(() => {
      void loadHideBalances().then(setHideBalances);
      void loadSpendable();
      void getZecUsdPrice().then((rate) => {
        setUsdPerZec(rate);
        setPriceReady(true);
      });
    }, [loadSpendable]),
  );

  useEffect(() => {
    return subscribeWalletSync((event) => {
      if (event.type === "done") void loadSpendable();
    });
  }, [loadSpendable]);

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
    const parsed = parseZecAmount(amount);
    if (!recipient.trim() || Number.isNaN(parsed) || parsed <= 0) {
      setError("Enter a valid recipient and amount");
      return;
    }

    const feeZatsNow = phoneSendFeeZats(memo);
    const sourceZatsNow = onDevice
      ? maxNoteZats
      : spendable == null
        ? null
        : zecToZats(spendable);
    const maxSendNow = sourceZatsNow == null ? null : maxSendableZats(sourceZatsNow, feeZatsNow);
    if (maxSendNow != null && zecToZats(parsed) > maxSendNow) {
      setError(feeExceedsSourceMessage(feeZatsNow, maxSendNow, onDevice && unspentNoteCount > 1));
      return;
    }

    setLoading(true);
    try {
      if (backendMode === "on_device") {
        const spendableNow = await onDeviceSpendableZec();
        if (spendableNow <= 0 || parsed > spendableNow) {
          throw new Error("No spendable ZEC yet.");
        }
        const resolved = await resolveRecipientOrThrow(recipient);
        if (resolved !== recipient.trim()) {
          setResolvedHint(`Resolved → ${resolved.slice(0, 16)}…`);
        }
        const txid = await onDeviceSendPayment({
          recipient: resolved,
          amountZec: parsed,
          memo: memo.trim() || undefined,
        });
        setResult(`Sent. TXID ${txid}`);
        setAmount("");
        setMemo("");
        return;
      }
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
      const message = e instanceof Error ? e.message : "Send failed";
      setError(message);
    } finally {
      setLoading(false);
    }
  }

  const feeZats = phoneSendFeeZats(memo);
  const sourceZats = onDevice
    ? maxNoteZats
    : spendable == null
      ? null
      : zecToZats(spendable);
  const maxSendZats = sourceZats == null ? null : maxSendableZats(sourceZats, feeZats);
  const parsedAmount = parseZecAmount(amount);
  const amountZats = zecToZats(parsedAmount);
  const feeBlocksSend = maxSendZats != null && amountZats > maxSendZats;
  const feeWarning = feeBlocksSend
    ? feeExceedsSourceMessage(feeZats, maxSendZats, onDevice && unspentNoteCount > 1)
    : "";
  const sendUsd =
    Number.isFinite(parsedAmount) && parsedAmount > 0
      ? usdForZec(parsedAmount, usdPerZec)
      : null;
  const balanceUsd = spendable != null ? usdForZec(spendable, usdPerZec) : null;
  const balanceText =
    spendable == null ? "—" : hideBalances ? "••••••••" : spendable.toFixed(8);

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

          <View style={styles.balanceBox}>
            <Text style={styles.eyebrow}>Spendable</Text>
            <Text style={styles.balance}>
              {balanceText}
              <Text style={styles.zec}> ZEC</Text>
            </Text>
            {hideBalances || !priceReady ? null : (
              <Text style={styles.fiat}>
                {balanceUsd != null ? `≈ ${balanceUsd}` : "USD price unavailable"}
              </Text>
            )}
          </View>

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
            onChangeText={(t) => {
              setAmount(normalizeZecAmountInput(t));
              setError("");
            }}
            keyboardType="decimal-pad"
            placeholder="0.001"
          />
          {amount.trim() && priceReady ? (
            <Text style={styles.sendUsd}>
              {sendUsd != null ? `≈ ${sendUsd} USD` : "USD price unavailable"}
            </Text>
          ) : null}

          <Input
            label="Memo (optional)"
            value={memo}
            onChangeText={setMemo}
            placeholder="Private memo"
          />

          <Text style={styles.fee}>
            Network fee {formatZats(feeZats)} ZEC (ZIP-317 × 4)
          </Text>
          {maxSendZats != null && !hideBalances ? (
            <Text style={styles.fee}>You can send up to {formatZats(maxSendZats)} ZEC</Text>
          ) : null}
          {feeWarning ? <Text style={styles.error}>{feeWarning}</Text> : null}

          {error ? <Text style={styles.error}>{error}</Text> : null}
          {result ? <Text style={styles.ok}>{result}</Text> : null}

          <Button
            label="Send ZEC"
            onPress={() => void handleSend()}
            loading={loading}
            disabled={feeBlocksSend}
          />
          {backendMode === "on_device" ? null : (
            <Button
              label="Send with Keystone"
              variant="secondary"
              onPress={() => openStackScreen("Keystone")}
            />
          )}
        </ScrollView>
      </KeyboardAvoidingView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  flex: { flex: 1 },
  container: { padding: spacing.lg, gap: spacing.md, paddingBottom: spacing.xl },
  balanceBox: { gap: spacing.xs },
  eyebrow: {
    color: colors.primary,
    fontSize: 10,
    fontWeight: "700",
    textTransform: "uppercase",
    letterSpacing: 2,
  },
  balance: {
    color: colors.primary,
    fontSize: fontSize.xl,
    fontWeight: "800",
  },
  zec: {
    fontSize: fontSize.sm,
    fontWeight: "600",
    color: colors.textFaint,
  },
  fiat: {
    color: "rgba(57, 255, 159, 0.7)",
    fontSize: fontSize.sm,
    fontWeight: "600",
  },
  sendUsd: {
    color: "rgba(57, 255, 159, 0.85)",
    fontSize: fontSize.sm,
    fontWeight: "600",
    marginTop: -spacing.sm,
  },
  fee: { color: colors.textFaint, fontSize: fontSize.sm },
  error: { color: colors.error, fontSize: fontSize.sm },
  ok: { color: colors.success, fontSize: fontSize.sm },
});
