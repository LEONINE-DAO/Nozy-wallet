import * as Clipboard from "expo-clipboard";
import { NativeStackScreenProps } from "@react-navigation/native-stack";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  ActivityIndicator,
  Linking,
  ScrollView,
  StyleSheet,
  Text,
  TextInput,
  View,
} from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "../components/Button";
import { Input } from "../components/Input";
import { useWalletSession } from "../context/WalletSessionContext";
import { api } from "../services/api";
import { colors, fontSize, spacing } from "../theme";
import type { RootStackParamList } from "../types";

type Props = NativeStackScreenProps<RootStackParamList, "Sell">;

export function SellScreen({ navigation }: Props) {
  const { password } = useWalletSession();
  const [role, setRole] = useState("personal");
  const [displayName, setDisplayName] = useState("");
  const [linkedDisplay, setLinkedDisplay] = useState<string | null>(null);
  const [businessAddress, setBusinessAddress] = useState("");
  const [linkName, setLinkName] = useState("");
  const [amount, setAmount] = useState("");
  const [waiting, setWaiting] = useState(false);
  const [paidMsg, setPaidMsg] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);
  const pollRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const baselineRef = useRef<number | null>(null);

  const load = useCallback(async () => {
    setError("");
    try {
      const profile = await api.getProfile(password || undefined);
      setRole(profile.role);
      setDisplayName(profile.business_display_name ?? "");
      setLinkedDisplay(profile.linked_zns_display ?? null);
      setBusinessAddress(profile.business_address ?? profile.receive_address ?? "");
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load profile");
    }
  }, [password]);

  useEffect(() => {
    void load();
    return () => {
      if (pollRef.current) clearInterval(pollRef.current);
    };
  }, [load]);

  async function switchToBusiness() {
    setBusy(true);
    setError("");
    try {
      await api.updateProfile({
        password: password || undefined,
        role: "business",
        business_display_name: displayName || undefined,
      });
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to switch profile");
    } finally {
      setBusy(false);
    }
  }

  async function linkNameAction() {
    setBusy(true);
    setError("");
    try {
      const res = await api.linkZnsName({
        name: linkName.trim(),
        password: password || undefined,
      });
      setLinkedDisplay(res.display);
      setBusinessAddress(res.business_address);
      setLinkName("");
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Link failed");
    } finally {
      setBusy(false);
    }
  }

  async function unlink() {
    setBusy(true);
    try {
      await api.unlinkZnsName();
      setLinkedDisplay(null);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Unlink failed");
    } finally {
      setBusy(false);
    }
  }

  async function startWaiting() {
    setPaidMsg("");
    setError("");
    try {
      const bal = await api.getBalance();
      const base = bal.available_zec ?? bal.balance_zec;
      baselineRef.current = base;
      setWaiting(true);
      if (pollRef.current) clearInterval(pollRef.current);
      pollRef.current = setInterval(async () => {
        try {
          await api.syncWallet(password || undefined).catch(() => undefined);
          const next = await api.getBalance();
          const now = next.available_zec ?? next.balance_zec;
          const start = baselineRef.current;
          if (start != null && now > start + 1e-8) {
            const delta = now - start;
            setPaidMsg(`Received ~${delta.toFixed(8)} ZEC`);
            setWaiting(false);
            if (pollRef.current) clearInterval(pollRef.current);
          }
        } catch {
          /* keep polling */
        }
      }, 8000);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not start wait");
    }
  }

  const identityLabel = linkedDisplay || businessAddress || "Switch to Business and generate address";
  const amt = parseFloat(amount);
  const qrPayload = (() => {
    if (!businessAddress) return linkedDisplay || "";
    if (Number.isFinite(amt) && amt > 0) {
      const trimmed = amt.toFixed(8).replace(/\.?0+$/, "");
      const memo = encodeURIComponent(`sell-${Date.now()}`);
      return `zcash:${businessAddress}?amount=${trimmed}&memo=${memo}`;
    }
    return `zcash:${businessAddress}`;
  })();

  return (
    <SafeAreaView style={styles.safe} edges={["bottom"]}>
      <ScrollView contentContainerStyle={styles.container}>
        <Text style={styles.subtitle}>
          Sell mode — show your Business identity. Claim a name on zcashnames.com pointing at your
          Business UA, then link it here.
        </Text>

        <View style={styles.card}>
          <Text style={styles.label}>Profile</Text>
          <Text style={styles.value}>
            {role === "business" ? "Business (account 1)" : "Personal (account 0)"}
          </Text>
          <Input
            label="Business display name (optional)"
            value={displayName}
            onChangeText={setDisplayName}
            placeholder="Taco stand"
          />
          <Button
            label="Use Business profile"
            onPress={() => void switchToBusiness()}
            loading={busy}
          />
        </View>

        <View style={styles.card}>
          <Text style={styles.label}>Zcash name</Text>
          {linkedDisplay ? (
            <>
              <Text style={styles.heroName}>{linkedDisplay}</Text>
              <Button label="Unlink name" variant="ghost" onPress={() => void unlink()} />
            </>
          ) : (
            <>
              <Text style={styles.hint}>
                Get a name at zcashnames.com, point it at your Business address, then link.
              </Text>
              <Button
                label="Open zcashnames.com"
                variant="secondary"
                onPress={() => void Linking.openURL("https://www.zcashnames.com")}
              />
              <Input
                label="Name to link"
                value={linkName}
                onChangeText={setLinkName}
                autoCapitalize="none"
                placeholder="mystore"
              />
              <Button
                label="Link name"
                onPress={() => void linkNameAction()}
                loading={busy}
              />
            </>
          )}
        </View>

        <View style={styles.card}>
          <Text style={styles.label}>Receive (ZIP-321 URI for QR)</Text>
          <Text style={styles.heroName}>{identityLabel}</Text>
          {businessAddress ? (
            <Text style={styles.mono} selectable>
              {businessAddress}
            </Text>
          ) : null}
          {qrPayload ? (
            <Text style={styles.mono} selectable>
              {qrPayload}
            </Text>
          ) : null}
          <Text style={styles.hint}>
            Paste the URI into a QR generator, or copy for the customer. Native on-screen QR lands with
            the invoice API polish pass.
          </Text>
          <TextInput
            style={styles.amount}
            value={amount}
            onChangeText={setAmount}
            placeholder="Amount ZEC (adds to ZIP-321 URI)"
            keyboardType="decimal-pad"
            placeholderTextColor={colors.textMuted}
          />
          <Button
            label="Copy name or address"
            onPress={async () => {
              const t = linkedDisplay || businessAddress;
              if (t) {
                await Clipboard.setStringAsync(t);
              }
            }}
          />
          <Button
            label="Copy ZIP-321 URI"
            variant="secondary"
            onPress={async () => {
              if (qrPayload) await Clipboard.setStringAsync(qrPayload);
            }}
          />
          {!waiting ? (
            <Button label="Waiting for payment…" onPress={() => void startWaiting()} />
          ) : (
            <View style={styles.waitRow}>
              <ActivityIndicator color={colors.primary} />
              <Text style={styles.hint}>Polling balance after sync…</Text>
            </View>
          )}
          {paidMsg ? <Text style={styles.ok}>{paidMsg}</Text> : null}
        </View>

        {error ? <Text style={styles.error}>{error}</Text> : null}
        <Button label="Back to Use mode" variant="ghost" onPress={() => navigation.goBack()} />
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  container: { padding: spacing.lg, gap: spacing.md },
  subtitle: { color: colors.textMuted, fontSize: fontSize.sm, marginBottom: spacing.sm },
  card: {
    backgroundColor: colors.surface,
    borderRadius: 12,
    padding: spacing.md,
    gap: spacing.sm,
  },
  label: { color: colors.textMuted, fontSize: fontSize.xs, textTransform: "uppercase" },
  value: { color: colors.text, fontSize: fontSize.md, fontWeight: "600" },
  heroName: {
    color: colors.primary,
    fontSize: fontSize.xl,
    fontWeight: "800",
    textAlign: "center",
    marginVertical: spacing.sm,
  },
  mono: { color: colors.text, fontSize: 11, fontFamily: "monospace" },
  hint: { color: colors.textMuted, fontSize: fontSize.sm },
  amount: {
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: 8,
    padding: spacing.sm,
    color: colors.text,
    fontSize: fontSize.lg,
    textAlign: "center",
  },
  waitRow: { flexDirection: "row", alignItems: "center", gap: spacing.sm },
  ok: { color: colors.success, fontSize: fontSize.md, fontWeight: "600" },
  error: { color: colors.error, fontSize: fontSize.sm },
});
