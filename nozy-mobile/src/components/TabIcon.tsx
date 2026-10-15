import { StyleSheet, View } from "react-native";
import { colors } from "../theme";

export type TabIconName = "home" | "send" | "receive" | "settings" | "more";

type Props = {
  name: TabIconName;
  size?: number;
  focused?: boolean;
};

type GlyphProps = {
  size: number;
  color: string;
  filled: boolean;
};

export function TabIcon({ name, size = 22, focused = false }: Props) {
  const color = focused ? colors.neon : colors.textFaint;

  return (
    <View style={[styles.wrap, { width: size, height: size }]}>
      {focused ? <View style={styles.accentDot} /> : null}
      <Glyph name={name} size={size} color={color} filled={focused} />
    </View>
  );
}

function Glyph({ name, size, color, filled }: GlyphProps & { name: TabIconName }) {
  switch (name) {
    case "home":
      return <HomeGlyph size={size} color={color} filled={filled} />;
    case "send":
      return <ArrowGlyph size={size} color={color} down={false} />;
    case "receive":
      return <ArrowGlyph size={size} color={color} down />;
    case "settings":
      return <SettingsGlyph size={size} color={color} />;
    case "more":
      return <MoreGlyph size={size} color={color} filled={filled} />;
  }
}

function Triangle({
  width,
  height,
  color,
}: {
  width: number;
  height: number;
  color: string;
}) {
  return (
    <View
      collapsable={false}
      style={{
        width: 0,
        height: 0,
        borderLeftWidth: width / 2,
        borderRightWidth: width / 2,
        borderBottomWidth: height,
        borderLeftColor: "transparent",
        borderRightColor: "transparent",
        borderBottomColor: color,
      }}
    />
  );
}

function HomeGlyph({ size, color, filled }: GlyphProps) {
  const stroke = Math.max(1.6, size * 0.08);
  const roofW = size * 0.9;
  const roofH = size * 0.4;
  const bodyW = size * 0.56;
  const bodyH = size * 0.38;
  const doorW = Math.max(3, size * 0.16);
  const doorH = size * 0.18;
  const innerW = roofW - stroke * 3;
  const innerH = Math.max(1, roofH - stroke * 1.7);

  return (
    <View style={{ width: size, height: size, alignItems: "center", justifyContent: "flex-end" }}>
      <View style={{ width: roofW, height: roofH }}>
        <View style={{ position: "absolute", left: roofW / 2, top: 0 }}>
          <View style={{ position: "absolute", left: -roofW / 2, top: 0 }}>
            <Triangle width={roofW} height={roofH} color={color} />
          </View>
        </View>
        {!filled ? (
          <View style={{ position: "absolute", left: roofW / 2, top: stroke * 1.15 }}>
            <View style={{ position: "absolute", left: -innerW / 2, top: 0 }}>
              <Triangle width={innerW} height={innerH} color={colors.surface} />
            </View>
          </View>
        ) : null}
      </View>
      <View
        style={{
          width: bodyW,
          height: bodyH,
          marginTop: -1,
          borderColor: color,
          borderWidth: filled ? 0 : stroke,
          borderTopWidth: 0,
          backgroundColor: filled ? color : "transparent",
          alignItems: "center",
          justifyContent: "flex-end",
        }}
      >
        <View
          style={{
            width: doorW,
            height: doorH,
            backgroundColor: filled ? colors.surface : color,
          }}
        />
      </View>
    </View>
  );
}

function ArrowGlyph({ size, color, down }: { size: number; color: string; down: boolean }) {
  const headW = size * 0.58;
  const headH = size * 0.36;
  const shaftW = Math.max(2, size * 0.14);
  const shaftH = size * 0.46;

  return (
    <View
      style={{
        width: size,
        height: size,
        alignItems: "center",
        justifyContent: "center",
        transform: [{ rotate: down ? "180deg" : "0deg" }],
      }}
    >
      <Triangle width={headW} height={headH} color={color} />
      <View
        style={{
          width: shaftW,
          height: shaftH,
          marginTop: -1,
          borderRadius: shaftW / 2,
          backgroundColor: color,
        }}
      />
    </View>
  );
}

function SettingsGlyph({ size, color }: Pick<GlyphProps, "size" | "color">) {
  const stroke = Math.max(1.5, size * 0.075);
  const knob = size * 0.3;
  const offsets = [0.62, 0.16, 0.4];

  return (
    <View style={{ width: size, height: size, justifyContent: "space-between", paddingVertical: 1 }}>
      {offsets.map((offset) => (
        <View key={offset} style={{ height: knob, justifyContent: "center" }}>
          <View
            style={{
              height: stroke,
              borderRadius: stroke,
              backgroundColor: color,
            }}
          />
          <View
            style={{
              position: "absolute",
              left: (size - knob) * offset,
              width: knob,
              height: knob,
              borderRadius: knob / 2,
              backgroundColor: colors.surface,
              borderWidth: stroke,
              borderColor: color,
            }}
          />
        </View>
      ))}
    </View>
  );
}

function MoreGlyph({ size, color, filled }: GlyphProps) {
  const cell = size * 0.34;
  const gap = size * 0.12;
  const stroke = Math.max(1.5, size * 0.08);

  return (
    <View style={{ width: size, height: size, alignItems: "center", justifyContent: "center" }}>
      <View style={{ width: cell * 2 + gap, height: cell * 2 + gap, flexDirection: "row", flexWrap: "wrap", gap }}>
        {[0, 1, 2, 3].map((index) => (
          <View
            key={index}
            style={{
              width: cell,
              height: cell,
              borderRadius: 2,
              backgroundColor: filled ? color : "transparent",
              borderWidth: filled ? 0 : stroke,
              borderColor: color,
            }}
          />
        ))}
      </View>
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
