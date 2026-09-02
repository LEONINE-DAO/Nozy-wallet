import { createNativeStackNavigator } from "@react-navigation/native-stack";
import { AboutScreen } from "../screens/About";
import { AddressBookScreen } from "../screens/AddressBook";
import { CreateWalletScreen } from "../screens/CreateWallet";
import { KeystoneScreen } from "../screens/Keystone";
import { MnemonicBackupScreen } from "../screens/MnemonicBackup";
import { RestoreWalletScreen } from "../screens/RestoreWallet";
import { TransactionDetailScreen } from "../screens/TransactionDetail";
import { TransactionHistoryScreen } from "../screens/TransactionHistory";
import { UnlockScreen } from "../screens/Unlock";
import { WelcomeScreen } from "../screens/Welcome";
import { IronwoodScreen } from "../screens/Ironwood";
import { VoteScreen } from "../screens/Vote";
import { colors } from "../theme";
import type { RootStackParamList } from "../types";
import { MainTabNavigator } from "./MainTabNavigator";

const Stack = createNativeStackNavigator<RootStackParamList>();

export function AppNavigator() {
  return (
    <Stack.Navigator
      initialRouteName="Welcome"
      screenOptions={{
        headerStyle: { backgroundColor: colors.surface },
        headerTintColor: colors.text,
        headerTitleStyle: { fontWeight: "700" },
        contentStyle: { backgroundColor: colors.background },
      }}
    >
      <Stack.Screen
        name="Welcome"
        component={WelcomeScreen}
        options={{ headerShown: false }}
      />
      <Stack.Screen
        name="CreateWallet"
        component={CreateWalletScreen}
        options={{ title: "Create wallet" }}
      />
      <Stack.Screen
        name="MnemonicBackup"
        component={MnemonicBackupScreen}
        options={{ title: "Backup seed", headerBackVisible: false }}
      />
      <Stack.Screen
        name="RestoreWallet"
        component={RestoreWalletScreen}
        options={{ title: "Restore wallet" }}
      />
      <Stack.Screen
        name="Unlock"
        component={UnlockScreen}
        options={{ title: "Unlock", headerBackVisible: false }}
      />
      <Stack.Screen
        name="Main"
        component={MainTabNavigator}
        options={{ headerShown: false }}
      />
      <Stack.Screen
        name="TransactionHistory"
        component={TransactionHistoryScreen}
        options={{ title: "History" }}
      />
      <Stack.Screen
        name="TransactionDetail"
        component={TransactionDetailScreen}
        options={{ title: "Transaction" }}
      />
      <Stack.Screen
        name="About"
        component={AboutScreen}
        options={{ title: "About & privacy" }}
      />
      <Stack.Screen
        name="AddressBook"
        component={AddressBookScreen}
        options={{ title: "Address book" }}
      />
      <Stack.Screen
        name="Keystone"
        component={KeystoneScreen}
        options={{ title: "Keystone" }}
      />
      <Stack.Screen
        name="Ironwood"
        component={IronwoodScreen}
        options={{ title: "Ironwood" }}
      />
      <Stack.Screen
        name="Vote"
        component={VoteScreen}
        options={{ title: "NU7 Vote" }}
      />
    </Stack.Navigator>
  );
}
