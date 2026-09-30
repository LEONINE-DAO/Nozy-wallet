import { StatusBar } from "expo-status-bar";
import { useEffect } from "react";
import { Linking } from "react-native";
import {
  DarkTheme,
  NavigationContainer,
  type Theme,
} from "@react-navigation/native";
import { SafeAreaProvider } from "react-native-safe-area-context";
import { WalletSessionProvider } from "./src/context/WalletSessionContext";
import { AppNavigator } from "./src/navigation/AppNavigator";
import { setPendingPaymentUri } from "./src/lib/pendingPaymentUri";
import { extractZip321Uri } from "./src/lib/zip321";
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
  useEffect(() => {
    const capture = (url: string | null) => {
      if (!url) return;
      const uri = extractZip321Uri(url) ?? (/^zcash:/i.test(url.trim()) ? url.trim() : null);
      if (uri) setPendingPaymentUri(uri);
    };
    void Linking.getInitialURL().then(capture);
    const sub = Linking.addEventListener("url", (event) => capture(event.url));
    return () => sub.remove();
  }, []);

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
