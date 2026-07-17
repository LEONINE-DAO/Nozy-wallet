import { NativeStackScreenProps } from "@react-navigation/native-stack";
import * as Clipboard from "expo-clipboard";
import { useEffect, useState } from "react";
import {
  Image,
  Pressable,
  ScrollView,
  StyleSheet,
  Text,
  View,
} from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "../components/Button";
import { Card } from "../components/Card";
import { ConnectionSetupFields } from "../components/ConnectionSetupFields";
import { Input } from "../components/Input";
import { Textarea } from "../components/Textarea";
import { useWalletSession } from "../context/WalletSessionContext";
import { requireHostedApiKey } from "../lib/buildProfile";
import { isHostedApiUrl } from "../lib/connectionPresets";
import { api } from "../services/api";
import { colors, fontSize, radius, spacing } from "../theme";
import type { RootStackParamList } from "../types";

type Props = NativeStackScreenProps<RootStackParamList, "Welcome">;
type ViewState = "initial" | "create" | "restore" | "securityTips";

/**
 * Same login flow as desktop-client/src/pages/Welcome.tsx:
 * logo, Privacy by Default, Unlock / Create / Restore / mnemonic / security tips.
 */
export function WelcomeScreen({ navigation }: Props) {
  const { unlockSession, apiUrl, setApiUrl, apiKey, setApiKey } =
    useWalletSession();
  const [view, setView] = useState<ViewState>("initial");
  const [isLoading, setIsLoading] = useState(false);
  const [checking, setChecking] = useState(true);
  const [apiReachable, setApiReachable] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [walletExistsOnDisk, setWalletExistsOnDisk] = useState(false);
  const [requiresPassword, setRequiresPassword] = useState(false);
  const [urlDraft, setUrlDraft] = useState(apiUrl);
  const [keyDraft, setKeyDraft] = useState(apiKey);
  const [connStatus, setConnStatus] = useState("");
  const [connError, setConnError] = useState("");
  const [connSaving, setConnSaving] = useState(false);

  const [createPassword, setCreatePassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [showCreatePass, setShowCreatePass] = useState(false);
  const [showConfirmPass, setShowConfirmPass] = useState(false);
  const [generatedMnemonic, setGeneratedMnemonic] = useState<string | null>(
    null,
  );
  const [mnemonicCopied, setMnemonicCopied] = useState(false);

  const [mnemonic, setMnemonic] = useState("");
  const [restorePassword, setRestorePassword] = useState("");
  const [showRestorePass, setShowRestorePass] = useState(false);
  const [unlockPassword, setUnlockPassword] = useState("");
  const [showUnlockPass, setShowUnlockPass] = useState(false);

  useEffect(() => {
    setUrlDraft(apiUrl);
    setKeyDraft(apiKey);
  }, [apiUrl, apiKey]);

  // Re-check after apiUrl/apiKey hydrate from storage (first mount often races).
  useEffect(() => {
    let cancelled = false;
    setChecking(true);
    void (async () => {
      try {
        await Promise.race([
          api.health(),
          new Promise<never>((_, reject) =>
            setTimeout(() => reject(new Error("API health timed out")), 8000),
          ),
        ]);
        if (cancelled) return;
        setApiReachable(true);
        const info = await api.walletExists();
        if (cancelled) return;
        setWalletExistsOnDisk(info.exists);
        setRequiresPassword(Boolean(info.has_password));
      } catch {
        if (cancelled) return;
        setApiReachable(false);
        // Still offer Unlock — user may already have a wallet on this API.
        setWalletExistsOnDisk(true);
        setRequiresPassword(false);
      } finally {
        if (!cancelled) setChecking(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [apiUrl, apiKey]);

  async function handleSaveConnection() {
    setConnSaving(true);
    setConnError("");
    setConnStatus("");
    setError(null);
    try {
      if (
        requireHostedApiKey() &&
        isHostedApiUrl(urlDraft) &&
        !keyDraft.trim()
      ) {
        setConnError("API key is required for the hosted API.");
        return;
      }
      await setApiUrl(urlDraft);
      await setApiKey(keyDraft);
      await api.health();
      setApiReachable(true);
      setConnStatus("Connected to API");
      const info = await api.walletExists();
      setWalletExistsOnDisk(info.exists);
      setRequiresPassword(Boolean(info.has_password));
    } catch (e) {
      setApiReachable(false);
      setConnError(
        e instanceof Error
          ? e.message
          : "Could not reach API. Check URL / key.",
      );
    } finally {
      setConnSaving(false);
    }
  }

  async function handleUnlockWallet() {
    setIsLoading(true);
    setError(null);
    try {
      const info = await api.walletExists();
      setWalletExistsOnDisk(info.exists);
      setRequiresPassword(Boolean(info.has_password));
      if (!info.exists) {
        setError("No wallet found on this API. Create or restore one first.");
        return;
      }
      const pwd = info.has_password ? unlockPassword : "";
      if (info.has_password && !pwd.trim()) {
        setError("Enter your wallet password to unlock.");
        return;
      }
      await api.unlockWallet(pwd);
      await unlockSession(pwd);
      navigation.replace("Dashboard");
    } catch (err: unknown) {
      setError(
        err instanceof Error
          ? err.message
          : "Failed to unlock wallet. Check your password.",
      );
    } finally {
      setIsLoading(false);
    }
  }

  async function handleCreateWallet() {
    if (createPassword !== confirmPassword) {
      setError("Passwords do not match");
      return;
    }
    setIsLoading(true);
    setError(null);
    try {
      const response = await api.createWallet(createPassword);
      if (response.mnemonic) {
        setGeneratedMnemonic(response.mnemonic);
        setWalletExistsOnDisk(true);
        setRequiresPassword(Boolean(createPassword));
        await unlockSession(createPassword);
      } else {
        await unlockSession(createPassword);
        navigation.replace("Dashboard");
      }
    } catch (err: unknown) {
      setError(
        err instanceof Error
          ? err.message
          : "Failed to create wallet. Please try again.",
      );
    } finally {
      setIsLoading(false);
    }
  }

  async function handleRestoreWallet() {
    if (!mnemonic.trim()) {
      setError("Mnemonic is required");
      return;
    }
    setIsLoading(true);
    setError(null);
    try {
      await api.restoreWallet(mnemonic.trim(), restorePassword);
      setWalletExistsOnDisk(true);
      setRequiresPassword(Boolean(restorePassword));
      await unlockSession(restorePassword);
      setView("securityTips");
    } catch (err: unknown) {
      setError(
        err instanceof Error
          ? err.message
          : "Failed to restore wallet. Please check your mnemonic.",
      );
    } finally {
      setIsLoading(false);
    }
  }

  function handleBack() {
    setView("initial");
    setError(null);
    setCreatePassword("");
    setConfirmPassword("");
    setMnemonic("");
    setRestorePassword("");
    setGeneratedMnemonic(null);
  }

  async function handleMnemonicCopied() {
    if (!generatedMnemonic) return;
    await Clipboard.setStringAsync(generatedMnemonic);
    setMnemonicCopied(true);
    setTimeout(() => setMnemonicCopied(false), 2000);
  }

  function handleMnemonicConfirmed() {
    setGeneratedMnemonic(null);
    setView("securityTips");
  }

  async function handleGetStarted() {
    try {
      await api.walletStatus();
    } catch {
      // Best-effort; session should already be open after create/restore.
    }
    setView("initial");
    navigation.replace("Dashboard");
  }

  return (
    <SafeAreaView style={styles.safe}>
      <ScrollView
        contentContainerStyle={styles.scroll}
        keyboardShouldPersistTaps="handled"
      >
        <View style={styles.heroGlow} />

        <Image
          source={require("../../assets/logo.png")}
          style={styles.logo}
          resizeMode="contain"
        />
        <Text style={styles.brandTitle}>Privacy by Default.</Text>

        {generatedMnemonic ? (
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>Save Your Recovery Phrase</Text>
            <Text style={styles.subtitle}>
              Write down these 24 words in order. Store them in a safe place.
              Away from nozy people. You'll need this to recover your wallet.
            </Text>
            <View style={styles.warnBox}>
              <Text style={styles.warnTitle}>
                Important: Never share your recovery phrase with anyone!
              </Text>
              <Text style={styles.warnBody}>
                Anyone with access to these words can control your wallet.
              </Text>
            </View>
            <Card variant="elevated" padding="md">
              <View style={styles.mnemonicGrid}>
                {generatedMnemonic.split(" ").map((word, index) => (
                  <View key={`${index}-${word}`} style={styles.mnemonicCell}>
                    <Text style={styles.mnemonicIndex}>{index + 1}.</Text>
                    <Text style={styles.mnemonicWord}>{word}</Text>
                  </View>
                ))}
              </View>
            </Card>
            <Button
              label={mnemonicCopied ? "Copied!" : "Copy Recovery Phrase"}
              onPress={() => void handleMnemonicCopied()}
              variant={mnemonicCopied ? "secondary" : "primary"}
              size="lg"
            />
            <Button
              label="I've Saved My Recovery Phrase"
              onPress={handleMnemonicConfirmed}
              disabled={!mnemonicCopied}
              size="lg"
            />
          </View>
        ) : null}

        {!generatedMnemonic && view === "initial" ? (
          <View style={styles.section}>
            {!walletExistsOnDisk && !checking ? (
              <Text style={styles.lead}>
                Experience the future of private finance. Nozy Wallet combines
                speed and anonymity in a beautiful, easy-to-use interface.
              </Text>
            ) : null}

            {!checking && !apiReachable ? (
              <Card variant="elevated" padding="lg">
                <Text style={styles.sectionTitle}>Connect API first</Text>
                <Text style={styles.subtitle}>
                  This phone cannot reach the companion API. Emulator default
                  (10.0.2.2) does not work on a real device — use Nozy hosted
                  HTTPS or your own VPS URL.
                </Text>
                <ConnectionSetupFields
                  urlDraft={urlDraft}
                  keyDraft={keyDraft}
                  onUrlChange={setUrlDraft}
                  onKeyChange={setKeyDraft}
                  onSave={handleSaveConnection}
                  saving={connSaving}
                  status={connStatus}
                  error={connError}
                />
              </Card>
            ) : null}

            <View style={styles.unlockBlock}>
              <Text style={styles.sectionTitle}>Welcome back</Text>
              <Text style={styles.subtitle}>
                {requiresPassword
                  ? "Enter your password to unlock your wallet"
                  : "Unlock to open your existing wallet"}
              </Text>
              {checking ? (
                <Text style={styles.subtitle}>Checking wallet…</Text>
              ) : null}
              {requiresPassword ? (
                <>
                  <Input
                    label="Password"
                    placeholder="Enter your wallet password"
                    value={unlockPassword}
                    onChangeText={setUnlockPassword}
                    secureTextEntry={!showUnlockPass}
                    error={error || undefined}
                  />
                  <Pressable onPress={() => setShowUnlockPass((v) => !v)}>
                    <Text style={styles.toggle}>
                      {showUnlockPass ? "Hide password" : "Show password"}
                    </Text>
                  </Pressable>
                </>
              ) : error ? (
                <Text style={styles.error}>{error}</Text>
              ) : null}
              <Button
                label={isLoading ? "Unlocking..." : "Unlock Wallet"}
                onPress={() => void handleUnlockWallet()}
                loading={isLoading}
                disabled={
                  isLoading ||
                  !apiReachable ||
                  (requiresPassword && !unlockPassword.trim())
                }
                size="lg"
              />
            </View>

            <Text style={styles.orSetup}>or set up a different wallet</Text>

            <View style={styles.actions}>
              <Button
                label="Create New Wallet"
                onPress={() => {
                  setError(null);
                  setView("create");
                }}
                disabled={!apiReachable}
                size="lg"
              />
              <Button
                label="Restore Wallet"
                variant="secondary"
                onPress={() => {
                  setError(null);
                  setView("restore");
                }}
                disabled={!apiReachable}
                size="lg"
              />
            </View>
          </View>
        ) : null}

        {!generatedMnemonic && view === "create" ? (
          <View style={styles.section}>
            <Card variant="elevated" padding="lg">
              <Text style={styles.sectionTitle}>Create New Wallet</Text>
              <Text style={styles.subtitle}>
                Set a password to secure your wallet. You can create as many
                wallets as you need.
              </Text>
              <Input
                label="Password"
                placeholder="Enter a strong password"
                value={createPassword}
                onChangeText={setCreatePassword}
                secureTextEntry={!showCreatePass}
              />
              <Pressable onPress={() => setShowCreatePass((v) => !v)}>
                <Text style={styles.toggle}>
                  {showCreatePass ? "Hide password" : "Show password"}
                </Text>
              </Pressable>
              <Input
                label="Confirm Password"
                placeholder="Confirm your password"
                value={confirmPassword}
                onChangeText={setConfirmPassword}
                secureTextEntry={!showConfirmPass}
                error={error || undefined}
              />
              <Pressable onPress={() => setShowConfirmPass((v) => !v)}>
                <Text style={styles.toggle}>
                  {showConfirmPass ? "Hide password" : "Show password"}
                </Text>
              </Pressable>
              <Button
                label={isLoading ? "Creating..." : "Create Wallet"}
                onPress={() => void handleCreateWallet()}
                loading={isLoading}
                size="lg"
              />
              <Button label="Back" variant="ghost" onPress={handleBack} />
            </Card>
          </View>
        ) : null}

        {!generatedMnemonic && view === "restore" ? (
          <View style={styles.section}>
            <Card variant="elevated" padding="lg">
              <Text style={styles.sectionTitle}>Restore Wallet</Text>
              <Text style={styles.subtitle}>
                Enter your seed phrase to recover your wallet
              </Text>
              <Textarea
                label="Seed Phrase (Mnemonic)"
                placeholder="Enter your 24-word seed phrase..."
                value={mnemonic}
                onChangeText={setMnemonic}
                autoCapitalize="none"
                autoCorrect={false}
              />
              <Input
                label="New Password"
                placeholder="Set a password for this device"
                value={restorePassword}
                onChangeText={setRestorePassword}
                secureTextEntry={!showRestorePass}
              />
              <Pressable onPress={() => setShowRestorePass((v) => !v)}>
                <Text style={styles.toggle}>
                  {showRestorePass ? "Hide password" : "Show password"}
                </Text>
              </Pressable>
              {error ? <Text style={styles.error}>{error}</Text> : null}
              <Button
                label={isLoading ? "Restoring..." : "Restore Wallet"}
                onPress={() => void handleRestoreWallet()}
                loading={isLoading}
                size="lg"
              />
              <Button label="Back" variant="ghost" onPress={handleBack} />
            </Card>
          </View>
        ) : null}

        {!generatedMnemonic && view === "securityTips" ? (
          <View style={styles.section}>
            <Text style={styles.sectionTitle}>A few security tips</Text>
            <Text style={styles.subtitle}>Keep your wallet and funds safe</Text>
            <View style={styles.tip}>
              <Text style={styles.tipTitle}>
                Never share your recovery phrase
              </Text>
              <Text style={styles.tipBody}>
                Nozy Wallet and no one else will ever ask for it. Anyone with
                these words can control your funds.
              </Text>
            </View>
            <View style={styles.tip}>
              <Text style={styles.tipTitle}>Verify addresses before sending</Text>
              <Text style={styles.tipBody}>
                Always double-check the recipient address. Transactions cannot
                be reversed.
              </Text>
            </View>
            <View style={styles.tip}>
              <Text style={styles.tipTitle}>Sync your wallet</Text>
              <Text style={styles.tipBody}>
                After unlocking, sync to load your balance and transaction
                history from the network.
              </Text>
            </View>
            <View style={styles.tip}>
              <Text style={styles.tipTitle}>Keep your app updated</Text>
              <Text style={styles.tipBody}>
                Updates include security fixes. Download only from official
                sources.
              </Text>
            </View>
            <Button
              label="Get started"
              onPress={() => void handleGetStarted()}
              size="lg"
            />
          </View>
        ) : null}
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: {
    flex: 1,
    backgroundColor: colors.background,
  },
  scroll: {
    flexGrow: 1,
    padding: spacing.lg,
    paddingBottom: spacing.xl,
    alignItems: "center",
  },
  heroGlow: {
    position: "absolute",
    top: 0,
    left: 0,
    right: 0,
    height: 220,
    backgroundColor: "rgba(212, 175, 55, 0.08)",
  },
  logo: {
    width: 160,
    height: 160,
    marginTop: spacing.md,
    marginBottom: spacing.md,
  },
  brandTitle: {
    color: colors.text,
    fontSize: fontSize.xl,
    fontWeight: "800",
    textAlign: "center",
    marginBottom: spacing.lg,
  },
  section: {
    width: "100%",
    maxWidth: 480,
    gap: spacing.md,
  },
  unlockBlock: {
    gap: spacing.md,
  },
  sectionTitle: {
    color: colors.text,
    fontSize: fontSize.lg,
    fontWeight: "700",
    textAlign: "center",
  },
  lead: {
    color: colors.textMuted,
    fontSize: fontSize.md,
    lineHeight: 24,
    textAlign: "center",
    marginBottom: spacing.sm,
  },
  subtitle: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
    lineHeight: 20,
    textAlign: "center",
  },
  orSetup: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
    textAlign: "center",
  },
  actions: {
    gap: spacing.md,
    marginTop: spacing.sm,
  },
  error: {
    color: colors.error,
    fontSize: fontSize.sm,
    textAlign: "center",
  },
  toggle: {
    color: colors.primary,
    fontSize: fontSize.sm,
    marginTop: -spacing.sm,
  },
  warnBox: {
    backgroundColor: colors.warnBg,
    borderWidth: 1,
    borderColor: colors.warn,
    borderRadius: radius.lg,
    padding: spacing.md,
    gap: spacing.xs,
  },
  warnTitle: {
    color: colors.warn,
    fontSize: fontSize.sm,
    fontWeight: "700",
  },
  warnBody: {
    color: colors.textMuted,
    fontSize: fontSize.xs,
  },
  mnemonicGrid: {
    flexDirection: "row",
    flexWrap: "wrap",
    gap: spacing.sm,
  },
  mnemonicCell: {
    width: "30%",
    flexGrow: 1,
    flexDirection: "row",
    alignItems: "center",
    gap: spacing.xs,
    backgroundColor: colors.surface,
    borderRadius: radius.sm,
    padding: spacing.sm,
  },
  mnemonicIndex: {
    color: colors.textMuted,
    fontSize: fontSize.xs,
    fontFamily: "monospace",
    width: 22,
  },
  mnemonicWord: {
    color: colors.text,
    fontSize: fontSize.sm,
    fontWeight: "600",
  },
  tip: {
    backgroundColor: colors.surface,
    borderRadius: radius.xl,
    borderWidth: 1,
    borderColor: colors.border,
    padding: spacing.md,
    gap: spacing.xs,
  },
  tipTitle: {
    color: colors.text,
    fontSize: fontSize.md,
    fontWeight: "700",
  },
  tipBody: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
    lineHeight: 20,
  },
});
