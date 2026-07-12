import { useEffect, useMemo, useState } from "react";
import { ArrowRightUp, ArrowLeftDown, Calendar } from "@solar-icons/react";
import { walletApi } from "../lib/api";
import { Input } from "../components/Input";
import { Select } from "../components/Select";
import { Modal } from "../components/Modal";
import { Button } from "../components/Button";
import { Card } from "../components/Card";
import { PageHeader } from "../components/PageHeader";
import { Tooltip } from "../components/Tooltip";
import { useSettingsStore } from "../store/settingsStore";
import { getZecPriceInFiat, formatFiatAmount } from "../utils/price";
import { formatErrorForDisplay } from "../utils/errors";
import toast from "react-hot-toast";
import {
  formatHistoryDate,
  formatHistoryDetailDate,
  historyAmountPrefix,
  historyTypeLabel,
  normalizeHistoryTx,
  sortHistoryNewestFirst,
  type HistoryTx,
} from "../lib/history";
import { TransactionIdDetail, TxExplorerLink } from "../components/TxExplorerLink";

type FilterType = "all" | "sent" | "received";
type FilterStatus = "all" | "confirmed" | "pending" | "failed" | "expired";
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

  const filtered = txs
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
    });
  return sortHistoryNewestFirst(filtered);
}

function formatStatusLabel(status: string): string {
  if (!status || status === "unknown") return "—";
  return status.charAt(0).toUpperCase() + status.slice(1);
}

function txIconClass(tx: HistoryTx): string {
  if (tx.type === "received") return "bg-green-100 dark:bg-green-900/30 text-green-600 dark:text-green-400";
  if (tx.type === "change") return "bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400";
  return "bg-red-100 dark:bg-red-900/30 text-red-600 dark:text-red-400";
}

