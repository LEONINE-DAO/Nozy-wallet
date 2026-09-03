import { useEffect, useMemo, useState } from "react";
import {
  extensionApi,
  getCompanionPrefs,
  setCompanionPrefs,
  type PendingApproval,
  type TxStateEntry,
  type WalletScanProgressResult,
  type WalletStatus
} from "./lib/extensionApi";
import QRCode from "react-qr-code";
import { AppHeader } from "./components/AppHeader";
import { BottomNav, MORE_VIEWS, MoreSheet } from "./components/BottomNav";
import { FullWalletShell } from "./components/FullWalletShell";
import { NetworkPrivacyPanel, SendEgressBadge } from "./components/NetworkPrivacyPanel";
import { BrowserView, NymVpnPromoCard } from "./components/BrowserView";
import { NodeConnectCard } from "./components/NodeConnectCard";
import { VoteView } from "./components/VoteView";
import { CrosslinkView, HomeStakedPanel } from "./components/CrosslinkView";
import {
  Button,
  Callout,
  Card,
  CopyButton,
  EmptyState,
  Eyebrow,
  Hint,
  Icon,
  Input,
  ListRow,
  LogBlock,
  PageHeader,
  Pill,
  CyberpunkSyncPanel,
  Screen,
  SectionTitle,
  SegmentedControl,
  StatRow,
  Textarea
} from "./components/ui";
import {
  isScanInProgress,
  scanPercentDisplay,
  scanPercentLabel,
  scanRateLabel
} from "./lib/scanFormat";
import { useUiStore } from "./store/uiStore";
import {
  isLikelyZnsName,
  isUnifiedZcashAddress,
  normalizeUnifiedAddress,
  resolveSendRecipient,
  type ZnsRegistration
} from "./lib/zns";
import zecMark from "./assets/zec.svg";
import { fiatForZec, formatFiat, useZecFiatPrice } from "./lib/zecPrice";
import { DEFAULT_RPC } from "./lib/nodeConnect";
import { isFullPage, openWalletPage, viewFromUrl } from "./lib/walletPage";

/** Local lightwalletd only — public hosts (e.g. zec.rocks) are not offered here; they cannot scan blocks. */
const DEFAULT_LWD_URL = "http://127.0.0.1:9067";

function WelcomeView({
  onCreated,
  onRestored
}: {
  onCreated: () => void;
  onRestored: (address: string) => void;
}) {
  const [password, setPassword] = useState("");
  const [mnemonic, setMnemonic] = useState("");
  const [restoreBirthday, setRestoreBirthday] = useState("");
  const [mode, setMode] = useState<"restore" | "create">("restore");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [nodeConnected, setNodeConnected] = useState(false);

  const submit = async () => {
    if (!nodeConnected) {
      setError("Connect your Zebrad node first — use Find my node above.");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      if (mode === "create") {
        await extensionApi.walletCreate(password);
        onCreated();
        return;
      }
      let restoreOpts: { birthdayHeight: number } | undefined;
      const rb = restoreBirthday.trim().replace(/,/g, "");
      if (rb) {
        const n = Number(rb);
        if (!Number.isFinite(n) || n < 0 || !Number.isInteger(n)) {
          setError("Optional birthday must be a non-negative integer (block height).");
          setBusy(false);
          return;
        }
        restoreOpts = { birthdayHeight: n };
      }
      const restored = await extensionApi.walletRestore(mnemonic.trim(), password, restoreOpts);
      onRestored(restored.address);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setBusy(false);
    }
  };

  const canSubmit =
    nodeConnected && !busy && !!password && (mode !== "restore" || !!mnemonic.trim());

  return (
    <Screen>
      <div className="pt-1 text-center">
        <img className="nw-hero-logo nw-hero-logo--welcome" src="./logo.jpg" alt="Nozy Wallet" />
        <h1 className="nw-title mt-3">Privacy by default</h1>
        <Hint className="mt-1.5">
          Restore your recovery phrase, or create a new wallet.
        </Hint>
      </div>

      <NodeConnectCard
        variant="welcome"
        onStateChange={(s) => setNodeConnected(s.connected)}
      />

      <SegmentedControl
        value={mode}
        onChange={setMode}
        options={[
          { value: "restore", label: "Restore" },
          { value: "create", label: "Create" }
        ]}
      />

      {mode === "restore" && (
        <>
          <Textarea
            label="Recovery phrase"
            rows={3}
            placeholder="Enter your 24-word mnemonic"
            value={mnemonic}
            onChange={(e) => setMnemonic(e.target.value)}
            disabled={!nodeConnected}
          />
          <Input
            label="Birthday height (optional)"
            hint="Leave blank to scan from block 3,050,000 (same floor as Desktop). Only set this if your notes are older."
            mono
            placeholder="3050000"
            value={restoreBirthday}
            onChange={(e) => setRestoreBirthday(e.target.value)}
            disabled={!nodeConnected}
          />
        </>
      )}

      {mode === "create" && (
        <Callout>
          Create makes a new recovery phrase. Restore instead if you already have one.
        </Callout>
      )}

      <Input
        label="Password"
        type="password"
        placeholder="Encrypts this copy on Chrome"
        value={password}
        onChange={(e) => setPassword(e.target.value)}
        disabled={!nodeConnected}
      />

      {error && <Callout tone="danger">{error}</Callout>}

      <Button variant="primary" fullWidth onClick={submit} disabled={!canSubmit}>
        {busy
          ? "Working…"
          : !nodeConnected
            ? "Connect node to continue"
            : mode === "create"
              ? "Create wallet"
              : "Restore wallet"}
      </Button>

      <Hint className="text-center">
        Write the recovery phrase down offline. Nobody can recover it for you.
      </Hint>
    </Screen>
  );
}

function UnlockView({ onUnlocked }: { onUnlocked: () => void }) {
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const unlock = async () => {
    setBusy(true);
    setError(null);
    try {
      await extensionApi.walletUnlock(password);
      onUnlocked();
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setBusy(false);
    }
  };

  return (
    <Screen className="pt-4">
      <div className="text-center">
        <img className="nw-hero-logo" src="./logo.jpg" alt="Nozy Wallet" />
        <h1 className="nw-title mt-3">Welcome back</h1>
        <Hint className="mt-1">Unlock NozyWallet to see your shielded balance.</Hint>
      </div>
      <Input
        label="Password"
        type="password"
        autoFocus
        placeholder="Wallet password"
        value={password}
        onChange={(e) => setPassword(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter" && password && !busy) void unlock();
        }}
      />
      {error && <Callout tone="danger">{error}</Callout>}
      <Button variant="primary" fullWidth onClick={unlock} disabled={!password || busy}>
        {busy ? "Unlocking…" : "Unlock"}
      </Button>
    </Screen>
  );
}

function zatsToZec(zats: number): string {
  return (zats / 1e8).toFixed(8);
}

function shortAddress(addr: string): string {
  const a = addr.trim();
  if (!a) return "No address yet";
  if (a.length <= 20) return a;
  return `${a.slice(0, 8)}…${a.slice(-6)}`;
}

