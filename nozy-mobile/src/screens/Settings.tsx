import type { BottomTabScreenProps } from "@react-navigation/bottom-tabs";
import { useEffect, useState } from "react";
import { BackHandler, ScrollView, StyleSheet, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { PageHeader } from "../components/PageHeader";
import { AccountSettings } from "../components/settings/AccountSettings";
import { DisplaySettings } from "../components/settings/DisplaySettings";
import { MobileConnectionSettings } from "../components/settings/MobileConnectionSettings";
import { OnDeviceWalletSettings } from "../components/settings/OnDeviceWalletSettings";
import { NetworkPrivacySettings } from "../components/settings/NetworkPrivacySettings";
import { NetworkSettings } from "../components/settings/NetworkSettings";
import { SettingsItem } from "../components/settings/SettingsItem";
import { SyncSettings } from "../components/settings/SyncSettings";
import { WalletsAccountsSettings } from "../components/settings/WalletsAccountsSettings";
import { colors, spacing } from "../theme";
import type { MainTabParamList } from "../types";

type Props = BottomTabScreenProps<MainTabParamList, "Settings">;

type SettingsSection =
  | "main"
  | "network"
  | "privacy"
  | "mobile"
  | "ondevice"
  | "display"
  | "sync"
  | "wallets"
  | "account";

export function SettingsScreen({}: Props) {
  const [section, setSection] = useState<SettingsSection>("main");

  useEffect(() => {
    if (section === "main") return;
    const sub = BackHandler.addEventListener("hardwareBackPress", () => {
      setSection("main");
      return true;
    });
    return () => sub.remove();
  }, [section]);

  if (section === "network") {
    return <NetworkSettings onBack={() => setSection("main")} />;
  }
  if (section === "privacy") {
    return <NetworkPrivacySettings onBack={() => setSection("main")} />;
  }
  if (section === "mobile") {
    return <MobileConnectionSettings onBack={() => setSection("main")} />;
  }
  if (section === "ondevice") {
    return <OnDeviceWalletSettings onBack={() => setSection("main")} />;
  }
  if (section === "display") {
    return <DisplaySettings onBack={() => setSection("main")} />;
  }
  if (section === "sync") {
    return <SyncSettings onBack={() => setSection("main")} />;
  }
  if (section === "wallets") {
    return <WalletsAccountsSettings onBack={() => setSection("main")} />;
  }
  if (section === "account") {
    return <AccountSettings onBack={() => setSection("main")} />;
  }

  return (
    <SafeAreaView style={styles.safe} edges={["top"]}>
      <ScrollView contentContainerStyle={styles.container}>
        <PageHeader title="Settings" description="Connection, sync, and account." />
        <View style={styles.list}>
          <SettingsItem
            title="On-device wallet"
            onPress={() => setSection("ondevice")}
          />
          <SettingsItem
            title="Mobile connection"
            onPress={() => setSection("mobile")}
          />
          <SettingsItem
            title="Account"
            onPress={() => setSection("account")}
          />
          <SettingsItem
            title="Wallets & accounts"
            onPress={() => setSection("wallets")}
          />
          <SettingsItem
            title="Network & node"
            onPress={() => setSection("network")}
          />
          <SettingsItem
            title="Network privacy"
            onPress={() => setSection("privacy")}
          />
          <SettingsItem
            title="Sync"
            onPress={() => setSection("sync")}
          />
          <SettingsItem
            title="Display"
            onPress={() => setSection("display")}
          />
        </View>
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  container: { padding: spacing.lg, paddingBottom: spacing.xl },
  list: { gap: spacing.xs },
});
