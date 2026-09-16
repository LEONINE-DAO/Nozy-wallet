import { useCallback, useEffect, useState } from "react";
import { ScrollView, StyleSheet, Text, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "../Button";
import { Input } from "../Input";
import { SettingsBackButton } from "./SettingsBackButton";
import {
  addOnDeviceAccount,
  getActiveOnDeviceAccount,
  listOnDeviceAccounts,
  renameOnDeviceAccount,
  setActiveOnDeviceAccount,
  type OnDeviceAccount,
} from "../../services/onDeviceAccounts";
import {
  onDeviceReceiveAddress,
  onDeviceWalletMeta,
} from "../../services/onDeviceWallet";
import { colors, fontSize, radius, spacing } from "../../theme";

type Props = { onBack: () => void };

function shortUa(address: string): string {
  if (address.length < 20) return address;
  return `${address.slice(0, 10)}…${address.slice(-8)}`;
}

export function WalletsAccountsSettings({ onBack }: Props) {
  const [accounts, setAccounts] = useState<OnDeviceAccount[]>([]);
  const [activeIndex, setActiveIndex] = useState(0);
  const [preview, setPreview] = useState("");
  const [exists, setExists] = useState(false);
  const [error, setError] = useState("");
  const [status, setStatus] = useState("");
  const [newName, setNewName] = useState("");
  const [creating, setCreating] = useState(false);
  const [renameIndex, setRenameIndex] = useState<number | null>(null);
  const [renameDraft, setRenameDraft] = useState("");

  const load = useCallback(async () => {
    setError("");
    try {
      const meta = await onDeviceWalletMeta();
      setExists(meta.exists);
      const list = await listOnDeviceAccounts();
      const active = await getActiveOnDeviceAccount();
      setAccounts(list);
      setActiveIndex(active.index);
      if (meta.exists) {
        const ua = await onDeviceReceiveAddress();
        setPreview(ua.address);
      } else {
        setPreview("");
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not load accounts");
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  async function handleCreate() {
    setError("");
    setStatus("");
    if (!exists) {
      setError("Create or restore a wallet on this phone first.");
      return;
    }
    setCreating(true);
    try {
      const created = await addOnDeviceAccount(newName || undefined);
      setNewName("");
      setStatus(`${created.name} is ready. Receive uses this account now.`);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not create account");
    } finally {
      setCreating(false);
    }
  }

  async function handleRename() {
    if (renameIndex == null) return;
    try {
      await renameOnDeviceAccount(renameIndex, renameDraft);
      setRenameIndex(null);
      setRenameDraft("");
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not rename");
    }
  }

  async function handleSwitch(index: number) {
    try {
      await setActiveOnDeviceAccount(index);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not switch account");
    }
  }

  return (
    <SafeAreaView style={styles.safe} edges={["bottom"]}>
      <SettingsBackButton onPress={onBack} />
      <ScrollView
        contentContainerStyle={styles.container}
        keyboardShouldPersistTaps="handled"
      >
        <Text style={styles.title}>Wallets & accounts</Text>
        <Text style={styles.body}>
          Create more accounts under this seed. Each account has its own
          shielded receive address. The recovery phrase stays the same.
        </Text>

        <View style={styles.createCard}>
          <Text style={styles.createTitle}>Create account</Text>
          <Input
            label="Name"
            value={newName}
            onChangeText={setNewName}
            placeholder={`Account ${accounts.length + 1}`}
            autoCapitalize="words"
          />
          <Button
            label="Create account"
            onPress={() => void handleCreate()}
            loading={creating}
            disabled={creating}
          />
        </View>

        {accounts.map((account) => {
          const active = account.index === activeIndex;
          const renaming = renameIndex === account.index;
          return (
            <View
              key={account.index}
              style={[styles.card, active && styles.cardActive]}
            >
              {renaming ? (
                <>
                  <Input
                    label="Name"
                    value={renameDraft}
                    onChangeText={setRenameDraft}
                    autoCapitalize="words"
                  />
                  <View style={styles.row}>
                    <Button
                      label="Save"
                      size="sm"
                      onPress={() => void handleRename()}
                    />
                    <Button
                      label="Cancel"
                      size="sm"
                      variant="ghost"
                      onPress={() => setRenameIndex(null)}
                    />
                  </View>
                </>
              ) : (
                <>
                  <Text style={styles.name}>{account.name}</Text>
                  <Text style={styles.meta}>
                    Account {account.index}
                    {active ? " · Active" : ""}
                  </Text>
                  {active && preview ? (
                    <Text style={styles.addr} selectable>
                      {shortUa(preview)}
                    </Text>
                  ) : null}
                  <View style={styles.row}>
                    {active ? null : (
                      <Button
                        label="Use"
                        size="sm"
                        onPress={() => void handleSwitch(account.index)}
                      />
                    )}
                    <Button
                      label="Rename"
                      size="sm"
                      variant="ghost"
                      onPress={() => {
                        setRenameIndex(account.index);
                        setRenameDraft(account.name);
                      }}
                    />
                  </View>
                </>
              )}
            </View>
          );
        })}
        {status ? <Text style={styles.ok}>{status}</Text> : null}
        {error ? <Text style={styles.error}>{error}</Text> : null}
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  container: { padding: spacing.lg, gap: spacing.md, paddingBottom: spacing.xl },
  title: { color: colors.text, fontSize: fontSize.xl, fontWeight: "700" },
  body: { color: colors.textMuted, fontSize: fontSize.sm, lineHeight: 20 },
  createCard: {
    backgroundColor: colors.surface,
    borderRadius: radius.md,
    borderWidth: 1,
    borderColor: colors.platinumLine,
    padding: spacing.md,
    gap: spacing.md,
  },
  createTitle: {
    color: colors.text,
    fontSize: fontSize.md,
    fontWeight: "700",
  },
  meta: { color: colors.textMuted, fontSize: fontSize.sm, lineHeight: 20 },
  card: {
    backgroundColor: colors.surface,
    borderRadius: radius.md,
    borderWidth: 1,
    borderColor: colors.border,
    padding: spacing.md,
    gap: 4,
  },
  cardActive: {
    borderColor: colors.platinumLine,
    backgroundColor: colors.primarySoft,
  },
  name: { color: colors.text, fontSize: fontSize.md, fontWeight: "700" },
  addr: {
    color: colors.textFaint,
    fontSize: 12,
    fontFamily: "monospace",
    marginTop: 4,
  },
  row: { flexDirection: "row", gap: spacing.sm, marginTop: spacing.sm },
  ok: { color: colors.success, fontSize: fontSize.sm },
  error: { color: colors.error, fontSize: fontSize.sm },
});
