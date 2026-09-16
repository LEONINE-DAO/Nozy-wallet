import { NativeStackScreenProps } from "@react-navigation/native-stack";
import { Ionicons } from "@expo/vector-icons";
import { useCallback, useEffect, useLayoutEffect, useState } from "react";
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
import { Modal } from "../components/Modal";
import { useWalletSession } from "../context/WalletSessionContext";
import { api } from "../services/api";
import {
  addOnDeviceAddressBookEntry,
  listOnDeviceAddressBook,
  removeOnDeviceAddressBookEntry,
} from "../services/onDeviceAddressBook";
import { colors, fontSize, spacing } from "../theme";
import type { AddressBookEntry, RootStackParamList } from "../types";

type Props = NativeStackScreenProps<RootStackParamList, "AddressBook">;

export function AddressBookScreen({ navigation }: Props) {
  const { backendMode } = useWalletSession();
  const onDevice = backendMode === "on_device";
  const [entries, setEntries] = useState<AddressBookEntry[]>([]);
  const [name, setName] = useState("");
  const [address, setAddress] = useState("");
  const [notes, setNotes] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [showAdd, setShowAdd] = useState(false);
  const [error, setError] = useState("");

  const load = useCallback(async () => {
    setError("");
    try {
      const next = onDevice
        ? await listOnDeviceAddressBook()
        : await api.listAddressBook();
      setEntries(next);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load address book");
    } finally {
      setLoading(false);
    }
  }, [onDevice]);

  useEffect(() => {
    void load();
  }, [load]);

  const openAdd = useCallback(() => {
    setError("");
    setShowAdd(true);
  }, []);

  useLayoutEffect(() => {
    navigation.setOptions({
      headerRight: () => (
        <Pressable
          onPress={openAdd}
          hitSlop={12}
          accessibilityRole="button"
          accessibilityLabel="Add contact"
          style={styles.plusBtn}
        >
          <Ionicons name="add" size={28} color={colors.primary} />
        </Pressable>
      ),
    });
  }, [navigation, openAdd]);

  function resetForm() {
    setName("");
    setAddress("");
    setNotes("");
    setError("");
  }

  async function handleAdd() {
    if (!name.trim() || !address.trim()) {
      setError("Name and address are required");
      return;
    }
    setSaving(true);
    setError("");
    try {
      if (onDevice) {
        await addOnDeviceAddressBookEntry(
          name.trim(),
          address.trim(),
          notes.trim() || undefined,
        );
      } else {
        await api.addAddressBookEntry(
          name.trim(),
          address.trim(),
          notes.trim() || undefined,
        );
      }
      resetForm();
      setShowAdd(false);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to add contact");
    } finally {
      setSaving(false);
    }
  }

  async function handleRemove(entryName: string) {
    try {
      if (onDevice) {
        await removeOnDeviceAddressBookEntry(entryName);
      } else {
        await api.removeAddressBookEntry(entryName);
      }
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
        ListEmptyComponent={
          !loading ? (
            <View style={styles.emptyWrap}>
              <Text style={styles.empty}>No contacts yet.</Text>
              <Text style={styles.emptyHint}>Tap + to add an address.</Text>
            </View>
          ) : null
        }
        renderItem={({ item }) => (
          <View style={styles.row}>
            <Pressable
              style={styles.rowMain}
              onPress={() =>
                navigation.navigate("Main", {
                  screen: "Send",
                  params: { recipient: item.address },
                })
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
      <Modal
        visible={showAdd}
        onClose={() => {
          if (!saving) {
            resetForm();
            setShowAdd(false);
          }
        }}
        title="Add contact"
      >
        <View style={styles.form}>
          <Input
            label="Name"
            value={name}
            onChangeText={setName}
            placeholder="Alice"
            autoCapitalize="words"
          />
          <Input
            label="Address"
            value={address}
            onChangeText={setAddress}
            autoCapitalize="none"
            autoCorrect={false}
            placeholder="u1..."
          />
          <Input
            label="Notes (optional)"
            value={notes}
            onChangeText={setNotes}
            placeholder="Friend, exchange, etc."
          />
          {error ? <Text style={styles.error}>{error}</Text> : null}
          <Button
            label="Save contact"
            onPress={() => void handleAdd()}
            loading={saving}
            disabled={saving}
          />
        </View>
      </Modal>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  plusBtn: { paddingHorizontal: spacing.sm, marginRight: 4 },
  list: { padding: spacing.lg, flexGrow: 1 },
  emptyWrap: { alignItems: "center", marginTop: spacing.xl, gap: spacing.sm },
  empty: { color: colors.textMuted, textAlign: "center", fontSize: fontSize.md },
  emptyHint: { color: colors.textFaint, textAlign: "center", fontSize: fontSize.sm },
  form: { gap: spacing.md },
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
  error: { color: colors.error, fontSize: fontSize.sm },
});
