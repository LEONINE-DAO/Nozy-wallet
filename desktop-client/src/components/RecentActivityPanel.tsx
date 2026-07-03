import { ArrowLeftDown, ArrowRightUp } from "@solar-icons/react";
import type { HistoryTx } from "../lib/history";
import { formatHistoryDate, historyAmountPrefix, historyTypeLabel } from "../lib/history";
import type { FiatCurrency } from "../lib/fiatCurrencies";
import { formatFiatAmount } from "../utils/price";
import { TxExplorerLink } from "./TxExplorerLink";

interface RecentActivityPanelProps {
  transactions: HistoryTx[];
  loading: boolean;
  error: string | null;
  tokenSymbol?: string;
  fiatRate?: number | null;
  fiatCurrency?: FiatCurrency;
  onViewAll: () => void;
  onSelect?: (tx: HistoryTx) => void;
}

function formatAmount(amount: number): string {
  return amount.toLocaleString(undefined, {
    minimumFractionDigits: 2,
    maximumFractionDigits: 8,
  });
}

function fiatLine(
  amountZec: number,
  fiatRate: number | null | undefined,
  fiatCurrency: FiatCurrency
): string | null {
  if (fiatRate == null || fiatRate <= 0) return null;
  return formatFiatAmount(amountZec * fiatRate, fiatCurrency);
}

export function RecentActivityPanel({
  transactions,
  loading,
  error,
  tokenSymbol = "ZEC",
  fiatRate,
  fiatCurrency = "USD",
  onViewAll,
  onSelect,
}: RecentActivityPanelProps) {
  return (
    <div className="w-full flex flex-col">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-xl font-bold text-gray-900 dark:text-gray-100">Recent Activity</h3>
        <button
          type="button"
          onClick={onViewAll}
          className="text-sm text-primary hover:text-primary-700 font-medium hover:underline"
        >
          Transaction history
        </button>
      </div>

      <div className="space-y-4">
        {loading ? (
          <div className="p-12 flex items-center justify-center gap-3 text-gray-600 dark:text-gray-300 rounded-2xl bg-white/60 dark:bg-gray-800/60 border border-white/50 dark:border-gray-700/50">
            <div className="w-6 h-6 border-2 border-primary/30 border-t-primary rounded-full animate-spin" />
            <span className="text-base">Loading activity…</span>
          </div>
        ) : error ? (
          <div className="p-10 text-center rounded-2xl bg-white/60 dark:bg-gray-800/60 border border-white/50 dark:border-gray-700/50">
            <p className="text-base text-red-600 dark:text-red-400">{error}</p>
          </div>
        ) : transactions.length === 0 ? (
          <div className="p-12 text-center text-gray-500 dark:text-gray-400 rounded-2xl bg-white/60 dark:bg-gray-800/60 border border-white/50 dark:border-gray-700/50">
            <p className="text-base">No recent transactions</p>
            <p className="text-sm mt-1">Your transaction history will appear here</p>
            <p className="text-xs mt-2 text-gray-400 dark:text-gray-500">
              Sent transactions are recorded when you send from this wallet.
            </p>
          </div>
        ) : (
          transactions.map((tx) => (
            <button
              key={tx.id}
              type="button"
              onClick={() => onSelect?.(tx)}
              className="w-full p-5 sm:p-6 rounded-2xl bg-white/60 dark:bg-gray-800/60 backdrop-blur-sm border border-white/50 dark:border-gray-700/50 hover:bg-white dark:hover:bg-gray-700/60 transition-all cursor-pointer group text-left"
            >
              <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
                <div className="flex items-center gap-4 min-w-0">
                  <div
                    className={`w-12 h-12 shrink-0 rounded-full flex items-center justify-center ${
                      tx.type === "received"
                        ? "bg-green-100 text-green-600 dark:bg-green-900/40 dark:text-green-400"
                        : tx.type === "change"
                          ? "bg-blue-100 text-blue-600 dark:bg-blue-900/40 dark:text-blue-400"
                          : "bg-red-100 text-red-600 dark:bg-red-900/40 dark:text-red-400"
                    }`}
                  >
                    {tx.type === "received" ? (
                      <ArrowLeftDown size={22} />
                    ) : (
                      <ArrowRightUp size={22} />
                    )}
                  </div>
                  <div className="min-w-0">
                    <p className="font-semibold text-lg text-gray-900 dark:text-gray-100">
                      {historyTypeLabel(tx)}
                    </p>
                    <p
                      className="font-mono text-sm text-gray-500 dark:text-gray-400 truncate mt-0.5"
                      title={tx.address}
                    >
                      {tx.address || tx.id || "Unknown"}
                    </p>
                  </div>
                </div>

                <div className="flex sm:flex-col items-start sm:items-end gap-1 sm:gap-2 shrink-0 pl-16 sm:pl-0">
                  <div className="text-right">
                    <span
                      className={`font-bold text-lg ${
                        tx.type === "received"
                          ? "text-green-600 dark:text-green-400"
                          : tx.type === "change"
                            ? "text-gray-900 dark:text-gray-100"
                            : "text-gray-900 dark:text-gray-100"
                      }`}
                    >
                      {historyAmountPrefix(tx)}
                      {formatAmount(tx.amount)} {tokenSymbol}
                    </span>
                    {fiatLine(tx.amount, fiatRate, fiatCurrency) && (
                      <span className="block text-sm font-normal text-gray-500 dark:text-gray-400">
                        ≈ {fiatLine(tx.amount, fiatRate, fiatCurrency)}
                      </span>
                    )}
                  </div>
                  <span className="text-sm text-gray-500 dark:text-gray-400">
                    {formatHistoryDate(tx)}
                  </span>
                  <span
                    className="mt-1"
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
            </button>
          ))
        )}
      </div>
    </div>
  );
}
