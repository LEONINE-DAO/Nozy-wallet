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
import { TopNav } from "./components/TopNav";
import { useUiStore } from "./store/uiStore";

const NETWORK_RPC_PRESETS = {
  mainnet: "https://zec.rocks:443",
  testnet: "https://testnet.zec.rocks:443",
  local18232: "http://127.0.0.1:18232",
  local8232: "http://127.0.0.1:8232"
} as const;

const NETWORK_LWD_PRESETS = {
  mainnet: "https://zec.rocks:443/",
  testnet: "https://testnet.zec.rocks:443/",
  local: "http://127.0.0.1:9067"
} as const;

function WelcomeView({
  onCreated,
  onRestored
}: {
  onCreated: () => void;
  onRestored: () => void;
}) {
  const [password, setPassword] = useState("");
  const [mnemonic, setMnemonic] = useState("");
  const [restoreBirthday, setRestoreBirthday] = useState("");
  const [mode, setMode] = useState<"create" | "restore">("create");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const submit = async () => {
    setBusy(true);
    setError(null);
    try {
      if (mode === "create") {
        await extensionApi.walletCreate(password);
        onCreated();
      } else {
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
        await extensionApi.walletRestore(mnemonic.trim(), password, restoreOpts);
        onRestored();
      }
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="space-y-3 p-4">
      <h1 className="text-lg font-semibold">NozyWallet Setup</h1>
      <div className="flex gap-2">
        <button
          className={`rounded px-3 py-1 text-sm ${mode === "create" ? "bg-amber-500 text-black" : "bg-white/10"}`}
          onClick={() => setMode("create")}
        >
          Create
        </button>
        <button
          className={`rounded px-3 py-1 text-sm ${mode === "restore" ? "bg-amber-500 text-black" : "bg-white/10"}`}
          onClick={() => setMode("restore")}
        >
          Restore
        </button>
      </div>
      {mode === "restore" && (
        <div className="space-y-2">
          <textarea
            className="h-24 w-full rounded bg-white/10 p-2 text-sm outline-none"
            placeholder="Enter 24-word mnemonic"
            value={mnemonic}
            onChange={(e) => setMnemonic(e.target.value)}
          />
          <input
            className="w-full rounded bg-white/10 p-2 text-[11px] outline-none font-mono"
            placeholder="Optional: Orchard birthday block (default = current RPC tip if reachable)"
            value={restoreBirthday}
            onChange={(e) => setRestoreBirthday(e.target.value)}
          />
        </div>
      )}
      <input
        className="w-full rounded bg-white/10 p-2 text-sm outline-none"
        placeholder="Password"
        type="password"
        value={password}
        onChange={(e) => setPassword(e.target.value)}
      />
      {error && <div className="text-xs text-red-300">{error}</div>}
      <button
        onClick={submit}
        disabled={busy || !password || (mode === "restore" && !mnemonic.trim())}
        className="w-full rounded bg-amber-500 p-2 text-sm font-medium text-black disabled:opacity-50"
      >
        {busy ? "Working..." : mode === "create" ? "Create Wallet" : "Restore Wallet"}
      </button>
    </div>
  );
}

function UnlockView({ onUnlocked }: { onUnlocked: () => void }) {
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  return (
    <div className="space-y-3 p-4">
      <h1 className="text-lg font-semibold">Unlock Wallet</h1>
      <input
        className="w-full rounded bg-white/10 p-2 text-sm outline-none"
        placeholder="Password"
        type="password"
        value={password}
        onChange={(e) => setPassword(e.target.value)}
      />
      {error && <div className="text-xs text-red-300">{error}</div>}
      <button
        onClick={async () => {
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
        }}
        disabled={!password || busy}
        className="w-full rounded bg-amber-500 p-2 text-sm font-medium text-black disabled:opacity-50"
      >
        {busy ? "Unlocking..." : "Unlock"}
      </button>
    </div>
  );
}

function DashboardView({
  status,
  txs,
  onRetry,
  scan
}: {
  status: WalletStatus | null;
  txs: TxStateEntry[];
  onRetry: (id: string) => Promise<void>;
  scan: WalletScanProgressResult | null;
}) {
  const [balanceZats, setBalanceZats] = useState<number | null>(null);

  const scanLabel = useMemo(() => {
    const p = scan;
    if (!p || p.status === "idle") return "No scan yet — go to Receive tab to start";
    if (p.status === "scanning") return `Scanning… ${p.percent ?? 0}%`;
    if (p.status === "done") return `Scanned ${p.scannedBlocks ?? 0} blocks (complete)`;
    if (p.status === "stopped") return `Scan stopped at ${p.percent ?? 0}%`;
    if (p.status === "failed") return p.scanError ? `Scan failed — ${p.scanError}` : "Scan failed";
    return "";
  }, [scan]);

  useEffect(() => {
    if (!scan) return;
    if (scan.status === "scanning" || scan.status === "done" || scan.status === "stopped") {
      setBalanceZats(scan.totalBalanceZats ?? 0);
    }
  }, [scan]);

  const zec = balanceZats !== null ? (balanceZats / 1e8).toFixed(8) : "—";

  return (
    <div className="space-y-3 p-4">
      <h1 className="text-lg font-semibold">Dashboard</h1>
      <div className="rounded border border-white/10 bg-white/5 p-3 text-sm">
        <div className="text-white/70">Address</div>
        <div className="break-all">{status?.address || "-"}</div>
      </div>
      <div className="rounded border border-white/10 bg-white/5 p-3 text-sm">
        <div className="text-white/70">Balance</div>
        <div className="text-xl font-semibold">{zec} <span className="text-sm font-normal text-white/50">ZEC</span></div>
        {scanLabel && <div className="text-xs text-white/40 mt-1">{scanLabel}</div>}
      </div>
      <div className="rounded border border-white/10 bg-white/5 p-3 text-sm">
        <div className="mb-1 text-white/70">Recent transactions</div>
        <div className="space-y-1 text-xs">
          {txs.length === 0 && <div className="text-white/60">No transactions yet.</div>}
          {txs.slice(-5).reverse().map((tx) => (
            <div key={tx.id} className="rounded bg-black/20 px-2 py-1">
              <div className="flex items-center justify-between">
                <span className="flex items-center gap-2 uppercase">
                  {tx.state}
                  {tx.inputMode && (
                    <span
                      className={`rounded px-1.5 py-0.5 text-[10px] ${
                        tx.inputMode === "multi"
                          ? "bg-amber-500/20 text-amber-200"
                          : "bg-green-500/20 text-green-200"
                      }`}
                    >
                      {tx.inputMode}{typeof tx.inputsUsed === "number" ? ` x${tx.inputsUsed}` : ""}
                    </span>
                  )}
                </span>
                <span>{tx.amount} zats</span>
              </div>
              <div className="truncate text-white/70">{tx.recipientAddress || tx.txid || "n/a"}</div>
              {tx.state === "failed" && (
                <button
                  className="mt-1 rounded bg-white/10 px-2 py-1 text-[10px]"
                  onClick={() => onRetry(tx.id)}
                >
                  Retry Broadcast
                </button>
              )}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function SendView() {
  const [status, setStatus] = useState<string>("");
  const [recipient, setRecipient] = useState("");
  const [amount, setAmount] = useState("");
  const [feeZats, setFeeZats] = useState("10000");
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

  async function runPreflight() {
    setBusy(true);
    setPreflight(null);
    setRawTxHex(null);
    try {
      const requestedAmount = Number(amount) || 0;
      if (requestedAmount <= 0) throw new Error("Enter an amount in zats");
      const fee = Math.max(0, Math.floor(Number(feeZats) || 0));
      if (fee <= 0) throw new Error("Enter a positive fee in zats (e.g. 10000)");
      const result = await extensionApi.walletProveTransaction({
        to: recipient.trim() || undefined,
        amount: requestedAmount,
        fee,
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

  return (
    <div className="space-y-3 p-4 text-sm">
      <h2 className="text-base font-semibold">Send ZEC</h2>

      <div>
        <label className="mb-1 block text-xs text-white/50">Recipient address (u1…)</label>
        <input
          className="w-full rounded bg-white/10 p-2 text-sm outline-none placeholder:text-white/30"
          placeholder="Leave blank to send to own address"
          value={recipient}
          onChange={(e) => setRecipient(e.target.value)}
        />
      </div>

      <div>
        <label className="mb-1 block text-xs text-white/50">Amount (zats)</label>
        <input
          className="w-full rounded bg-white/10 p-2 text-sm outline-none placeholder:text-white/30"
          placeholder="e.g. 100000 (= 0.001 ZEC)"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
        />
      </div>

      <div>
        <label className="mb-1 block text-xs text-white/50">Fee (zats)</label>
        <input
          className="w-full rounded bg-white/10 p-2 text-sm outline-none placeholder:text-white/30"
          placeholder="10000"
          value={feeZats}
          onChange={(e) => setFeeZats(e.target.value)}
        />
        <p className="mt-1 text-[10px] text-white/40">
          Zebrad has no estimatefee RPC; set fee manually (10000 zats is a common default).
        </p>
      </div>

      <div>
        <label className="mb-1 block text-xs text-white/50">Memo (optional, max 512 bytes)</label>
        <textarea
          className="w-full rounded bg-white/10 p-2 text-sm outline-none placeholder:text-white/30 resize-none"
          rows={2}
          placeholder="Private message attached to the transaction"
          value={memo}
          onChange={(e) => setMemo(e.target.value)}
        />
      </div>

      {!preflight ? (
        <button
          className="w-full rounded bg-amber-500 py-2 font-medium text-black disabled:opacity-50"
          disabled={busy || !amount || !feeZats}
          onClick={runPreflight}
        >
          {busy ? "Building transaction…" : "Preview Transaction"}
        </button>
      ) : (
        <div className="space-y-2">
          <div className="rounded border border-white/10 bg-white/5 p-3 text-xs space-y-1">
            <div className="flex justify-between">
              <span className="text-white/50">Amount</span>
              <span>{preflight.requestedAmount} zats ({(preflight.requestedAmount / 1e8).toFixed(8)} ZEC)</span>
            </div>
            <div className="flex justify-between">
              <span className="text-white/50">Fee</span>
              <span>{preflight.fee} zats</span>
            </div>
            <div className="flex justify-between">
              <span className="text-white/50">Input notes</span>
              <span>{preflight.selectedNotesCount} ({preflight.selectedNotesTotalValue} zats)</span>
            </div>
            {memo && (
              <div className="flex justify-between">
                <span className="text-white/50">Memo</span>
                <span className="max-w-[180px] truncate">{memo}</span>
              </div>
            )}
          </div>
          <div className="flex gap-2">
            <button
              className="flex-1 rounded bg-green-600 py-2 font-medium text-white disabled:opacity-50"
              disabled={busy}
              onClick={broadcast}
            >
              {busy ? "Broadcasting…" : "Confirm & Send"}
            </button>
            <button
              className="rounded border border-white/20 px-3 py-2 text-white/60"
              onClick={() => { setPreflight(null); setRawTxHex(null); setStatus(""); }}
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {status && <div className="text-xs text-white/70 mt-1">{status}</div>}
    </div>
  );
}

/** Orchard (NU5) activation — use as scan start when you need “all Orchard” on that network. */
const NU5_ORCHARD_START_MAINNET = 1_687_104;
const NU5_ORCHARD_START_TESTNET = 1_842_420;

function ReceiveView({
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
  const percent = Math.min(100, Math.max(0, scan?.percent ?? 0));

  const scanInfo = useMemo(() => {
    if (!scan) return "";
    const range =
      typeof scan.startHeight === "number" && typeof scan.endHeight === "number"
        ? ` (${scan.startHeight.toLocaleString()}–${scan.endHeight.toLocaleString()})`
        : "";
    if (scan.status === "scanning") {
      const elapsed = ((scan.elapsed ?? 0) / 1000).toFixed(0);
      const warn =
        typeof scan.lastRpcError === "string" && scan.lastRpcError.trim()
          ? ` — RPC: ${scan.lastRpcError.slice(0, 120)}${scan.lastRpcError.length > 120 ? "…" : ""}`
          : "";
      return `Scanning… ${scan.percent ?? 0}% (${scan.scannedBlocks ?? 0}/${scan.totalBlocks ?? 0} blocks, ${scan.discoveredNotes ?? 0} notes, ${elapsed}s)${range}${warn}`;
    }
    if (scan.status === "done") {
      const elapsed = ((scan.elapsed ?? 0) / 1000).toFixed(1);
      const zec = ((scan.totalBalanceZats ?? 0) / 1e8).toFixed(8);
      return `Done in ${elapsed}s — ${scan.scannedBlocks ?? 0} blocks, ${scan.discoveredNotes ?? 0} notes, balance: ${zec} ZEC${range}`;
    }
    if (scan.status === "stopped") {
      return `Scan stopped at ${scan.percent ?? 0}% (${scan.scannedBlocks ?? 0}/${scan.totalBlocks ?? 0} blocks)${range}`;
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
    <div className="space-y-2 p-4 text-sm">
      <h2 className="text-base font-semibold">Receive</h2>
      <div className="rounded border border-white/10 bg-white/5 p-3 break-all">
        {status?.address || "No address yet"}
      </div>

      {scanning && (
        <div className="h-1.5 w-full rounded bg-white/10 overflow-hidden">
          <div
            className="h-full bg-amber-500 transition-all duration-500"
            style={{ width: `${percent}%` }}
          />
        </div>
      )}

      <p className="text-[10px] leading-snug text-white/45">
        <span className="text-white/60">Orchard-only</span> scan. Default scan uses your saved{" "}
        <span className="text-white/60">creation height</span> (chain tip at create/restore, unless you change it) through
        tip — no blocks before the wallet existed. Restored seed with older funds: lower birthday or use{" "}
        <span className="text-white/60">NU5 → tip</span> / <span className="text-white/60">Full chain</span>.
      </p>

      <div className="rounded border border-white/10 bg-white/5 p-2 space-y-2 text-[11px]">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <span className="text-white/55">
            Chain tip:{" "}
            <span className="font-mono text-white/85">{chainTip !== null ? chainTip.toLocaleString() : "—"}</span>
          </span>
          <button
            type="button"
            className="rounded bg-white/10 px-2 py-0.5 text-[10px] text-white/80"
            onClick={() => refreshChainTip()}
          >
            Refresh tip
          </button>
        </div>
        <div className="grid grid-cols-2 gap-2">
          <label className="space-y-0.5">
            <span className="text-white/45">Start height</span>
            <input
              className="w-full rounded bg-black/30 px-2 py-1 font-mono text-[11px] outline-none"
              value={scanStartStr}
              onChange={(e) => setScanStartStr(e.target.value)}
              inputMode="numeric"
              disabled={scanning}
            />
          </label>
          <label className="space-y-0.5">
            <span className="text-white/45">End height</span>
            <input
              className="w-full rounded bg-black/30 px-2 py-1 font-mono text-[11px] outline-none"
              value={scanEndStr}
              onChange={(e) => setScanEndStr(e.target.value)}
              inputMode="numeric"
              disabled={scanning}
            />
          </label>
        </div>
        <button
          type="button"
          disabled={scanning}
          className="w-full rounded bg-amber-500 py-1.5 text-[12px] font-medium text-black disabled:opacity-40"
          onClick={() => void startScanBirthdayToTip()}
        >
          Start scan (saved birthday → tip)
        </button>
        <div className="border-t border-white/10 pt-2 space-y-1">
          <div className="text-[10px] text-white/45">
            Saved Orchard birthday (default scan start):{" "}
            <span className="font-mono text-white/80">
              {typeof status?.orchardBirthdayHeight === "number"
                ? status.orchardBirthdayHeight.toLocaleString()
                : "not set"}
            </span>
          </div>
          <div className="flex gap-1">
            <input
              className="min-w-0 flex-1 rounded bg-black/30 px-2 py-1 font-mono text-[11px] outline-none"
              value={birthdayEditStr}
              onChange={(e) => setBirthdayEditStr(e.target.value)}
              placeholder="Block height"
              disabled={scanning}
            />
            <button
              type="button"
              className="shrink-0 rounded bg-white/15 px-2 py-1 text-[10px] text-white/85 disabled:opacity-40"
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
            </button>
          </div>
        </div>
      </div>

      <div className="text-[10px] text-white/40 space-y-0.5">
        <div>Quick presets (end = current chain tip)</div>
        <div className="text-white/35 leading-snug">
          <span className="text-white/50">Last N:</span> scans only block heights{" "}
          <span className="font-mono text-white/55">tip − N</span> through <span className="font-mono text-white/55">tip</span>{" "}
          (both inclusive; that is <span className="font-mono text-white/55">N + 1</span> heights when the chain is long
          enough). Your saved birthday is <span className="text-white/50">not</span> used for these buttons. Nothing
          outside that interval is scanned.
        </div>
      </div>
      <div className="flex flex-wrap gap-2 items-center">
        {!scan ? (
          <span className="text-xs text-white/50 py-1">Checking scan status…</span>
        ) : !scanning ? (
          <>
            <button
              type="button"
              className="rounded bg-emerald-600 px-2 py-1 text-[11px] font-medium text-white"
              onClick={() => void startScanBirthdayToTip()}
            >
              Birthday → tip
            </button>
            <button
              type="button"
              className="rounded bg-amber-500 px-2 py-1 text-[11px] text-black"
              onClick={() => void startScanWindow(20_000)}
            >
              Last 20k
            </button>
            <button
              type="button"
              className="rounded bg-amber-600/90 px-2 py-1 text-[11px] text-black"
              onClick={() => void startScanWindow(100_000)}
            >
              Last 100k
            </button>
            <button
              type="button"
              className="rounded bg-amber-700/80 px-2 py-1 text-[11px] text-white"
              onClick={() => void startScanWindow(500_000)}
            >
              Last 500k
            </button>
            {chainTip !== null && (
              <>
                <button
                  type="button"
                  className="rounded bg-violet-600/90 px-2 py-1 text-[10px] text-white"
                  title="Mainnet NU5 / Orchard activation height"
                  onClick={() =>
                    void startScanRange(Math.min(NU5_ORCHARD_START_MAINNET, chainTip), chainTip)
                  }
                >
                  NU5 → tip (mainnet)
                </button>
                <button
                  type="button"
                  className="rounded bg-violet-700/70 px-2 py-1 text-[10px] text-white"
                  title="Testnet NU5 / Orchard activation height"
                  onClick={() =>
                    void startScanRange(Math.min(NU5_ORCHARD_START_TESTNET, chainTip), chainTip)
                  }
                >
                  NU5 → tip (testnet)
                </button>
                <button
                  type="button"
                  className="rounded border border-amber-500/40 px-2 py-1 text-[10px] text-amber-100"
                  onClick={() => void startScanRange(0, chainTip)}
                >
                  Full chain 0 → tip
                </button>
              </>
            )}
          </>
        ) : (
          <button
            className="rounded bg-red-600 px-3 py-1 text-white"
            onClick={async () => {
              setActionMsg("");
              try {
                await extensionApi.walletStopScan();
              } catch (_) {}
            }}
          >
            Stop Scan
          </button>
        )}
      </div>

      {actionMsg && <div className="text-xs text-red-300">{actionMsg}</div>}
      {scanInfo && <div className="text-xs text-white/70">{scanInfo}</div>}
    </div>
  );
}

function CompanionView() {
  const [baseUrl, setBaseUrl] = useState("http://127.0.0.1:3000");
  const [lwdUrl, setLwdUrl] = useState("https://testnet.zec.rocks:443/");
  const [log, setLog] = useState("");
  const [busy, setBusy] = useState(false);
  const [syncStart, setSyncStart] = useState("0");
  const [syncEnd, setSyncEnd] = useState("");

  useEffect(() => {
    getCompanionPrefs()
      .then((p) => {
        setBaseUrl(p.baseUrl);
        setLwdUrl(p.lightwalletdUrl);
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

  return (
    <div className="space-y-3 p-4 text-sm">
      <h2 className="text-base font-semibold">Local API (lightwalletd)</h2>
      <p className="text-[11px] leading-relaxed text-white/55">
        <span className="font-medium text-white/75">Easiest full wallet:</span> install{" "}
        <a
          className="text-amber-300 underline"
          href="https://github.com/LEONINE-DAO/Nozy-wallet/releases"
          target="_blank"
          rel="noreferrer"
        >
          Nozy Desktop
        </a>{" "}
        (Tauri). This tab is the <span className="text-white/70">lighter path</span>: WASM in the
        extension + <span className="font-mono text-white/70">nozywallet-api</span> on your PC for
        zeaking/lightwalletd compact sync—no Zebrad required on the same machine for that sync step.
      </p>

      <div className="rounded border border-white/10 bg-white/5 p-3 space-y-2">
        <div>
          <div className="mb-1 text-[11px] text-white/60">Nozy API base URL</div>
          <input
            className="w-full rounded bg-white/10 p-2 text-xs outline-none font-mono"
            value={baseUrl}
            onChange={(e) => setBaseUrl(e.target.value)}
            placeholder="http://127.0.0.1:3000"
          />
        </div>
        <div>
          <div className="mb-1 text-[11px] text-white/60">lightwalletd gRPC (optional override)</div>
          <div className="mb-2">
            <div className="mb-1 text-[11px] text-white/60">Network preset</div>
            <select
              className="w-full rounded bg-white/10 p-2 text-xs outline-none"
              onChange={(e) => {
                const v = e.target.value;
                if (v === "mainnet") setLwdUrl(NETWORK_LWD_PRESETS.mainnet);
                else if (v === "testnet") setLwdUrl(NETWORK_LWD_PRESETS.testnet);
                else if (v === "local") setLwdUrl(NETWORK_LWD_PRESETS.local);
              }}
              value={
                lwdUrl === NETWORK_LWD_PRESETS.mainnet
                  ? "mainnet"
                  : lwdUrl === NETWORK_LWD_PRESETS.testnet
                    ? "testnet"
                    : lwdUrl === NETWORK_LWD_PRESETS.local
                      ? "local"
                      : "custom"
              }
            >
              <option value="testnet">Testnet</option>
              <option value="mainnet">Mainnet</option>
              <option value="local">Local</option>
              <option value="custom">Custom</option>
            </select>
          </div>
          <input
            className="w-full rounded bg-white/10 p-2 text-xs outline-none font-mono"
            value={lwdUrl}
            onChange={(e) => setLwdUrl(e.target.value)}
            placeholder="https://testnet.zec.rocks:443/"
          />
          <div className="mt-2 flex flex-wrap gap-2 text-[11px]">
            <button
              type="button"
              className="rounded bg-white/10 px-2 py-1"
              onClick={() => setLwdUrl("https://testnet.zec.rocks:443/")}
            >
              Testnet zec.rocks
            </button>
            <button
              type="button"
              className="rounded bg-white/10 px-2 py-1"
              onClick={() => setLwdUrl("https://zec.rocks:443/")}
            >
              Mainnet zec.rocks
            </button>
            <button
              type="button"
              className="rounded bg-white/10 px-2 py-1"
              onClick={() => setLwdUrl("http://127.0.0.1:9067")}
            >
              Local 9067
            </button>
          </div>
        </div>
        <button
          type="button"
          disabled={busy}
          className="rounded bg-white/15 px-3 py-1 text-xs"
          onClick={() =>
            run(async () => {
              await setCompanionPrefs({ baseUrl, lightwalletdUrl: lwdUrl });
              setLog("Saved companion URLs to extension storage.");
            })
          }
        >
          Save URLs
        </button>
      </div>

      <div className="flex flex-wrap gap-2">
        <button
          type="button"
          disabled={busy}
          className="rounded bg-amber-500 px-3 py-1 text-xs font-medium text-black"
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
            })
          }
        >
          Check API
        </button>
        <button
          type="button"
          disabled={busy}
          className="rounded bg-white/15 px-3 py-1 text-xs"
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
          GetLightdInfo
        </button>
        <button
          type="button"
          disabled={busy}
          className="rounded bg-white/15 px-3 py-1 text-xs"
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
        </button>
      </div>

      <div className="rounded border border-white/10 bg-white/5 p-3 space-y-2">
        <div className="text-[11px] text-white/60">POST compact sync (writes desktop SQLite via API)</div>
        <div className="flex gap-2">
          <input
            className="w-24 rounded bg-white/10 p-1.5 text-xs outline-none"
            value={syncStart}
            onChange={(e) => setSyncStart(e.target.value)}
            placeholder="start"
          />
          <input
            className="w-24 rounded bg-white/10 p-1.5 text-xs outline-none"
            value={syncEnd}
            onChange={(e) => setSyncEnd(e.target.value)}
            placeholder="end (opt)"
          />
        </div>
        <button
          type="button"
          disabled={busy}
          className="rounded bg-green-700 px-3 py-1 text-xs text-white"
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
            })
          }
        >
          Sync compact range
        </button>
      </div>

      {log && (
        <pre className="max-h-48 overflow-auto rounded bg-black/30 p-2 text-[10px] text-white/80 whitespace-pre-wrap">
          {log}
        </pre>
      )}
    </div>
  );
}

function SettingsView({
  endpoint,
  onEndpointChange,
  onLock,
  onSetAutoLock
}: {
  endpoint: string;
  onEndpointChange: (url: string) => void;
  onLock: () => void;
  onSetAutoLock: (ms: number) => Promise<void>;
}) {
  const [value, setValue] = useState(endpoint);
  const [msg, setMsg] = useState<string | null>(null);
  const [autoLockMin, setAutoLockMin] = useState("15");

  return (
    <div className="space-y-3 p-4 text-sm">
      <h2 className="text-base font-semibold">Settings</h2>
      <div>
        <div className="mb-1 text-white/70">RPC endpoint</div>
        <div className="mb-2">
          <div className="mb-1 text-[11px] text-white/60">Network preset</div>
          <select
            className="w-full rounded bg-white/10 p-2 text-xs outline-none"
            onChange={(e) => {
              const v = e.target.value;
              if (v === "mainnet") setValue(NETWORK_RPC_PRESETS.mainnet);
              else if (v === "testnet") setValue(NETWORK_RPC_PRESETS.testnet);
              else if (v === "local18232") setValue(NETWORK_RPC_PRESETS.local18232);
              else if (v === "local8232") setValue(NETWORK_RPC_PRESETS.local8232);
            }}
            value={
              value === NETWORK_RPC_PRESETS.mainnet
                ? "mainnet"
                : value === NETWORK_RPC_PRESETS.testnet
                  ? "testnet"
                  : value === NETWORK_RPC_PRESETS.local18232
                    ? "local18232"
                    : value === NETWORK_RPC_PRESETS.local8232
                      ? "local8232"
                      : "custom"
            }
          >
            <option value="mainnet">Mainnet</option>
            <option value="testnet">Testnet</option>
            <option value="local18232">Local 18232</option>
            <option value="local8232">Local 8232</option>
            <option value="custom">Custom</option>
          </select>
        </div>
        <input
          className="w-full rounded bg-white/10 p-2 outline-none"
          value={value}
          onChange={(e) => setValue(e.target.value)}
        />
        <div className="mt-2 flex flex-wrap gap-2 text-xs">
          <button
            className="rounded bg-white/10 px-2 py-1"
            onClick={() => setValue("http://127.0.0.1:18232")}
          >
            Local 18232
          </button>
          <button
            className="rounded bg-white/10 px-2 py-1"
            onClick={() => setValue("http://127.0.0.1:8232")}
          >
            Local 8232
          </button>
          <button
            className="rounded bg-white/10 px-2 py-1"
            onClick={() => setValue("https://zec.rocks:443")}
          >
            zec.rocks:443
          </button>
        </div>
        <button
          className="mt-2 rounded bg-amber-500 px-3 py-1 text-black"
          onClick={async () => {
            await onEndpointChange(value);
            setMsg("Saved.");
          }}
        >
          Save
        </button>
        {msg && <div className="mt-1 text-xs text-green-300">{msg}</div>}
        <div className="mt-1 text-[11px] text-white/60">
          Tip: `zec.rocks443` is accepted and auto-normalized to `https://zec.rocks:443`.
        </div>
      </div>
      <button className="rounded bg-white/10 px-3 py-1" onClick={onLock}>
        Lock wallet
      </button>
      <div className="rounded border border-white/10 bg-white/5 p-3">
        <div className="mb-1 text-white/70">Auto-lock timeout (minutes)</div>
        <div className="flex items-center gap-2">
          <input
            className="w-24 rounded bg-white/10 p-2 outline-none"
            value={autoLockMin}
            onChange={(e) => setAutoLockMin(e.target.value)}
          />
          <button
            className="rounded bg-amber-500 px-3 py-1 text-black"
            onClick={async () => {
              const mins = Math.max(1, Number(autoLockMin) || 15);
              await onSetAutoLock(mins * 60_000);
              setMsg("Session policy saved.");
            }}
          >
            Save
          </button>
        </div>
      </div>
    </div>
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
    <div className="space-y-2 border-b border-white/10 p-3">
      <div className="text-xs font-medium text-amber-300">Pending dApp requests</div>
      {approvals.map((a) => (
        <div key={a.id} className="rounded border border-white/10 bg-white/5 p-2 text-xs">
          <div>{a.kind.toUpperCase()} request</div>
          {a.kind === "transaction" && (
            <div className="my-1 flex items-center gap-2">
              {(() => {
                const p = (a.payload as Record<string, unknown>)?.preflight as
                  | { input_mode?: string; inputs_used?: number }
                  | undefined;
                if (!p) return <span className="text-white/60">Preflight pending</span>;
                const mode = String(p.input_mode || "single");
                const used = Number(p.inputs_used ?? 0);
                return (
                  <span
                    className={`rounded px-1.5 py-0.5 text-[10px] ${
                      mode === "multi"
                        ? "bg-amber-500/20 text-amber-200"
                        : "bg-green-500/20 text-green-200"
                    }`}
                  >
                    {mode} x{used}
                  </span>
                );
              })()}
            </div>
          )}
          {a.kind === "transaction" &&
            Boolean((a.payload as Record<string, unknown>)?.preflightError) && (
              <div className="mb-1 rounded bg-amber-500/10 px-2 py-1 text-[10px] text-amber-200">
                Preflight warning:{" "}
                {String((a.payload as Record<string, unknown>)?.preflightError)}
              </div>
            )}
          <div className="mb-2 break-all text-white/70">{JSON.stringify(a.payload)}</div>
          <div className="flex gap-2">
            <button className="rounded bg-amber-500 px-2 py-1 text-black" onClick={() => onApprove(a.id)}>
              Approve
            </button>
            <button className="rounded bg-white/10 px-2 py-1" onClick={() => onReject(a.id)}>
              Reject
            </button>
          </div>
        </div>
      ))}
    </div>
  );
}

export function App() {
  const { view, setView } = useUiStore();
  const [status, setStatus] = useState<WalletStatus | null>(null);
  const [approvals, setApprovals] = useState<PendingApproval[]>([]);
  const [txs, setTxs] = useState<TxStateEntry[]>([]);
  const [bootDebug, setBootDebug] = useState<string | null>(null);
  /** Orchard block scan progress; polled app-wide so it keeps updating when you leave Receive. */
  const [scanProgress, setScanProgress] = useState<WalletScanProgressResult | null>(null);
  const endpoint = useMemo(() => status?.rpcEndpoint || "http://127.0.0.1:8232", [status]);

  const refresh = async () => {
    try {
      setBootDebug("startup: wallet_status");
      const nextStatus = await extensionApi.walletStatus();
      setStatus(nextStatus);
      if (!nextStatus.exists) setView("welcome");
      else if (!nextStatus.unlocked) setView("unlock");
      else if (view === "welcome" || view === "unlock") setView("dashboard");

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
    const id = setInterval(tick, 2000);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, [status?.unlocked]);

  return (
    <div className="h-full overflow-auto">
      {bootDebug && (
        <div className="mx-3 mt-3 rounded border border-red-400/40 bg-red-500/10 px-2 py-1 text-[10px] text-red-200">
          {bootDebug}
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

      {status?.unlocked && (
        <TopNav
          view={view}
          onChange={(next) => {
            setView(next);
          }}
        />
      )}

      {status?.unlocked && scanProgress?.status === "scanning" && (
        <div className="mx-3 mt-2 rounded border border-amber-500/35 bg-amber-500/10 px-3 py-2 text-xs text-amber-100">
          <span className="font-medium">Orchard scan</span>{" "}
          {Math.min(100, Math.max(0, Number(scanProgress.percent ?? 0))).toFixed(2)}% ·{" "}
          {(scanProgress.scannedBlocks ?? 0).toLocaleString()}/{(scanProgress.totalBlocks ?? 0).toLocaleString()} blocks ·{" "}
          {scanProgress.discoveredNotes ?? 0} notes
          <span className="mt-1 block text-[10px] leading-snug text-white/45">
            Runs in the background while the wallet stays unlocked — switch tabs freely.
          </span>
        </div>
      )}
      {status?.unlocked && scanProgress?.status === "failed" && scanProgress.scanError && (
        <div className="mx-3 mt-2 rounded border border-red-400/40 bg-red-500/10 px-3 py-2 text-[11px] text-red-200">
          Scan failed: {scanProgress.scanError}
        </div>
      )}

      {view === "welcome" && (
        <WelcomeView
          onCreated={() => refresh().catch(console.error)}
          onRestored={() => refresh().catch(console.error)}
        />
      )}
      {view === "unlock" && <UnlockView onUnlocked={() => refresh().catch(console.error)} />}
      {view === "dashboard" && (
        <DashboardView
          status={status}
          txs={txs}
          scan={scanProgress}
          onRetry={async (id) => {
            await extensionApi.walletRetryBroadcast(id);
            await refresh();
          }}
        />
      )}
      {view === "send" && <SendView />}
      {view === "receive" && (
        <ReceiveView status={status} scan={scanProgress} onWalletMetaChanged={() => void refresh()} />
      )}
      {view === "companion" && <CompanionView />}
      {view === "settings" && (
        <SettingsView
          endpoint={endpoint}
          onEndpointChange={async (url) => {
            await extensionApi.rpcSetEndpoint(url);
            await refresh();
          }}
          onLock={async () => {
            await extensionApi.walletLock();
            await refresh();
          }}
          onSetAutoLock={async (ms) => {
            await extensionApi.walletSetSessionPolicy(ms);
          }}
        />
      )}
    </div>
  );
}

