import { useEffect, useState } from "react";
import { StyleSheet, Text, View } from "react-native";
import { Button } from "./Button";
import { Input } from "./Input";
import { PublicNodeRiskModal } from "./PublicNodeRiskModal";
import { Select } from "./Select";
import {
  type ApiConnectionMode,
  defaultHostedApiUrl,
  defaultSelfHostedApiUrl,
  inferApiConnectionMode,
} from "../lib/connectionPresets";
import { isProductionBuild, requireHostedApiKey } from "../lib/buildProfile";
import { colors, fontSize, spacing } from "../theme";

type Props = {
  urlDraft: string;
  keyDraft: string;
  onUrlChange: (url: string) => void;
  onKeyChange: (key: string) => void;
  onSave: () => void | Promise<void>;
  saving?: boolean;
  status?: string;
  error?: string;
};

export function ConnectionSetupFields({
  urlDraft,
  keyDraft,
  onUrlChange,
  onKeyChange,
  onSave,
  saving,
  status,
  error,
}: Props) {
  const [mode, setMode] = useState<ApiConnectionMode>(() =>
    inferApiConnectionMode(urlDraft),
  );
  const [riskOpen, setRiskOpen] = useState(false);
  const [pendingMode, setPendingMode] = useState<ApiConnectionMode | null>(
    null,
  );

  // Keep selector in sync when parent sets URL (e.g. “Use home PC API”).
  useEffect(() => {
    setMode(inferApiConnectionMode(urlDraft));
  }, [urlDraft]);

  function applyMode(next: ApiConnectionMode) {
    setMode(next);
    if (next === "hosted") {
      onUrlChange(defaultHostedApiUrl());
    } else {
      onUrlChange(defaultSelfHostedApiUrl());
    }
  }

  function requestMode(next: ApiConnectionMode) {
    if (next === mode) return;
    if (next === "hosted") {
      setPendingMode(next);
      setRiskOpen(true);
      return;
    }
    applyMode(next);
  }

  function confirmHosted() {
    if (pendingMode === "hosted") {
      applyMode("hosted");
    }
    setPendingMode(null);
    setRiskOpen(false);
  }

  function cancelHosted() {
    setPendingMode(null);
    setRiskOpen(false);
  }

  // Until Nozy funds its own Zebrad, “hosted” is only useful if that API’s
  // operator already points zebra_url at a live node. Prefer own API.
  const connectionOptions = [
    {
      value: "self" as const,
      label: "My own API (home PC / VPS)",
    },
    {
      value: "hosted" as const,
      label: isProductionBuild()
        ? "Nozy hosted API (no Nozy node yet)"
        : "Nozy hosted API (no Nozy Zebrad yet)",
    },
  ];

  return (
    <>
      <Select
        label="Connection type"
        value={mode}
        options={connectionOptions}
        onChange={(v) => requestMode(v as ApiConnectionMode)}
      />
      {mode === "hosted" ? (
        <View style={styles.banner}>
          <Text style={styles.bannerTitle}>Nozy hosted — limited</Text>
          <Text style={styles.bannerText}>
            The Nozy API server is up, but NozyWallet does not run a Zebrad yet
            (funding). Sync will fail unless that API is configured to a node
            you or another operator provide. Prefer “My own API” with your PC
            or VPS Zebrad.
          </Text>
        </View>
      ) : (
        <Text style={styles.hint}>
          Emulator → home PC: http://10.0.2.2:3000. Real phone → your PC LAN IP
          (http://192.168.x.x:3000) or HTTPS VPS. Leave API key blank for local
          API. Zebrad must be reachable from that API host (not from the phone).
        </Text>
      )}
      <Input
        label="API server URL"
        value={urlDraft}
        onChangeText={onUrlChange}
        autoCapitalize="none"
        autoCorrect={false}
        placeholder={
          mode === "hosted"
            ? defaultHostedApiUrl()
            : defaultSelfHostedApiUrl()
        }
      />
      <Input
        label={
          mode === "hosted" || requireHostedApiKey()
            ? "API key (required)"
            : "API key (optional)"
        }
        value={keyDraft}
        onChangeText={onKeyChange}
        autoCapitalize="none"
        autoCorrect={false}
        secureTextEntry
        placeholder={mode === "hosted" ? "From your host operator" : "Local API"}
      />
      <Button
        label="Save & connect"
        onPress={() => void onSave()}
        loading={saving}
      />
      {status ? <Text style={styles.ok}>{status}</Text> : null}
      {error ? <Text style={styles.bad}>{error}</Text> : null}

      <PublicNodeRiskModal
        visible={riskOpen}
        onCancel={cancelHosted}
        onConfirm={confirmHosted}
        context="api"
      />
    </>
  );
}

const styles = StyleSheet.create({
  hint: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
    lineHeight: 18,
  },
  banner: {
    backgroundColor: colors.warnBg,
    borderRadius: 12,
    padding: spacing.md,
    borderWidth: 1,
    borderColor: colors.warn,
    gap: spacing.xs,
  },
  bannerTitle: {
    color: colors.warn,
    fontSize: fontSize.sm,
    fontWeight: "700",
  },
  bannerText: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
    lineHeight: 18,
  },
  ok: {
    color: colors.success,
    fontSize: fontSize.sm,
    fontWeight: "600",
  },
  bad: {
    color: colors.error,
    fontSize: fontSize.sm,
  },
});
