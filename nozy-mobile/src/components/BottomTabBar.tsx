import { BottomTabBarProps } from "@react-navigation/bottom-tabs";
import { Pressable, StyleSheet, Text, View } from "react-native";
import { useSafeAreaInsets } from "react-native-safe-area-context";
import { TabIcon, type TabIconName } from "./TabIcon";
import { colors } from "../theme";

type TabKey = "Home" | "Send" | "Receive" | "Settings" | "More";

const TABS: Array<{
  name: TabKey;
  label: string;
  icon: TabIconName;
}> = [
  { name: "Home", label: "Home", icon: "home" },
  { name: "Send", label: "Send", icon: "send" },
  { name: "Receive", label: "Receive", icon: "receive" },
  { name: "Settings", label: "Settings", icon: "settings" },
  { name: "More", label: "More", icon: "more" },
];

export function BottomTabBar({ state, navigation }: BottomTabBarProps) {
  const insets = useSafeAreaInsets();

  return (
    <View style={[styles.wrap, { paddingBottom: Math.max(insets.bottom, 8) }]}>
      {TABS.map((tab) => {
        const routeIndex = state.routes.findIndex((r) => r.name === tab.name);
        if (routeIndex < 0) return null;
        const focused = state.index === routeIndex;
        const route = state.routes[routeIndex];

        return (
          <Pressable
            key={tab.name}
            accessibilityRole="button"
            accessibilityState={focused ? { selected: true } : {}}
            onPress={() => {
              const event = navigation.emit({
                type: "tabPress",
                target: route.key,
                canPreventDefault: true,
              });
              if (!focused && !event.defaultPrevented) {
                navigation.navigate(route.name);
              }
            }}
            style={styles.tab}
          >
            {focused ? <View style={styles.activeBar} /> : null}
            <View style={[styles.iconWrap, focused && styles.iconWrapActive]}>
              <TabIcon name={tab.icon} size={22} focused={focused} />
            </View>
            <Text style={[styles.label, focused && styles.labelActive]}>{tab.label}</Text>
          </Pressable>
        );
      })}
    </View>
  );
}

const styles = StyleSheet.create({
  wrap: {
    flexDirection: "row",
    borderTopWidth: 1,
    borderTopColor: colors.border,
    backgroundColor: colors.surface,
    paddingTop: 4,
  },
  tab: {
    flex: 1,
    alignItems: "center",
    justifyContent: "center",
    gap: 3,
    paddingVertical: 9,
    position: "relative",
  },
  activeBar: {
    position: "absolute",
    top: 0,
    left: "28%",
    right: "28%",
    height: 2,
    borderRadius: 2,
    backgroundColor: colors.neon,
    shadowColor: colors.neonGlow,
    shadowOffset: { width: 0, height: 0 },
    shadowOpacity: 1,
    shadowRadius: 8,
  },
  iconWrap: {
    alignItems: "center",
    justifyContent: "center",
  },
  iconWrapActive: {
    shadowColor: colors.neon,
    shadowOffset: { width: 0, height: 0 },
    shadowOpacity: 0.85,
    shadowRadius: 6,
  },
  label: {
    fontSize: 10,
    fontWeight: "600",
    color: colors.textFaint,
  },
  labelActive: {
    color: colors.neon,
  },
});
