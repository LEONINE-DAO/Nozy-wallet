import { NativeStackScreenProps } from "@react-navigation/native-stack";
import { useEffect, useState } from "react";
import { ScrollView, StyleSheet, Text } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { useWalletSession } from "../context/WalletSessionContext";
import { api } from "../services/api";
import { onDeviceShieldedSnapshot } from "../services/onDeviceWallet";
import { colors, fontSize, spacing } from "../theme";
import type { RootStackParamList, TransactionRecord } from "../types";

type Props = NativeStackScreenProps<RootStackParamList, "TransactionDetail">;

function row(label: string, value: string | number | null | undefined) {
  if (value === null || value === undefined || value === "") return null;
  return (
    <>
      <Text style={styles.label}>{label}</Text>
      <Text style={styles.value} selectable>
        {String(value)}
      </Text>
    </>
  );
}

export function TransactionDetailScreen({ route }: Props) {
  const { txid } = route.params;
  const { backendMode } = useWalletSession();
  const [tx, setTx] = useState<Record<string, unknown> | null>(null);
  const [note, setNote] = useState<TransactionRecord | null>(null);
  const [error, setError] = useState("");

  useEffect(() => {
    if (backendMode === "on_device") {
      void onDeviceShieldedSnapshot()
        .then((snap) => {
          const found = snap.notes.find((item) => item.txid === txid);
          setNote(
            found
              ? {
                  txid: found.txid,
                  status: found.spent ? "spent" : "received",
                  amount_zec: found.value_zec,
                  fee_zec: 0,
                  recipient: found.pool,
                  block_height: found.block_height,
                  confirmations: null,
                  broadcast_at: null,
                  memo: found.memo,
                }
              : null,
          );
        })
        .catch((e) =>
          setError(e instanceof Error ? e.message : "Failed to load transaction"),
        );
      return;
    }
    void api
      .getTransaction(txid)
      .then(setTx)
      .catch((e) =>
        setError(e instanceof Error ? e.message : "Failed to load transaction"),
      );
  }, [txid, backendMode]);

  if (backendMode === "on_device") {
    return (
      <SafeAreaView style={styles.safe} edges={["bottom"]}>
        <ScrollView contentContainerStyle={styles.container}>
          {error ? <Text style={styles.error}>{error}</Text> : null}
          {row("TXID", txid)}
          {row("Status", note?.status)}
          {row("Amount (ZEC)", note?.amount_zec)}
          {row("Pool", note?.recipient)}
          {row("Block height", note?.block_height)}
          {row("Memo", note?.memo)}
          {!note && !error ? (
            <Text style={styles.loading}>No details for this transaction yet.</Text>
          ) : null}
        </ScrollView>
      </SafeAreaView>
    );
  }

  return (
    <SafeAreaView style={styles.safe} edges={["bottom"]}>
      <ScrollView contentContainerStyle={styles.container}>
        {error ? <Text style={styles.error}>{error}</Text> : null}
        {tx ? (
          <>
            {row("TXID", tx.txid as string)}
            {row("Status", tx.status as string)}
            {row("Amount (ZEC)", tx.amount_zec as number)}
            {row("Fee (ZEC)", tx.fee_zec as number)}
            {row("Recipient", tx.recipient as string)}
            {row("Block height", tx.block_height as number)}
            {row("Confirmations", tx.confirmations as number)}
            {row("Broadcast at", tx.broadcast_at as string)}
            {row("Memo", tx.memo as string)}
          </>
        ) : !error ? (
          <Text style={styles.loading}>Loading...</Text>
        ) : null}
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  container: { padding: spacing.lg, gap: spacing.sm },
  label: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
    fontWeight: "600",
    marginTop: spacing.md,
  },
  value: { color: colors.text, fontSize: fontSize.md, lineHeight: 22 },
  error: { color: colors.error },
  loading: { color: colors.textMuted },
});
