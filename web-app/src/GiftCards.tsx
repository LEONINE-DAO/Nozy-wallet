import { useCallback, useEffect, useState } from "react";
import QRCode from "react-qr-code";
import { api, merchantMessage, type GiftCard, type GiftCardCreated } from "./api";

const fieldClass =
  "w-full rounded-lg bg-zinc-950 border border-zinc-700 px-3 py-2 text-sm";

function statusLabel(status: GiftCard["status"]): string {
  switch (status) {
    case "unfunded":
      return "Waiting for payment";
    case "funded":
      return "Paid — send the claim code";
    case "redeeming":
      return "Redeeming";
    case "redeemed":
      return "Redeemed";
    case "cancelled":
      return "Cancelled";
  }
}

export function GiftCards({ password }: { password: string }) {
  const [amount, setAmount] = useState("");
  const [label, setLabel] = useState("");
  const [cards, setCards] = useState<GiftCard[]>([]);
  const [created, setCreated] = useState<GiftCardCreated | null>(null);
  const [claimCode, setClaimCode] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [copied, setCopied] = useState(false);

  const refresh = useCallback(async () => {
    const res = await api.listGiftCards(20);
    setCards(res.gift_cards);
  }, []);

  useEffect(() => {
    void refresh().catch(() => undefined);
  }, [refresh]);

  async function createCard() {
    setError("");
    const amt = Number(amount);
    if (!Number.isFinite(amt) || amt <= 0 || amt > 21_000_000) {
      setError("Enter an amount in ZEC greater than 0.");
      return;
    }
    setBusy(true);
    try {
      const card = await api.createGiftCard({
        amount_zec: amt,
        label: label.trim() || undefined,
        password: password || undefined,
      });
      setCreated(card);
      setCards((prev) => [card, ...prev.filter((row) => row.gift_id !== card.gift_id)]);
      setAmount("");
      setLabel("");
    } catch (e) {
      setError(merchantMessage(e, "Could not create this gift card"));
    } finally {
      setBusy(false);
    }
  }

  async function cancel(id: string) {
    setBusy(true);
    setError("");
    try {
      const next = await api.cancelGiftCard(id);
      setCards((prev) => prev.map((row) => (row.gift_id === id ? next : row)));
      if (created?.gift_id === id) setCreated(null);
    } catch (e) {
      setError(merchantMessage(e, "Could not cancel this gift card"));
    } finally {
      setBusy(false);
    }
  }

  async function redeem() {
    setError("");
    if (!claimCode.trim()) {
      setError("Enter the claim code.");
      return;
    }
    setBusy(true);
    try {
      const next = await api.redeemGiftCard({
        claim_code: claimCode.trim(),
        password: password || undefined,
      });
      setClaimCode("");
      setCards((prev) => prev.map((row) => (row.gift_id === next.gift_id ? next : row)));
    } catch (e) {
      setError(merchantMessage(e, "Could not redeem this gift card"));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="rounded-2xl border border-zinc-800 bg-zinc-900/80 p-4 space-y-4">
      <div>
        <h2 className="font-semibold text-zinc-100">Gift cards</h2>
        <p className="mt-1 text-sm text-zinc-400">
          Buy a gift card with shielded ZEC, then send the claim code to someone. They paste it
          in their own Nozy and redeem it whenever they want. The redeem fee comes out of the
          card. Anyone who has the code can redeem it.
        </p>
      </div>

      <label className="block text-xs text-zinc-500">
        Amount (ZEC)
        <input
          className={`${fieldClass} mt-1`}
          inputMode="decimal"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          placeholder="0.05"
          disabled={busy}
        />
      </label>
      <label className="block text-xs text-zinc-500">
        Label
        <input
          className={`${fieldClass} mt-1`}
          value={label}
          onChange={(e) => setLabel(e.target.value)}
          placeholder="Coffee"
          disabled={busy}
        />
      </label>
      <div className="flex flex-wrap gap-2">
        <button
          type="button"
          disabled={busy}
          className="rounded-lg bg-primary px-3 py-2 text-sm font-semibold text-zinc-950 disabled:opacity-50"
          onClick={() => void createCard()}
        >
          {busy ? "Working…" : "Buy with shielded ZEC"}
        </button>
      </div>

      {error && <p className="text-sm text-red-400">{error}</p>}

      {created && (
        <div className="space-y-3 rounded-xl border border-zinc-700 p-3">
          <p className="text-sm text-amber-300">
            Claim code, shown once: <span className="font-mono">{created.claim_code}</span>
          </p>
          <div className="flex justify-center bg-white p-3 rounded-xl">
            <QRCode value={created.claim_code} size={180} />
          </div>
          <p className="text-xs text-zinc-400">
            Send this code. The recipient redeems it in their own Nozy.
          </p>
          <button
            type="button"
            className="rounded-lg border border-zinc-600 px-3 py-2 text-sm"
            onClick={() => {
              void navigator.clipboard.writeText(created.claim_code);
              setCopied(true);
              window.setTimeout(() => setCopied(false), 2000);
            }}
          >
            {copied ? "Copied" : "Copy claim code"}
          </button>
        </div>
      )}

      <div className="space-y-2 border-t border-zinc-800 pt-3">
        <h3 className="text-sm font-semibold text-zinc-200">Redeem into this wallet</h3>
        <label className="block text-xs text-zinc-500">
          Claim code
          <input
            className={`${fieldClass} mt-1 font-mono`}
            value={claimCode}
            onChange={(e) => setClaimCode(e.target.value)}
            placeholder="NZGC-…"
            disabled={busy}
          />
        </label>
        <button
          type="button"
          disabled={busy}
          className="rounded-lg bg-primary px-3 py-2 text-sm font-semibold text-zinc-950 disabled:opacity-50"
          onClick={() => void redeem()}
        >
          Redeem
        </button>
      </div>

      {cards.length > 0 && (
        <ul className="space-y-2">
          {cards.map((row) => (
            <li
              key={row.gift_id}
              className="rounded-lg border border-zinc-800 px-3 py-2"
            >
              <span className="text-sm text-zinc-100">
                {row.label || row.gift_id} · {row.amount_zec} ZEC
              </span>
              <span className="block text-xs text-zinc-400">{statusLabel(row.status)}</span>
              {row.status === "unfunded" && (
                <button
                  type="button"
                  disabled={busy}
                  className="mt-1 text-xs text-zinc-400 underline disabled:opacity-50"
                  onClick={() => void cancel(row.gift_id)}
                >
                  Cancel
                </button>
              )}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
