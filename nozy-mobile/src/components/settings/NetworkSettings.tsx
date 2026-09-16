import { useEffect, useState } from "react";
import { Alert, ScrollView, StyleSheet, Text, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "../Button";
import { Card } from "../Card";
import { Input } from "../Input";
import { PublicNodeRiskModal } from "../PublicNodeRiskModal";
import { Select } from "../Select";
import { SettingsBackButton } from "./SettingsBackButton";
import { useWalletSession } from "../../context/WalletSessionContext";
import { api } from "../../services/api";
import AsyncStorage from "@react-native-async-storage/async-storage";
import { lwdGetInfo } from "nozy-wallet";
import { ONDEVICE_LWD_URL_KEY } from "./OnDeviceWalletSettings";
import {
  type NodeConnectionMode,
  defaultHostedLwdUrl,
  defaultLocalZebraUrl,
  defaultPublicZebraUrl,
  inferNodeConnectionMode,
  isHostedApiUrl,
  isLocalZebraUrl,
} from "../../lib/connectionPresets";
import { colors, fontSize, spacing } from "../../theme";

type Props = { onBack: () => void };

export function NetworkSettings({ onBack }: Props) {
  const { apiUrl, backendMode } = useWalletSession();
  const apiIsHosted = isHostedApiUrl(apiUrl);
  const [zebraUrl, setZebraUrl] = useState("");
  const [initialZebraUrl, setInitialZebraUrl] = useState("");
  const [network, setNetwork] = useState("");
  const [nodeMode, setNodeMode] = useState<NodeConnectionMode>("local");
  const [status, setStatus] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const [riskOpen, setRiskOpen] = useState(false);
  const [pendingUrl, setPendingUrl] = useState<string | null>(null);
  const [lwdUrl, setLwdUrl] = useState(defaultHostedLwdUrl());

  useEffect(() => {
    if (backendMode === "on_device") {
      void AsyncStorage.getItem(ONDEVICE_LWD_URL_KEY).then((v) => {
        setLwdUrl(v?.trim() || defaultHostedLwdUrl());
      });
      return;
    }
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
  }, [backendMode]);

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
      const msg = "Enter a full-node RPC URL before saving.";
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
      const ok = `Full-node RPC URL saved on the API:\n${trimmed}`;
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
                  "Test timed out. The API could not reach that node (127.0.0.1 only works if Zebrad/Zakura runs on the same machine as the API).",
                ),
              ),
            20000,
          ),
        ),
      ]);
      setStatus(res.message);
      Alert.alert(res.ok ? "Node OK" : "Node test", res.message);
    } catch (e) {
      const msg = e instanceof Error ? e.message : "Connection test failed";
      setError(msg);
      Alert.alert("Node test failed", msg);
    } finally {
      setLoading(false);
    }
  }

  async function saveLwd() {
    setLoading(true);
    setError("");
    setStatus("");
    try {
      const url = lwdUrl.trim() || defaultHostedLwdUrl();
      await AsyncStorage.setItem(ONDEVICE_LWD_URL_KEY, url);
      setLwdUrl(url);
      const info = await lwdGetInfo(url);
      setStatus(
        `Connected · height ${Number(info.block_height || info.estimated_height || 0).toLocaleString()}`,
      );
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not reach lightwalletd");
    } finally {
      setLoading(false);
    }
  }

  if (backendMode === "on_device") {
    return (
      <SafeAreaView style={styles.safe} edges={["bottom"]}>
        <SettingsBackButton onPress={onBack} />
        <ScrollView
          contentContainerStyle={styles.container}
          keyboardShouldPersistTaps="handled"
        >
          <Text style={styles.title}>Network & node</Text>
          <Text style={styles.subtitle}>
            This phone syncs compact blocks from lightwalletd. Default is
            lwd.nozywallet.org. Set your own URL if you run a node.
          </Text>
          <Card>
            <Input
              label="lightwalletd URL"
              value={lwdUrl}
              onChangeText={setLwdUrl}
              autoCapitalize="none"
              autoCorrect={false}
              placeholder={defaultHostedLwdUrl()}
            />
            <Button
              label="Save & test"
              onPress={() => void saveLwd()}
              loading={loading}
            />
            {status ? <Text style={styles.ok}>{status}</Text> : null}
            {error ? <Text style={styles.error}>{error}</Text> : null}
          </Card>
        </ScrollView>
      </SafeAreaView>
    );
  }

  return (
    <SafeAreaView style={styles.safe} edges={["bottom"]}>
      <SettingsBackButton onPress={onBack} />
      <ScrollView
        contentContainerStyle={styles.container}
        keyboardShouldPersistTaps="handled"
      >
        <Text style={styles.title}>Network & Node</Text>
        <Text style={styles.subtitle}>
          This sets the full-node RPC URL on your API server (not on the phone).
          Use Zebrad or Zakura JSON-RPC (same port). Local node means the API
          machine’s localhost — only works if the node runs next to that API.
        </Text>
        {apiIsHosted ? (
          <View style={styles.banner}>
            <Text style={styles.bannerText}>
              You are on a remote API. http://127.0.0.1 here is that server,
              not your phone or home PC. To use your node, point Mobile
              connection at an API that sits next to Zebrad/Zakura.
            </Text>
          </View>
        ) : null}
        <Card>
          <Text style={styles.meta}>Network: {network || "…"}</Text>
          <Select
            label="Node type (API server → Zebrad/Zakura)"
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
                Public node: the operator may log server-side sync and broadcast
                timing. Nym smolmix only helps when submit goes to a remote node
                you do not control.
              </Text>
            </View>
          ) : null}
          <Input
            label="Full node RPC URL (Zebrad or Zakura)"
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
