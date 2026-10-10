import { useCallback, useEffect, useState } from "react";
import toast from "react-hot-toast";
import QRCode from "react-qr-code";
import { Button } from "./Button";
import { Input } from "./Input";
import { walletApi, type GiftCard, type GiftCardCreated } from "../lib/api";
import { formatErrorForDisplay } from "../utils/errors";

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

/** Gift cards sit on the same shielded receive path as merchant invoices. */
export function GiftCardPanel() {
  const [amount, setAmount] = useState("0.05");
  const [label, setLabel] = useState("");
  const [cards, setCards] = useState<GiftCard[]>([]);
  const [created, setCreated] = useState<GiftCardCreated | null>(null);
  const [claimCode, setClaimCode] = useState("");
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    const { data } = await walletApi.listGiftCards(20);
    setCards(data.gift_cards);
  }, []);

  useEffect(() => {
    void refresh().catch(() => undefined);
  }, [refresh]);

  const create = async () => {
    const amt = Number(amount);
    if (!(amt > 0)) {
      toast.error("Enter a positive amount");
      return;
    }
    setBusy(true);
    try {
      const { data } = await walletApi.createGiftCard({
        amount_zec: amt,
        label: label.trim() || undefined,
      });
      setCreated(data);
      setLabel("");
      toast.success("Gift card paid. Copy the claim code now and send it to them — it is not saved.");
      await refresh();
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Could not create gift card"));
    } finally {
      setBusy(false);
    }
  };

  const cancel = async (id: string) => {
    setBusy(true);
    try {
      await walletApi.cancelGiftCard(id);
      if (created?.gift_id === id) setCreated(null);
      toast.success("Gift card cancelled");
      await refresh();
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Could not cancel gift card"));
    } finally {
      setBusy(false);
    }
  };

  const redeem = async () => {
    if (!claimCode.trim()) {
      toast.error("Enter the claim code");
      return;
    }
    setBusy(true);
    try {
      const { data } = await walletApi.redeemGiftCard({
        claim_code: claimCode.trim(),
      });
      setClaimCode("");
      const paid = data.payout_zatoshis
        ? (data.payout_zatoshis / 100_000_000).toFixed(8)
        : data.amount_zec;
      toast.success(`Redeemed ${paid} ZEC into this wallet`);
      await refresh();
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Could not redeem gift card"));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="space-y-3 text-left">
      <div>
        <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100">Gift cards</h3>
        <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
          Buy a gift card with shielded ZEC, then send the claim code to someone. They paste it
          in their own Nozy and redeem it whenever they want. The redeem fee comes out of the
          card. Anyone who has the code can redeem it.
        </p>
      </div>

      <Input
        label="Amount (ZEC)"
        value={amount}
        onChange={(e) => setAmount(e.target.value)}
        inputMode="decimal"
      />
      <Input
        label="Label"
        value={label}
        onChange={(e) => setLabel(e.target.value)}
        placeholder="Optional, for example Coffee"
      />
      <div className="flex flex-wrap gap-2">
        <Button size="sm" onClick={() => void create()} disabled={busy}>
          {busy ? "Working…" : "Buy with shielded ZEC"}
        </Button>
      </div>

      {created ? (
        <div className="space-y-2 rounded-xl border border-gray-200 dark:border-gray-700 p-3">
          <p className="text-xs text-amber-700 dark:text-amber-300">
            Claim code (shown once): <span className="font-mono">{created.claim_code}</span>
          </p>
          <div className="flex justify-center bg-white p-2 rounded-lg">
            <QRCode value={created.claim_code} size={140} level="M" />
          </div>
          <p className="text-xs text-gray-500 dark:text-gray-400">
            Send this code. The recipient redeems it in their own Nozy.
          </p>
          <Button size="sm" variant="outline" onClick={() => void navigator.clipboard.writeText(created.claim_code)}>
            Copy claim code
          </Button>
        </div>
      ) : null}

      <div className="border-t border-gray-200 dark:border-gray-700 pt-3 space-y-2">
        <p className="text-xs font-semibold text-gray-700 dark:text-gray-200">
          Redeem into this wallet
        </p>
        <Input
          label="Claim code"
          value={claimCode}
          onChange={(e) => setClaimCode(e.target.value)}
          placeholder="NZGC-…"
        />
        <Button size="sm" onClick={() => void redeem()} disabled={busy}>
          Redeem
        </Button>
      </div>

      {cards.length > 0 ? (
        <ul className="space-y-2">
          {cards.map((card) => (
            <li
              key={card.gift_id}
              className="rounded-lg border border-gray-200 dark:border-gray-700 px-3 py-2"
            >
              <p className="text-sm text-gray-900 dark:text-gray-100">
                {card.label || card.gift_id} · {card.amount_zec} ZEC
              </p>
              <p className="text-xs text-gray-500">{statusLabel(card.status)}</p>
              {card.status === "unfunded" && (
                <Button
                  size="sm"
                  variant="ghost"
                  className="mt-1"
                  disabled={busy}
                  onClick={() => void cancel(card.gift_id)}
                >
                  Cancel
                </Button>
              )}
            </li>
          ))}
        </ul>
      ) : null}
    </div>
  );
}
