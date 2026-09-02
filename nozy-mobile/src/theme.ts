/**
 * Dark platinum shell — matches browser-extension/wasm-core/popup/src/styles.css
 * and desktop silver palette (not the old light/gold mobile theme).
 */
export const colors = {
  background: "#020b07",
  backgroundMid: "#0a1410",
  surface: "#0c1210",
  surfaceAlt: "#121a16",
  surfaceInset: "#080f0c",
  /** Back-compat alias used by older components */
  surfaceLight: "#121a16",
  primary: "#c8ccd4",
  primaryHover: "#b0b5c0",
  primaryText: "#18181b",
  primarySoft: "rgba(200, 204, 212, 0.12)",
  platinumLine: "rgba(200, 204, 212, 0.32)",
  platinumGlow: "rgba(200, 204, 212, 0.16)",
  text: "#e8eaed",
  textMuted: "#8b919c",
  textFaint: "#6e7480",
  border: "rgba(255, 255, 255, 0.08)",
  borderStrong: "rgba(255, 255, 255, 0.14)",
  success: "#22c55e",
  successSoft: "rgba(34, 197, 94, 0.14)",
  error: "#ef4444",
  warn: "#f59e0b",
  warnBg: "rgba(245, 158, 11, 0.14)",
  /** Cyberpunk tab / accent (matches home logo palette) */
  neon: "#39ff9f",
  neonGlow: "rgba(57, 255, 159, 0.5)",
  cyberPurple: "#b24bff",
  cyberPurpleGlow: "rgba(178, 75, 255, 0.4)",
};

export const radius = {
  sm: 10,
  md: 14,
  lg: 18,
  xl: 20,
  xxl: 24,
};

export const spacing = {
  xs: 4,
  sm: 8,
  md: 16,
  lg: 24,
  xl: 32,
};

export const fontSize = {
  xs: 12,
  sm: 14,
  md: 16,
  lg: 20,
  xl: 28,
  xxl: 36,
};

export const shadows = {
  card: {
    shadowColor: "#000",
    shadowOffset: { width: 0, height: 8 },
    shadowOpacity: 0.35,
    shadowRadius: 28,
    elevation: 6,
  },
  button: {
    shadowColor: "rgba(200, 204, 212, 0.2)",
    shadowOffset: { width: 0, height: 4 },
    shadowOpacity: 1,
    shadowRadius: 14,
    elevation: 4,
  },
};
