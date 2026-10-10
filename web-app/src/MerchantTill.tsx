import { useCallback, useEffect, useRef, useState } from "react";
import QRCode from "react-qr-code";
import {
  api,
  merchantMessage,
  type InvoiceStatus,
  type MerchantInvoice,
  type Profile,
} from "./api";

const fieldClass =
  "w-full rounded-lg bg-zinc-950 border border-zinc-700 px-3 py-2 text-sm";

function formatZec(n: number): string {
  if (!Number.isFinite(n)) return "—";
  return n.toFixed(8).replace(/0+$/, "").replace(/\.$/, "");
}

function statusLabel(status: InvoiceStatus): string {
  switch (status) {
    case "open":
      return "Waiting for payment";
    case "detected":
      return "Payment seen";
    case "confirmed":
      return "Paid";
    case "expired":
      return "Expired";
    case "cancelled":
      return "Cancelled";
  }
}

function statusClass(status: InvoiceStatus): string {
  switch (status) {
    case "open":
      return "text-amber-300";
    case "detected":
      return "text-sky-300";
    case "confirmed":
      return "text-emerald-300";
    default:
      return "text-zinc-400";
  }
}

export function MerchantTill({
  password,
  profile,
}: {
  password: string;
  profile: Profile | null;
}) {
  const business = profile?.role === "business";
  const [amount, setAmount] = useState("");
  const [product, setProduct] = useState("");
  const [memo, setMemo] = useState("");
  const [active, setActive] = useState<MerchantInvoice | null>(null);
  const [invoices, setInvoices] = useState<MerchantInvoice[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [copied, setCopied] = useState(false);
  const syncing = useRef(false);

  const refreshList = useCallback(async () => {
    if (!business) return;
    const res = await api.listInvoices(20);
    setInvoices(res.invoices);
  }, [business]);

  useEffect(() => {
    if (!business) {
      setInvoices([]);
      return;
    }
    void refreshList().catch((e) => {
      setError(merchantMessage(e, "Could not load charges"));
    });
  }, [business, refreshList]);

  useEffect(() => {
    if (!active || (active.status !== "open" && active.status !== "detected")) return;
    const id = active.invoice_id;
    let cancelled = false;
    const check = async () => {
      if (syncing.current || cancelled) return;
      syncing.current = true;
      try {
        await api.sync(password || undefined).catch(() => undefined);
        if (cancelled) return;
        const next = await api.getInvoice(id);
        if (cancelled) return;
        setActive((cur) => (cur?.invoice_id === id ? next : cur));
        setInvoices((prev) => prev.map((row) => (row.invoice_id === id ? next : row)));
      } catch {
        /* keep the code on screen; the next check will try again */
      } finally {
        syncing.current = false;
      }
    };
    void check();
    const timer = setInterval(() => void check(), 20000);
    return () => {
      cancelled = true;
      clearInterval(timer);
    };
  }, [active?.invoice_id, active?.status, password]);

  async function createCharge() {
    setError("");
    const amt = Number(amount);
    if (!Number.isFinite(amt) || amt <= 0 || amt > 21_000_000) {
      setError("Enter an amount in ZEC greater than 0.");
      return;
    }
    if (!business) {
      setError("Switch to Business before charging a customer.");
      return;
    }
    setBusy(true);
    try {
      const inv = await api.createInvoice({
        amount_zec: amt,
        product_name: product.trim() || undefined,
        memo: memo.trim() || undefined,
        password: password || undefined,
      });
      setActive(inv);
      setInvoices((prev) => [inv, ...prev.filter((row) => row.invoice_id !== inv.invoice_id)]);
      setAmount("");
      setProduct("");
      setMemo("");
    } catch (e) {
      setError(merchantMessage(e, "Could not start this sale"));
    } finally {
      setBusy(false);
    }
  }

  async function cancelCharge(id: string) {
    setBusy(true);
    setError("");
    try {
      const next = await api.cancelInvoice(id);
      setActive((cur) => (cur?.invoice_id === id ? next : cur));
      setInvoices((prev) => prev.map((row) => (row.invoice_id === id ? next : row)));
    } catch (e) {
      setError(merchantMessage(e, "Could not cancel this sale"));
    } finally {
      setBusy(false);
    }
  }

  async function copyUri(uri: string) {
    await navigator.clipboard.writeText(uri);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 2000);
  }

  return (
    <section className="rounded-2xl border border-zinc-800 bg-zinc-900/80 p-4 space-y-4">
      <div>
        <h2 className="font-semibold text-zinc-100">Charge a customer</h2>
        <p className="mt-1 text-sm text-zinc-400">
          Enter the amount, then show the customer the payment code. They scan it with their Zcash
          wallet. The money goes to your business wallet.
        </p>
      </div>

      {!profile ? (
        <p className="text-sm text-zinc-400">Open your wallet above to start a sale.</p>
      ) : !business ? (
        <p className="text-sm text-amber-300">
          Switch to your business wallet above before taking a payment.
        </p>
      ) : null}

      <label className="block text-xs text-zinc-500">
        Amount (ZEC)
        <input
          className={`${fieldClass} mt-1`}
          inputMode="decimal"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          placeholder="0.05"
          disabled={!business || busy}
        />
      </label>
      <label className="block text-xs text-zinc-500">
        Product
        <input
          className={`${fieldClass} mt-1`}
          value={product}
          onChange={(e) => setProduct(e.target.value)}
          placeholder="Coffee"
          disabled={!business || busy}
        />
      </label>
      <label className="block text-xs text-zinc-500">
        Memo
        <input
          className={`${fieldClass} mt-1`}
          value={memo}
          onChange={(e) => setMemo(e.target.value)}
          placeholder="Note on the payment (optional)"
          disabled={!business || busy}
        />
      </label>
      <button
        type="button"
        disabled={!business || busy}
        className="rounded-lg bg-primary px-3 py-2 text-sm font-semibold text-zinc-950 disabled:opacity-50"
        onClick={() => void createCharge()}
      >
        {busy ? "Working…" : "Show payment code"}
      </button>

      {error && <p className="text-sm text-red-400">{error}</p>}

      {active && (
        <div className="fixed inset-0 z-50 flex flex-col items-center justify-center gap-6 bg-zinc-950 px-6 text-center">
          <p className="text-sm uppercase tracking-widest text-zinc-400">
            {active.product_name || "Payment"}
          </p>
          <p className="text-5xl font-bold tabular-nums text-zinc-50">
            {formatZec(active.amount_zec)} ZEC
          </p>
          <p className={`text-2xl font-semibold ${statusClass(active.status)}`}>
            {active.status === "confirmed" ? "Paid" : statusLabel(active.status)}
          </p>
          {active.status === "open" || active.status === "detected" ? (
            <div className="bg-white p-4 rounded-2xl">
              <QRCode value={active.zcash_uri} size={280} />
            </div>
          ) : (
            <p className="max-w-sm text-zinc-300">
              {active.status === "confirmed"
                ? "The payment is in your business wallet."
                : active.status === "cancelled"
                  ? "This sale was cancelled. Start a new one when the customer is ready."
                  : "This payment code expired. Start a new sale."}
            </p>
          )}
          {(active.confirmed_txid || active.detected_txid) && (
            <p className="max-w-md text-xs text-zinc-500 break-all">
              Receipt {active.confirmed_txid || active.detected_txid}
            </p>
          )}
          <div className="flex flex-wrap justify-center gap-3">
            {(active.status === "open" || active.status === "detected") && (
              <button
                type="button"
                className="rounded-lg border border-zinc-600 px-4 py-2 text-sm"
                onClick={() => void copyUri(active.zcash_uri)}
              >
                {copied ? "Copied" : "Copy payment link"}
              </button>
            )}
            {active.status === "open" && (
              <button
                type="button"
                disabled={busy}
                className="rounded-lg border border-zinc-600 px-4 py-2 text-sm text-zinc-300 disabled:opacity-50"
                onClick={() => void cancelCharge(active.invoice_id)}
              >
                Cancel sale
              </button>
            )}
            <button
              type="button"
              className="rounded-lg bg-primary px-4 py-2 text-sm font-semibold text-zinc-950"
              onClick={() => setActive(null)}
            >
              {active.status === "open" || active.status === "detected" ? "Back" : "New sale"}
            </button>
          </div>
        </div>
      )}

      {invoices.length > 0 && (
        <div className="space-y-2">
          <h3 className="text-sm font-semibold text-zinc-200">Recent charges</h3>
          <ul className="space-y-2">
            {invoices.map((row) => (
              <li key={row.invoice_id}>
                <button
                  type="button"
                  className="w-full text-left rounded-lg border border-zinc-800 px-3 py-2 hover:border-zinc-600"
                  onClick={() => setActive(row)}
                >
                  <span className="text-sm text-zinc-100">
                    {row.product_name || row.invoice_id} · {formatZec(row.amount_zec)} ZEC
                  </span>
                  <span className={`block text-xs ${statusClass(row.status)}`}>
                    {statusLabel(row.status)}
                  </span>
                </button>
              </li>
            ))}
          </ul>
        </div>
      )}
    </section>
  );
}
