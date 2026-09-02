import * as Clipboard from "expo-clipboard";
import { useCallback, useEffect, useState } from "react";
import {
  Linking,
  ScrollView,
  Share,
  StyleSheet,
  Text,
  TextInput,
  View,
} from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { NativeStackScreenProps } from "@react-navigation/native-stack";
import { isNozyWalletNativeAvailable } from "nozy-wallet";
import { Button } from "../components/Button";
import { Card } from "../components/Card";
import { PageHeader } from "../components/PageHeader";
import {
  onDeviceVoteCalendar,
  onDeviceVoteExportNotes,
  onDeviceVoteSignDelegation,
} from "../services/onDeviceVote";
import { colors, fontSize, spacing } from "../theme";
import type { RootStackParamList } from "../types";

type Props = NativeStackScreenProps<RootStackParamList, "Vote">;

const ZAT = 100_000_000;

export function VoteScreen({}: Props) {
  const [calendar, setCalendar] = useState<{
    snapshot_utc: string;
    vote_start_utc: string;
    vote_end_utc: string;
    forum_url: string;
    message: string;
  } | null>(null);
  const [nativeReady, setNativeReady] = useState(false);
  const [busy, setBusy] = useState<"export" | "sign" | null>(null);
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");
  const [requestJson, setRequestJson] = useState("");
  const [lastNotesJson, setLastNotesJson] = useState<string | null>(null);
  const [lastSigJson, setLastSigJson] = useState<string | null>(null);

  const load = useCallback(async () => {
    setNativeReady(isNozyWalletNativeAvailable());
    try {
      setCalendar(await onDeviceVoteCalendar());
    } catch {
      setCalendar(null);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  async function exportNotes() {
    setBusy("export");
    setError("");
    setMessage("");
    try {
      const res = await onDeviceVoteExportNotes("mainnet");
      setLastNotesJson(res.notes_json);
      setMessage(
        `${res.message} (~${(res.total_value_zat / ZAT).toFixed(4)} ZEC across ${res.note_count} note(s)).`,
      );
    } catch (e) {
      setError(e instanceof Error ? e.message : "Export failed");
    } finally {
      setBusy(null);
    }
  }

  async function signDelegation() {
    setBusy("sign");
    setError("");
    setMessage("");
    try {
      const res = await onDeviceVoteSignDelegation(requestJson.trim());
      setLastSigJson(res.sig_json);
      setMessage(res.message);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Sign failed");
    } finally {
      setBusy(null);
    }
  }

  async function shareText(title: string, body: string) {
    await Share.share({ title, message: body });
  }

  return (
    <SafeAreaView style={styles.safe} edges={["bottom"]}>
      <ScrollView contentContainerStyle={styles.container}>
        <PageHeader
          title="NU7 Vote"
          description="On-device export + sign only. Prepare, prove, and cast on Desktop Vote or nozy-vote CLI."
        />

        <Card>
          <Text style={styles.label}>Calendar</Text>
          <Text style={styles.body}>
            Snapshot {calendar?.snapshot_utc ?? "2026-08-24T19:00:00Z"}
          </Text>
          <Text style={styles.body}>
            Vote {calendar?.vote_start_utc ?? "…"} →{" "}
            {calendar?.vote_end_utc ?? "…"}
          </Text>
          <Text style={styles.muted}>{calendar?.message}</Text>
          {calendar?.forum_url ? (
            <Button
              label="Forum thread"
              variant="ghost"
              onPress={() => void Linking.openURL(calendar.forum_url)}
            />
          ) : null}
        </Card>

        <Card>
          <Text style={styles.label}>Native FFI</Text>
          <Text style={styles.body}>
            {nativeReady
              ? "libnozy_ffi available — export/sign can run on device."
              : "Native module not loaded (Expo Go / missing .so). Companion mode: use Desktop Vote for the full flow."}
          </Text>
        </Card>

        <Card>
          <Text style={styles.label}>1. Export Ironwood notes</Text>
          <Text style={styles.muted}>
            Needs synced Ironwood notes in the on-device wallet data dir. Share
            JSON to desktop for import-notes.
          </Text>
          <Button
            label={busy === "export" ? "Exporting…" : "Export notes"}
            onPress={() => void exportNotes()}
            disabled={busy !== null}
          />
          {lastNotesJson ? (
            <View style={styles.row}>
              <Button
                label="Copy notes JSON"
                variant="secondary"
                onPress={() => void Clipboard.setStringAsync(lastNotesJson)}
              />
              <Button
                label="Share notes"
                variant="secondary"
                onPress={() =>
                  void shareText("nozy-vote-notes-v1", lastNotesJson)
                }
              />
            </View>
          ) : null}
        </Card>

        <Card>
          <Text style={styles.label}>2. Sign delegation request</Text>
          <Text style={styles.muted}>
            Paste signing-request JSON from desktop / nozy-vote delegate, then
            share the signature back for delegate-finish.
          </Text>
          <TextInput
            style={styles.input}
            multiline
            placeholder='{"format":"nozy-vote-delegation-sign-v1",...}'
            placeholderTextColor={colors.textMuted}
            value={requestJson}
            onChangeText={setRequestJson}
            autoCapitalize="none"
            autoCorrect={false}
          />
          <Button
            label={busy === "sign" ? "Signing…" : "Sign delegation"}
            onPress={() => void signDelegation()}
            disabled={busy !== null || !requestJson.trim()}
          />
          {lastSigJson ? (
            <View style={styles.row}>
              <Button
                label="Copy signature"
                variant="secondary"
                onPress={() => void Clipboard.setStringAsync(lastSigJson)}
              />
              <Button
                label="Share signature"
                variant="secondary"
                onPress={() =>
                  void shareText("nozy-vote-delegation-sig-v1", lastSigJson)
                }
              />
            </View>
          ) : null}
        </Card>

        {message ? <Text style={styles.ok}>{message}</Text> : null}
        {error ? <Text style={styles.err}>{error}</Text> : null}
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  container: { padding: spacing.lg, gap: spacing.md },
  label: {
    color: colors.text,
    fontSize: fontSize.md,
    fontWeight: "700",
    marginBottom: spacing.xs,
  },
  body: { color: colors.text, fontSize: fontSize.sm, marginBottom: spacing.xs },
  muted: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
    marginBottom: spacing.sm,
  },
  ok: { color: colors.success, fontSize: fontSize.sm },
  err: { color: colors.error, fontSize: fontSize.sm },
  input: {
    minHeight: 120,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: 12,
    padding: spacing.sm,
    color: colors.text,
    backgroundColor: colors.surface,
    textAlignVertical: "top",
    marginBottom: spacing.sm,
    fontFamily: "monospace",
    fontSize: fontSize.xs,
  },
  row: { flexDirection: "row", flexWrap: "wrap", gap: spacing.sm, marginTop: spacing.sm },
});
