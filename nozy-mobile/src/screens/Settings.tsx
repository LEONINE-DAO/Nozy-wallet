import { NativeStackScreenProps } from "@react-navigation/native-stack";
import { useState } from "react";
import { ScrollView, StyleSheet, View } from "react-native";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "../components/Button";
import { PageHeader } from "../components/PageHeader";
import { AccountSettings } from "../components/settings/AccountSettings";
import { DisplaySettings } from "../components/settings/DisplaySettings";
import { LightClientSettings } from "../components/settings/LightClientSettings";
import { MobileConnectionSettings } from "../components/settings/MobileConnectionSettings";
import { OnDeviceWalletSettings } from "../components/settings/OnDeviceWalletSettings";
import { NetworkPrivacySettings } from "../components/settings/NetworkPrivacySettings";
import { NetworkSettings } from "../components/settings/NetworkSettings";
import { SettingsItem } from "../components/settings/SettingsItem";
import { SyncSettings } from "../components/settings/SyncSettings";
import { WalletsAccountsSettings } from "../components/settings/WalletsAccountsSettings";
import { enableExperimentalFeatures } from "../lib/buildProfile";
import { colors, spacing } from "../theme";
import type { RootStackParamList } from "../types";

type Props = NativeStackScreenProps<RootStackParamList, "Settings">;

type SettingsSection =
  | "main"
  | "network"
  | "privacy"
  | "mobile"
  | "lightclient"
  | "ondevice"
  | "display"
  | "sync"
  | "wallets"
  | "account";

export function SettingsScreen({ navigation }: Props) {
  const [section, setSection] = useState<SettingsSection>("main");
  const showExperimental = enableExperimentalFeatures();

  if (section === "network") {
    return <NetworkSettings onBack={() => setSection("main")} />;
  }
  if (section === "privacy") {
    return <NetworkPrivacySettings onBack={() => setSection("main")} />;
  }
  if (section === "mobile") {
    return <MobileConnectionSettings onBack={() => setSection("main")} />;
  }
  if (section === "lightclient") {
    return <LightClientSettings onBack={() => setSection("main")} />;
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
    <SafeAreaView style={styles.safe} edges={["bottom"]}>
      <ScrollView contentContainerStyle={styles.container}>
        <PageHeader
          title="Settings"
          description="Manage your wallet, network, and preferences"
        />
        <View style={styles.list}>
          {showExperimental ? (
            <>
              <SettingsItem
                title="On-device wallet (Phase 2)"
                description="Keys on phone — create, unlock, sync via nozy-ffi"
                onPress={() => setSection("ondevice")}
              />
              <SettingsItem
                title="Light client (experimental)"
                description="On-device LWD sync via zeaking-ffi"
                onPress={() => setSection("lightclient")}
              />
            </>
          ) : null}
          <SettingsItem
            title="Mobile connection"
            description="Own API (home/VPS) or Nozy hosted — sync needs Zebrad behind the API"
            onPress={() => setSection("mobile")}
          />
          <SettingsItem
            title="Account Information"
            description="Address, seed, private key, change password"
            onPress={() => setSection("account")}
          />
          <SettingsItem
            title="Wallets & Accounts"
            description="Switch profiles and mainnet / testnet wallets"
            onPress={() => setSection("wallets")}
          />
          <SettingsItem
            title="Network & Node"
            description="Configure Zebra RPC on the API server"
            onPress={() => setSection("network")}
          />
          <SettingsItem
            title="Network privacy"
            description="Local Zebrad defaults, Nym/Tor attestation"
            onPress={() => setSection("privacy")}
          />
          <SettingsItem
            title="Sync"
            description="Keep wallet near tip while unlocked"
            onPress={() => setSection("sync")}
          />
          <SettingsItem
            title="Display"
            description="Fiat equivalent and balance visibility"
            onPress={() => setSection("display")}
          />
          <SettingsItem
            title="Address book"
            description="Saved contacts for shielded sends"
            onPress={() => navigation.navigate("AddressBook")}
          />
          <SettingsItem
            title="Ironwood"
            description="NU6.3 readiness and Orchard migration"
            onPress={() => navigation.navigate("Ironwood")}
          />
          <SettingsItem
            title="Keystone wallet"
            description="Hardware wallet PCZT signing"
            onPress={() => navigation.navigate("Keystone")}
          />
          <SettingsItem
            title="About & privacy"
            description="Privacy policy and support links"
            onPress={() => navigation.navigate("About")}
          />
        </View>
        <Button label="Back" variant="ghost" onPress={() => navigation.goBack()} />
      </ScrollView>
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  container: { padding: spacing.lg, gap: spacing.md },
  list: { gap: spacing.sm },
});
