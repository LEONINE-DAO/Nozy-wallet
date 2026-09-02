import { useEffect, useRef } from "react";
import { Animated, StyleSheet, View } from "react-native";
import { syncCyber } from "../syncCyberpunk";

type Props = {
  percent?: number | null;
  indeterminate?: boolean;
};

export function CyberpunkSyncBar({ percent, indeterminate = false }: Props) {
  const pulse = useRef(new Animated.Value(0)).current;
  const clamped =
    percent != null && Number.isFinite(percent)
      ? Math.min(100, Math.max(0, percent))
      : null;

  useEffect(() => {
    if (!indeterminate) return;
    const loop = Animated.loop(
      Animated.sequence([
        Animated.timing(pulse, { toValue: 1, duration: 1400, useNativeDriver: false }),
        Animated.timing(pulse, { toValue: 0, duration: 1400, useNativeDriver: false }),
      ]),
    );
    loop.start();
    return () => loop.stop();
  }, [indeterminate, pulse]);

  const indeterminateWidth = pulse.interpolate({
    inputRange: [0, 1],
    outputRange: ["18%", "82%"],
  });

  return (
    <View style={styles.track} accessibilityRole="progressbar">
      {indeterminate ? (
        <Animated.View style={[styles.fill, { width: indeterminateWidth }]} />
      ) : clamped != null ? (
        <View style={[styles.fill, { width: `${clamped}%` }]} />
      ) : null}
    </View>
  );
}

const styles = StyleSheet.create({
  track: {
    height: 8,
    borderRadius: 999,
    overflow: "hidden",
    backgroundColor: syncCyber.barTrack,
  },
  fill: {
    height: "100%",
    borderRadius: 999,
    backgroundColor: syncCyber.barFill,
    shadowColor: syncCyber.barGlow,
    shadowOffset: { width: 0, height: 0 },
    shadowOpacity: 1,
    shadowRadius: 8,
    elevation: 4,
  },
});
