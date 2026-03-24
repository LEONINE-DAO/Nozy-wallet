import { useEffect, useMemo, useState } from "react";
import QRCode from "qrcode";
import {
  extensionApi,
  getCompanionPrefs,
  setCompanionPrefs,
  type MobileSyncState,
  type PendingApproval,
  type TxStateEntry,
  type WalletStatus
} from "./lib/extensionApi";
import { TopNav } from "./components/TopNav";
import { useUiStore } from "./store/uiStore";

function WelcomeView({
  onCreated,
  onRestored
}: {
  onCreated: () => void;
  onRestored: () => void;
}) {
  const [password, setPassword] = useState("");
  const [mnemonic, setMnemonic] = useState("");
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
        await extensionApi.walletRestore(mnemonic.trim(), password);
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
        <textarea
          className="h-24 w-full rounded bg-white/10 p-2 text-sm outline-none"
          placeholder="Enter 24-word mnemonic"
          value={mnemonic}
          onChange={(e) => setMnemonic(e.target.value)}
        />
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
  onRetry
}: {
  status: WalletStatus | null;
  txs: TxStateEntry[];
  onRetry: (id: string) => Promise<void>;
}) {
  const [balanceZats, setBalanceZats] = useState<number | null>(null);
  const [scanLabel, setScanLabel] = useState("");

  useEffect(() => {
    let timer: ReturnType<typeof setInterval> | null = null;
    async function poll() {
      try {
        const p = await extensionApi.walletScanProgress();
        if (p.status === "done") {
          setBalanceZats(p.totalBalanceZats ?? 0);
          setScanLabel(`Scanned ${p.scannedBlocks} blocks`);
        } else if (p.status === "scanning") {
          setBalanceZats(p.totalBalanceZats ?? 0);
          setScanLabel(`Scanning… ${p.percent}%`);
        } else if (p.status === "stopped") {
          setBalanceZats(p.totalBalanceZats ?? 0);
          setScanLabel(`Scan stopped at ${p.percent}%`);
        } else {
          setScanLabel("No scan yet — go to Receive tab to start");
        }
      } catch (_) {}
    }
    poll();
    timer = setInterval(poll, 2000);
    return () => { if (timer) clearInterval(timer); };
  }, []);

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

function ReceiveView({ status }: { status: WalletStatus | null }) {
  const [scanInfo, setScanInfo] = useState<string>("");
  const [scanStatus, setScanStatus] = useState<string>("idle");
  const [percent, setPercent] = useState(0);

  useEffect(() => {
    let timer: ReturnType<typeof setInterval> | null = null;

    async function poll() {
      try {
        const p = await extensionApi.walletScanProgress();
        setScanStatus(p.status ?? "idle");
        setPercent(p.percent ?? 0);
        if (p.status === "scanning") {
          const elapsed = ((p.elapsed ?? 0) / 1000).toFixed(0);
          setScanInfo(
            `Scanning… ${p.percent}% (${p.scannedBlocks}/${p.totalBlocks} blocks, ${p.discoveredNotes} notes, ${elapsed}s)`
          );
        } else if (p.status === "done") {
          const elapsed = ((p.elapsed ?? 0) / 1000).toFixed(1);
          const zec = ((p.totalBalanceZats ?? 0) / 1e8).toFixed(8);
          setScanInfo(
            `Done in ${elapsed}s — ${p.scannedBlocks} blocks, ${p.discoveredNotes} notes, balance: ${zec} ZEC`
          );
        } else if (p.status === "stopped") {
          setScanInfo(`Scan stopped at ${p.percent}% (${p.scannedBlocks}/${p.totalBlocks} blocks)`);
        }
      } catch (_) {}
    }

    poll();
    timer = setInterval(poll, 1500);
    return () => { if (timer) clearInterval(timer); };
  }, []);

  const scanning = scanStatus === "scanning";

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

      <div className="flex gap-2">
        {!scanning ? (
          <button
            className="rounded bg-amber-500 px-3 py-1 text-black"
            onClick={async () => {
              try {
                await extensionApi.walletStartScan(20_000);
                setScanStatus("scanning");
                setScanInfo("Starting scan…");
              } catch (e) {
                setScanInfo((e as Error).message);
              }
            }}
          >
            Scan Blocks (20k)
          </button>
        ) : (
          <button
            className="rounded bg-red-600 px-3 py-1 text-white"
            onClick={async () => {
              try {
                await extensionApi.walletStopScan();
                setScanStatus("stopped");
              } catch (_) {}
            }}
          >
            Stop Scan
          </button>
        )}
      </div>

      {scanInfo && <div className="text-xs text-white/70">{scanInfo}</div>}
    </div>
  );
}