function DashboardView({
  status,
  txs,
  onRetry,
  onSpeedUp,
  scan,
  pageMode = false
}: {
  status: WalletStatus | null;
  txs: TxStateEntry[];
  onRetry: (id: string) => Promise<void>;
  onSpeedUp: (id: string) => Promise<void>;
  scan: WalletScanProgressResult | null;
  pageMode?: boolean;
}) {
  const setView = useUiStore((s) => s.setView);
  const { currency: fiatCode, rate: zecFiat } = useZecFiatPrice();
  const [homeTab, setHomeTab] = useState<"available" | "staked">("available");

  const syncPill = useMemo<{ label: string; tone: "neutral" | "accent" | "success" | "danger" } | null>(() => {
    const p = scan;
    if (p?.status === "stopped") return { label: `Paused ${scanPercentLabel(p)}%`, tone: "neutral" };
    if (p?.status === "scanning" && p.sessionWaitingSince)
      return { label: "Unlock to resume sync", tone: "neutral" };
    if (p?.status === "failed") return { label: "Sync failed", tone: "danger" };
    return null;
  }, [scan]);

  const scanDetail = useMemo(() => {
    const p = scan;
    if (p?.status === "scanning" && p.sessionWaitingSince)
      return "Sync is paused — unlock your wallet and it will resume automatically.";
    if (p?.status === "failed") return p.scanError || "Sync failed.";
    if (p?.status === "stopped") return "Sync stopped before reaching the chain tip. Resume from Settings.";
    if (!p || p.status === "idle") return "Run a sync from Settings to load your notes.";
    return "";
  }, [scan]);

  const poolBalances = useMemo(() => {
    const orchard = scan?.orchardBalanceZats ?? 0;
    const ironwood = scan?.ironwoodBalanceZats ?? 0;
    const sapling = scan?.saplingBalanceZats ?? 0;
    const total = scan?.totalBalanceZats ?? orchard + ironwood + sapling;
    return { total, orchard, ironwood, sapling };
  }, [scan]);

  const totalZec = scan ? poolBalances.total / 1e8 : null;
  const balanceFiat = totalZec != null ? fiatForZec(totalZec, zecFiat, fiatCode) : null;
  const unitPrice = zecFiat != null ? `1 ZEC ≈ ${formatFiat(zecFiat, fiatCode)}` : null;
  const recentTxs = txs.slice(-3).reverse();
  const address = status?.address || "";

  const assets = [
    {
      id: "orchard",
      label: "Orchard",
      hint: "Shielded · spendable",
      zats: poolBalances.orchard
    },
    {
      id: "ironwood",
      label: "Ironwood",
      hint: "Current send pool",
      zats: poolBalances.ironwood
    },
    {
      id: "sapling",
      label: "Sapling",
      hint: "Legacy · shield to spend",
      zats: poolBalances.sapling
    }
  ] as const;

  return (
    <Screen className={pageMode ? "max-w-5xl px-6 pt-5" : undefined}>
      {!pageMode && (
      <SegmentedControl
        options={[
          { value: "available", label: "Available" },
          { value: "staked", label: "Staked" }
        ]}
        value={homeTab}
        onChange={setHomeTab}
      />
      )}

      {!pageMode && homeTab === "staked" ? (
        <HomeStakedPanel
          onManage={(hex) => {
            if (isFullPage()) {
              if (hex) useUiStore.getState().setPendingFinalizer(hex);
              setView("crosslink");
              return;
            }
            void openWalletPage({ view: "crosslink", finalizer: hex });
          }}
        />
      ) : (
        <>
      <div className="flex items-center justify-between gap-2">
        <div className="flex min-w-0 items-center gap-2">
          <img className="nw-zec-ticker__icon" src={zecMark} alt="" />
          <div className="min-w-0">
            <Eyebrow>Available</Eyebrow>
            <p className="nw-mono truncate text-[11px]" style={{ color: "var(--nw-muted)" }}>
              {shortAddress(address)}
            </p>
          </div>
        </div>
        <div className="flex shrink-0 items-center gap-1.5">
          {syncPill && <Pill tone={syncPill.tone}>{syncPill.label}</Pill>}
          <CopyButton value={address} label="Copy" />
        </div>
      </div>

      <div>
        <div className="nw-dash-fiat">
          {balanceFiat ?? (totalZec != null ? `${zatsToZec(poolBalances.total)} ZEC` : "—")}
        </div>
        {balanceFiat && totalZec != null ? (
          <p className="mt-1 text-[15px] font-semibold tabular-nums" style={{ color: "var(--nw-platinum)" }}>
            {zatsToZec(poolBalances.total)}{" "}
            <span className="text-sm font-medium" style={{ color: "var(--nw-faint)" }}>
              ZEC
            </span>
          </p>
        ) : null}
        {unitPrice ? <Hint className="mt-1">{unitPrice}</Hint> : <Hint className="mt-1">Price unavailable</Hint>}
        {scanDetail ? <Hint className="mt-1">{scanDetail}</Hint> : null}
      </div>

      <div className="grid grid-cols-2 gap-2">
        <Button variant="secondary" icon="receive" onClick={() => setView("receive")}>
          Deposit
        </Button>
        <Button variant="primary" icon="send" onClick={() => setView("send")}>
          Send
        </Button>
      </div>

      <Card flush>
        <div className="px-3.5 py-2.5">
          <SectionTitle>Assets</SectionTitle>
        </div>
        {assets.map((asset, i) => {
          const zec = scan ? asset.zats / 1e8 : null;
          const fiat = zec != null ? fiatForZec(zec, zecFiat, fiatCode) : null;
          return (
            <div
              key={asset.id}
              className="nw-asset-row"
              style={i === 0 ? { borderTop: "1px solid var(--nw-border)" } : undefined}
            >
              <img className="nw-zec-ticker__icon" src={zecMark} alt="" />
              <div className="min-w-0 flex-1">
                <p className="text-[13px] font-semibold leading-tight">{asset.label}</p>
                <p className="nw-hint mt-0.5 truncate">{asset.hint}</p>
              </div>
              <div className="shrink-0 text-right">
                <p className="text-[13px] font-semibold tabular-nums">
                  {zec != null ? `${zatsToZec(asset.zats)} ZEC` : "—"}
                </p>
                {fiat ? (
                  <p className="nw-hint mt-0.5 tabular-nums">{fiat}</p>
                ) : null}
              </div>
            </div>
          );
        })}
      </Card>

      {pageMode ? (
        <NymVpnPromoCard
          onOpenBrowser={() => {
            setView("browser");
          }}
        />
      ) : null}

      <Card>
        <SectionTitle>Activity</SectionTitle>
        <div className="mt-2 space-y-2">
          {recentTxs.length === 0 && <EmptyState>No transactions yet.</EmptyState>}
          {recentTxs.map((tx) => (
            <div
              key={tx.id}
              className="rounded-xl p-2.5"
              style={{ background: "var(--nw-surface-alt)" }}
            >
              <div className="flex items-center justify-between gap-2">
                <span className="flex items-center gap-1.5">
                  <Pill
                    tone={
                      tx.state === "confirmed"
                        ? "success"
                        : tx.state === "failed" || tx.state === "expired"
                          ? "danger"
                          : "neutral"
                    }
                  >
                    {tx.state}
                  </Pill>
                  {tx.inputMode && (
                    <Pill tone={tx.inputMode === "multi" ? "accent" : "success"}>
                      {tx.inputMode}
                      {typeof tx.inputsUsed === "number" ? ` ×${tx.inputsUsed}` : ""}
                    </Pill>
                  )}
                </span>
                <span className="text-[12px] font-semibold tabular-nums">
                  {(tx.amount / 1e8).toFixed(4)} ZEC
                </span>
              </div>
              <p
                className="nw-mono mt-1.5 truncate text-[10px]"
                style={{ color: "var(--nw-faint)" }}
              >
                {shortAddress(tx.recipientAddress || tx.txid || "n/a")}
              </p>
              {tx.state === "failed" && (
                <Button size="sm" className="mt-2" onClick={() => onRetry(tx.id)}>
                  Retry broadcast
                </Button>
              )}
              {tx.state === "expired" && (
                <Button size="sm" className="mt-2" onClick={() => onSpeedUp(tx.id)}>
                  Speed up
                </Button>
              )}
            </div>
          ))}
        </div>
      </Card>
        </>
      )}
    </Screen>
  );
}

