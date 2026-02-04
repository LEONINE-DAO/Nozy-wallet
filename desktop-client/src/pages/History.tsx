import { useEffect, useMemo, useState } from "react";
import { ArrowRightUp, ArrowLeftDown, Calendar } from "@solar-icons/react";
import { walletApi } from "../lib/api";
import { Input } from "../components/Input";
import { Modal } from "../components/Modal";
import { Button } from "../components/Button";
import { Tooltip } from "../components/Tooltip";
import { useSettingsStore } from "../store/settingsStore";
import { getZecPriceInFiat, formatFiatAmount } from "../utils/price";
import toast from "react-hot-toast";

export interface HistoryTx {
  id: string;
  type: "sent" | "received";
  amount: number;
  date: string;
  address: string;
  status: string;
  memo?: string;
}

function normalizeTx(raw: any): HistoryTx {
  const txid = raw.txid ?? raw.id ?? "";
  const amountZec = typeof raw.amount_zec === "number"
    ? raw.amount_zec
    : (raw.amount_zatoshis != null ? raw.amount_zatoshis / 100_000_000 : 0);
  const recipient = raw.recipient_address ?? raw.recipient ?? "";
  const status = raw.status != null
    ? String(raw.status).toLowerCase().replace(/\s/g, "_")
    : "unknown";
  const dateRaw = raw.broadcast_at ?? raw.created_at ?? raw.date ?? raw.block_time;
  const date =
    typeof dateRaw === "string"
      ? dateRaw.slice(0, 10)
      : dateRaw != null && typeof dateRaw === "object" && "secs_since_epoch" in dateRaw
        ? new Date((dateRaw as { secs_since_epoch: number }).secs_since_epoch * 1000).toISOString().slice(0, 10)
        : raw.timestamp != null
          ? new Date(raw.timestamp * 1000).toISOString().slice(0, 10)
          : "";
  const type = (raw.transaction_type ?? raw.type ?? "sent").toString().toLowerCase();
  const memo = typeof raw.memo === "string" ? raw.memo : undefined;

  return {
    id: txid,
    type: type === "received" ? "received" : "sent",
    amount: amountZec,
    date,
    address: recipient || (txid ? `${txid.slice(0, 8)}...${txid.slice(-4)}` : "—"),
    status: status === "confirmed" ? "confirmed" : status === "pending" ? "pending" : status,
    memo,
  };
}

type FilterType = "all" | "sent" | "received";
type FilterStatus = "all" | "confirmed" | "pending";
type FilterDateRange = "all" | "7" | "30" | "90";

function filterAndSortTxs(
  txs: HistoryTx[],
  filterType: FilterType,
  filterStatus: FilterStatus,
  filterDateRange: FilterDateRange,
  searchQuery: string
): HistoryTx[] {
  const q = searchQuery.trim().toLowerCase();
  const now = new Date();
  const cutoffDays = filterDateRange === "all" ? null : parseInt(filterDateRange, 10);
  const cutoffDate = cutoffDays != null ? (() => {
    const d = new Date(now);
    d.setDate(d.getDate() - cutoffDays);
    return d;
  })() : null;

  return txs
    .filter((t) => filterType === "all" || t.type === filterType)
    .filter((t) => filterStatus === "all" || t.status === filterStatus)
    .filter((t) => {
      if (!cutoffDate) return true;
      const txDate = t.date ? new Date(t.date) : new Date(0);
      return txDate >= cutoffDate;
    })
    .filter((t) => {
      if (!q) return true;
      return (
        t.address.toLowerCase().includes(q) ||
        t.id.toLowerCase().includes(q) ||
        (t.memo ?? "").toLowerCase().includes(q)
      );
    })
    .sort((a, b) => {
      const da = a.date ? new Date(a.date).getTime() : 0;
      const db = b.date ? new Date(b.date).getTime() : 0;
      return db - da;
    });
}

