import * as Clipboard from "expo-clipboard";
import { useCallback, useEffect, useMemo, useState } from "react";
import {
  Alert,
  Pressable,
  ScrollView,
  Share,
  StyleSheet,
  Text,
  View,
} from "react-native";
import { NativeStackScreenProps } from "@react-navigation/native-stack";
import { SafeAreaView } from "react-native-safe-area-context";
import { Button } from "../components/Button";
import { Card } from "../components/Card";
import { Input } from "../components/Input";
import { PageHeader } from "../components/PageHeader";
import { useWalletSession } from "../context/WalletSessionContext";
import {
  atRiskDelegations,
  fetchHybridPosScoreboard,
  finalizerCrowdCopy,
  finalizerCrowdLabel,
  finalizerDisplayName,
  indexScoreboard,
  isValidFinalizerHex,
  normalizeFinalizerHex,
  retargetRiskLabel,
  type AtRiskDelegation,
  type HybridPosFinalizer,
} from "../lib/hybridPos";
import { api } from "../services/api";
import { colors, fontSize, spacing } from "../theme";
import type {
  CrosslinkGuardianSnapshot,
  CrosslinkNextAction,
  CrosslinkRosterEntry,
  RootStackParamList,
} from "../types";

type Props = NativeStackScreenProps<RootStackParamList, "Crosslink">;

const ZAT = 100_000_000;

function zatToCtaz(zat: number): string {
  return (zat / ZAT).toFixed(4);
}

function shortHex(hex: string, head = 8, tail = 6): string {
  const h = hex.trim();
  if (h.length <= head + tail + 1) return h;
  return `${h.slice(0, head)}…${h.slice(-tail)}`;
}

function formatNextAction(action: CrosslinkNextAction): string {
  if (typeof action === "string") {
    switch (action) {
      case "unbond_to_exit":
        return "Window open with active stake — unbond to start exit, or retarget anytime.";
      case "stake_or_guardian":
        return "Window open — stake to a finalizer, or back your own identity from the monolith UI.";
      case "retarget_if_needed":
        return "Window closed — you can still retarget if a finalizer misbehaves.";
      default:
        return "Refresh status for guidance.";
    }
  }
  if ("wait_for_staking_day" in action) {
    return `Wait ~${action.wait_for_staking_day.blocks} blocks for Staking Day, then stake / unbond / withdraw.`;
  }
  if ("withdraw_ready" in action) {
    return `${action.withdraw_ready.count} bond(s) ready to withdraw while the window is open.`;
  }
  return "Refresh status for guidance.";
}

function confirm(title: string, message: string): Promise<boolean> {
  return new Promise((resolve) => {
    Alert.alert(title, message, [
      { text: "Cancel", style: "cancel", onPress: () => resolve(false) },
      { text: "Confirm", onPress: () => resolve(true) },
    ]);
  });
}