function SendView() {
  const { currency: fiatCode, rate: zecFiat } = useZecFiatPrice();
  const [status, setStatus] = useState<string>("");
  const [recipient, setRecipient] = useState("");
  const [resolvedName, setResolvedName] = useState<string | null>(null);
  const [amount, setAmount] = useState("");
  const [feeZats, setFeeZats] = useState("40000");
  const [coreVersion, setCoreVersion] = useState<string>("");
  const [memo, setMemo] = useState("");
  const [rawTxHex, setRawTxHex] = useState<string | null>(null);
  const [preflight, setPreflight] = useState<{
    txid: string;
    requestedAmount: number;
    fee: number;
    selectedNotesCount: number;
    selectedNotesTotalValue: number;
    selectedNotes: Array<{ value: number; cmx: string; block_height: number }>;
  } | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    extensionApi
      .walletEstimateSendFee({ memo: memo || undefined })
      .then((r) => {
        setFeeZats(String(r.fee));
        setCoreVersion(r.core_version);
      })
      .catch(() => undefined);
  }, [memo]);

  async function resolveRecipientToAddress(): Promise<string | undefined> {
    const trimmed = recipient.trim();
    if (!trimmed) return undefined;
    if (isUnifiedZcashAddress(trimmed)) {
      setResolvedName(null);
      return normalizeUnifiedAddress(trimmed);
    }
    if (!isLikelyZnsName(trimmed)) {
      throw new Error("Enter a unified address (u1…) or a Zcash name (e.g. zoie).");
    }
    const prefs = await getCompanionPrefs();
    const result = await resolveSendRecipient(
      trimmed,
      async (name, network) => {
        const res = await extensionApi.companionZnsResolve({
          name,
          network,
          baseUrl: prefs.baseUrl
        });
        if (!res.found || !res.registration?.address) return null;
        return res.registration as ZnsRegistration;
      },
      "mainnet"
    );
    if (result.kind === "address") {
      setResolvedName(null);
      return result.address;
    }
    if (result.kind === "name") {
      setResolvedName(result.name);
      return result.registration.address;
    }
    if (result.kind === "unresolved") {
      throw new Error(`No Zcash name registered for “${result.name}”.`);
    }
    throw new Error(result.message);
  }

  async function runPreflight() {
    setBusy(true);
    setPreflight(null);
    setRawTxHex(null);
    try {
      const requestedAmount = Number(amount) || 0;
      if (requestedAmount <= 0) throw new Error("Enter an amount in zats");
      const to = await resolveRecipientToAddress();
      const result = await extensionApi.walletProveTransaction({
        to,
        amount: requestedAmount,
        memo: memo || undefined
      });
      setRawTxHex(result.rawTxHex || null);
      setPreflight({
        txid: result.txid,
        requestedAmount,
        fee: Number(result.fee ?? 0),
        selectedNotesCount: Number(result.selected_notes_count ?? 0),
        selectedNotesTotalValue: Number(result.selected_notes_total_value ?? 0),
        selectedNotes: result.selected_notes ?? []
      });
      setStatus("Transaction built — review and confirm below.");
    } catch (e) {
      setStatus((e as Error).message);
    } finally {
      setBusy(false);
    }
  }

  async function broadcast() {
    if (!rawTxHex) return;
    setBusy(true);
    try {
      const txid = await extensionApi.rpcSendRawTx(rawTxHex);
      setStatus(`Broadcast OK — txid: ${txid}`);
      setPreflight(null);
      setRawTxHex(null);
    } catch (e) {
      setStatus(`Broadcast failed: ${(e as Error).message}`);
    } finally {
      setBusy(false);
    }
  }

  const amountZec = (() => {
    const n = Number(amount);
    return Number.isFinite(n) && n > 0 ? n / 1e8 : null;
  })();
  const amountFiat = amountZec != null ? fiatForZec(amountZec, zecFiat, fiatCode) : null;

  const statusTone: "success" | "accent" | "danger" = status.startsWith("Broadcast OK")
    ? "success"
    : status.startsWith("Transaction built")
      ? "accent"
      : "danger";

  return (
    <Screen>
      <PageHeader title="Send" description="Shielded Orchard/Ironwood transfer from your synced notes." />
      <SendEgressBadge compact />

      <Input
        label="Recipient"
        placeholder="u1… or Zcash name (e.g. zoie)"
        value={recipient}
        onChange={(e) => {
          setRecipient(e.target.value);
          setResolvedName(null);
        }}
      />
      {resolvedName ? (
        <Hint style={{ color: "var(--nw-success)" }}>Resolved {resolvedName}</Hint>
      ) : (
        <Hint>Leave blank to send to your own address.</Hint>
      )}

      <Input
        label="Amount (zats)"
        placeholder="e.g. 100000"
        inputMode="numeric"
        mono
        value={amount}
        onChange={(e) => setAmount(e.target.value)}
      />
      {amountZec != null && (
        <Hint>
          ≈ {amountZec.toFixed(8)} ZEC{amountFiat ? ` · ${amountFiat}` : ""}
        </Hint>
      )}

      <div>
        <span className="nw-label">Network fee</span>
        <Hint className="mb-1.5">
          ZIP-317 × 4 is required on every Nozy send
          {coreVersion ? ` · ${coreVersion}` : ""}.
        </Hint>
        <StatRow
          label="Fee"
          value={
            feeZats
              ? `${feeZats} zats · ${(Number(feeZats) / 1e8).toFixed(8)} ZEC`
              : "Estimating…"
          }
        />
      </div>

      <Textarea
        label="Memo (optional, max 512 bytes)"
        rows={2}
        placeholder="Private message attached to the transaction"
        value={memo}
        onChange={(e) => setMemo(e.target.value)}
      />

      {!preflight ? (
        <Button
          variant="primary"
          fullWidth
          disabled={busy || !amount}
          onClick={runPreflight}
        >
          {busy ? "Building transaction…" : "Review transaction"}
        </Button>
      ) : (
        <div className="space-y-2">
          <Card tone="accent" className="space-y-1.5">
            <Eyebrow>Confirm details</Eyebrow>
            <StatRow
              label="Amount"
              value={`${preflight.requestedAmount} zats · ${(preflight.requestedAmount / 1e8).toFixed(8)} ZEC`}
            />
            <StatRow
              label="Fee"
              value={`${preflight.fee} zats · ${(preflight.fee / 1e8).toFixed(8)} ZEC (ZIP-317 × 4)`}
            />
            <StatRow
              label="Input notes"
              value={`${preflight.selectedNotesCount} · ${preflight.selectedNotesTotalValue} zats`}
            />
            {memo && <StatRow label="Memo" value={memo} />}
          </Card>
          <div className="flex gap-2">
            <Button
              variant="success"
              className="flex-1"
              disabled={busy}
              onClick={broadcast}
            >
              {busy ? "Broadcasting…" : "Confirm & send"}
            </Button>
            <Button
              variant="secondary"
              onClick={() => {
                setPreflight(null);
                setRawTxHex(null);
                setStatus("");
              }}
            >
              Cancel
            </Button>
          </div>
        </div>
      )}

      {status && <Callout tone={statusTone}>{status}</Callout>}
    </Screen>
  );
}

