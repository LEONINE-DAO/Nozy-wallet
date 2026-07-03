import * as Clipboard from "expo-clipboard";
import { NativeStackScreenProps } from "@react-navigation/native-stack";
import { useEffect, useState } from "react";
import { ScrollView, StyleSheet, Text, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "../components/Button";
import { Input } from "../components/Input";
import { useWalletSession } from "../context/WalletSessionContext";
import { api } from "../services/api";
import { colors, fontSize, spacing } from "../theme";
import type { RootStackParamList } from "../types";

type Props = NativeStackScreenProps<RootStackParamList, "Keystone">;

export function KeystoneScreen({ navigation }: Props) {
  const { password } = useWalletSession();
  const [status, setStatus] = useState<Awaited<ReturnType<typeof api.keystoneStatus>> | null>(null);
  const [deviceLabel, setDeviceLabel] = useState("My Keystone");
  const [recipient, setRecipient] = useState("");
  const [amount, setAmount] = useState("");
  const [signedInput, setSignedInput] = useState("");
  const [prepareResult, setPrepareResult] = useState("");
  const [urPreview, setUrPreview] = useState("");
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  async function refreshStatus() {
    try {
      const s = await api.keystoneStatus();
      setStatus(s);
      if (s.device_label) setDeviceLabel(s.device_label);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load Keystone status");
    }
  }

  useEffect(() => {
    void refreshStatus();
  }, []);

  async function toggleEnabled() {
    setLoading(true);
    setError("");
    try {
      await api.keystoneEnable(!status?.enabled, deviceLabel);
      await refreshStatus();
      setMessage(status?.enabled ? "Keystone disabled" : "Keystone enabled");
    } catch (e) {
      setError(e instanceof Error ? e.message : "Enable failed");
    } finally {
      setLoading(false);
    }
  }

  async function exportUfvk() {
    setLoading(true);
    setError("");
    try {
      const res = await api.keystoneExportUfvk(password || undefined);
      setMessage("UFVK exported — pair this with your Keystone device");
      setUrPreview(res.ufvk);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Export failed");
    } finally {
      setLoading(false);
    }
  }

  async function prepareSend() {
    const parsed = parseFloat(amount);
    if (!recipient.trim() || Number.isNaN(parsed) || parsed <= 0) {
      setError("Enter recipient and amount");
      return;
    }
    setLoading(true);
    setError("");
    try {
      const res = await api.keystonePrepareSend({
        recipient: recipient.trim(),
        amount: parsed,
        password: password || undefined,
      });
      if (!res.success) {
        setError(res.message ?? "Prepare failed");
        return;
      }
      setPrepareResult(res.summary ?? "Transaction prepared");
      const frames = res.ur_frames?.join("\n") ?? res.pczt_hex ?? "";
      setUrPreview(frames);
      setMessage("Scan UR frames on Keystone, then paste signed data below");
    } catch (e) {
      setError(e instanceof Error ? e.message : "Prepare failed");
    } finally {
      setLoading(false);
    }
  }

  async function completeSend() {
    if (!signedInput.trim()) {
      setError("Paste signed PCZT hex or UR frames from Keystone");
      return;
    }
    setLoading(true);
    setError("");
    try {
      const lines = signedInput
        .split(/\n/)
        .map((l) => l.trim())
        .filter(Boolean);
      const isHex = /^[0-9a-fA-F]+$/.test(lines[0] ?? "");
      const res = await api.keystoneCompleteSend(
        isHex
          ? { pcztHex: lines[0], broadcast: true }
          : { urFrames: lines, broadcast: true },
      );
      setMessage(res.txid ? `Broadcast OK: ${res.txid}` : "Send completed");
      setSignedInput("");
      await refreshStatus();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Complete send failed");
    } finally {
      setLoading(false);
    }
  }

  return (
    <SafeAreaView style={styles.safe} edges={["bottom"]}>
      <ScrollView contentContainerStyle={styles.container}>
        <Text style={styles.subtitle}>
          NozyWallet Keystone signing on Zcash mainnet. Prepare on phone, sign on device, then
          complete here.
        </Text>

        <View style={styles.card}>
          <Text style={styles.label}>Status</Text>
          <Text style={styles.value}>
            {status
              ? `${status.enabled ? "Enabled" : "Disabled"} · ${
                  status.network === "testnet" ? "testnet" : "mainnet"
                } · UFVK ${status.has_ufvk ? "paired" : "not paired"}${status.pending_send ? " · pending send" : ""}`
              : "Loading..."}
          </Text>
          <Input label="Device label" value={deviceLabel} onChangeText={setDeviceLabel} />
          <Button
            label={status?.enabled ? "Disable Keystone" : "Enable Keystone"}
            variant="secondary"
            onPress={() => void toggleEnabled()}
            loading={loading}
          />
          <Button label="Export UFVK for pairing" onPress={() => void exportUfvk()} loading={loading} />
        </View>

        <Text style={styles.section}>Prepare send (unsigned)</Text>
        <Input label="Recipient" value={recipient} onChangeText={setRecipient} autoCapitalize="none" />
        <Input label="Amount (ZEC)" value={amount} onChangeText={setAmount} keyboardType="decimal-pad" />
        <Button label="Prepare for Keystone" onPress={() => void prepareSend()} loading={loading} />

        {prepareResult ? <Text style={styles.ok}>{prepareResult}</Text> : null}
        {urPreview ? (
          <View style={styles.card}>
            <Text style={styles.label}>Data for Keystone (copy / scan)</Text>
            <Text style={styles.mono} selectable numberOfLines={8}>
              {urPreview}
            </Text>
            <Button
              label="Copy"
              variant="ghost"
              onPress={() => void Clipboard.setStringAsync(urPreview)}
            />
          </View>
        ) : null}

        <Text style={styles.section}>Complete send (signed)</Text>
        <Input
          label="Signed PCZT hex or UR frames"
          value={signedInput}
          onChangeText={setSignedInput}
          multiline
          numberOfLines={4}
          style={styles.multiline}
          placeholder="Paste from Keystone after signing"
        />
        <Button label="Broadcast signed tx" onPress={() => void completeSend()} loading={loading} />

        {message ? <Text style={styles.ok}>{message}</Text> : null}
        {error ? <Text style={styles.error}>{error}</Text> : null}
        <Button label="Back" variant="ghost" onPress={() => navigation.goBack()} />
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  container: { padding: spacing.lg, gap: spacing.md },
  subtitle: { color: colors.textMuted, fontSize: fontSize.md, lineHeight: 22 },
  card: {
    backgroundColor: colors.surface,
    borderRadius: 12,
    padding: spacing.md,
    gap: spacing.sm,
    borderWidth: 1,
    borderColor: colors.border,
  },
  section: {
    color: colors.primary,
    fontSize: fontSize.sm,
    fontWeight: "700",
    letterSpacing: 1,
    textTransform: "uppercase",
    marginTop: spacing.sm,
  },
  label: { color: colors.textMuted, fontSize: fontSize.sm },
  value: { color: colors.text, fontSize: fontSize.md },
  mono: { color: colors.text, fontSize: 11, lineHeight: 16 },
  multiline: { minHeight: 100, textAlignVertical: "top" },
  ok: { color: colors.success, fontSize: fontSize.sm },
  error: { color: colors.error, fontSize: fontSize.sm },
});