export function CrosslinkScreen({}: Props) {
  const { password, backendMode } = useWalletSession();
  const [snap, setSnap] = useState<CrosslinkGuardianSnapshot | null>(null);
  const [roster, setRoster] = useState<CrosslinkRosterEntry[]>([]);
  const [hybridRows, setHybridRows] = useState<HybridPosFinalizer[]>([]);
  const [amount, setAmount] = useState("0.01");
  const [finalizer, setFinalizer] = useState("");
  const [bondKey, setBondKey] = useState("");
  const [force, setForce] = useState(false);
  const [payoutAddress, setPayoutAddress] = useState("");
  const [cutoffHeight, setCutoffHeight] = useState("");
  const [mobileUfvk, setMobileUfvk] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");

  const hybridByKey = useMemo(() => indexScoreboard(hybridRows), [hybridRows]);
  const atRisk = useMemo(
    () => atRiskDelegations(snap?.positions.active ?? {}, hybridRows),
    [snap, hybridRows],
  );

  const load = useCallback(async () => {
    if (backendMode === "on_device") return;
    setError("");
    try {
      const status = await api.crosslinkStatus();
      setSnap(status);
    } catch (e) {
      setSnap(null);
      setError(
        e instanceof Error
          ? e.message
          : "Crosslink companion unreachable. Point the API at a Season 1 node.",
      );
      return;
    }
    try {
      setRoster(await api.crosslinkRoster(false));
    } catch {
      setRoster([]);
    }
  }, [backendMode]);

  useEffect(() => {
    void load();
    const id = setInterval(() => void load(), 90_000);
    return () => clearInterval(id);
  }, [load]);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const rows = await fetchHybridPosScoreboard();
        if (!cancelled) setHybridRows(rows);
      } catch {
        if (!cancelled) setHybridRows([]);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  async function run(kind: string, fn: () => Promise<string>) {
    setBusy(kind);
    setError("");
    setMessage("");
    try {
      setMessage(await fn());
      await load();
    } catch (e) {
      const msg = e instanceof Error ? e.message : `${kind} failed`;
      if (msg === "Cancelled") setMessage("Cancelled");
      else setError(msg);
    } finally {
      setBusy(null);
    }
  }

  function prepareRetarget(item: AtRiskDelegation) {
    const pk = item.bondPks[0];
    if (pk) setBondKey(pk);
    if (item.suggestion) setFinalizer(item.suggestion.pubkey);
    setMessage(
      `Prepared retarget off ${retargetRiskLabel(item.reason)} finalizer. Confirm Retarget (allowed anytime).`,
    );
  }

  const dayOpen = snap?.staking_day.open === true;
  const activeCount = Object.values(snap?.positions.active ?? {}).reduce(
    (n, a) => n + a.length,
    0,
  );

  if (backendMode === "on_device") {
    return (
      <SafeAreaView style={styles.safe} edges={["bottom"]}>
        <ScrollView contentContainerStyle={styles.container}>
          <PageHeader
            title="Crosslink"
            description="Public finalizer scoreboard. This phone has no stake."
          />
          {hybridRows.length === 0 ? (
            <Text style={styles.muted}>No scoreboard rows yet. Pull to refresh later.</Text>
          ) : (
            <Card>
              <Text style={styles.kicker}>Finalizers · {hybridRows.length}</Text>
              {hybridRows.slice(0, 25).map((row) => (
                <View key={row.pubkey} style={styles.rosterRow}>
                  <Text style={styles.rosterTitle}>
                    #{row.rank} {row.name ?? shortHex(row.pubkey)}
                  </Text>
                  <Text style={styles.muted}>
                    Grade {row.grade ?? "—"}
                    {row.live ? " · live" : ""}
                  </Text>
                </View>
              ))}
            </Card>
          )}
        </ScrollView>
      </SafeAreaView>
    );
  }

  return (
    <SafeAreaView style={styles.safe} edges={["bottom"]}>
    <ScrollView contentContainerStyle={styles.container}>
      <PageHeader
        title="Crosslink"
        description="Protocol Guardian via your companion API. Feature-net cTAZ; node wallet signs. Needs a Season 1 Crosslink node on that companion."
      />

      <Button
        label={busy === "refresh" ? "Refreshing…" : "Refresh"}
        variant="secondary"
        size="sm"
        loading={busy === "refresh"}
        disabled={!!busy}
        onPress={() => {
          setBusy("refresh");
          setMessage("");
          void load().finally(() => setBusy(null));
        }}
      />

      {error ? <Text style={styles.error}>{error}</Text> : null}
      {message ? <Text style={styles.ok}>{message}</Text> : null}

      {snap ? (
        <Card>
          <Text style={styles.kicker}>
            {dayOpen ? "Staking Day open" : "Staking Day closed"}
          </Text>
          <Text style={styles.metric}>Height {snap.height.toLocaleString()}</Text>
          <Text style={styles.body}>{formatNextAction(snap.next_action)}</Text>
          <Text style={styles.muted}>
            {activeCount} active bonds · {snap.positions.withdrawable.length}{" "}
            withdrawable · earned {zatToCtaz(earnedZat(snap))} cTAZ
          </Text>
          {snap.wallet ? (
            <Text style={styles.muted}>
              Available to stake{" "}
              {zatToCtaz(
                snap.wallet.user_shielded_spendable_zats +
                  snap.wallet.user_unshielded_zats,
              )}{" "}
              cTAZ
            </Text>
          ) : (
            <Text style={styles.muted}>
              Spendable balance unavailable on this node build.
            </Text>
          )}
        </Card>
      ) : null}

      {atRisk.length > 0 ? (
        <Card>
          <Text style={styles.kicker}>Guardian — retarget</Text>
          <Text style={styles.body}>
            Bonds on offline or below-B− finalizers. Prepare fills the form;
            Confirm still goes through Retarget.
          </Text>
          {atRisk.map((item) => (
            <View key={item.finalizer} style={styles.riskRow}>
              <Text style={styles.body}>
                {retargetRiskLabel(item.reason)} · {shortHex(item.finalizer)}
                {item.suggestion
                  ? ` → ${finalizerDisplayName(item.suggestion) ?? shortHex(item.suggestion.pubkey)}`
                  : " · no safer pick"}
              </Text>
              <Button
                label="Prepare retarget"
                variant="secondary"
                size="sm"
                disabled={!!busy}
                onPress={() => prepareRetarget(item)}
              />
            </View>
          ))}
        </Card>
      ) : null}

      <Card>
        <Text style={styles.kicker}>Actions</Text>
        <Input
          label="Amount (cTAZ)"
          value={amount}
          onChangeText={setAmount}
          keyboardType="decimal-pad"
          editable={!busy}
        />
        <Input
          label="Finalizer (64 hex)"
          value={finalizer}
          onChangeText={(t) => setFinalizer(t.trim())}
          autoCapitalize="none"
          autoCorrect={false}
          editable={!busy}
        />
        <Input
          label="Bond pk"
          value={bondKey}
          onChangeText={(t) => setBondKey(t.trim())}
          autoCapitalize="none"
          autoCorrect={false}
          editable={!busy}
        />
        <Pressable
          onPress={() => setForce((v) => !v)}
          style={styles.forceRow}
          disabled={!!busy}
        >
          <Text style={styles.body}>
            {force ? "Force outside window: on" : "Force outside window: off"}
          </Text>
        </Pressable>
        <View style={styles.row}>
          <Button
            label="Stake"
            size="sm"
            disabled={!!busy}
            loading={busy === "stake"}
            onPress={() =>
              void run("stake", async () => {
                const amt = Number(amount);
                if (!Number.isFinite(amt) || amt <= 0) {
                  throw new Error("Amount must be a positive number");
                }
                if (!isValidFinalizerHex(finalizer)) {
                  throw new Error("Finalizer must be 64 hex characters");
                }
                const ok = await confirm(
                  "Stake",
                  `Create a ${amt} cTAZ bond to ${shortHex(finalizer)}? Node wallet signs.`,
                );
                if (!ok) throw new Error("Cancelled");
                const res = await api.crosslinkStake({
                  amount_ctaz: amt,
                  finalizer: normalizeFinalizerHex(finalizer),
                  force,
                });
                return `${res.action} submitted`;
              })
            }
          />
          <Button
            label="Retarget"
            variant="secondary"
            size="sm"
            disabled={!!busy}
            loading={busy === "retarget"}
            onPress={() =>
              void run("retarget", async () => {
                if (!bondKey.trim()) throw new Error("Bond pk required");
                if (!isValidFinalizerHex(finalizer)) {
                  throw new Error("Finalizer must be 64 hex characters");
                }
                const ok = await confirm(
                  "Retarget",
                  `Move bond ${shortHex(bondKey)} to ${shortHex(finalizer)}? Allowed anytime.`,
                );
                if (!ok) throw new Error("Cancelled");
                const res = await api.crosslinkRetarget({
                  bond: bondKey.trim(),
                  finalizer: normalizeFinalizerHex(finalizer),
                });
                return `${res.action} submitted`;
              })
            }
          />
        </View>
        <View style={styles.row}>
          <Button
            label="Unbond"
            variant="secondary"
            size="sm"
            disabled={!!busy}
            loading={busy === "unbond"}
            onPress={() =>
              void run("unbond", async () => {
                if (!bondKey.trim()) throw new Error("Bond pk required");
                const ok = await confirm(
                  "Unbond",
                  `Begin unbonding ${shortHex(bondKey)}? Needs Staking Day unless Force is on.`,
                );
                if (!ok) throw new Error("Cancelled");
                const res = await api.crosslinkUnbond({
                  bond: bondKey.trim(),
                  force,
                });
                return `${res.action} submitted`;
              })
            }
          />
          <Button
            label="Withdraw"
            variant="secondary"
            size="sm"
            disabled={!!busy}
            loading={busy === "withdraw"}
            onPress={() =>
              void run("withdraw", async () => {
                if (!bondKey.trim()) throw new Error("Bond pk required");
                const ok = await confirm(
                  "Withdraw",
                  `Withdraw ${shortHex(bondKey)}? Needs Staking Day unless Force is on.`,
                );
                if (!ok) throw new Error("Cancelled");
                const res = await api.crosslinkWithdraw({
                  bond: bondKey.trim(),
                  force,
                });
                return `${res.action} submitted`;
              })
            }
          />
        </View>
      </Card>

      <Card>
        <Text style={styles.kicker}>Payout claim pack</Text>
        <Text style={styles.muted}>
          Feature-net UFVK + mainnet u1… for Shielded Labs. Earned cTAZ, not bonded
          principal.
        </Text>
        <Input
          label="Mainnet Orchard payout (u1…)"
          value={payoutAddress}
          onChangeText={(t) => setPayoutAddress(t.trim())}
          autoCapitalize="none"
          autoCorrect={false}
          editable={!busy}
        />
        <Input
          label="Cutoff height (optional)"
          value={cutoffHeight}
          onChangeText={(t) => setCutoffHeight(t.trim())}
          keyboardType="number-pad"
          editable={!busy}
        />
        <Input
          label="Second UFVK (this phone / delegator)"
          value={mobileUfvk}
          onChangeText={setMobileUfvk}
          autoCapitalize="none"
          autoCorrect={false}
          multiline
          editable={!busy}
        />
        <Button
          label="Build + copy pack"
          size="sm"
          disabled={!!busy}
          loading={busy === "payout"}
          onPress={() =>
            void run("payout", async () => {
              const cutoffRaw = cutoffHeight.trim();
              const cutoff_height = cutoffRaw ? Number(cutoffRaw) : undefined;
              if (
                cutoffRaw &&
                (!Number.isInteger(cutoff_height) || (cutoff_height ?? 0) <= 0)
              ) {
                throw new Error("Cutoff height must be a positive whole number");
              }
              let address = payoutAddress.trim();
              if (!address) {
                try {
                  const addr = await api.generateAddress(password || undefined);
                  const got = (addr.address ?? "").trim();
                  if (got.startsWith("u1") && !got.startsWith("utest")) {
                    address = got;
                    setPayoutAddress(got);
                  }
                } catch {
                  /* pack still works without u1 */
                }
              }
              const pack = await api.crosslinkPayoutClaim({
                payout_address: address || undefined,
                mobile_ufvk: mobileUfvk.trim() || undefined,
                cutoff_height,
              });
              await Clipboard.setStringAsync(pack.paste_body);
              return pack.complete
                ? "Payout pack copied"
                : "Pack copied — add a mainnet u1… to finish";
            })
          }
        />
      </Card>

      <Card>
        <Text style={styles.kicker}>Diagnostics</Text>
        <Text style={styles.muted}>
          Tip, TFL lag, bonds, UFVK fingerprint. Does not unbond or withdraw.
        </Text>
        <Button
          label="Copy dump"
          variant="secondary"
          size="sm"
          disabled={!!busy}
          loading={busy === "doctor"}
          onPress={() =>
            void run("doctor", async () => {
              const report = await api.crosslinkDoctor(true);
              await Clipboard.setStringAsync(report.paste_body);
              try {
                await Share.share({
                  title: "Nozy × Crosslink doctor",
                  message: report.paste_body,
                });
              } catch {
                /* share optional */
              }
              return report.ok
                ? "Diagnostics copied"
                : "Diagnostics copied — check fail items";
            })
          }
        />
      </Card>

      {roster.length > 0 ? (
        <Card>
          <Text style={styles.kicker}>Roster · {roster.length}</Text>
          <Text style={styles.muted}>
            All finalizers · independent observer grade. Tap to paste. Prefer
            spread over the largest third.
          </Text>
          {roster.map((e, i) => {
            const row = hybridByKey.get(normalizeFinalizerHex(e.finalizer));
            const crowd = finalizerCrowdLabel(row);
            const name = finalizerDisplayName(row);
            const rank = row?.rank ?? i + 1;
            return (
              <Pressable
                key={e.finalizer}
                onPress={() => {
                  setFinalizer(e.finalizer);
                  setMessage("Finalizer pasted");
                }}
                style={styles.rosterRow}
              >
                <Text style={styles.rosterTitle}>
                  #{rank} {name ?? shortHex(e.finalizer)}
                </Text>
                <Text style={styles.muted}>
                  {zatToCtaz(e.stake_zat)} cTAZ · {(e.share * 100).toFixed(1)}%
                  {crowd ? ` · ${finalizerCrowdCopy(crowd)}` : ""}
                  {` · Grade ${row?.grade ?? "—"}`}
                </Text>
              </Pressable>
            );
          })}
        </Card>
      ) : null}
    </ScrollView>
    </SafeAreaView>
  );
}