function WalletSyncPanel({
  status,
  scan,
  onScanProgress,
  beforeSync
}: {
  status: WalletStatus | null;
  scan: WalletScanProgressResult | null;
  onScanProgress?: (p: WalletScanProgressResult) => void;
  beforeSync?: () => Promise<void>;
}) {
  const [actionMsg, setActionMsg] = useState("");
  const [infoMsg, setInfoMsg] = useState("");
  const [busy, setBusy] = useState(false);
  const busyScanning = busy || scan?.status === "scanning";
  const percent = scanPercentDisplay(scan);

  const statusLine = useMemo(() => {
    if (!scan || scan.status === "idle") return null;
    if (scan.status === "scanning") {
      const done = scan.scannedBlocks ?? 0;
      const total = scan.totalBlocks ?? 0;
      const pct = scanPercentLabel(scan);
      const rate = scanRateLabel(scan);
      const range =
        typeof scan.startHeight === "number" && typeof scan.endHeight === "number"
          ? ` · blocks ${scan.startHeight.toLocaleString()}–${scan.endHeight.toLocaleString()}`
          : "";
      return `${done.toLocaleString()} / ${total.toLocaleString()} blocks · ${pct}% · ${scan.discoveredNotes ?? 0} notes${rate ? ` · ${rate}` : ""}${range}`;
    }
    if (scan.status === "done") {
      const zec = ((scan.totalBalanceZats ?? 0) / 1e8).toFixed(8);
      return `Done — ${zec} ZEC · ${scan.discoveredNotes ?? 0} notes`;
    }
    if (scan.status === "failed" && scan.scanError) return scan.scanError;
    if (scan.status === "stopped") return `Stopped at ${scan.percent ?? 0}%`;
    return null;
  }, [scan]);

  const refreshScanProgress = async () => {
    try {
      const p = await extensionApi.walletScanProgress();
      onScanProgress?.(p);
      return p;
    } catch {
      return null;
    }
  };

  const runSync = async (start: () => Promise<{ startHeight: number; endHeight: number }>) => {
    setActionMsg("");
    setInfoMsg("");
    setBusy(true);
    try {
      if (beforeSync) await beforeSync();
      const result = await start();
      const blockCount = Math.max(1, result.endHeight - result.startHeight + 1);
      const rpcNote =
        "rpcEndpoint" in result && typeof result.rpcEndpoint === "string"
          ? ` RPC: ${result.rpcEndpoint}.`
          : "";
      setInfoMsg(
        `Scan started: ${result.startHeight.toLocaleString()} → ${result.endHeight.toLocaleString()} (${blockCount.toLocaleString()} blocks).${rpcNote} Large scans stay at 0% for a while — watch block counts below.`
      );
      await refreshScanProgress();
      // Poll a few times quickly so the UI updates before the 2s app interval.
      for (let i = 0; i < 5; i += 1) {
        await new Promise((r) => setTimeout(r, 400));
        const p = await refreshScanProgress();
        if (p?.status === "scanning" && (p.scannedBlocks ?? 0) > 0) break;
        if (p?.status === "done" || p?.status === "failed") break;
      }
    } catch (e) {
      setActionMsg((e as Error).message);
    } finally {
      setBusy(false);
    }
  };

  if (!status?.unlocked) return null;

  return (
    <Card className="space-y-2.5">
      <SectionTitle>Sync wallet</SectionTitle>
      <Hint>
        Scans Orchard + Ironwood via your RPC node. Sapling legacy notes sync in the
        background when nozywallet-api is running (Settings → Local API). A restored
        Desktop wallet starts around block 3,050,000, not the chain tip — that takes a
        while. Do not stop at 1 block.
      </Hint>
      {isScanInProgress(scan) && (
        <div className="space-y-2">
          <CyberpunkSyncPanel
            headline={
              statusLine ??
              `Syncing ${scanPercentLabel(scan)}% · ${(scan?.scannedBlocks ?? 0).toLocaleString()} blocks`
            }
            detail={
              typeof scan?.startHeight === "number" && typeof scan?.endHeight === "number"
                ? `Range ${scan.startHeight.toLocaleString()} → ${scan.endHeight.toLocaleString()}`
                : undefined
            }
            percent={Math.max(busyScanning && percent === 0 ? 0.8 : 0, percent)}
          />
        </div>
      )}
      <div className="flex flex-wrap gap-2">
        {scan?.status !== "scanning" && !busy ? (
          <>
            <Button
              variant="primary"
              size="sm"
              icon="refresh"
              disabled={busy}
              onClick={() =>
                void runSync(() => extensionApi.walletStartScan({ useBirthdayRange: true }))
              }
            >
              Sync to tip
            </Button>
            <Button
              size="sm"
              disabled={busy}
              onClick={() => void runSync(() => extensionApi.walletStartScan(20_000))}
            >
              Last 20k blocks
            </Button>
          </>
        ) : (
          <Button
            variant="danger"
            size="sm"
            onClick={async () => {
              setActionMsg("");
              setInfoMsg("");
              try {
                await extensionApi.walletStopScan();
                await refreshScanProgress();
              } catch (_) {
                /* ignore */
              }
            }}
          >
            Stop sync
          </Button>
        )}
      </div>
      {typeof status.orchardBirthdayHeight === "number" && (
        <StatRow
          label="Birthday height"
          value={status.orchardBirthdayHeight.toLocaleString()}
          tone="muted"
        />
      )}
      {infoMsg && <Callout tone="success">{infoMsg}</Callout>}
      {statusLine && scan?.status !== "done" && <Hint>{statusLine}</Hint>}
      {scan?.lastRpcError && scan.status === "scanning" && (
        <Hint style={{ color: "var(--nw-warn)" }}>RPC: {scan.lastRpcError}</Hint>
      )}
      {actionMsg && <Callout tone="danger">{actionMsg}</Callout>}
    </Card>
  );
}

function ReceiveView({ status }: { status: WalletStatus | null }) {
  const address = status?.address || "";
  return (
    <Screen>
      <PageHeader title="Receive" description="Unified address for shielded ZEC on mainnet." />

      <Card className="flex flex-col items-center gap-3">
        {address ? (
          <div className="rounded-2xl bg-white p-3">
            <QRCode value={address} size={148} bgColor="#ffffff" fgColor="#0b0b0e" />
          </div>
        ) : (
          <EmptyState>No address yet — unlock and sync your wallet.</EmptyState>
        )}
        <p
          className="nw-mono break-all text-center text-[10px]"
          style={{ color: "var(--nw-muted)" }}
        >
          {address || "—"}
        </p>
        <CopyButton value={address} label="Copy address" fullWidth />
      </Card>

      <Callout>
        After someone pays you, run a sync from <strong>Settings</strong> so the new note shows up in
        your balance. This address must match Nozy Desktop Wallet 1 (same recovery phrase).
      </Callout>
    </Screen>
  );
}

