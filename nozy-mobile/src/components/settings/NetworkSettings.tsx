import { useEffect, useState } from "react";
import { Alert, ScrollView, StyleSheet, Text, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "../Button";
import { Card } from "../Card";
import { Input } from "../Input";
import { PublicNodeRiskModal } from "../PublicNodeRiskModal";
import { Select } from "../Select";
import { SettingsBackButton } from "./SettingsBackButton";
import { api } from "../../services/api";
import {
  type NodeConnectionMode,
  defaultLocalZebraUrl,
  defaultPublicZebraUrl,
  inferNodeConnectionMode,
  isLocalZebraUrl,
} from "../../lib/connectionPresets";
import { colors, fontSize, spacing } from "../../theme";

type Props = { onBack: () => void };

export function NetworkSettings({ onBack }: Props) {
  const [zebraUrl, setZebraUrl] = useState("");
  const [initialZebraUrl, setInitialZebraUrl] = useState("");
  const [network, setNetwork] = useState("");
  const [nodeMode, setNodeMode] = useState<NodeConnectionMode>("local");
  const [status, setStatus] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const [riskOpen, setRiskOpen] = useState(false);
  const [pendingUrl, setPendingUrl] = useState<string | null>(null);

  useEffect(() => {
    void api
      .getConfig()
      .then((c) => {
        setZebraUrl(c.zebra_url);
        setInitialZebraUrl(c.zebra_url);
        setNetwork(c.network);
        setNodeMode(inferNodeConnectionMode(c.zebra_url));
      })
      .catch((e) => {
        setError(
          e instanceof Error ? e.message : "Could not load network config",
        );
      });
  }, []);

  function applyNodeMode(next: NodeConnectionMode) {
    setNodeMode(next);
    if (next === "public") {
      const pub = defaultPublicZebraUrl().trim();
      if (pub) setZebraUrl(pub);
    } else {
      setZebraUrl(defaultLocalZebraUrl());
    }
  }

  function requestNodeMode(next: NodeConnectionMode) {
    if (next === nodeMode) return;
    if (next === "public") {
      const pub = defaultPublicZebraUrl().trim();
      setPendingUrl(pub || zebraUrl || "");
      setRiskOpen(true);
      return;
    }
    applyNodeMode(next);
  }

  async function persistUrl(url: string) {
    const trimmed = url.trim();
    if (!trimmed) {
      const msg = "Enter a Zebra RPC URL before saving.";
      setError(msg);
      Alert.alert("Cannot save", msg);
      return;
    }
    if (!/^https?:\/\//i.test(trimmed)) {
      const msg = "URL must start with http:// or https://";
      setError(msg);
      Alert.alert("Cannot save", msg);
      return;
    }

    setLoading(true);
    setError("");
    setStatus("");
    try {
      await Promise.race([
        api.setZebraUrl(trimmed),
        new Promise<never>((_, reject) =>
          setTimeout(
            () => reject(new Error("Save timed out talking to the API.")),
            15000,
          ),
        ),
      ]);
      setZebraUrl(trimmed);
      setInitialZebraUrl(trimmed);
      setNodeMode(inferNodeConnectionMode(trimmed));
      const ok = `Zebra URL saved on the API:\n${trimmed}`;
      setStatus(ok);
      Alert.alert("Saved", ok);
    } catch (e) {
      const msg = e instanceof Error ? e.message : "Save failed";
      setError(msg);
      Alert.alert("Save failed", msg);
    } finally {
      setLoading(false);
    }
  }

  async function save() {
    const trimmed = zebraUrl.trim();
    const wasLocal = isLocalZebraUrl(initialZebraUrl);
    const goingPublic = wasLocal && !isLocalZebraUrl(trimmed);

    if (goingPublic) {
      setPendingUrl(trimmed);
      setRiskOpen(true);
      return;
    }

    await persistUrl(trimmed);
  }

  function confirmPublic() {
    const url = pendingUrl;
    setPendingUrl(null);
    setRiskOpen(false);
    if (url) {
      setZebraUrl(url);
      setNodeMode(inferNodeConnectionMode(url));
      void persistUrl(url);
    } else {
      applyNodeMode("public");
    }
  }

  function cancelPublic() {
    setPendingUrl(null);
    setRiskOpen(false);
  }

  async function test() {
    setLoading(true);
    setError("");
    setStatus("");
    try {
      const res = await Promise.race([
        api.testZebra(zebraUrl.trim() || undefined),
        new Promise<never>((_, reject) =>
          setTimeout(
            () =>
              reject(
                new Error(
                  "Test timed out. The API could not reach that Zebrad (127.0.0.1 only works if Zebrad runs on the same machine as the API).",
                ),
              ),
            20000,
          ),
        ),
      ]);
      setStatus(res.message);
      Alert.alert(res.ok ? "Zebra OK" : "Zebra test", res.message);
    } catch (e) {
      const msg = e instanceof Error ? e.message : "Connection test failed";
      setError(msg);
      Alert.alert("Zebra test failed", msg);
    } finally {
      setLoading(false);
    }
  }

  return (
    <SafeAreaView style={styles.safe} edges={["bottom"]}>
      <ScrollView
        contentContainerStyle={styles.container}
        keyboardShouldPersistTaps="handled"
      >
        <SettingsBackButton onPress={onBack} />
        <Text style={styles.title}>Network & Node</Text>
        <Text style={styles.subtitle}>
          This sets the Zebrad URL on your API server (not on the phone). Local
          node means the API machine’s localhost — only works if Zebrad runs
          next to that API.
        </Text>
        <Card>
          <Text style={styles.meta}>Network: {network || "…"}</Text>
          <Select
            label="Node type (API server → Zebrad)"
            value={nodeMode}
            options={[
              { value: "local", label: "Local / own node (recommended)" },
              { value: "public", label: "Public operator node" },
            ]}
            onChange={(v) => requestNodeMode(v as NodeConnectionMode)}
          />
          {nodeMode === "public" ? (
            <View style={styles.banner}>
              <Text style={styles.bannerText}>
                Public Zebrad: the operator may log server-side sync and
                broadcast timing. Nym smolmix only helps when submit goes to a
                remote node you do not control.
              </Text>
            </View>
          ) : null}
          <Input
            label="Zebra RPC URL"
            value={zebraUrl}
            onChangeText={setZebraUrl}
            autoCapitalize="none"
            autoCorrect={false}
            placeholder={defaultLocalZebraUrl()}
          />
          <View style={styles.row}>
            <Button
              label="Save"
              onPress={() => void save()}
              loading={loading}
              style={styles.half}
            />
            <Button
              label="Test"
              variant="secondary"
              onPress={() => void test()}
              loading={loading}
              style={styles.half}
            />
          </View>
          {status ? <Text style={styles.ok}>{status}</Text> : null}
          {error ? <Text style={styles.error}>{error}</Text> : null}
        </Card>
      </ScrollView>

      <PublicNodeRiskModal
        visible={riskOpen}
        onCancel={cancelPublic}
        onConfirm={confirmPublic}
        context="zebra"
      />
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  container: { padding: spacing.lg, gap: spacing.md },
  title: { color: colors.text, fontSize: fontSize.xl, fontWeight: "700" },
  subtitle: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
    marginBottom: spacing.sm,
  },
  meta: { color: colors.textMuted, fontSize: fontSize.sm },
  banner: {
    backgroundColor: colors.warnBg,
    borderRadius: 12,
    padding: spacing.md,
    borderWidth: 1,
    borderColor: colors.warn,
  },
  bannerText: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
    lineHeight: 18,
  },
  row: { flexDirection: "row", gap: spacing.sm },
  half: { flex: 1 },
  ok: { color: colors.success, fontSize: fontSize.sm },
  error: { color: colors.error, fontSize: fontSize.sm },
});
