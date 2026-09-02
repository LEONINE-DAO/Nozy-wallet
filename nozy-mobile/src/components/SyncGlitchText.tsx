import { useEffect, useRef } from "react";
import {
  Animated,
  StyleSheet,
  Text,
  type StyleProp,
  type TextStyle,
  View,
} from "react-native";
import { syncCyber } from "../syncCyberpunk";

type Props = {
  children: string;
  style?: StyleProp<TextStyle>;
  numberOfLines?: number;
};

export function SyncGlitchText({ children, style, numberOfLines = 2 }: Props) {
  const flicker = useRef(new Animated.Value(1)).current;
  const shiftA = useRef(new Animated.Value(-2)).current;
  const shiftB = useRef(new Animated.Value(2)).current;

  useEffect(() => {
    const flickerAnim = Animated.loop(
      Animated.sequence([
        Animated.delay(2600),
        Animated.timing(flicker, { toValue: 0.65, duration: 60, useNativeDriver: true }),
        Animated.timing(flicker, { toValue: 1, duration: 60, useNativeDriver: true }),
        Animated.timing(flicker, { toValue: 0.5, duration: 50, useNativeDriver: true }),
        Animated.timing(flicker, { toValue: 1, duration: 80, useNativeDriver: true }),
      ]),
    );
    const glitchA = Animated.loop(
      Animated.sequence([
        Animated.timing(shiftA, { toValue: 2, duration: 420, useNativeDriver: true }),
        Animated.timing(shiftA, { toValue: -2, duration: 420, useNativeDriver: true }),
      ]),
    );
    const glitchB = Animated.loop(
      Animated.sequence([
        Animated.timing(shiftB, { toValue: -2, duration: 360, useNativeDriver: true }),
        Animated.timing(shiftB, { toValue: 2, duration: 360, useNativeDriver: true }),
      ]),
    );
    flickerAnim.start();
    glitchA.start();
    glitchB.start();
    return () => {
      flickerAnim.stop();
      glitchA.stop();
      glitchB.stop();
    };
  }, [flicker, shiftA, shiftB]);

  const base = StyleSheet.flatten([styles.base, style]);

  return (
    <View style={styles.wrap}>
      <Text style={[base, styles.measure]} numberOfLines={numberOfLines}>
        {children}
      </Text>
      <View pointerEvents="none" style={styles.layer}>
        <Animated.Text
          numberOfLines={numberOfLines}
          style={[
            base,
            { color: syncCyber.glitchA, opacity: 0.55, transform: [{ translateX: shiftA }] },
          ]}
        >
          {children}
        </Animated.Text>
      </View>
      <View pointerEvents="none" style={styles.layer}>
        <Animated.Text
          numberOfLines={numberOfLines}
          style={[
            base,
            { color: syncCyber.glitchB, opacity: 0.45, transform: [{ translateX: shiftB }] },
          ]}
        >
          {children}
        </Animated.Text>
      </View>
      <Animated.Text
        numberOfLines={numberOfLines}
        style={[
          styles.layer,
          base,
          styles.main,
          { opacity: flicker },
        ]}
      >
        {children}
      </Animated.Text>
    </View>
  );
}

const styles = StyleSheet.create({
  wrap: {
    position: "relative",
    alignSelf: "stretch",
  },
  measure: {
    opacity: 0,
  },
  layer: {
    position: "absolute",
    left: 0,
    right: 0,
    top: 0,
  },
  base: {
    fontFamily: "monospace",
    letterSpacing: 0.6,
  },
  main: {
    color: syncCyber.headline,
    textShadowColor: "rgba(0, 255, 140, 0.45)",
    textShadowOffset: { width: 0, height: 0 },
    textShadowRadius: 8,
  },
});
