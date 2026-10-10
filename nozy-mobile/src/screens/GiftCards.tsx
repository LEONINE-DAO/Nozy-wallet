import * as Clipboard from "expo-clipboard";
import { useFocusEffect } from "@react-navigation/native";
import { useCallback, useMemo, useState } from "react";
import {
  Image,
  KeyboardAvoidingView,
  Platform,
  ScrollView,
  Share,
  StyleSheet,
  Text,
  View,
} from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "../components/Button";
import { Card } from "../components/Card";
import { Input } from "../components/Input";
import { PageHeader } from "../components/PageHeader";
import { useWalletSession } from "../context/WalletSessionContext";
import { addressQrDataUrl } from "../lib/qrPng";
import { api } from "../services/api";
import {
  buyOnDeviceGiftCard,
  cancelOnDeviceGiftCard,
  listOnDeviceGiftCards,
  redeemOnDeviceGiftCard,
} from "../services/onDeviceGiftCards";
import { colors, fontSize, radius, spacing } from "../theme";
import type { GiftCard, GiftCardCreated, GiftCardStatus } from "../types";

function statusLabel(status: GiftCardStatus): string {
  switch (status) {
    case "unfunded":
      return "Waiting for payment";
    case "funded":
      return "Paid — send the claim code";
    case "redeeming":
      return "Redeeming";
    case "redeemed":
      return "Redeemed";
    case "cancelled":
      return "Cancelled";
  }
}

