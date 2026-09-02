import { Ionicons } from "@expo/vector-icons";
import { View, StyleSheet } from "react-native";
import { colors } from "../theme";

export type TabIconName = "home" | "send" | "receive" | "settings" | "more";

type Props = {
  name: TabIconName;
  size?: number;
  focused?: boolean;
};

const ICONS: Record<
  TabIconName,
  { outline: keyof typeof Ionicons.glyphMap; filled: keyof typeof Ionicons.glyphMap }
> = {
  home: { outline: "home-outline", filled: "home" },
  send: { outline: "arrow-up-outline", filled: "arrow-up" },
  receive: { outline: "arrow-down-outline", filled: "arrow-down" },
  settings: { outline: "settings-outline", filled: "settings" },
  more: { outline: "grid-outline", filled: "grid" },
};

export function TabIcon({ name, size = 22, focused = false }: Props) {
  const icon = ICONS[name];
  const glyph = focused ? icon.filled : icon.outline;
  const color = focused ? colors.neon : colors.textFaint;

  return (
    <View style={styles.wrap}>
      {focused ? <View style={styles.accentDot} /> : null}
      <Ionicons name={glyph} size={size} color={color} />
    </View>
  );
}

const styles = StyleSheet.create({
  wrap: {
    alignItems: "center",
    justifyContent: "center",
    position: "relative",
  },
  accentDot: {
    position: "absolute",
    top: -1,
    right: -2,
    width: 5,
    height: 5,
    borderRadius: 3,
    backgroundColor: colors.cyberPurple,
  },
});
