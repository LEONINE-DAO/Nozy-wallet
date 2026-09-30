import { useEffect, useMemo, useState } from "react";
import { ArrowRightUp, ArrowLeftDown, Calendar } from "@solar-icons/react";
import { walletApi } from "../lib/api";
import { Input } from "../components/Input";
import { Select } from "../components/Select";
import { Modal } from "../components/Modal";
import { Button } from "../components/Button";
import { PageHeader } from "../components/PageHeader";
import { FeatureShell, MetricTile, Panel } from "../components/FeatureSurface";
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
  const cutoffDate =
    cutoffDays != null
      ? (() => {
          const d = new Date(now);
          d.setDate(d.getDate() - cutoffDays);
          return d;
        })()
      : null;

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
  if (tx.type === "received") return "border-emerald-400/30 bg-emerald-500/15 text-emerald-300";
  if (tx.type === "change") return "border-sky-400/30 bg-sky-500/15 text-sky-300";
  return "border-rose-400/30 bg-rose-500/15 text-rose-300";
}

function amountClass(tx: HistoryTx): string {
  if (tx.type === "received") return "text-emerald-300";
  return "text-gray-100";
}

function statusPillClass(status: string): string {
  if (status === "confirmed") return "border-emerald-400/30 bg-emerald-500/15 text-emerald-200";
  if (status === "pending") return "border-amber-400/30 bg-amber-500/15 text-amber-200";
  if (status === "failed") return "border-rose-400/30 bg-rose-500/15 text-rose-200";
  if (status === "expired") return "border-orange-400/30 bg-orange-500/15 text-orange-200";
  return "border-white/10 bg-white/5 text-gray-300";
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
    ]
      .map(escapeCsvField)
      .join(",")
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
  const [detailExtra, setDetailExtra] = useState<{
    confirmations?: number;
    block_height?: number;
    fee_zec?: number;
    broadcast_at?: string;
  } | null>(null);
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
      .catch(() => ({
        data: { pending_updated: 0, expired_updated: 0, confirmations_updated: 0 },
      }))
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
            fee_zec:
              typeof d.fee_zec === "number"
                ? d.fee_zec
                : typeof d.fee_zatoshis === "number"
                  ? d.fee_zatoshis / 100_000_000
                  : undefined,
            broadcast_at: typeof d.broadcast_at === "string" ? d.broadcast_at : undefined,
          });
        }
      })
      .catch(() => setDetailExtra(null))
      .finally(() => setDetailLoading(false));
  }, [selectedTx?.id]);

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
    return formatFiatAmount(amountZec * effectiveFiatRate, fiatCurrency);
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

  const filterSelectClass =
    "rounded-xl border border-white/10 bg-black/30 px-3 py-2 text-sm text-gray-100 focus:border-primary/40 focus:outline-none focus:ring-2 focus:ring-primary/20";

  return (
    <div className="flex flex-col gap-8 animate-fade-in w-full pb-4">
      <PageHeader
        title="History"
        description="Local shielded transaction log"
        actions={
          <Button type="button" variant="outline" onClick={() => loadHistory()} disabled={loading}>
            {loading ? "Refreshing…" : "Refresh"}
          </Button>
        }
      />

      <FeatureShell>
        <div className="flex flex-col gap-5 lg:flex-row lg:items-start lg:justify-between">
          <div className="min-w-0 space-y-3">
            <div className="flex flex-wrap items-center gap-2">
              <span className="rounded-full border border-primary/35 bg-primary/10 px-3 py-1 text-[0.65rem] font-bold uppercase tracking-[0.22em] text-primary">
                Activity
              </span>
              {!loading && !error ? (
                <span className="rounded-full border border-white/10 bg-white/5 px-3 py-1 text-[0.65rem] font-bold uppercase tracking-[0.16em] text-gray-300">
                  {txs.length} tx{txs.length === 1 ? "" : "s"}
                </span>
              ) : null}
            </div>
            <div>
              <h3 className="text-2xl font-extrabold tracking-tight text-primary-100">
                Transaction history
              </h3>
              <p className="mt-2 max-w-2xl text-sm leading-relaxed text-gray-400">
                Stored on this device. Explorer links are optional for confirmation checks.
              </p>
            </div>
          </div>
        </div>

        <div className="mt-4 grid gap-3 sm:grid-cols-3">
          <MetricTile
            label="Total"
            value={loading ? "…" : String(txs.length)}
            accent="gold"
            hint="All recorded txs"
          />
          <MetricTile label="Sent" value={loading ? "…" : String(sentCount)} hint="Outgoing" />
          <MetricTile
            label="Received"
            value={loading ? "…" : String(receivedCount)}
            accent="emerald"
            hint="Incoming"
          />
        </div>

        {!loading && !error && txs.length > 0 ? (
          <Panel title="Filters" subtitle="Search and narrow the list" className="mt-4">
            <div className="flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-center">
              <div className="min-w-[200px] flex-1">
                <Input
                  type="search"
                  placeholder="Search by address, txid, or memo"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  aria-label="Search transactions"
                  className="border-white/10 bg-black/30 text-gray-100"
                />
              </div>
              <Select
                value={filterType}
                onChange={(e) => setFilterType(e.target.value as FilterType)}
                aria-label="Filter by type"
                className={`w-auto min-w-[8rem] ${filterSelectClass}`}
              >
                <option value="all">All types</option>
                <option value="sent">Sent</option>
                <option value="received">Received</option>
              </Select>
              <Select
                value={filterStatus}
                onChange={(e) => setFilterStatus(e.target.value as FilterStatus)}
                aria-label="Filter by status"
                className={`w-auto min-w-[8rem] ${filterSelectClass}`}
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
                className={`w-auto min-w-[8rem] ${filterSelectClass}`}
              >
                <option value="all">All time</option>
                <option value="7">Last 7 days</option>
                <option value="30">Last 30 days</option>
                <option value="90">Last 90 days</option>
              </Select>
              <Tooltip content="Download filtered transactions as CSV">
                <Button type="button" variant="outline" onClick={() => downloadCsv(filteredTxs)} size="sm">
                  Export CSV
                </Button>
              </Tooltip>
            </div>
          </Panel>
        ) : null}

        <div className="mt-4 overflow-hidden rounded-2xl border border-white/10 bg-black/20 backdrop-blur-sm">
          {loading ? (
            <div className="flex items-center justify-center gap-2 p-12 text-gray-400">
              <div className="h-6 w-6 animate-spin rounded-full border-2 border-primary/30 border-t-primary" />
              <span>Loading history…</span>
            </div>
          ) : error ? (
            <div className="p-12 text-center">
              <p className="mb-2 text-rose-300">{error}</p>
              <p className="text-sm text-gray-500">
                History is stored locally. You do not need a Zebra node to view past transactions.
              </p>
            </div>
          ) : txs.length === 0 ? (
            <div className="p-12 text-center text-gray-400">
              <p>No transactions found</p>
              <p className="mt-1 text-sm text-gray-500">
                Received deposits appear after sync. Sent transactions appear after you send.
              </p>
            </div>
          ) : filteredTxs.length === 0 ? (
            <div className="p-12 text-center text-gray-400">
              <p>No transactions match your filters or search</p>
              <p className="mt-1 text-sm text-gray-500">Try changing the filters or search term.</p>
            </div>
          ) : (
            <>
              {(filterType !== "all" ||
                filterStatus !== "all" ||
                filterDateRange !== "all" ||
                searchQuery.trim()) && (
                <div className="border-b border-white/8 px-4 py-2 text-sm text-gray-400">
                  Showing {filteredTxs.length} of {txs.length} transactions
                </div>
              )}
              <div className="divide-y divide-white/8">
                {filteredTxs.map((tx) => (
                  <div
                    key={tx.id}
                    role="button"
                    tabIndex={0}
                    onClick={() => setSelectedTx(tx)}
                    onKeyDown={(e) => e.key === "Enter" && setSelectedTx(tx)}
                    className="flex cursor-pointer items-center justify-between p-4 transition-colors hover:bg-white/5 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-primary/30"
                  >
                    <div className="flex min-w-0 items-center gap-4">
                      <div
                        className={`flex h-10 w-10 shrink-0 items-center justify-center rounded-full border ${txIconClass(tx)}`}
                      >
                        {tx.type === "received" ? (
                          <ArrowLeftDown size={20} />
                        ) : (
                          <ArrowRightUp size={20} />
                        )}
                      </div>
                      <div className="min-w-0">
                        <p className="font-semibold text-gray-100">{historyTypeLabel(tx)}</p>
                        <div className="flex items-center gap-2 text-xs text-gray-500">
                          <Calendar size={12} />
                          <span>{formatHistoryDate(tx)}</span>
                          <span className="h-1 w-1 rounded-full bg-gray-600" />
                          <span className="max-w-[140px] truncate font-mono" title={tx.address}>
                            {tx.address}
                          </span>
                        </div>
                        {tx.memo ? (
                          <p
                            className="mt-0.5 max-w-[200px] truncate text-xs text-gray-500"
                            title={tx.memo}
                          >
                            {tx.memo}
                          </p>
                        ) : null}
                      </div>
                    </div>

                    <div className="shrink-0 pl-3 text-right">
                      <p className={`font-bold uppercase ${amountClass(tx)}`}>
                        {historyAmountPrefix(tx)}
                        {tx.amount.toFixed(4)} ZEC
                        {fiatLine(tx.amount) ? (
                          <span className="mt-0.5 block text-xs font-normal normal-case text-gray-500">
                            ≈ {fiatLine(tx.amount)}
                          </span>
                        ) : null}
                      </p>
                      <span
                        className={`mt-1 inline-block rounded-full border px-2 py-0.5 text-xs capitalize ${statusPillClass(tx.status)}`}
                      >
                        {tx.status}
                      </span>
                      <span
                        className="mt-1.5 block"
                        onClick={(e) => e.stopPropagation()}
                        onKeyDown={(e) => e.stopPropagation()}
                      >
                        <TxExplorerLink txid={tx.id} label="View on explorer" variant="pill" />
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            </>
          )}
        </div>
      </FeatureShell>

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
                  className="shrink-0 font-semibold text-primary hover:bg-primary/20"
                  onClick={() => setSaveContactOpen(true)}
                >
                  Save to contacts
                </Button>
              )}
            </div>
            <DetailRow label="Status" value={formatStatusLabel(selectedTx.status)} />
            {selectedTx.memo && <DetailRow label="Memo" value={selectedTx.memo} />}
            {detailLoading && <p className="text-sm text-gray-300">Loading extra details…</p>}
            {!detailLoading && detailExtra && (
              <div className="mt-3 space-y-3 border-t border-gray-600 pt-3">
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
              <div className="mt-2 space-y-3 border-t border-gray-600 pt-4">
                <p className="text-sm text-gray-200">
                  This transaction expired unmined. Speed up rebuilds a new transaction at priority
                  fee (×4).
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
          <div className="flex justify-end gap-2 pt-2">
            <Button
              variant="outline"
              onClick={() => setSaveContactOpen(false)}
              disabled={saveContactSaving}
            >
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
      <p className="text-xs font-semibold uppercase tracking-wide text-gray-300">{label}</p>
      <p
        className={`mt-1 break-all text-base font-medium text-primary-100 ${mono ? "font-mono text-sm" : ""}`}
      >
        {value}
      </p>
    </div>
  );
}
