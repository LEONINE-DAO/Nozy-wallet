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
import { Input } from "../components/Input";
import { api } from "../services/api";
import { colors, fontSize, spacing } from "../theme";
import type { AddressBookEntry, RootStackParamList } from "../types";

type Props = NativeStackScreenProps<RootStackParamList, "AddressBook">;

export function AddressBookScreen({ navigation }: Props) {
  const [entries, setEntries] = useState<AddressBookEntry[]>([]);
  const [name, setName] = useState("");
  const [address, setAddress] = useState("");
  const [notes, setNotes] = useState("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  const load = useCallback(async () => {
    setError("");
    try {
      setEntries(await api.listAddressBook());
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load address book");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  async function handleAdd() {
    if (!name.trim() || !address.trim()) {
      setError("Name and address are required");
      return;
    }
    setError("");
    try {
      await api.addAddressBookEntry(name.trim(), address.trim(), notes.trim() || undefined);
      setName("");
      setAddress("");
      setNotes("");
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to add contact");
    }
  }

  async function handleRemove(entryName: string) {
    try {
      await api.removeAddressBookEntry(entryName);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to remove contact");
    }
  }

  return (
    <SafeAreaView style={styles.safe} edges={["bottom"]}>
      <FlatList
        data={entries}
        keyExtractor={(item) => item.name}
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
        ListHeaderComponent={
          <View style={styles.header}>
            <Text style={styles.title}>Add contact</Text>
            <Input label="Name" value={name} onChangeText={setName} placeholder="Alice" />
            <Input
              label="Address"
              value={address}
              onChangeText={setAddress}
              autoCapitalize="none"
              placeholder="u1..."
            />
            <Input
              label="Notes (optional)"
              value={notes}
              onChangeText={setNotes}
              placeholder="Friend, exchange, etc."
            />
            <Button label="Add to address book" onPress={() => void handleAdd()} />
            {error ? <Text style={styles.error}>{error}</Text> : null}
            <Text style={styles.section}>Saved contacts</Text>
          </View>
        }
        ListEmptyComponent={
          !loading ? <Text style={styles.empty}>No contacts yet.</Text> : null
        }
        renderItem={({ item }) => (
          <View style={styles.row}>
            <Pressable
              style={styles.rowMain}
              onPress={() =>
                navigation.navigate("Send", { recipient: item.address })
              }
            >
              <Text style={styles.name}>{item.name}</Text>
              <Text style={styles.address} numberOfLines={1}>
                {item.address}
              </Text>
            </Pressable>
            <Button
              label="Remove"
              variant="ghost"
              onPress={() => void handleRemove(item.name)}
            />
          </View>
        )}
      />
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  list: { padding: spacing.lg },
  header: { gap: spacing.md, marginBottom: spacing.md },
  title: { color: colors.text, fontSize: fontSize.lg, fontWeight: "700" },
  section: {
    color: colors.textMuted,
    fontSize: fontSize.sm,
    fontWeight: "600",
    marginTop: spacing.md,
  },
  row: {
    backgroundColor: colors.surface,
    borderRadius: 12,
    padding: spacing.md,
    borderWidth: 1,
    borderColor: colors.border,
    marginBottom: spacing.sm,
    gap: spacing.sm,
  },
  rowMain: { gap: 4 },
  name: { color: colors.text, fontWeight: "700", fontSize: fontSize.md },
  address: { color: colors.textMuted, fontSize: fontSize.sm },
  empty: { color: colors.textMuted, textAlign: "center" },
  error: { color: colors.error, fontSize: fontSize.sm },
});
