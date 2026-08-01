import { useCallback, useEffect, useState } from "react";
import QRCode from "react-qr-code";
import { api, getApiBase, setApiBase, type Profile } from "./api";

export default function App() {
  const [apiUrl, setApiUrl] = useState(getApiBase());
  const [password, setPassword] = useState("");
  const [profile, setProfile] = useState<Profile | null>(null);
  const [linkName, setLinkName] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [balance, setBalance] = useState<number | null>(null);
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    setError("");
    try {
      const p = await api.getProfile(password || undefined);
      setProfile(p);
      setDisplayName(p.business_display_name ?? "");
      const b = await api.getBalance();
      setBalance(b.available_zec ?? b.balance_zec);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load");
    }
  }, [password]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const saveApi = () => {
    setApiBase(apiUrl);
    void refresh();
  };

  return (
    <div className="min-h-screen bg-gradient-to-b from-zinc-950 via-zinc-900 to-black px-4 py-10">
      <div className="mx-auto max-w-lg space-y-6">
        <header>
          <p className="text-primary text-sm tracking-widest uppercase">NozyWallet</p>
          <h1 className="text-3xl font-bold text-zinc-50">Merchant dashboard</h1>
          <p className="mt-2 text-sm text-zinc-400">
            Companion API only — Business profile, ZNS link, receive QR. Claim names on
            zcashnames.com.
          </p>
        </header>

        <section className="rounded-2xl border border-zinc-800 bg-zinc-900/80 p-4 space-y-3">
          <label className="block text-xs text-zinc-500">API base URL</label>
          <div className="flex gap-2">
            <input
              className="flex-1 rounded-lg bg-zinc-950 border border-zinc-700 px-3 py-2 text-sm"
              value={apiUrl}
              onChange={(e) => setApiUrl(e.target.value)}
            />
            <button
              type="button"
              className="rounded-lg bg-primary px-3 py-2 text-sm font-semibold text-zinc-950"
              onClick={saveApi}
            >
              Save
            </button>
          </div>
          <label className="block text-xs text-zinc-500">Wallet password (for UAs / link)</label>
          <input
            type="password"
            className="w-full rounded-lg bg-zinc-950 border border-zinc-700 px-3 py-2 text-sm"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            placeholder="Optional if unlocked session"
          />
          <button
            type="button"
            className="text-sm text-primary underline"
            onClick={() => void refresh()}
          >
            Refresh
          </button>
        </section>

        {profile && (
          <section className="rounded-2xl border border-zinc-800 bg-zinc-900/80 p-4 space-y-3">
            <p className="text-sm text-zinc-300">
              Role:{" "}
              <strong>
                {profile.role === "business" ? "Business (account 1)" : "Personal (account 0)"}
              </strong>
              {profile.linked_zns_display ? (
                <span className="ml-2 text-primary font-semibold">
                  {profile.linked_zns_display}
                </span>
              ) : null}
            </p>
            {balance != null && (
              <p className="text-lg font-semibold tabular-nums">{balance.toFixed(8)} ZEC</p>
            )}
            <input
              className="w-full rounded-lg bg-zinc-950 border border-zinc-700 px-3 py-2 text-sm"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              placeholder="Business display name"
            />
            <div className="flex flex-wrap gap-2">
              <button
                type="button"
                disabled={busy}
                className="rounded-lg bg-primary px-3 py-2 text-sm font-semibold text-zinc-950 disabled:opacity-50"
                onClick={async () => {
                  setBusy(true);
                  try {
                    await api.updateProfile({
                      password: password || undefined,
                      role: "business",
                      business_display_name: displayName || undefined,
                    });
                    await refresh();
                  } catch (e) {
                    setError(e instanceof Error ? e.message : "Update failed");
                  } finally {
                    setBusy(false);
                  }
                }}
              >
                Use Business
              </button>
              <button
                type="button"
                disabled={busy}
                className="rounded-lg border border-zinc-600 px-3 py-2 text-sm"
                onClick={async () => {
                  setBusy(true);
                  try {
                    await api.updateProfile({
                      password: password || undefined,
                      role: "personal",
                    });
                    await refresh();
                  } catch (e) {
                    setError(e instanceof Error ? e.message : "Update failed");
                  } finally {
                    setBusy(false);
                  }
                }}
              >
                Use Personal
              </button>
              <button
                type="button"
                className="rounded-lg border border-zinc-600 px-3 py-2 text-sm"
                onClick={async () => {
                  try {
                    await api.sync(password || undefined);
                    await refresh();
                  } catch (e) {
                    setError(e instanceof Error ? e.message : "Sync failed");
                  }
                }}
              >
                Sync
              </button>
            </div>
          </section>
        )}

        <section className="rounded-2xl border border-zinc-800 bg-zinc-900/80 p-4 space-y-3">
          <h2 className="font-semibold text-zinc-100">Zcash name link</h2>
          {profile?.linked_zns_display ? (
            <button
              type="button"
              className="text-sm text-zinc-400 underline"
              onClick={async () => {
                await api.unlinkZns();
                await refresh();
              }}
            >
              Unlink {profile.linked_zns_display}
            </button>
          ) : (
            <>
              <a
                className="text-sm text-primary underline"
                href="https://www.zcashnames.com"
                target="_blank"
                rel="noreferrer"
              >
                Claim on zcashnames.com
              </a>
              <input
                className="w-full rounded-lg bg-zinc-950 border border-zinc-700 px-3 py-2 text-sm"
                value={linkName}
                onChange={(e) => setLinkName(e.target.value)}
                placeholder="mystore"
              />
              <button
                type="button"
                disabled={busy || !linkName.trim()}
                className="rounded-lg bg-primary px-3 py-2 text-sm font-semibold text-zinc-950 disabled:opacity-50"
                onClick={async () => {
                  setBusy(true);
                  try {
                    await api.linkZns({
                      name: linkName.trim(),
                      password: password || undefined,
                    });
                    setLinkName("");
                    await refresh();
                  } catch (e) {
                    setError(e instanceof Error ? e.message : "Link failed");
                  } finally {
                    setBusy(false);
                  }
                }}
              >
                Link name
              </button>
            </>
          )}
        </section>

        {(profile?.business_address || profile?.receive_address) && (
          <section className="rounded-2xl border border-zinc-800 bg-zinc-900/80 p-4 flex flex-col items-center gap-3">
            <h2 className="font-semibold self-start">Receive QR</h2>
            {profile.linked_zns_display && (
              <p className="text-primary text-xl font-bold">{profile.linked_zns_display}</p>
            )}
            <div className="bg-white p-3 rounded-xl">
              <QRCode
                value={profile.business_address || profile.receive_address || ""}
                size={180}
              />
            </div>
            <p className="text-xs font-mono break-all text-zinc-400 text-center">
              {profile.business_address || profile.receive_address}
            </p>
          </section>
        )}

        {error && <p className="text-sm text-red-400">{error}</p>}
      </div>
    </div>
  );
}
