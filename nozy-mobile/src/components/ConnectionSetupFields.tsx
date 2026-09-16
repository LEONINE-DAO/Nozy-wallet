import { StyleSheet, Text } from "react-native";
import { Button } from "./Button";
import { Input } from "./Input";
import { defaultSelfHostedApiUrl } from "../lib/connectionPresets";
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
  return (
    <>
      <Text style={styles.hint}>
        Nozy keeps keys on this phone. Compact sync uses our public
        lightwalletd (https://lwd.nozywallet.org:443). To use a node you run,
        set your own lightwalletd URL in On-device settings. Optional:
        connect your own nozywallet-api (home PC) under Advanced — never a
        shared hosted wallet.
      </Text>
      <Input
        label="API server URL"
        value={urlDraft}
        onChangeText={onUrlChange}
        autoCapitalize="none"
        autoCorrect={false}
        placeholder={defaultSelfHostedApiUrl()}
      />
      <Input
        label="API key (optional)"
        value={keyDraft}
        onChangeText={onKeyChange}
        autoCapitalize="none"
        autoCorrect={false}
        secureTextEntry
        placeholder="Local API"
      />
      <Button
        label="Save & connect"
        onPress={() => void onSave()}
        loading={saving}
      />
      {status ? <Text style={styles.ok}>{status}</Text> : null}
      {error ? <Text style={styles.bad}>{error}</Text> : null}
    </>
  );
}

const styles = StyleSheet.create({
  hint: {
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
