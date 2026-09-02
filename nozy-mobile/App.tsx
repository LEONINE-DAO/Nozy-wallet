import { StatusBar } from "expo-status-bar";
import {
  DarkTheme,
  NavigationContainer,
  type Theme,
} from "@react-navigation/native";
import { SafeAreaProvider } from "react-native-safe-area-context";
import { WalletSessionProvider } from "./src/context/WalletSessionContext";
import { AppNavigator } from "./src/navigation/AppNavigator";
import { colors } from "./src/theme";

const NozyNavTheme: Theme = {
  ...DarkTheme,
  colors: {
    ...DarkTheme.colors,
    primary: colors.primary,
    background: colors.background,
    card: colors.surface,
    text: colors.text,
    border: colors.border,
    notification: colors.primary,
  },
};

export default function App() {
  return (
    <SafeAreaProvider>
      <WalletSessionProvider>
        <NavigationContainer theme={NozyNavTheme}>
          <StatusBar style="light" />
          <AppNavigator />
        </NavigationContainer>
      </WalletSessionProvider>
    </SafeAreaProvider>
  );
}
