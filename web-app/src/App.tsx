import { useCallback, useEffect, useState } from "react";
import QRCode from "react-qr-code";
import {
  api,
  getWalletCode,
  merchantMessage,
  needsRegisterCode,
  setWalletCode,
  type Profile,
} from "./api";
import { GiftCards } from "./GiftCards";
import { MerchantTill } from "./MerchantTill";

export default function App() {
  const [walletCode, setWalletCodeDraft] = useState(getWalletCode());
  const [needsCode, setNeedsCode] = useState(false);
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
      setNeedsCode(false);
      setProfile(p);
      setDisplayName(p.business_display_name ?? "");
      const b = await api.getBalance();
      setBalance(b.available_zec ?? b.balance_zec);
    } catch (e) {
      setNeedsCode(needsRegisterCode(e));
      setError(merchantMessage(e, "Could not open your wallet"));
    }
  }, [password]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const openWallet = () => {
    if (needsCode) setWalletCode(walletCode);
    void refresh();
  };

  return (
    <div className="min-h-screen bg-gradient-to-b from-zinc-950 via-zinc-900 to-black px-4 py-10">
      <div className="mx-auto max-w-lg space-y-6">
        <header>
          <p className="text-primary text-sm tracking-widest uppercase">NozyWallet</p>
          <h1 className="text-3xl font-bold text-zinc-50">Merchant dashboard</h1>
        </header>

        <section className="rounded-2xl border border-zinc-800 bg-zinc-900/80 p-4 space-y-3">
          <h2 className="font-semibold text-zinc-100">Your wallet</h2>
          <label className="block text-xs text-zinc-500">
            Wallet password
            <input
              type="password"
              className="mt-1 w-full rounded-lg bg-zinc-950 border border-zinc-700 px-3 py-2 text-sm"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Unlock to take payments"
            />
          </label>
          {needsCode && (
            <label className="block text-xs text-zinc-500">
              Wallet code
              <input
                type="password"
                className="mt-1 w-full rounded-lg bg-zinc-950 border border-zinc-700 px-3 py-2 text-sm"
                value={walletCode}
                onChange={(e) => setWalletCodeDraft(e.target.value)}
                placeholder="Paste the code from this computer's wallet"
              />
              <span className="mt-1 block text-zinc-500">
                You only need this once. Ask whoever set up the register if you don't have it.
              </span>
            </label>
          )}
          <button
            type="button"
            className="rounded-lg bg-primary px-3 py-2 text-sm font-semibold text-zinc-950"
            onClick={openWallet}
          >
            Open wallet
          </button>
        </section>

        {profile && (
          <section className="rounded-2xl border border-zinc-800 bg-zinc-900/80 p-4 space-y-3">
            <p className="text-sm text-zinc-300">
              Wallet:{" "}
              <strong>{profile.role === "business" ? "Business" : "Personal"}</strong>
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
                    setError(merchantMessage(e, "Could not switch wallet"));
                  } finally {
                    setBusy(false);
                  }
                }}
              >
                Business wallet
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
                    setError(merchantMessage(e, "Could not switch wallet"));
                  } finally {
                    setBusy(false);
                  }
                }}
              >
                Personal wallet
              </button>
            </div>
          </section>
        )}

        <MerchantTill password={password} profile={profile} />
        <GiftCards password={password} />

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
                    setError(merchantMessage(e, "Could not link that name"));
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
            <h2 className="font-semibold self-start">Store address</h2>
            <p className="self-start text-xs text-zinc-500">
              Bare receive address. Use Charge a customer when the sale has an amount.
            </p>
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