function earnedZat(snap: CrosslinkGuardianSnapshot): number {
  const active = Object.values(snap.positions.active ?? {})
    .flat()
    .reduce((sum, b) => sum + Math.max(0, b.latest_val - b.initial_val), 0);
  const withdr = (snap.positions.withdrawable ?? []).reduce(
    (sum, b) => sum + Math.max(0, b.latest_val - b.initial_val),
    0,
  );
  return active + withdr;
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: colors.background },
  container: {
    padding: spacing.lg,
    paddingBottom: spacing.xl,
    gap: spacing.md,
    backgroundColor: colors.background,
  },
  kicker: {
    color: colors.primary,
    fontSize: 11,
    fontWeight: "800",
    letterSpacing: 1.4,
    textTransform: "uppercase",
    marginBottom: spacing.xs,
  },
  metric: {
    color: colors.text,
    fontSize: fontSize.lg,
    fontWeight: "800",
    marginBottom: 4,
  },
  body: {
    color: colors.text,
    fontSize: fontSize.sm,
    lineHeight: 20,
  },
  muted: {
    color: colors.textFaint,
    fontSize: 11,
    lineHeight: 16,
    marginTop: 4,
  },
  error: { color: colors.error, fontSize: fontSize.sm, lineHeight: 20 },
  ok: { color: colors.success, fontSize: fontSize.sm, lineHeight: 20 },
  row: { flexDirection: "row", gap: spacing.sm, marginTop: spacing.sm },
  forceRow: {
    paddingVertical: spacing.sm,
  },
  riskRow: {
    marginTop: spacing.sm,
    gap: spacing.xs,
  },
  rosterRow: {
    paddingVertical: 8,
    borderTopWidth: StyleSheet.hairlineWidth,
    borderTopColor: colors.border,
  },
  rosterTitle: {
    color: colors.text,
    fontSize: fontSize.sm,
    fontWeight: "600",
  },
});
