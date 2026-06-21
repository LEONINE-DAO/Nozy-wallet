import { NativeStackScreenProps } from "@react-navigation/native-stack";
import { useCallback, useEffect, useState } from "react";
import {
  FlatList,
  Pressable,
  RefreshControl,
  StyleSheet,
  Text,
  View,
} from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "../components/Button";
import { api } from "../services/api";
import { colors, fontSize, spacing } from "../theme";
import type { RootStackParamList, TransactionRecord } from "../types";

type Props = NativeStackScreenProps<RootStackParamList, "TransactionHistory">;

export function TransactionHistoryScreen({ navigation }: Props) {
  const [items, setItems] = useState<TransactionRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  const load = useCallback(async () => {
    setError("");
    try {
      const res = await api.getTransactionHistory();
      setItems(res.transactions);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load history");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  return (
    <SafeAreaView style={styles.safe} edges={["bottom"]}>
      <FlatList
        data={items}
        keyExtractor={(item) => item.txid}
        contentContainerStyle={styles.list}
        refreshControl={
          <RefreshControl
            refreshing={loading}
            onRefresh={() => {
              setLoading(true);
              void load();
            }}
            tintColor={colors.primary}
          />
        }
        ListEmptyComponent={
          !loading ? (
            <Text style={styles.empty}>No transactions yet.</Text>
          ) : null
        }
        ListHeaderComponent={
          error ? <Text style={styles.error}>{error}</Text> : null
        }
        renderItem={({ item }) => (
          <Pressable
            style={styles.row}
            onPress={() =>
              navigation.navigate("TransactionDetail", { txid: item.txid })
            }
          >
            <View style={styles.rowTop}>
              <Text style={styles.amount}>-{item.amount_zec.toFixed(8)} ZEC</Text>
              <Text style={styles.status}>{item.status}</Text>
            </View>
            <Text style={styles.recipient} numberOfLines={1}>
              {item.recipient}
            </Text>
            <Text style={styles.meta}>
              Fee {item.fee_zec.toFixed(8)} ·{" "}
              {item.confirmations ?? 0} conf
            </Text>
          </Pressable>
        )}
      />
      <View style={styles.footer}>
        <Button label="Back to dashboard" variant="ghost" onPress={() => navigation.goBack()} />
      </View>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  list: { padding: spacing.lg, gap: spacing.sm },
  row: {
    backgroundColor: colors.surface,
    borderRadius: 12,
    padding: spacing.md,
    borderWidth: 1,
    borderColor: colors.border,
    marginBottom: spacing.sm,
  },
  rowTop: { flexDirection: "row", justifyContent: "space-between" },
  amount: { color: colors.text, fontWeight: "700", fontSize: fontSize.md },
  status: { color: colors.primary, fontSize: fontSize.sm },
  recipient: { color: colors.textMuted, fontSize: fontSize.sm, marginTop: 4 },
  meta: { color: colors.textMuted, fontSize: 12, marginTop: 4 },
  empty: { color: colors.textMuted, textAlign: "center", marginTop: spacing.xl },
  error: { color: colors.error, marginBottom: spacing.md },
  footer: { padding: spacing.lg },
});