function escapeCsvField(value: string): string {
  if (value.includes(",") || value.includes('"') || value.includes("\n") || value.includes("\r")) {
    return `"${value.replace(/"/g, '""')}"`;
  }
  return value;
}

function downloadCsv(txs: HistoryTx[]): void {
  const header = "Date,Type,Amount (ZEC),Address,Status,Memo,Transaction ID";
  const rows = txs.map((tx) =>
    [
      tx.date || "",
      tx.type,
      tx.amount.toFixed(8),
      tx.address,
      tx.status,
      tx.memo ?? "",
      tx.id,
    ].map(escapeCsvField).join(",")
  );
  const csv = [header, ...rows].join("\r\n");
  const blob = new Blob([csv], { type: "text/csv;charset=utf-8;" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `nozy-transactions-${new Date().toISOString().slice(0, 10)}.csv`;
  a.click();
  URL.revokeObjectURL(url);
}

export function HistoryPage() {
  const [txs, setTxs] = useState<HistoryTx[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filterType, setFilterType] = useState<FilterType>("all");
  const [filterStatus, setFilterStatus] = useState<FilterStatus>("all");
  const [filterDateRange, setFilterDateRange] = useState<FilterDateRange>("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedTx, setSelectedTx] = useState<HistoryTx | null>(null);
  const [detailExtra, setDetailExtra] = useState<{ confirmations?: number; block_height?: number; fee_zec?: number; broadcast_at?: string } | null>(null);
  const [detailLoading, setDetailLoading] = useState(false);
  const [fiatRate, setFiatRate] = useState<number | null>(null);
  const [saveContactOpen, setSaveContactOpen] = useState(false);
  const [saveContactName, setSaveContactName] = useState("");
  const [saveContactNotes, setSaveContactNotes] = useState("");
  const [saveContactSaving, setSaveContactSaving] = useState(false);

  const {
    showFiatEquivalent,
    fiatCurrency,
    useLiveFiatPrice,
    customFiatPerZec,
  } = useSettingsStore();

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    walletApi
      .getTransactionHistory()
      .then((res) => {
        if (cancelled) return;
        const raw = res?.data;
        if (Array.isArray(raw)) {
          const normalized = raw.map(normalizeTx).filter((t) => t.id);
          setTxs(normalized);
        } else {
          setTxs([]);
        }
      })
      .catch((e) => {
        if (!cancelled) {
          setError(e?.message ?? "Failed to load history");
          setTxs([]);
        }
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const filteredTxs = useMemo(
    () => filterAndSortTxs(txs, filterType, filterStatus, filterDateRange, searchQuery),
    [txs, filterType, filterStatus, filterDateRange, searchQuery]
  );

  useEffect(() => {
    if (!selectedTx) {
      setDetailExtra(null);
      return;
    }
    setDetailLoading(true);
    setDetailExtra(null);
    walletApi
      .getTransaction(selectedTx.id)
      .then((res) => {
        const d = res?.data;
        if (d && typeof d === "object") {
          setDetailExtra({
            confirmations: typeof d.confirmations === "number" ? d.confirmations : undefined,
            block_height: typeof d.block_height === "number" ? d.block_height : undefined,
            fee_zec: typeof d.fee_zec === "number" ? d.fee_zec : (typeof d.fee_zatoshis === "number" ? d.fee_zatoshis / 100_000_000 : undefined),
            broadcast_at: typeof d.broadcast_at === "string" ? d.broadcast_at : undefined,
          });
        }
      })
      .catch(() => setDetailExtra(null))
      .finally(() => setDetailLoading(false));
  }, [selectedTx?.id]);

  // Fiat rate: live (CoinGecko) or custom
  useEffect(() => {
    if (!showFiatEquivalent) {
      setFiatRate(null);
      return;
    }
    if (!useLiveFiatPrice && customFiatPerZec != null) {
      setFiatRate(customFiatPerZec);
      return;
    }
    if (!useLiveFiatPrice) {
      setFiatRate(null);
      return;
    }
    getZecPriceInFiat(fiatCurrency).then((rate) => setFiatRate(rate));
  }, [showFiatEquivalent, useLiveFiatPrice, customFiatPerZec, fiatCurrency]);

  const effectiveFiatRate = showFiatEquivalent
    ? (useLiveFiatPrice ? fiatRate : customFiatPerZec)
    : null;

  function fiatLine(amountZec: number): string | null {
    if (effectiveFiatRate == null || effectiveFiatRate <= 0) return null;
    const fiat = amountZec * effectiveFiatRate;
    return formatFiatAmount(fiat, fiatCurrency);
  }

  const handleSaveToContacts = async () => {
    if (!selectedTx) return;
    const name = saveContactName.trim();
    const addr = selectedTx.address.trim();
    if (!name || !addr) {
      toast.error("Name is required");
      return;
    }
    if (!addr.startsWith("u1") && !addr.startsWith("zs1")) {
      toast.error("Address must be a shielded address (u1 or zs1)");
      return;
    }
    setSaveContactSaving(true);
    try {
      await walletApi.addAddressBookEntry({
        name,
        address: addr,
        notes: saveContactNotes.trim() || undefined,
      });
      toast.success("Saved to contacts");
      setSaveContactOpen(false);
      setSaveContactName("");
      setSaveContactNotes("");
    } catch (e) {
      toast.error((e as Error)?.message ?? "Failed to save contact");
    } finally {
      setSaveContactSaving(false);
    }
  };

  return (
    <div className="space-y-6 animate-fade-in max-w-4xl mx-auto">
      <div className="flex items-center justify-between">
        <h2 className="text-3xl font-bold text-gray-900">
          Transaction History
        </h2>
      </div>

      {!loading && !error && txs.length > 0 && (
        <div className="flex flex-col sm:flex-row gap-4 flex-wrap">
          <div className="flex-1 min-w-[200px] max-w-md">
            <Input
              type="search"
              placeholder="Search by address, txid, or memo"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              aria-label="Search transactions"
              className="bg-white/60 border-white/50"
            />
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <select
              value={filterType}
              onChange={(e) => setFilterType(e.target.value as FilterType)}
              aria-label="Filter by type"
              className="h-11 rounded-lg border border-gray-200/60 bg-white/60 px-3 py-2 text-sm text-gray-700 focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2"
            >
              <option value="all">All types</option>
              <option value="sent">Sent</option>
              <option value="received">Received</option>
            </select>
            <select
              value={filterStatus}
              onChange={(e) => setFilterStatus(e.target.value as FilterStatus)}
              aria-label="Filter by status"
              className="h-11 rounded-lg border border-gray-200/60 bg-white/60 px-3 py-2 text-sm text-gray-700 focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2"
            >
              <option value="all">All statuses</option>
              <option value="confirmed">Confirmed</option>
              <option value="pending">Pending</option>
            </select>
            <select
              value={filterDateRange}
              onChange={(e) => setFilterDateRange(e.target.value as FilterDateRange)}
              aria-label="Filter by date range"
              className="h-11 rounded-lg border border-gray-200/60 bg-white/60 px-3 py-2 text-sm text-gray-700 focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2"
            >
              <option value="all">All time</option>
              <option value="7">Last 7 days</option>
              <option value="30">Last 30 days</option>
              <option value="90">Last 90 days</option>
            </select>
            <Tooltip content="Download filtered transactions as CSV">
              <Button
                type="button"
                variant="outline"
                onClick={() => downloadCsv(filteredTxs)}
                className="h-11 gap-2 bg-white/60 border-white/50 text-gray-700 hover:bg-white/90"
              >
                Export CSV
              </Button>
            </Tooltip>
          </div>
        </div>
      )}

      <div className="bg-white/60 backdrop-blur-md rounded-2xl border border-white/50 shadow-sm overflow-hidden">
        {loading ? (
          <div className="p-12 flex items-center justify-center gap-2 text-gray-600">
            <div className="w-6 h-6 border-2 border-primary/30 border-t-primary rounded-full animate-spin" />
            <span>Loading history…</span>
          </div>
        ) : error ? (
          <div className="p-12 text-center">
            <p className="text-red-600 mb-2">{error}</p>
            <p className="text-sm text-gray-500">
              History is stored locally. You do not need a Zebra node to view past transactions.
            </p>
          </div>
        ) : txs.length === 0 ? (
          <div className="p-12 text-center text-gray-500">
            <p>No transactions found</p>
            <p className="text-sm mt-1">Sent and received transactions will appear here.</p>
          </div>
        ) : filteredTxs.length === 0 ? (
          <div className="p-12 text-center text-gray-500">
            <p>No transactions match your filters or search</p>
            <p className="text-sm mt-1">Try changing the filters or search term.</p>
          </div>
        ) : (
          <>
            {(filterType !== "all" || filterStatus !== "all" || filterDateRange !== "all" || searchQuery.trim()) && (
              <div className="px-4 py-2 border-b border-gray-100/50 text-sm text-gray-600 bg-white/30">
                Showing {filteredTxs.length} of {txs.length} transactions
              </div>
            )}
            <div className="divide-y divide-gray-100/50">
            {filteredTxs.map((tx) => (
              <div
                key={tx.id}
                role="button"
                tabIndex={0}
                onClick={() => setSelectedTx(tx)}
                onKeyDown={(e) => e.key === "Enter" && setSelectedTx(tx)}
                className="p-4 hover:bg-white/40 transition-colors flex items-center justify-between group cursor-pointer focus:outline-none focus:ring-2 focus:ring-primary/30 focus:ring-inset"
              >
                <div className="flex items-center gap-4">
                  <div
                    className={`w-10 h-10 rounded-full flex items-center justify-center ${
                      tx.type === "received"
                        ? "bg-green-100 text-green-600"
                        : "bg-red-100 text-red-600"
                    }`}
                  >
                    {tx.type === "received" ? (
                      <ArrowLeftDown size={20} />
                    ) : (
                      <ArrowRightUp size={20} />
                    )}
                  </div>
                  <div>
                    <p className="font-semibold text-gray-900">
                      {tx.type === "received" ? "Received" : "Sent"}
                    </p>
                    <div className="flex items-center gap-2 text-xs text-gray-500">
                      <Calendar size={12} />
                      <span>{tx.date || "—"}</span>
                      <span className="w-1 h-1 rounded-full bg-gray-300" />
                      <span className="font-mono truncate max-w-[140px]" title={tx.address}>
                        {tx.address}
                      </span>
                    </div>
                    {tx.memo && (
                      <p className="text-xs text-gray-500 mt-0.5 truncate max-w-[200px]" title={tx.memo}>
                        {tx.memo}
                      </p>
                    )}
                  </div>
                </div>

                <div className="text-right">
                  <p
                    className={`font-bold uppercase ${
                      tx.type === "received"
                        ? "text-green-600"
                        : "text-gray-900"
                    }`}
                  >
                    {tx.type === "received" ? "+" : "-"}
                    {tx.amount.toFixed(4)} ZEC
                    {fiatLine(tx.amount) && (
                      <span className="block text-xs font-normal normal-case text-gray-500 mt-0.5">
                        ≈ {fiatLine(tx.amount)}
                      </span>
                    )}
                  </p>
                  <span
                    className={`text-xs px-2 py-0.5 rounded-full ${
                      tx.status === "confirmed"
                        ? "bg-green-100 text-green-700"
                        : tx.status === "pending"
                          ? "bg-yellow-100 text-yellow-700"
                          : "bg-gray-100 text-gray-700"
                    }`}
                  >
                    {tx.status}
                  </span>
                </div>
              </div>
            ))}
            </div>
          </>
        )}
      </div>

      <Modal
        isOpen={selectedTx !== null}
        onClose={() => setSelectedTx(null)}
        title="Transaction details"
      >
        {selectedTx && (
          <div className="space-y-4">
            <DetailRow label="Transaction ID" value={selectedTx.id} mono />
            <DetailRow label="Type" value={selectedTx.type === "received" ? "Received" : "Sent"} />
            <DetailRow
              label="Amount"
              value={
                `${selectedTx.type === "received" ? "+" : "-"}${selectedTx.amount.toFixed(8)} ZEC` +
                (fiatLine(selectedTx.amount) ? ` (≈ ${fiatLine(selectedTx.amount)})` : "")
              }
            />
            <DetailRow label="Date" value={selectedTx.date ? formatDetailDate(selectedTx.date) : "—"} />
            <div className="flex items-start justify-between gap-2">
              <div className="min-w-0 flex-1">
                <DetailRow label="Address" value={selectedTx.address} mono />
              </div>
              {(selectedTx.address.startsWith("u1") || selectedTx.address.startsWith("zs1")) && (
                <Button
                  variant="ghost"
                  size="sm"
                  className="shrink-0 text-primary hover:bg-primary/10"
                  onClick={() => setSaveContactOpen(true)}
                >
                  Save to contacts
                </Button>
              )}
            </div>
            <DetailRow label="Status" value={selectedTx.status} />
            {selectedTx.memo && <DetailRow label="Memo" value={selectedTx.memo} />}
            {detailLoading && (
              <p className="text-sm text-gray-500">Loading extra details…</p>
            )}
            {!detailLoading && detailExtra && (
              <div className="pt-3 mt-3 border-t border-gray-200 space-y-2">
                {detailExtra.confirmations != null && (
                  <DetailRow label="Confirmations" value={String(detailExtra.confirmations)} />
                )}
                {detailExtra.block_height != null && (
                  <DetailRow label="Block height" value={String(detailExtra.block_height)} />
                )}
                {detailExtra.fee_zec != null && (
                  <DetailRow
                    label="Fee"
                    value={
                      `${detailExtra.fee_zec.toFixed(8)} ZEC` +
                      (fiatLine(detailExtra.fee_zec) ? ` (≈ ${fiatLine(detailExtra.fee_zec)})` : "")
                    }
                  />
                )}
                {detailExtra.broadcast_at && (
                  <DetailRow label="Broadcast at" value={formatDetailDate(detailExtra.broadcast_at)} />
                )}
              </div>
            )}
          </div>
        )}
      </Modal>

      <Modal
        isOpen={saveContactOpen}
        onClose={() => !saveContactSaving && setSaveContactOpen(false)}
        title="Save to contacts"
      >
        <div className="space-y-4">
          <Input
            label="Name"
            placeholder="e.g. Exchange"
            value={saveContactName}
            onChange={(e) => setSaveContactName(e.target.value)}
          />
          <Input
            label="Notes (optional)"
            placeholder="e.g. Withdrawal"
            value={saveContactNotes}
            onChange={(e) => setSaveContactNotes(e.target.value)}
          />
          <div className="flex gap-2 justify-end pt-2">
            <Button variant="outline" onClick={() => setSaveContactOpen(false)} disabled={saveContactSaving}>
              Cancel
            </Button>
            <Button onClick={handleSaveToContacts} disabled={saveContactSaving}>
              {saveContactSaving ? "Saving…" : "Save"}
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}

function DetailRow({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return (
    <div>
      <p className="text-xs font-medium text-gray-500 uppercase tracking-wide">{label}</p>
      <p className={`mt-0.5 text-gray-900 break-all ${mono ? "font-mono text-sm" : ""}`}>{value}</p>
    </div>
  );
}

function formatDetailDate(dateStr: string): string {
  if (!dateStr) return "—";
  const d = new Date(dateStr);
  if (Number.isNaN(d.getTime())) return dateStr;
  return d.toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" });
}
