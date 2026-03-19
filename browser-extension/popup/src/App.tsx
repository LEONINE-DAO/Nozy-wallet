import { useEffect, useMemo, useState } from "react";
import { extensionApi, type PendingApproval, type WalletStatus } from "./lib/extensionApi";
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

function DashboardView({ status }: { status: WalletStatus | null }) {
  return (
    <div className="space-y-3 p-4">
      <h1 className="text-lg font-semibold">Dashboard</h1>
      <div className="rounded border border-white/10 bg-white/5 p-3 text-sm">
        <div className="text-white/70">Address</div>
        <div className="break-all">{status?.address || "-"}</div>
      </div>
      <div className="rounded border border-white/10 bg-white/5 p-3 text-sm">
        <div className="text-white/70">Balance</div>
        <div>Scanning pipeline is next phase.</div>
      </div>
    </div>
  );
}

function SendView() {
  const [status, setStatus] = useState<string>("");
  const [busy, setBusy] = useState(false);
  return (
    <div className="space-y-2 p-4 text-sm">
      <h2 className="text-base font-semibold">Send</h2>
      <div className="rounded border border-white/10 bg-white/5 p-3">
        Transaction proving worker channel is wired. Full Orchard proving internals are the
        remaining step in Phase 3c.
      </div>
      <button
        className="rounded bg-amber-500 px-3 py-1 text-black disabled:opacity-50"
        disabled={busy}
        onClick={async () => {
          setBusy(true);
          try {
            const result = await extensionApi.walletProveTransaction({
              to: "u1example",
              amount: 1
            });
            setStatus(`Proving worker response: ${result.txid.slice(0, 16)}...`);
          } catch (e) {
            setStatus((e as Error).message);
          } finally {
            setBusy(false);
          }
        }}
      >
        Test Proving Worker
      </button>
      {status && <div className="text-xs text-white/70">{status}</div>}
    </div>
  );
}

function ReceiveView({ status }: { status: WalletStatus | null }) {
  const [scanInfo, setScanInfo] = useState<string>("");
  const [busy, setBusy] = useState(false);
  return (
    <div className="space-y-2 p-4 text-sm">
      <h2 className="text-base font-semibold">Receive</h2>
      <div className="rounded border border-white/10 bg-white/5 p-3 break-all">
        {status?.address || "No address yet"}
      </div>
      <button
        className="rounded bg-amber-500 px-3 py-1 text-black disabled:opacity-50"
        disabled={busy}
        onClick={async () => {
          setBusy(true);
          try {
            const count = await extensionApi.rpcGetBlockCount();
            const start = Math.max(0, count - 25);
            const scanned = await extensionApi.walletScanNotes(start, count);
            setScanInfo(
              `Scanned ${scanned.scannedBlocks} blocks, notes found: ${scanned.discoveredNotes.length}`
            );
          } catch (e) {
            setScanInfo((e as Error).message);
          } finally {
            setBusy(false);
          }
        }}
      >
        Scan Recent Blocks
      </button>
      {scanInfo && <div className="text-xs text-white/70">{scanInfo}</div>}
    </div>
  );
}

function SettingsView({
  endpoint,
  onEndpointChange,
  onLock
}: {
  endpoint: string;
  onEndpointChange: (url: string) => void;
  onLock: () => void;
}) {
  const [value, setValue] = useState(endpoint);
  const [msg, setMsg] = useState<string | null>(null);
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
  const endpoint = useMemo(() => status?.rpcEndpoint || "http://127.0.0.1:8232", [status]);

  const refresh = async () => {
    const nextStatus = await extensionApi.walletStatus();
    setStatus(nextStatus);
    if (!nextStatus.exists) setView("welcome");
    else if (!nextStatus.unlocked) setView("unlock");
    else if (view === "welcome" || view === "unlock") setView("dashboard");
    setApprovals(await extensionApi.walletGetPendingApprovals());
  };

  useEffect(() => {
    refresh().catch(console.error);
    const id = setInterval(() => {
      extensionApi.walletGetPendingApprovals().then(setApprovals).catch(() => undefined);
    }, 1500);
    return () => clearInterval(id);
  }, []);

  return (
    <div className="h-full overflow-auto">
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
      {view === "dashboard" && <DashboardView status={status} />}
      {view === "send" && <SendView />}
      {view === "receive" && <ReceiveView status={status} />}
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
        />
      )}
    </div>
  );
}

