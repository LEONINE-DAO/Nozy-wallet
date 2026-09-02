import { ActivityIndicator, StyleSheet, Text, View } from "react-native";
import { syncCyber } from "../syncCyberpunk";
import { CyberpunkSyncBar } from "./CyberpunkSyncBar";
import { SyncGlitchText } from "./SyncGlitchText";

export type CyberpunkSyncTone = "syncing" | "ok" | "warn" | "offline";

type Props = {
  headline: string;
  detail?: string;
  percent?: number | null;
  tone?: CyberpunkSyncTone;
  indeterminate?: boolean;
  showSpinner?: boolean;
  footer?: string;
};

export function CyberpunkSyncPanel({
  headline,
  detail,
  percent = null,
  tone = "syncing",
  indeterminate = false,
  showSpinner = false,
  footer,
}: Props) {
  const borderColor =
    tone === "warn"
      ? syncCyber.warnBorder
      : tone === "offline"
        ? syncCyber.offlineBorder
        : syncCyber.border;

  return (
    <View style={[styles.panel, { borderColor }]} accessibilityLiveRegion="polite">
      <View style={styles.row}>
        {showSpinner ? (
          <ActivityIndicator color={syncCyber.barFill} size="small" style={styles.spinner} />
        ) : null}
        <View style={styles.body}>
          <Text style={styles.eyebrow}>CHAIN SYNC</Text>
          <SyncGlitchText style={styles.headline} numberOfLines={2}>
            {headline}
          </SyncGlitchText>
          {detail ? (
            <SyncGlitchText style={styles.detail} numberOfLines={2}>
              {detail}
            </SyncGlitchText>
          ) : null}
        </View>
        {percent != null && !indeterminate ? (
          <SyncGlitchText style={styles.percent}>{`${Math.floor(percent)}%`}</SyncGlitchText>
        ) : null}
      </View>
      {indeterminate || percent != null ? (
        <CyberpunkSyncBar percent={percent} indeterminate={indeterminate} />
      ) : null}
      {footer ? <Text style={styles.footer}>{footer}</Text> : null}
    </View>
  );
}

const styles = StyleSheet.create({
  panel: {
    borderRadius: 14,
    borderWidth: 1,
    backgroundColor: syncCyber.panelBg,
    padding: 16,
    gap: 10,
    shadowColor: "rgba(0, 255, 140, 0.2)",
    shadowOffset: { width: 0, height: 0 },
    shadowOpacity: 1,
    shadowRadius: 14,
    elevation: 3,
  },
  row: {
    flexDirection: "row",
    alignItems: "flex-start",
    gap: 10,
  },
  spinner: {
    marginTop: 18,
  },
  body: {
    flex: 1,
    minWidth: 0,
    gap: 4,
  },
  eyebrow: {
    color: syncCyber.label,
    fontSize: 10,
    fontWeight: "700",
    letterSpacing: 2.2,
  },
  headline: {
    fontSize: 14,
    fontWeight: "700",
  },
  detail: {
    fontSize: 12,
    fontWeight: "500",
  },
  percent: {
    fontSize: 18,
    fontWeight: "800",
    marginTop: 12,
    minWidth: 48,
    textAlign: "right",
  },
  footer: {
    color: syncCyber.detail,
    fontSize: 12,
    lineHeight: 18,
    marginTop: 2,
  },
});