/** Advanced scan controls (birthday, custom heights) — tucked under Settings → Advanced. */
function ScanView({
  status,
  scan,
  onWalletMetaChanged
}: {
  status: WalletStatus | null;
  scan: WalletScanProgressResult | null;
  onWalletMetaChanged?: () => void;
}) {
  const [actionMsg, setActionMsg] = useState<string>("");
  const [chainTip, setChainTip] = useState<number | null>(null);
  const [scanStartStr, setScanStartStr] = useState("");
  const [scanEndStr, setScanEndStr] = useState("");
  const [birthdayEditStr, setBirthdayEditStr] = useState("");

  const scanning = scan?.status === "scanning";
  const percent = scanPercentDisplay(scan);
  const NU5_ORCHARD_START_MAINNET = 1_687_104;
  const NU5_ORCHARD_START_TESTNET = 1_842_420;

  const scanInfo = useMemo(() => {
    if (!scan) return "";
    const range =
      typeof scan.startHeight === "number" && typeof scan.endHeight === "number"
        ? ` (${scan.startHeight.toLocaleString()}–${scan.endHeight.toLocaleString()})`
        : "";
    if (isScanInProgress(scan)) {
      const elapsed = ((scan.elapsed ?? 0) / 1000).toFixed(0);
      const warn =
        typeof scan.lastRpcError === "string" && scan.lastRpcError.trim()
          ? ` — RPC: ${scan.lastRpcError.slice(0, 120)}${scan.lastRpcError.length > 120 ? "…" : ""}`
          : "";
      return `Scanning… ${scanPercentLabel(scan)}% (${scan.scannedBlocks ?? 0}/${scan.totalBlocks ?? 0} blocks, ${scan.discoveredNotes ?? 0} notes, ${elapsed}s)${range}${warn}`;
    }
    if (scan.status === "done") {
      const elapsed = ((scan.elapsed ?? 0) / 1000).toFixed(1);
      const zec = ((scan.totalBalanceZats ?? 0) / 1e8).toFixed(8);
      return `Done in ${elapsed}s — ${scan.scannedBlocks ?? 0} blocks, ${scan.discoveredNotes ?? 0} notes, balance: ${zec} ZEC${range}`;
    }
    if (scan.status === "stopped") {
      return `Scan stopped at ${scanPercentLabel(scan)}% (${scan.scannedBlocks ?? 0}/${scan.totalBlocks ?? 0} blocks)${range}`;
    }
    if (scan.status === "failed") {
      return scan.scanError ? `Scan failed: ${scan.scanError}` : "Scan failed.";
    }
    return "";
  }, [scan]);

  useEffect(() => {
    if (typeof status?.orchardBirthdayHeight === "number" && Number.isFinite(status.orchardBirthdayHeight)) {
      setBirthdayEditStr(String(status.orchardBirthdayHeight));
    } else {
      setBirthdayEditStr("");
    }
  }, [status?.orchardBirthdayHeight]);

  useEffect(() => {
    if (!status?.unlocked) {
      setChainTip(null);
      return;
    }
    let cancelled = false;
    extensionApi
      .rpcGetBlockCount()
      .then((n) => {
        if (cancelled || typeof n !== "number" || !Number.isFinite(n)) return;
        setChainTip(n);
        setScanEndStr(String(n));
        const b = status.orchardBirthdayHeight;
        if (typeof b === "number" && Number.isFinite(b)) {
          setScanStartStr(String(Math.min(b, n)));
        } else {
          setScanStartStr(String(Math.max(0, n - 20_000)));
        }
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, [status?.unlocked, status?.orchardBirthdayHeight]);

  const refreshChainTip = async () => {
    setActionMsg("");
    try {
      const n = await extensionApi.rpcGetBlockCount();
      if (typeof n !== "number" || !Number.isFinite(n)) throw new Error("Invalid chain tip from RPC");
      setChainTip(n);
      setScanEndStr(String(n));
    } catch (e) {
      setActionMsg((e as Error).message);
    }
  };

  const parseHeight = (s: string, label: string): number => {
    const t = s.trim().replace(/,/g, "");
    const n = Number(t);
    if (!Number.isFinite(n) || n < 0 || !Number.isInteger(n)) {
      throw new Error(`${label} must be a non-negative integer`);
    }
    return n;
  };

  const formatScanStartedMsg = (
    startHeight: number,
    endHeight: number,
    label: string
  ): string => {
    const n = Math.max(1, endHeight - startHeight + 1);
    return `${label}: heights ${startHeight.toLocaleString()}–${endHeight.toLocaleString()} (${n.toLocaleString()} blocks). Only this inclusive range is scanned.`;
  };

  const startScanWindow = async (windowBlocks: number) => {
    setActionMsg("");
    try {
      const r = await extensionApi.walletStartScan(windowBlocks);
      setActionMsg(
        formatScanStartedMsg(r.startHeight, r.endHeight, `Last ${windowBlocks.toLocaleString()}`)
      );
    } catch (e) {
      setActionMsg((e as Error).message);
    }
  };

  const startScanRange = async (startHeight: number, endHeight: number) => {
    setActionMsg("");
    try {
      const r = await extensionApi.walletStartScan({ startHeight, endHeight });
      setActionMsg(formatScanStartedMsg(r.startHeight, r.endHeight, "Preset range"));
    } catch (e) {
      setActionMsg((e as Error).message);
    }
  };

  const startScanBirthdayToTip = async () => {
    setActionMsg("");
    try {
      const r = await extensionApi.walletStartScan({ useBirthdayRange: true });
      setActionMsg(
        formatScanStartedMsg(r.startHeight, r.endHeight, "Birthday → tip") +
          " (start from saved creation/birthday height, not from a window preset)."
      );
    } catch (e) {
      setActionMsg((e as Error).message);
    }
  };

  const startScanCustomFields = async () => {
    setActionMsg("");
    try {
      let endH: number;
      if (scanEndStr.trim() === "") {
        if (chainTip === null) {
          throw new Error('Set end height or tap "Refresh tip" first.');
        }
        endH = chainTip;
      } else {
        endH = parseHeight(scanEndStr, "End height");
      }
      const startH = parseHeight(scanStartStr, "Start height");
      const r = await extensionApi.walletStartScan({ startHeight: startH, endHeight: endH });
      setActionMsg(formatScanStartedMsg(r.startHeight, r.endHeight, "Custom range"));
    } catch (e) {
      setActionMsg((e as Error).message);
    }
  };

  return (
    <div className="space-y-2.5">
      <SectionTitle>Custom scan</SectionTitle>

      {isScanInProgress(scan) && (
        <CyberpunkSyncPanel
          headline={`${scanPercentLabel(scan)}% · custom scan`}
          detail={
            typeof scan?.scannedBlocks === "number" && typeof scan?.totalBlocks === "number"
              ? `${scan.scannedBlocks.toLocaleString()} / ${scan.totalBlocks.toLocaleString()} blocks`
              : undefined
          }
          percent={percent}
        />
      )}

      <Hint>
        For recovery or older funds: set a birthday height or pick a block range. Normal use is{" "}
        <strong>Sync to tip</strong> above.
      </Hint>

      <div
        className="space-y-2 rounded-xl p-2.5"
        style={{ background: "var(--nw-surface-alt)", border: "1px solid var(--nw-border)" }}
      >
        <div className="flex items-center justify-between gap-2">
          <StatRow
            label="Chain tip"
            value={chainTip !== null ? chainTip.toLocaleString() : "—"}
          />
          <Button size="sm" icon="refresh" onClick={() => refreshChainTip()}>
            Refresh
          </Button>
        </div>
        <div className="grid grid-cols-2 gap-2">
          <Input
            label="Start height"
            mono
            value={scanStartStr}
            onChange={(e) => setScanStartStr(e.target.value)}
            inputMode="numeric"
            disabled={scanning}
          />
          <Input
            label="End height"
            mono
            value={scanEndStr}
            onChange={(e) => setScanEndStr(e.target.value)}
            inputMode="numeric"
            disabled={scanning}
          />
        </div>
        <div className="nw-divider" />
        <StatRow
          label="Saved Orchard birthday"
          value={
            typeof status?.orchardBirthdayHeight === "number"
              ? status.orchardBirthdayHeight.toLocaleString()
              : "not set"
          }
          tone="muted"
        />
        <div className="flex items-end gap-2">
          <div className="min-w-0 flex-1">
            <Input
              mono
              value={birthdayEditStr}
              onChange={(e) => setBirthdayEditStr(e.target.value)}
              placeholder="Block height"
              disabled={scanning}
            />
          </div>
          <Button
            size="sm"
            disabled={scanning}
            onClick={async () => {
              setActionMsg("");
              try {
                const h = parseHeight(birthdayEditStr, "Birthday");
                await extensionApi.walletSetBirthdayHeight(h);
                onWalletMetaChanged?.();
              } catch (e) {
                setActionMsg((e as Error).message);
              }
            }}
          >
            Save birthday
          </Button>
        </div>
      </div>

      <div className="flex flex-wrap items-center gap-2">
        {!scan ? (
          <Hint>Checking scan status…</Hint>
        ) : !scanning ? (
          <>
            <Button variant="primary" size="sm" onClick={() => void startScanWindow(20_000)}>
              Last 20k
            </Button>
            <Button size="sm" onClick={() => void startScanCustomFields()}>
              Custom range
            </Button>
            {chainTip !== null && (
              <>
                <Button
                  size="sm"
                  title="Mainnet NU5 / Orchard activation height"
                  onClick={() =>
                    void startScanRange(Math.min(NU5_ORCHARD_START_MAINNET, chainTip), chainTip)
                  }
                >
                  NU5 → tip (mainnet)
                </Button>
                <Button
                  size="sm"
                  title="Testnet NU5 / Orchard activation height"
                  onClick={() =>
                    void startScanRange(Math.min(NU5_ORCHARD_START_TESTNET, chainTip), chainTip)
                  }
                >
                  NU5 → tip (testnet)
                </Button>
                <Button size="sm" onClick={() => void startScanRange(0, chainTip)}>
                  Full chain 0 → tip
                </Button>
              </>
            )}
          </>
        ) : (
          <Button
            variant="danger"
            size="sm"
            onClick={async () => {
              setActionMsg("");
              try {
                await extensionApi.walletStopScan();
              } catch (_) {}
            }}
          >
            Stop scan
          </Button>
        )}
      </div>

      {actionMsg && <Callout tone="danger">{actionMsg}</Callout>}
      {scanInfo && <Hint>{scanInfo}</Hint>}
    </div>
  );
}

function CompanionView({ nested = false }: { nested?: boolean }) {
  const [baseUrl, setBaseUrl] = useState("http://127.0.0.1:3000");
  const [lwdUrl, setLwdUrl] = useState<string>(DEFAULT_LWD_URL);
  const [apiKey, setApiKey] = useState("");
  const [log, setLog] = useState("");
  const [busy, setBusy] = useState(false);
  const [syncStart, setSyncStart] = useState("0");
  const [syncEnd, setSyncEnd] = useState("");
  const [syncTipFloor, setSyncTipFloor] = useState("");
  const [legacyZec, setLegacyZec] = useState<number | null>(null);
  const [legacyFeeZec, setLegacyFeeZec] = useState(0);

  const refreshLegacyStatus = async (prefsBase?: string) => {
    try {
      const prefs = prefsBase
        ? { baseUrl: prefsBase }
        : await getCompanionPrefs();
      const s = await extensionApi.companionSaplingStatus(prefs.baseUrl);
      if (s.has_legacy_balance && s.unspent_zec > 0) {
        setLegacyZec(s.unspent_zec);
        setLegacyFeeZec(s.fee_zec);
      } else {
        setLegacyZec(null);
      }
    } catch {
      setLegacyZec(null);
    }
  };

  useEffect(() => {
    getCompanionPrefs()
      .then((p) => {
        setBaseUrl(p.baseUrl);
        setLwdUrl(p.lightwalletdUrl);
        setApiKey(p.apiKey || "");
        void refreshLegacyStatus(p.baseUrl);
      })
      .catch(() => undefined);
  }, []);

  const run = async (fn: () => Promise<void>) => {
    setBusy(true);
    try {
      await fn();
    } finally {
      setBusy(false);
    }
  };

  const body = (
    <div className="space-y-3">
      <Hint>
        Optional: run <span className="nw-mono">nozywallet-api</span> or Nozy Desktop for
        lightwalletd compact sync. Zebrad JSON-RPC in Settings is enough for scan + send.
      </Hint>
      <Hint>
        On Windows, if lightwalletd runs in WSL, set{" "}
        <span className="nw-mono">LIGHTWALLETD_GRPC</span> on the API process (e.g.{" "}
        <span className="nw-mono">http://&lt;wsl-ip&gt;:9067</span>), not only{" "}
        <span className="nw-mono">127.0.0.1</span>.
      </Hint>

      {legacyZec !== null && (
        <Card tone="accent" className="space-y-2">
          <Eyebrow>Legacy funds</Eyebrow>
          <Hint>
            {legacyZec.toFixed(8)} ZEC available to move into shielded balance (fee ~
            {legacyFeeZec.toFixed(8)} ZEC). Uses the companion wallet data dir — not the in-extension
            WASM wallet.
          </Hint>
          <Button
            variant="primary"
            size="sm"
            disabled={busy}
            onClick={() =>
              run(async () => {
                const prefs = await getCompanionPrefs();
                const password =
                  window.prompt("Companion wallet password (to move legacy funds)") ?? "";
                if (!password) {
                  setLog("Cancelled — password required to move legacy funds.");
                  return;
                }
                await extensionApi.companionSaplingScan({
                  baseUrl: prefs.baseUrl,
                  password
                });
                const res = await extensionApi.companionSaplingShield({
                  baseUrl: prefs.baseUrl,
                  password
                });
                setLog(JSON.stringify(res, null, 2));
                await refreshLegacyStatus(prefs.baseUrl);
              })
            }
          >
            Move to shielded
          </Button>
        </Card>
      )}

      <Card className="space-y-2.5">
        <Input
          label="Nozy API base URL"
          mono
          value={baseUrl}
          onChange={(e) => setBaseUrl(e.target.value)}
          placeholder="http://127.0.0.1:3000"
        />
        <Input
          label="Companion API key (required for send/shield)"
          hint={
            <>
              Copy from the wallet data dir file <span className="nw-mono">companion_api_key</span>{" "}
              (printed when nozywallet-api starts), or set{" "}
              <span className="nw-mono">NOZY_API_KEY</span>.
            </>
          }
          mono
          type="password"
          autoComplete="off"
          value={apiKey}
          onChange={(e) => setApiKey(e.target.value)}
          placeholder="Paste companion API key"
        />
        <Input
          label="lightwalletd gRPC (optional, local only)"
          hint="For compact sync via the desktop API — not used for in-extension block scan (use Zebrad RPC in Settings)."
          mono
          value={lwdUrl}
          onChange={(e) => setLwdUrl(e.target.value)}
          placeholder={DEFAULT_LWD_URL}
        />
        <div className="flex flex-wrap gap-2">
          <Button size="sm" onClick={() => setLwdUrl(DEFAULT_LWD_URL)}>
            Reset to local
          </Button>
          <Button
            variant="primary"
            size="sm"
            disabled={busy}
            onClick={() =>
              run(async () => {
                await setCompanionPrefs({ baseUrl, lightwalletdUrl: lwdUrl, apiKey });
                setLog("Saved companion URLs and API key to extension storage.");
              })
            }
          >
            Save settings
          </Button>
        </div>
      </Card>

      <div className="flex flex-wrap gap-2">
        <Button
          variant="primary"
          size="sm"
          disabled={busy}
          onClick={() =>
            run(async () => {
              const prefs = await getCompanionPrefs();
              const s = await extensionApi.companionStatus(prefs.baseUrl);
              setLog(
                JSON.stringify(
                  {
                    companionReachable: s.companionReachable,
                    healthStatus: s.healthStatus,
                    lwdChainTip: s.lwdChainTip
                  },
                  null,
                  2
                )
              );
              await refreshLegacyStatus(prefs.baseUrl);
            })
          }
        >
          Check API
        </Button>
        <Button
          size="sm"
          disabled={busy}
          onClick={() =>
            run(async () => {
              const prefs = await getCompanionPrefs();
              const q = prefs.lightwalletdUrl.trim();
              const info = await extensionApi.companionLwdInfo(
                prefs.baseUrl,
                q || undefined
              );
              setLog(JSON.stringify(info, null, 2));
            })
          }
        >
          Lightd info
        </Button>
        <Button
          size="sm"
          disabled={busy}
          onClick={() =>
            run(async () => {
              const prefs = await getCompanionPrefs();
              const q = prefs.lightwalletdUrl.trim();
              const tip = await extensionApi.companionLwdChainTip(
                prefs.baseUrl,
                q || undefined
              );
              setLog(JSON.stringify(tip, null, 2));
            })
          }
        >
          Chain tip
        </Button>
      </div>

      <Card className="space-y-2.5">
        <SectionTitle>Compact sync</SectionTitle>
        <Hint>Writes the desktop SQLite store through the companion API.</Hint>
        <div className="grid grid-cols-2 gap-2">
          <Input
            label="Start"
            mono
            value={syncStart}
            onChange={(e) => setSyncStart(e.target.value)}
            placeholder="0"
          />
          <Input
            label="End (optional)"
            mono
            value={syncEnd}
            onChange={(e) => setSyncEnd(e.target.value)}
            placeholder="tip"
          />
        </div>
        <Button
          size="sm"
          disabled={busy}
          onClick={() =>
            run(async () => {
              const prefs = await getCompanionPrefs();
              const start = Math.max(0, Math.floor(Number(syncStart) || 0));
              const endRaw = syncEnd.trim();
              const end =
                endRaw === "" ? undefined : Math.max(start, Math.floor(Number(endRaw) || 0));
              const q = prefs.lightwalletdUrl.trim();
              const res = await extensionApi.companionLwdSyncCompact({
                baseUrl: prefs.baseUrl,
                start,
                end,
                lightwalletd_url: q || undefined
              });
              setLog(JSON.stringify(res, null, 2));
              await refreshLegacyStatus(prefs.baseUrl);
            })
          }
        >
          Sync compact range
        </Button>
        <div className="nw-divider" />
        <Input
          label="Birthday floor for “to tip”"
          hint="Empty = default 1."
          mono
          value={syncTipFloor}
          onChange={(e) => setSyncTipFloor(e.target.value)}
          placeholder="start_floor (optional)"
        />
        <Button
          size="sm"
          disabled={busy}
          onClick={() =>
            run(async () => {
              const prefs = await getCompanionPrefs();
              const q = prefs.lightwalletdUrl.trim();
              const floorRaw = syncTipFloor.trim();
              const start_floor =
                floorRaw === "" ? undefined : Math.max(0, Math.floor(Number(floorRaw) || 0));
              const res = await extensionApi.companionLwdSyncCompactToTip({
                baseUrl: prefs.baseUrl,
                lightwalletd_url: q || undefined,
                start_floor
              });
              setLog(JSON.stringify(res, null, 2));
              await refreshLegacyStatus(prefs.baseUrl);
            })
          }
        >
          Sync compact to tip
        </Button>
      </Card>

      {log && <LogBlock>{log}</LogBlock>}
    </div>
  );

  if (nested) return body;

  return (
    <Screen>
      <PageHeader
        title="Local API"
        description="Optional companion for lightwalletd compact sync and legacy funds."
      />
      {body}
    </Screen>
  );
}

function SettingsView({
  endpoint,
  onEndpointChange,
  onLock,
  onSetAutoLock,
  status,
  scan,
  onWalletMetaChanged,
  onScanProgress
}: {
  endpoint: string;
  onEndpointChange: (url: string) => void;
  onLock: () => void;
  onSetAutoLock: (ms: number) => Promise<void>;
  status: WalletStatus | null;
  scan: WalletScanProgressResult | null;
  onWalletMetaChanged?: () => void;
  onScanProgress?: (p: WalletScanProgressResult) => void;
}) {
  const [msg, setMsg] = useState<string | null>(null);
  const [autoLockMin, setAutoLockMin] = useState("15");
  const [showAdvanced, setShowAdvanced] = useState(false);

  return (
    <Screen>
      <PageHeader
        title="Settings"
        trailing={<Pill tone="neutral">v{chrome.runtime.getManifest().version}</Pill>}
      />

      <NodeConnectCard
        variant="settings"
        initialEndpoint={endpoint}
        onConnected={(url) => {
          void onEndpointChange(url);
          setMsg("Node connected and saved.");
        }}
      />
      {msg && <Callout tone="success">{msg}</Callout>}

      <NetworkPrivacyPanel />

      <WalletSyncPanel
        status={status}
        scan={scan}
        onScanProgress={onScanProgress}
        beforeSync={async () => {
          try {
            const res = await extensionApi.rpcConnect({ tryCompanion: true });
            if (res.rpcEndpoint !== endpoint) {
              await onEndpointChange(res.rpcEndpoint);
            }
            setMsg(
              `Using Zebrad at ${res.rpcEndpoint} (${res.blockCount.toLocaleString()} blocks) for sync.`
            );
          } catch (e) {
            setMsg((e as Error).message);
            throw e;
          }
        }}
      />

      <Card className="space-y-2.5">
        <SectionTitle>Security</SectionTitle>
        <div className="flex items-end gap-2">
          <div className="w-28">
            <Input
              label="Auto-lock (min)"
              mono
              value={autoLockMin}
              onChange={(e) => setAutoLockMin(e.target.value)}
            />
          </div>
          <Button
            size="sm"
            onClick={async () => {
              const mins = Math.max(1, Number(autoLockMin) || 15);
              await onSetAutoLock(mins * 60_000);
              setMsg("Session policy saved.");
            }}
          >
            Save
          </Button>
        </div>
        <Button variant="secondary" icon="lock" fullWidth onClick={onLock}>
          Lock wallet
        </Button>
      </Card>

      <Card flush>
        <ListRow
          title="Advanced"
          description="Birthday height, custom scan, local API"
          onClick={() => setShowAdvanced((v) => !v)}
          trailing={<Pill tone={showAdvanced ? "accent" : "neutral"}>{showAdvanced ? "Open" : "Show"}</Pill>}
        />
      </Card>
      {showAdvanced && (
        <Card className="space-y-3">
          <ScanView status={status} scan={scan} onWalletMetaChanged={onWalletMetaChanged} />
          <div className="nw-divider" />
          <SectionTitle>Local API (optional)</SectionTitle>
          <CompanionView nested />
        </Card>
      )}
    </Screen>
  );
}

function PendingApprovals({
  approvals,
  onApprove,
  onReject
}: {
  approvals: PendingApproval[];
  onApprove: (id: string) => Promise<void>;
  onReject: (id: string) => Promise<void>;
}) {
  if (approvals.length === 0) return null;
  return (
    <div
      className="space-y-2 px-4 py-3"
      style={{ borderBottom: "1px solid var(--nw-border)", background: "var(--nw-platinum-soft)" }}
    >
      <Eyebrow>Pending dApp requests</Eyebrow>
      {approvals.map((a) => (
        <Card key={a.id} className="space-y-2">
          <div className="flex items-center gap-2">
            <span className="text-[13px] font-semibold">{a.kind.toUpperCase()} request</span>
            {a.kind === "transaction" &&
              (() => {
                const p = (a.payload as Record<string, unknown>)?.preflight as
                  | { input_mode?: string; inputs_used?: number }
                  | undefined;
                if (!p) return <Pill tone="neutral">Preflight pending</Pill>;
                const mode = String(p.input_mode || "single");
                const used = Number(p.inputs_used ?? 0);
                return (
                  <Pill tone={mode === "multi" ? "accent" : "success"}>
                    {mode} ×{used}
                  </Pill>
                );
              })()}
          </div>
          {a.kind === "transaction" &&
            Boolean((a.payload as Record<string, unknown>)?.preflightError) && (
              <Callout tone="warn">
                Preflight warning:{" "}
                {String((a.payload as Record<string, unknown>)?.preflightError)}
              </Callout>
            )}
          <LogBlock>{JSON.stringify(a.payload, null, 2)}</LogBlock>
          <div className="flex gap-2">
            <Button variant="primary" size="sm" className="flex-1" onClick={() => onApprove(a.id)}>
              Approve
            </Button>
            <Button size="sm" className="flex-1" onClick={() => onReject(a.id)}>
              Reject
            </Button>
          </div>
        </Card>
      ))}
    </div>
  );
}

function ChainSyncPopup({
  scan,
  onOpenSync
}: {
  scan: WalletScanProgressResult | null;
  onOpenSync: () => void;
}) {
  if (!isScanInProgress(scan) || !scan) return null;
  const tip = Math.floor(Number(scan.endHeight ?? 0));
  const cursor = Number(scan.currentHeight ?? 0);
  const synced =
    Number.isFinite(tip) && tip > 0 && Number.isFinite(cursor)
      ? Math.max(0, Math.min(tip, Math.floor(cursor) - 1))
      : null;
  const detail =
    synced != null
      ? `height ${synced.toLocaleString()} / ${tip.toLocaleString()} · ${scan.discoveredNotes ?? 0} notes · ${scanRateLabel(scan) ?? "estimating…"}`
      : `${scan.discoveredNotes ?? 0} notes · ${scanRateLabel(scan) ?? "estimating…"}`;
  return (
    <div className="shrink-0 px-3 pb-1 pt-2">
      <button type="button" className="w-full text-left" onClick={onOpenSync}>
        <CyberpunkSyncPanel
          headline={`${scanPercentLabel(scan)}% · ${(scan.scannedBlocks ?? 0).toLocaleString()} / ${(scan.totalBlocks ?? 0).toLocaleString()} blocks`}
          detail={detail}
          percent={scanPercentDisplay(scan)}
        />
      </button>
    </div>
  );
}

export function App() {
  const { view, setView } = useUiStore();
  const [status, setStatus] = useState<WalletStatus | null>(null);
  const [approvals, setApprovals] = useState<PendingApproval[]>([]);
  const [txs, setTxs] = useState<TxStateEntry[]>([]);
  const [bootDebug, setBootDebug] = useState<string | null>(null);
  const [moreOpen, setMoreOpen] = useState(false);
  /** Orchard block scan progress; polled app-wide so it keeps updating when you leave Receive. */
  const [scanProgress, setScanProgress] = useState<WalletScanProgressResult | null>(null);
  const endpoint = useMemo(() => status?.rpcEndpoint || DEFAULT_RPC, [status]);

  const refresh = async () => {
    try {
      setBootDebug("startup: wallet_status");
      const nextStatus = await extensionApi.walletStatus();
      setStatus(nextStatus);
      if (!nextStatus.exists) setView("welcome");
      else if (!nextStatus.unlocked) setView("unlock");
      else if (view === "welcome" || view === "unlock") {
        setView(viewFromUrl() ?? "dashboard");
      }

      setBootDebug("startup: wallet_get_pending_approvals");
      const nextApprovals = await extensionApi.walletGetPendingApprovals();
      setApprovals(nextApprovals);

      setBootDebug("startup: wallet_get_transactions");
      const txState = await extensionApi.walletGetTransactions();
      setTxs(Array.isArray(txState.txs) ? txState.txs : []);
      setBootDebug(null);

      return;
    } catch (err) {
      const msg = String((err as Error)?.message || err || "unknown");
      setBootDebug(`startup-error: ${msg}`);
      throw err;
    }
  };

  useEffect(() => {
    refresh().catch((err) => {
      console.error(err);
    });
    const id = setInterval(() => {
      extensionApi.walletGetPendingApprovals().then(setApprovals).catch(() => undefined);
    }, 1500);
    return () => clearInterval(id);
  }, []);

  useEffect(() => {
    if (!status?.unlocked) {
      setScanProgress(null);
      return;
    }
    let cancelled = false;
    async function tick() {
      try {
        const p = await extensionApi.walletScanProgress();
        if (!cancelled) setScanProgress(p);
      } catch {
        if (!cancelled) setScanProgress({ status: "idle" });
      }
    }
    tick();
    let id = setInterval(tick, 2000);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, [status?.unlocked]);

  // Poll faster while scanning so each integer % step is visible.
  useEffect(() => {
    if (!status?.unlocked || scanProgress?.status !== "scanning") return;
    let cancelled = false;
    const fast = setInterval(async () => {
      try {
        const p = await extensionApi.walletScanProgress();
        if (!cancelled) setScanProgress(p);
      } catch {
        /* ignore transient poll errors */
      }
    }, 400);
    return () => {
      cancelled = true;
      clearInterval(fast);
    };
  }, [status?.unlocked, scanProgress?.status]);

  const showAppBar = view !== "welcome" && view !== "unlock";
  const pageUnlocked = isFullPage() && Boolean(status?.unlocked);

  const lockWallet = async () => {
    await extensionApi.walletLock();
    await refresh();
  };

  const chainSync = showAppBar && status?.unlocked && view !== "settings" && (
    <ChainSyncPopup
      scan={scanProgress}
      onOpenSync={() => {
        setMoreOpen(false);
        setView("settings");
      }}
    />
  );

  const mainViews = (
    <>
      {bootDebug && (
        <div className="px-4 pt-3">
          <Callout tone="danger">{bootDebug}</Callout>
        </div>
      )}
      <PendingApprovals
        approvals={approvals}
        onApprove={async (id) => {
          await extensionApi.walletApproveRequest(id);
          setApprovals(await extensionApi.walletGetPendingApprovals());
        }}
        onReject={async (id) => {
          await extensionApi.walletRejectRequest(id);
          setApprovals(await extensionApi.walletGetPendingApprovals());
        }}
      />

      {!pageUnlocked && status?.unlocked && MORE_VIEWS.includes(view) && (
        <div
          className="px-2 py-1.5"
          style={{ borderBottom: "1px solid var(--nw-border)" }}
        >
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setView("dashboard")}
            style={{ color: "var(--nw-platinum)" }}
          >
            <span className="rotate-180">
              <Icon name="chevron" size={13} />
            </span>
            Home
          </Button>
        </div>
      )}

      {view === "welcome" && (
        <WelcomeView
          onCreated={() => {
            refresh().catch(console.error);
          }}
          onRestored={() => {
            refresh().catch(console.error);
          }}
        />
      )}
      {view === "unlock" && (
        <UnlockView onUnlocked={() => refresh().catch(console.error)} />
      )}
      {view === "dashboard" && (
          <DashboardView
            pageMode={pageUnlocked}
            status={status}
            txs={txs}
            scan={scanProgress}
          onRetry={async (id) => {
            await extensionApi.walletRetryBroadcast(id);
            await refresh();
          }}
          onSpeedUp={async (id) => {
            await extensionApi.walletSpeedUp(id);
            await refresh();
          }}
        />
      )}
      {view === "send" && <SendView />}
      {view === "receive" && <ReceiveView status={status} />}
      {view === "companion" && <CompanionView />}
      {view === "vote" && <VoteView />}
      {view === "crosslink" && <CrosslinkView />}
      {view === "browser" && <BrowserView />}
      {view === "settings" && (
        <SettingsView
          endpoint={endpoint}
          status={status}
          scan={scanProgress}
          onWalletMetaChanged={() => void refresh()}
          onScanProgress={setScanProgress}
          onEndpointChange={async (url) => {
            await extensionApi.rpcSetEndpoint(url);
            await refresh();
          }}
          onLock={lockWallet}
          onSetAutoLock={async (ms) => {
            await extensionApi.walletSetSessionPolicy(ms);
          }}
        />
      )}
    </>
  );

  if (pageUnlocked) {
    return (
      <FullWalletShell
        view={view}
        address={status?.address ?? null}
        onNavigate={(next) => {
          setMoreOpen(false);
          setView(next);
        }}
        onLock={() => void lockWallet()}
      >
        {chainSync}
        {mainViews}
      </FullWalletShell>
    );
  }

  return (
    <div className="relative flex h-full flex-col overflow-hidden">
      {showAppBar && (
        <AppHeader
          scan={scanProgress}
          unlocked={Boolean(status?.unlocked)}
          showBrand={view !== "dashboard"}
          onOpenSync={() => {
            setMoreOpen(false);
            setView("settings");
          }}
          onLock={lockWallet}
        />
      )}
      {chainSync}
      <div className="min-h-0 flex-1 overflow-y-auto">{mainViews}</div>

      {status?.unlocked && (
        <BottomNav
          view={view}
          moreOpen={moreOpen}
          onChange={(next) => {
            setMoreOpen(false);
            setView(next);
          }}
          onOpenMore={() => setMoreOpen((v) => !v)}
        />
      )}

      {moreOpen && (
        <MoreSheet
          onSelect={(next) => {
            setMoreOpen(false);
            if ((next === "crosslink" || next === "browser") && !isFullPage()) {
              void openWalletPage({ view: next });
              return;
            }
            setView(next);
          }}
          onClose={() => setMoreOpen(false)}
          onOpenFull={
            isFullPage()
              ? undefined
              : () => {
                  void openWalletPage({ view: "dashboard" });
                }
          }
          onLogout={lockWallet}
        />
      )}
    </div>
  );
}