export function GiftCardsScreen() {
  const { password, backendMode } = useWalletSession();
  const onDevice = backendMode === "on_device";
  const [amount, setAmount] = useState("");
  const [label, setLabel] = useState("");
  const [claimCode, setClaimCode] = useState("");
  const [cards, setCards] = useState<GiftCard[]>([]);
  const [created, setCreated] = useState<GiftCardCreated | null>(null);
  const [action, setAction] = useState<"buy" | "redeem" | "cancel" | null>(null);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [copied, setCopied] = useState(false);

  const claimQr = useMemo(() => {
    if (!created?.claim_code) return null;
    try {
      return addressQrDataUrl(created.claim_code);
    } catch {
      return null;
    }
  }, [created?.claim_code]);

  const refresh = useCallback(async () => {
    if (onDevice) {
      setCards(await listOnDeviceGiftCards());
      return;
    }
    const res = await api.listGiftCards(20);
    setCards(res.gift_cards);
  }, [onDevice]);

  useFocusEffect(
    useCallback(() => {
      void refresh().catch((e) => {
        setError(e instanceof Error ? e.message : "Could not load gift cards");
      });
    }, [refresh]),
  );

  async function buy() {
    setError("");
    setNotice("");
    const parsed = Number(amount);
    if (!Number.isFinite(parsed) || parsed <= 0) {
      setError("Enter an amount in ZEC greater than 0.");
      return;
    }
    setAction("buy");
    try {
      const card = onDevice
        ? await buyOnDeviceGiftCard({
            amountZec: parsed,
            label: label.trim() || undefined,
          })
        : await api.createGiftCard({
            amount_zec: parsed,
            label: label.trim() || undefined,
            password: password || undefined,
          });
      setCreated(card);
      setAmount("");
      setLabel("");
      setCopied(false);
      setNotice("Paid. Copy the claim code now and send it. It is not saved.");
      await refresh();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not buy gift card");
    } finally {
      setAction(null);
    }
  }

  async function redeem() {
    setError("");
    setNotice("");
    if (!claimCode.trim()) {
      setError("Enter the claim code.");
      return;
    }
    setAction("redeem");
    try {
      const card = onDevice
        ? await redeemOnDeviceGiftCard(claimCode.trim())
        : await api.redeemGiftCard({
            claim_code: claimCode.trim(),
            password: password || undefined,
          });
      setClaimCode("");
      const paid = card.payout_zatoshis
        ? (card.payout_zatoshis / 100_000_000).toFixed(8)
        : String(card.amount_zec);
      setNotice(`Redeemed ${paid} ZEC into this wallet.`);
      setCards((prev) => {
        const known = prev.some((row) => row.gift_id === card.gift_id);
        if (!known) return [card, ...prev];
        return prev.map((row) => (row.gift_id === card.gift_id ? card : row));
      });
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not redeem gift card");
    } finally {
      setAction(null);
    }
  }

  async function cancel(id: string) {
    setError("");
    setNotice("");
    setAction("cancel");
    try {
      const next = onDevice
        ? await cancelOnDeviceGiftCard(id)
        : await api.cancelGiftCard(id);
      if (created?.gift_id === id) setCreated(null);
      setCards((prev) => prev.map((row) => (row.gift_id === id ? next : row)));
      setNotice("Gift card cancelled.");
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not cancel gift card");
    } finally {
      setAction(null);
    }
  }

  async function copyCode() {
    if (!created?.claim_code) return;
    await Clipboard.setStringAsync(created.claim_code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  async function shareCode() {
    if (!created?.claim_code) return;
    try {
      await Share.share({ message: created.claim_code });
    } catch {
      setError("Could not open share.");
    }
  }

  const busy = action !== null;

  return (
    <SafeAreaView style={styles.safe} edges={["bottom"]}>
      <KeyboardAvoidingView
        style={styles.flex}
        behavior={Platform.OS === "ios" ? "padding" : undefined}
      >
        <ScrollView
          contentContainerStyle={styles.container}
          keyboardShouldPersistTaps="handled"
        >
          <PageHeader
            title="Gift cards"
            description="Buy with shielded ZEC from this wallet, then send the claim code. The recipient pastes it in their own Nozy and redeems whenever they want. The redeem fee comes out of the card. Anyone with the code can redeem it."
          />

          <Input
            label="Amount (ZEC)"
            value={amount}
            onChangeText={setAmount}
            keyboardType="decimal-pad"
            placeholder="0.05"
            editable={!busy}
          />
          <Input
            label="Label"
            value={label}
            onChangeText={setLabel}
            placeholder="Coffee"
            editable={!busy}
          />
          <Button
            label="Buy with shielded ZEC"
            onPress={() => void buy()}
            loading={action === "buy"}
            disabled={busy}
          />

          {created ? (
            <Card>
              <Text style={styles.warn}>
                Claim code, shown once. Send it to them. It is not saved on
                this phone.
              </Text>
              <Text style={styles.code} selectable>
                {created.claim_code}
              </Text>
              {claimQr ? (
                <View style={styles.qrFrame}>
                  <Image
                    source={{ uri: claimQr }}
                    style={styles.qr}
                    accessibilityLabel="Gift card claim code QR"
                  />
                </View>
              ) : null}
              <View style={styles.row}>
                <Button
                  label={copied ? "Copied" : "Copy code"}
                  variant="secondary"
                  size="sm"
                  onPress={() => void copyCode()}
                  style={styles.rowButton}
                />
                <Button
                  label="Share code"
                  variant="secondary"
                  size="sm"
                  onPress={() => void shareCode()}
                  style={styles.rowButton}
                />
              </View>
            </Card>
          ) : null}

          <Text style={styles.section}>Redeem into this wallet</Text>
          <Input
            label="Claim code"
            value={claimCode}
            onChangeText={setClaimCode}
            autoCapitalize="characters"
            autoCorrect={false}
            placeholder="NZGC-…"
            editable={!busy}
          />
          <Button
            label="Redeem"
            onPress={() => void redeem()}
            loading={action === "redeem"}
            disabled={busy}
          />

          {error ? <Text style={styles.error}>{error}</Text> : null}
          {notice ? <Text style={styles.ok}>{notice}</Text> : null}

          {cards.length > 0 ? (
            <View style={styles.list}>
              {cards.map((card) => (
                <Card key={card.gift_id}>
                  <Text style={styles.cardTitle}>
                    {card.label || card.gift_id} · {card.amount_zec} ZEC
                  </Text>
                  <Text style={styles.meta}>{statusLabel(card.status)}</Text>
                  {card.status === "unfunded" ? (
                    <Button
                      label="Cancel"
                      variant="ghost"
                      size="sm"
                      disabled={busy}
                      loading={action === "cancel"}
                      onPress={() => void cancel(card.gift_id)}
                    />
                  ) : null}
                </Card>
              ))}
            </View>
          ) : null}
        </ScrollView>
      </KeyboardAvoidingView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  flex: { flex: 1 },
  container: { padding: spacing.lg, gap: spacing.md, paddingBottom: spacing.xl },
  body: { color: colors.textMuted, fontSize: fontSize.sm, lineHeight: 20 },
  section: {
    color: colors.text,
    fontSize: fontSize.sm,
    fontWeight: "700",
    marginTop: spacing.sm,
  },
  warn: { color: colors.warn, fontSize: fontSize.sm, lineHeight: 20 },
  code: {
    color: colors.text,
    fontSize: fontSize.xs,
    fontFamily: Platform.OS === "ios" ? "Menlo" : "monospace",
    marginTop: spacing.sm,
  },
  qrFrame: {
    alignSelf: "center",
    marginTop: spacing.md,
    backgroundColor: "#ffffff",
    borderRadius: radius.lg,
    padding: spacing.sm,
  },
  qr: { width: 196, height: 196 },
  row: { flexDirection: "row", gap: spacing.sm, marginTop: spacing.md },
  rowButton: { flex: 1 },
  error: { color: colors.error, fontSize: fontSize.sm },
  ok: { color: colors.success, fontSize: fontSize.sm },
  list: { gap: spacing.sm, marginTop: spacing.sm },
  cardTitle: { color: colors.text, fontSize: fontSize.sm, fontWeight: "600" },
  meta: { color: colors.textFaint, fontSize: fontSize.xs, marginTop: spacing.xs },
});
