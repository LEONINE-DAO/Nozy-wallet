import { useCallback, useEffect, useState } from "react";
import { ScrollView, StyleSheet, Text, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { NativeStackScreenProps } from "@react-navigation/native-stack";
import { Button } from "../components/Button";
import { Card } from "../components/Card";
import { Input } from "../components/Input";
import { PageHeader } from "../components/PageHeader";
import { useWalletSession } from "../context/WalletSessionContext";
import { api } from "../services/api";
import { colors, fontSize, spacing } from "../theme";
import type { IronwoodStatusResponse, RootStackParamList } from "../types";

type Props = NativeStackScreenProps<RootStackParamList, "Ironwood">;

const ZAT = 100_000_000;

function formatZec(zat: number): string {
  return `${(zat / ZAT).toLocaleString(undefined, {
    minimumFractionDigits: 2,
    maximumFractionDigits: 8,
  })} ZEC`;
}

function formatOptZec(value: number | null | undefined): string {
  if (value == null) return "Unavailable";
  return `${value.toLocaleString(undefined, {
    minimumFractionDigits: 2,
    maximumFractionDigits: 8,
  })} ZEC`;
}

export function IronwoodScreen({}: Props) {
  const { password } = useWalletSession();
  const [status, setStatus] = useState<IronwoodStatusResponse | null>(null);
  const [actionPassword, setActionPassword] = useState(password);
  const [busy, setBusy] = useState<"plan" | "split" | "migrate" | "broadcast" | null>(null);
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");

  const load = useCallback(async () => {
    try {
      setStatus(await api.getIronwoodStatus());
      setError("");
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load Ironwood status");
    }
  }, []);

  useEffect(() => {
    void load();
    const id = setInterval(() => void load(), 60_000);
    return () => clearInterval(id);
  }, [load]);

  async function run(
    kind: "plan" | "split" | "migrate" | "broadcast",
    fn: () => Promise<{ message: string }>,
  ) {
    setBusy(kind);
    setError("");
    setMessage("");
    try {
      const res = await fn();
      setMessage(res.message);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : `${kind} failed`);
    } finally {
      setBusy(null);
    }
  }

  const tone =
    status?.ironwood_active && status.ironwood_rpc_detected
      ? "Ironwood detected"
      : status?.ironwood_rpc_detected
        ? "RPC ready"
        : "Mainnet pending";

  return (
    <SafeAreaView style={styles.safe} edges={["bottom"]}>
      <ScrollView contentContainerStyle={styles.container}>
        <PageHeader
          title="Ironwood"
          description="NU6.3 readiness and Orchard → Ironwood migration (Plan → Split → Migrate → Broadcast)."
        />

        <Card>
          <Text style={styles.badge}>{tone}</Text>
          <Text style={styles.meta}>
            Network {status?.network ?? "—"} · Tip{" "}
            {status?.chain_tip?.toLocaleString() ?? "—"} · Activation{" "}
            {status?.activation_height?.toLocaleString() ?? "pending"}
          </Text>
          <Text style={styles.meta}>Target {status?.activation_target_date ?? "—"}</Text>
        </Card>

        {status?.activation_notice ? (
          <Card>
            <Text style={styles.cardTitle}>Activation notice</Text>
            <Text style={styles.body}>{status.activation_notice}</Text>
            {(status.migration_privacy_warnings ?? []).map((warning) => (
              <Text key={warning} style={styles.blocker}>
                • {warning}
              </Text>
            ))}
          </Card>
        ) : null}

        <View style={styles.stats}>
          <Card style={styles.stat}>
            <Text style={styles.statLabel}>Orchard chain</Text>
            <Text style={styles.statValue}>
              {formatOptZec(status?.orchard_chain_value_zec)}
            </Text>
          </Card>
          <Card style={styles.stat}>
            <Text style={styles.statLabel}>Ironwood chain</Text>
            <Text style={styles.statValue}>
              {formatOptZec(status?.ironwood_chain_value_zec)}
            </Text>
          </Card>
          <Card style={styles.stat}>
            <Text style={styles.statLabel}>Wallet Orchard</Text>
            <Text style={styles.statValue}>
              {formatZec(status?.orchard_wallet_zat ?? 0)}
            </Text>
          </Card>
          <Card style={styles.stat}>
            <Text style={styles.statLabel}>Wallet Ironwood</Text>
            <Text style={styles.statValue}>
              {formatZec(status?.ironwood_wallet_zat ?? 0)}
            </Text>
          </Card>
        </View>

        {status?.blockers?.length ? (
          <Card>
            <Text style={styles.cardTitle}>Blockers</Text>
            {status.blockers.map((b) => (
              <Text key={b} style={styles.blocker}>
                • {b}
              </Text>
            ))}
          </Card>
        ) : null}

        <Card>
          <Text style={styles.cardTitle}>Migration actions</Text>
          <Text style={styles.body}>
            Notes to migrate: {status?.migration_note_count ?? 0} (
            {formatZec(status?.migration_zat ?? 0)}). Split required:{" "}
            {status?.zip318_note_split_required ? "yes" : "no"}.
          </Text>
          <Input
            label="Wallet password (if encrypted)"
            value={actionPassword}
            onChangeText={setActionPassword}
            secureTextEntry
          />
          <View style={styles.actions}>
            <Button
              label="Plan"
              size="sm"
              variant="secondary"
              loading={busy === "plan"}
              disabled={!!busy}
              onPress={() => void run("plan", () => api.ironwoodPlan())}
            />
            <Button
              label="Split"
              size="sm"
              variant="secondary"
              loading={busy === "split"}
              disabled={!!busy}
              onPress={() =>
                void run("split", () =>
                  api.ironwoodSplit({ password: actionPassword || undefined }),
                )
              }
            />
            <Button
              label="Migrate"
              size="sm"
              variant="secondary"
              loading={busy === "migrate"}
              disabled={!!busy}
              onPress={() =>
                void run("migrate", () => api.ironwoodMigrate(actionPassword || undefined))
              }
            />
            <Button
              label="Broadcast"
              size="sm"
              loading={busy === "broadcast"}
              disabled={!!busy}
              onPress={() =>
                void run("broadcast", () =>
                  api.ironwoodBroadcast({ password: actionPassword || undefined }),
                )
              }
            />
          </View>
          <Button
            label="Refresh status"
            size="sm"
            variant="ghost"
            onPress={() => void load()}
          />
        </Card>

        {message ? <Text style={styles.ok}>{message}</Text> : null}
        {error ? <Text style={styles.error}>{error}</Text> : null}
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  container: { padding: spacing.lg, gap: spacing.md, paddingBottom: spacing.xl },
  badge: {
    color: colors.primary,
    fontWeight: "800",
    textTransform: "uppercase",
    letterSpacing: 1,
    fontSize: fontSize.xs,
  },
  meta: { color: colors.textMuted, fontSize: fontSize.sm, marginTop: 4 },
  stats: { flexDirection: "row", flexWrap: "wrap", gap: spacing.sm },
  stat: { width: "48%", flexGrow: 1 },
  statLabel: {
    color: colors.textMuted,
    fontSize: fontSize.xs,
    fontWeight: "700",
    textTransform: "uppercase",
  },
  statValue: { color: colors.text, fontWeight: "700", marginTop: 6 },
  cardTitle: { color: colors.text, fontWeight: "700", marginBottom: spacing.sm },
  body: { color: colors.textMuted, fontSize: fontSize.sm, marginBottom: spacing.sm },
  blocker: { color: colors.warn, fontSize: fontSize.sm, marginBottom: 4 },
  actions: { flexDirection: "row", flexWrap: "wrap", gap: spacing.sm, marginBottom: spacing.sm },
  ok: { color: colors.success, fontSize: fontSize.sm },
  error: { color: colors.error, fontSize: fontSize.sm },
});
