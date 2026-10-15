import { useEffect } from "react";
import { AppState } from "react-native";
import { createBottomTabNavigator } from "@react-navigation/bottom-tabs";
import { BottomTabBar } from "../components/BottomTabBar";
import { useWalletSession } from "../context/WalletSessionContext";
import { DashboardScreen } from "../screens/Dashboard";
import { MoreScreen } from "../screens/More";
import { ReceiveScreen } from "../screens/Receive";
import { SendScreen } from "../screens/Send";
import { SettingsScreen } from "../screens/Settings";
import {
  INCOMING_SYNC_MS,
  requestWalletSync,
} from "../services/walletAutoSync";
import { warmReceiveAddressCache } from "../services/onDeviceWallet";
import { colors } from "../theme";
import type { MainTabParamList } from "../types";

const Tab = createBottomTabNavigator<MainTabParamList>();

export function MainTabNavigator() {
  const { autoSync, backendMode, password } = useWalletSession();
  const onDevice = backendMode === "on_device";

  useEffect(() => {
    if (!autoSync) return;
    let cancelled = false;
    let timer: ReturnType<typeof setInterval> | undefined;
    let ready = false;
    const tick = () => {
      if (cancelled || !ready) return;
      void requestWalletSync({ onDevice, password }).catch(() => {});
    };
    void (async () => {
      if (onDevice) {
        try {
          await warmReceiveAddressCache();
        } catch {
          // Address can still be derived later. Sync should not wait forever.
        }
      }
      if (cancelled) return;
      ready = true;
      tick();
      timer = setInterval(tick, INCOMING_SYNC_MS);
    })();
    const sub = AppState.addEventListener("change", (state) => {
      if (state === "active") tick();
    });
    return () => {
      cancelled = true;
      if (timer) clearInterval(timer);
      sub.remove();
    };
  }, [autoSync, onDevice, password]);

  return (
    <Tab.Navigator
      tabBar={(props) => <BottomTabBar {...props} />}
      screenOptions={{
        headerShown: false,
        sceneStyle: { backgroundColor: colors.background },
      }}
    >
      <Tab.Screen name="Home" component={DashboardScreen} />
      <Tab.Screen name="Send" component={SendScreen} />
      <Tab.Screen name="Receive" component={ReceiveScreen} />
      <Tab.Screen name="Settings" component={SettingsScreen} />
      <Tab.Screen name="More" component={MoreScreen} />
    </Tab.Navigator>
  );
}