function CompanionView() {
  const [baseUrl, setBaseUrl] = useState("http://127.0.0.1:3000");
  const [lwdUrl, setLwdUrl] = useState("");
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
          <input
            className="w-full rounded bg-white/10 p-2 text-xs outline-none font-mono"
            value={lwdUrl}
            onChange={(e) => setLwdUrl(e.target.value)}
            placeholder="http://127.0.0.1:9067"
          />
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
  const [syncState, setSyncState] = useState<MobileSyncState | null>(null);
  const [syncMsg, setSyncMsg] = useState<string | null>(null);
  const [schemaMsg, setSchemaMsg] = useState<string | null>(null);
  const [autoLockMin, setAutoLockMin] = useState("15");
  const [pairingSignature, setPairingSignature] = useState("");
  const [pairingQrDataUrl, setPairingQrDataUrl] = useState<string | null>(null);
  const [mobileResponsePayload, setMobileResponsePayload] = useState("");
  const [deviceDraftNames, setDeviceDraftNames] = useState<Record<string, string>>({});

  const refreshSyncState = async () => {
    const state = await extensionApi.mobileSyncGetState();
    setSyncState(state);
  };

  useEffect(() => {
    refreshSyncState().catch(() => undefined);
  }, []);

  useEffect(() => {
    const payload = syncState?.pairingPayload || "";
    if (!payload) {
      setPairingQrDataUrl(null);
      return;
    }
    QRCode.toDataURL(payload, { margin: 1, width: 180 })
      .then((dataUrl) => setPairingQrDataUrl(dataUrl))
      .catch(() => setPairingQrDataUrl(null));
  }, [syncState?.pairingPayload]);

  return (
    <div className="space-y-3 p-4 text-sm">
      <h2 className="text-base font-semibold">Settings</h2>
      <div>
        <div className="mb-1 text-white/70">RPC endpoint</div>
        <input
          className="w-full rounded bg-white/10 p-2 outline-none"
          value={value}
          onChange={(e) => setValue(e.target.value)}
        />
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

      <div className="rounded border border-white/10 bg-white/5 p-3">
        <div className="mb-2 text-white/70">Mobile Sync (Protocol v1 seed-on-device)</div>
        <div className="flex flex-wrap gap-2">
          <button
            className="rounded bg-amber-500 px-3 py-1 text-black"
            onClick={async () => {
              try {
                const pairing = await extensionApi.mobileSyncInitPairing();
                setSyncMsg(
                  `Pairing session ${pairing.sessionId.slice(0, 10)}... code ${pairing.verifyCode}`
                );
                await refreshSyncState();
              } catch (e) {
                setSyncMsg((e as Error).message);
              }
            }}
          >
            Start Pairing
          </button>
          <button
            className="rounded bg-white/10 px-3 py-1"
            onClick={async () => {
              try {
                const state = await extensionApi.mobileSyncGetState();
                if (!state.activePairing) {
                  setSyncMsg("No active pairing session.");
                  return;
                }
                await extensionApi.mobileSyncConfirmPairing(
                  state.activePairing.sessionId,
                  "Nozy Mobile",
                  "ios-android",
                  pairingSignature.trim()
                );
                setSyncMsg("Pairing confirmed (signature verified).");
                setPairingSignature("");
                await refreshSyncState();
              } catch (e) {
                setSyncMsg((e as Error).message);
              }
            }}
          >
            Confirm Pairing (Signed)
          </button>
          <button
            className="rounded bg-white/10 px-3 py-1"
            onClick={async () => {
              try {
                const schema = await extensionApi.mobileSyncGetPairingSchema();
                setSchemaMsg(JSON.stringify(schema, null, 2));
              } catch (e) {
                setSchemaMsg((e as Error).message);
              }
            }}
          >
            Show Pairing Schema
          </button>
        </div>

        {syncState?.activePairing && (
          <div className="mt-2 break-all text-xs text-white/80">
            Active session: {syncState.activePairing.sessionId} (code{" "}
            {syncState.activePairing.verifyCode})
          </div>
        )}
        <div className="mt-2">
          <div className="mb-1 text-xs text-white/70">Challenge signature (from mobile)</div>
          <input
            className="w-full rounded bg-white/10 p-2 text-xs outline-none"
            value={pairingSignature}
            onChange={(e) => setPairingSignature(e.target.value)}
            placeholder="Paste mobile signature for active challenge"
          />
        </div>

        {syncState?.pairingPayload && (
          <div className="mt-2">
            <div className="mb-1 text-xs text-white/70">Scan-ready pairing payload</div>
            <textarea
              readOnly
              className="h-20 w-full rounded bg-black/20 p-2 text-xs outline-none"
              value={syncState.pairingPayload}
            />
            <button
              className="mt-1 rounded bg-white/10 px-2 py-1 text-xs"
              onClick={async () => {
                await navigator.clipboard.writeText(syncState.pairingPayload || "");
                setSyncMsg("Pairing payload copied.");
              }}
            >
              Copy Payload
            </button>
            {pairingQrDataUrl && (
              <div className="mt-2">
                <div className="mb-1 text-xs text-white/70">QR code (mobile scans this)</div>
                <img
                  src={pairingQrDataUrl}
                  alt="Pairing QR"
                  className="h-44 w-44 rounded border border-white/10 bg-white p-1"
                />
              </div>
            )}
          </div>
        )}

        <div className="mt-2">
          <div className="mb-1 text-xs text-white/70">Mobile response payload (QR scan result)</div>
          <textarea
            className="h-16 w-full rounded bg-black/20 p-2 text-xs outline-none"
            value={mobileResponsePayload}
            onChange={(e) => setMobileResponsePayload(e.target.value)}
            placeholder='{"sessionId":"...","challengeSignature":"...","deviceName":"Nozy Mobile","platform":"ios"}'
          />
          <button
            className="mt-1 rounded bg-amber-500 px-2 py-1 text-xs text-black"
            onClick={async () => {
              try {
                const parsed = JSON.parse(mobileResponsePayload || "{}") as Record<string, unknown>;
                const sessionId = String(parsed.sessionId ?? "");
                const challengeSignature = String(parsed.challengeSignature ?? "");
                const deviceName = String(parsed.deviceName ?? "Nozy Mobile");
                const platform = String(parsed.platform ?? "unknown");
                await extensionApi.mobileSyncConfirmPairing(
                  sessionId,
                  deviceName,
                  platform,
                  challengeSignature
                );
                setSyncMsg("Pairing confirmed from mobile response payload.");
                setMobileResponsePayload("");
                setPairingSignature("");
                await refreshSyncState();
              } catch (e) {
                setSyncMsg((e as Error).message);
              }
            }}
          >
            Confirm From Response Payload
          </button>
        </div>

        <div className="mt-2 space-y-1">
          {(syncState?.pairedDevices || []).map((d) => (
            <div key={d.id} className="rounded bg-black/20 px-2 py-2">
              <div className="mb-2 flex items-center justify-between">
                <div className="text-xs">
                  {d.name} ({d.platform})
                  <span
                    className={`ml-2 rounded px-1.5 py-0.5 text-[10px] ${
                      d.status === "revoked"
                        ? "bg-red-500/20 text-red-200"
                        : "bg-green-500/20 text-green-200"
                    }`}
                  >
                    {d.status}
                  </span>
                </div>
                <div className="text-[10px] text-white/50">
                  trust: {d.trustLevel || "signed-challenge-v1"}
                </div>
              </div>
              <div className="flex items-center gap-2">
                <input
                  className="flex-1 rounded bg-white/10 p-1.5 text-xs outline-none"
                  value={deviceDraftNames[d.id] ?? d.name}
                  onChange={(e) =>
                    setDeviceDraftNames((prev) => ({ ...prev, [d.id]: e.target.value }))
                  }
                  placeholder="Device name"
                />
                <button
                  className="rounded bg-white/10 px-2 py-1 text-xs"
                  onClick={async () => {
                    const name = (deviceDraftNames[d.id] ?? d.name).trim();
                    await extensionApi.mobileSyncRenameDevice(d.id, name);
                    setSyncMsg("Device name updated.");
                    await refreshSyncState();
                  }}
                >
                  Rename
                </button>
                <button
                  className="rounded bg-red-500/20 px-2 py-1 text-xs text-red-200"
                  onClick={async () => {
                    await extensionApi.mobileSyncRevokeDevice(d.id);
                    setSyncMsg("Device revoked.");
                    await refreshSyncState();
                  }}
                >
                  Revoke
                </button>
              </div>
              <div className="mt-1 text-[10px] text-white/50">
                paired {new Date(d.pairedAt).toLocaleString()}
                {d.renamedAt ? ` | renamed ${new Date(d.renamedAt).toLocaleString()}` : ""}
                {d.revokedAt ? ` | revoked ${new Date(d.revokedAt).toLocaleString()}` : ""}
              </div>
            </div>
          ))}
        </div>

        {syncMsg && <div className="mt-2 text-xs text-white/70">{syncMsg}</div>}
        {schemaMsg && (
          <pre className="mt-2 max-h-40 overflow-auto rounded bg-black/20 p-2 text-[10px] text-white/70">
            {schemaMsg}
          </pre>
        )}
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
          onRetry={async (id) => {
            await extensionApi.walletRetryBroadcast(id);
            await refresh();
          }}
        />
      )}
      {view === "send" && <SendView />}
      {view === "receive" && <ReceiveView status={status} />}
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

