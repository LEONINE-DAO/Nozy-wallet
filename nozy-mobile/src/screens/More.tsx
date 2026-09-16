import { useNavigation } from "@react-navigation/native";
import type { NativeStackNavigationProp } from "@react-navigation/native-stack";
import { ScrollView, StyleSheet, Text } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { SettingsItem } from "../components/settings/SettingsItem";
import { PageHeader } from "../components/PageHeader";
import { useWalletSession } from "../context/WalletSessionContext";
import { colors, spacing } from "../theme";
import type { RootStackParamList } from "../types";

type Nav = NativeStackNavigationProp<RootStackParamList>;

export function MoreScreen() {
  const navigation = useNavigation<Nav>();
  const { clearPassword, backendMode } = useWalletSession();
  const onDevice = backendMode === "on_device";

  async function logout() {
    await clearPassword();
    navigation.reset({ index: 0, routes: [{ name: "Welcome" }] });
  }

  return (
    <SafeAreaView style={styles.safe} edges={["top"]}>
      <ScrollView contentContainerStyle={styles.container}>
        <PageHeader title="More" description="History and tools." />
        <SettingsItem
          title="Transaction history"
          onPress={() => navigation.navigate("TransactionHistory")}
        />
        <SettingsItem
          title="Address book"
          onPress={() => navigation.navigate("AddressBook")}
        />
        <SettingsItem
          title="Ironwood"
          onPress={() => navigation.navigate("Ironwood")}
        />
        <SettingsItem
          title="Crosslink"
          description="Finalizer scoreboard"
          onPress={() => navigation.navigate("Crosslink")}
        />
        <SettingsItem
          title="NU7 Vote"
          onPress={() => navigation.navigate("Vote")}
        />
        {onDevice ? null : (
          <SettingsItem
            title="Keystone wallet"
            onPress={() => navigation.navigate("Keystone")}
          />
        )}
        <SettingsItem
          title="About & privacy"
          onPress={() => navigation.navigate("About")}
        />
        <SettingsItem
          title="Log out"
          onPress={() => void logout()}
        />
        <Text style={styles.footer}>LEONINE DAO · NozyWallet</Text>
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  container: { padding: spacing.lg, paddingBottom: spacing.xl },
  footer: {
    color: colors.textFaint,
    fontSize: 11,
    textAlign: "center",
    marginTop: spacing.lg,
  },
});