function amountClass(tx: HistoryTx): string {
  if (tx.type === "received") return "text-green-600 dark:text-green-400";
  if (tx.type === "change") return "text-gray-900 dark:text-gray-100";
  return "text-gray-900 dark:text-gray-100";
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
      historyTypeLabel(tx),
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
  const [speedUpPassword, setSpeedUpPassword] = useState("");
  const [speedUpBusy, setSpeedUpBusy] = useState(false);

  const { fiatCurrency, useLiveFiatPrice, customFiatPerZec } = useSettingsStore();

  const loadHistory = () => {
    setLoading(true);
    setError(null);
    return walletApi
      .checkTransactionConfirmations()
      .catch(() => ({ data: { pending_updated: 0, expired_updated: 0, confirmations_updated: 0 } }))
      .then(() => walletApi.getTransactionHistory())
      .then((res) => {
        const raw = res?.data;
        if (Array.isArray(raw)) {
          const normalized = raw
            .map((row) => normalizeHistoryTx(row as Record<string, unknown>))
            .filter((t) => t.id);
          setTxs(normalized);
        } else {
          setTxs([]);
        }
      })
      .catch((e) => {
        setError(formatErrorForDisplay(e, "Failed to load transaction history"));
        setTxs([]);
      })
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    let cancelled = false;
    loadHistory().then(() => {
      if (cancelled) return;
    });
    return () => {
      cancelled = true;
    };
  }, []);

  const filteredTxs = useMemo(
    () => filterAndSortTxs(txs, filterType, filterStatus, filterDateRange, searchQuery),
    [txs, filterType, filterStatus, filterDateRange, searchQuery]
  );

  const sentCount = useMemo(() => txs.filter((t) => t.type === "sent").length, [txs]);
  const receivedCount = useMemo(() => txs.filter((t) => t.type === "received").length, [txs]);

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
    if (!useLiveFiatPrice && customFiatPerZec != null) {
      setFiatRate(customFiatPerZec);
      return;
    }
    if (!useLiveFiatPrice) {
      setFiatRate(null);
      return;
    }
    getZecPriceInFiat(fiatCurrency).then((rate) => setFiatRate(rate));
  }, [useLiveFiatPrice, customFiatPerZec, fiatCurrency]);

  const effectiveFiatRate = useLiveFiatPrice ? fiatRate : customFiatPerZec;

  function fiatLine(amountZec: number): string | null {
    if (effectiveFiatRate == null || effectiveFiatRate <= 0) return null;
    const fiat = amountZec * effectiveFiatRate;
    return formatFiatAmount(fiat, fiatCurrency);
  }

  const handleSpeedUp = async () => {
    if (!selectedTx) return;
    if (!speedUpPassword.trim()) {
      toast.error("Enter your wallet password to speed up");
      return;
    }
    setSpeedUpBusy(true);
    try {
      const { data } = await walletApi.speedUpTransaction({
        originalTxid: selectedTx.id,
        password: speedUpPassword,
      });
      if (!data.success) {
        toast.error(data.message || "Speed-up failed");
        return;
      }
      toast.success(data.message || "Speed-up transaction broadcast");
      setSpeedUpPassword("");
      setSelectedTx(null);
      await loadHistory();
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Speed-up failed"));
    } finally {
      setSpeedUpBusy(false);
    }
  };

  const handleSaveToContacts = async () => {
    if (!selectedTx) return;
    const name = saveContactName.trim();
    const addr = selectedTx.address.trim();
    if (!name || !addr) {
      toast.error("Name is required");
      return;
    }
    if (!addr.startsWith("u1") && !addr.startsWith("utest1") && !addr.startsWith("zs1")) {
      toast.error("Address must be a shielded address (u1, utest1, or zs1)");
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
      toast.error(formatErrorForDisplay(e, "Failed to save contact"));
    } finally {
      setSaveContactSaving(false);
    }
  };

  return (
    <div className="space-y-6 animate-fade-in max-w-3xl mx-auto text-left pb-8">
      <PageHeader
        title="Transaction History"
        description={
          !loading && !error && txs.length > 0
            ? `${sentCount} sent · ${receivedCount} received`
            : undefined
        }
        actions={
          <Button
            type="button"
            variant="outline"
            onClick={() => loadHistory()}
            disabled={loading}
          >
            {loading ? "Refreshing…" : "Refresh"}
          </Button>
        }
      />

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
            <Select
              value={filterType}
              onChange={(e) => setFilterType(e.target.value as FilterType)}
              aria-label="Filter by type"
              className="w-auto min-w-[8rem]"
            >
              <option value="all">All types</option>
              <option value="sent">Sent</option>
              <option value="received">Received</option>
            </Select>
            <Select
              value={filterStatus}
              onChange={(e) => setFilterStatus(e.target.value as FilterStatus)}
              aria-label="Filter by status"
              className="w-auto min-w-[8rem]"
            >
              <option value="all">All statuses</option>
              <option value="confirmed">Confirmed</option>
              <option value="pending">Pending</option>
              <option value="failed">Failed</option>
              <option value="expired">Expired</option>
            </Select>
            <Select
              value={filterDateRange}
              onChange={(e) => setFilterDateRange(e.target.value as FilterDateRange)}
              aria-label="Filter by date range"
              className="w-auto min-w-[8rem]"
            >
              <option value="all">All time</option>
              <option value="7">Last 7 days</option>
              <option value="30">Last 30 days</option>
              <option value="90">Last 90 days</option>
            </Select>
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

      <Card padding="none" className="overflow-hidden">
        {loading ? (
          <div className="p-12 flex items-center justify-center gap-2 text-gray-600 dark:text-gray-400">
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
          <div className="p-12 text-center text-gray-500 dark:text-gray-400">
            <p>No transactions found</p>
            <p className="text-sm mt-1">
              Received deposits appear after sync. Sent transactions appear after you send from this wallet.
            </p>
          </div>
        ) : filteredTxs.length === 0 ? (
          <div className="p-12 text-center text-gray-500 dark:text-gray-400">
            <p>No transactions match your filters or search</p>
            <p className="text-sm mt-1">Try changing the filters or search term.</p>
          </div>
        ) : (
          <>
            {(filterType !== "all" || filterStatus !== "all" || filterDateRange !== "all" || searchQuery.trim()) && (
              <div className="px-4 py-2 border-b border-gray-100/50 dark:border-gray-700/50 text-sm text-gray-600 dark:text-gray-300 bg-white/30 dark:bg-gray-800/40">
                Showing {filteredTxs.length} of {txs.length} transactions
              </div>
            )}
            <div className="divide-y divide-gray-100/50 dark:divide-gray-700/50">
            {filteredTxs.map((tx) => (
              <div
                key={tx.id}
                role="button"
                tabIndex={0}
                onClick={() => setSelectedTx(tx)}
                onKeyDown={(e) => e.key === "Enter" && setSelectedTx(tx)}
                className="p-4 hover:bg-white/40 dark:hover:bg-gray-700/30 transition-colors flex items-center justify-between group cursor-pointer focus:outline-none focus:ring-2 focus:ring-primary/30 focus:ring-inset"
              >
                <div className="flex items-center gap-4">
                  <div
                    className={`w-10 h-10 rounded-full flex items-center justify-center ${txIconClass(tx)}`}
                  >
                    {tx.type === "received" ? (
                      <ArrowLeftDown size={20} />
                    ) : (
                      <ArrowRightUp size={20} />
                    )}
                  </div>
                  <div>
                    <p className="font-semibold text-gray-900 dark:text-gray-100">
                      {historyTypeLabel(tx)}
                    </p>
                    <div className="flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400">
                      <Calendar size={12} />
                      <span>{formatHistoryDate(tx)}</span>
                      <span className="w-1 h-1 rounded-full bg-gray-300 dark:bg-gray-600" />
                      <span className="font-mono truncate max-w-[140px]" title={tx.address}>
                        {tx.address}
                      </span>
                    </div>
                    {tx.memo && (
                      <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5 truncate max-w-[200px]" title={tx.memo}>
                        {tx.memo}
                      </p>
                    )}
                  </div>
                </div>

                <div className="text-right">
                  <p
                    className={`font-bold uppercase ${amountClass(tx)}`}
                  >
                    {historyAmountPrefix(tx)}
                    {tx.amount.toFixed(4)} ZEC
                    {fiatLine(tx.amount) && (
                      <span className="block text-xs font-normal normal-case text-gray-500 dark:text-gray-400 mt-0.5">
                        ≈ {fiatLine(tx.amount)}
                      </span>
                    )}
                  </p>
                  <span
                    className={`text-xs px-2 py-0.5 rounded-full capitalize ${
                      tx.status === "confirmed"
                        ? "bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300"
                        : tx.status === "pending"
                          ? "bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-300"
                          : tx.status === "failed"
                          ? "bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300"
                          : tx.status === "expired"
                            ? "bg-orange-100 dark:bg-orange-900/30 text-orange-700 dark:text-orange-300"
                            : "bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300"
                    }`}
                  >
                    {tx.status}
                  </span>
                  <span
                    className="block mt-1.5"
                    onClick={(e) => e.stopPropagation()}
                    onKeyDown={(e) => e.stopPropagation()}
                  >
                    <TxExplorerLink
                      txid={tx.id}
                      label="View on explorer"
                      variant="pill"
                    />
                  </span>
                </div>
              </div>
            ))}
            </div>
          </>
        )}
      </Card>

      <Modal
        isOpen={selectedTx !== null}
        onClose={() => setSelectedTx(null)}
        title="Transaction details"
      >
        {selectedTx && (
          <div className="space-y-4">
            <TransactionIdDetail txid={selectedTx.id} />
            <DetailRow label="Type" value={historyTypeLabel(selectedTx)} />
            <DetailRow
              label="Amount"
              value={
                `${historyAmountPrefix(selectedTx)}${selectedTx.amount.toFixed(8)} ZEC` +
                (fiatLine(selectedTx.amount) ? ` (≈ ${fiatLine(selectedTx.amount)})` : "")
              }
            />
            <DetailRow label="Date" value={formatHistoryDetailDate(selectedTx)} />
            <div className="flex items-start justify-between gap-2">
              <div className="min-w-0 flex-1">
                <DetailRow label="Address" value={selectedTx.address} mono />
              </div>
              {(selectedTx.address.startsWith("u1") ||
                selectedTx.address.startsWith("utest1") ||
                selectedTx.address.startsWith("zs1")) && (
                <Button
                  variant="ghost"
                  size="sm"
                  className="shrink-0 text-primary hover:bg-primary/20 font-semibold"
                  onClick={() => setSaveContactOpen(true)}
                >
                  Save to contacts
                </Button>
              )}
            </div>
            <DetailRow label="Status" value={formatStatusLabel(selectedTx.status)} />
            {selectedTx.memo && <DetailRow label="Memo" value={selectedTx.memo} />}
            {detailLoading && (
              <p className="text-sm text-gray-300">Loading extra details…</p>
            )}
            {!detailLoading && detailExtra && (
              <div className="pt-3 mt-3 border-t border-gray-600 space-y-3">
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
                  <DetailRow
                    label="Broadcast at"
                    value={formatHistoryDetailDate({ date: detailExtra.broadcast_at })}
                  />
                )}
              </div>
            )}
            {selectedTx.type === "sent" && selectedTx.status === "expired" && (
              <div className="pt-4 mt-2 border-t border-gray-600 space-y-3">
                <p className="text-sm text-gray-200">
                  This transaction expired unmined. Speed up rebuilds a new transaction at priority fee (×4).
                </p>
                <Input
                  type="password"
                  label="Wallet password"
                  placeholder="Required to sign the new transaction"
                  value={speedUpPassword}
                  onChange={(e) => setSpeedUpPassword(e.target.value)}
                />
                <Button
                  onClick={handleSpeedUp}
                  disabled={speedUpBusy || !speedUpPassword.trim()}
                  className="w-full"
                >
                  {speedUpBusy ? "Building priority transaction…" : "Speed up (priority fee ×4)"}
                </Button>
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
      <p className="text-xs font-semibold text-gray-300 uppercase tracking-wide">{label}</p>
      <p className={`mt-1 text-base font-medium text-white break-all ${mono ? "font-mono text-sm" : ""}`}>
        {value}
      </p>
    </div>
  );
}
