import { useState, useEffect } from "react";
import { useWalletStore } from "../store/walletStore";
import { useTokenStore } from "../store/tokenStore";
import { Button } from "../components/Button";
import {
  Eye,
  EyeClosed,
  Copy,
  CheckCircle,
  ArrowRightUp,
  ArrowLeftDown,
} from "@solar-icons/react";
import { TabId } from "../components/Header";
import { Modal } from "../components/Modal";
import { ReceiveContent } from "../components/ReceiveContent";
import { Tooltip } from "../components/Tooltip";
import { RecentActivityPanel } from "../components/RecentActivityPanel";
import { BlockSyncPanel } from "../components/BlockSyncPanel";
import { walletApi } from "../lib/api";
import {
  formatHistoryDetailDate,
  historyAmountPrefix,
  historyTypeLabel,
  normalizeHistoryTx,
  sortHistoryNewestFirst,
  type HistoryTx,
} from "../lib/history";
import { TransactionIdDetail } from "../components/TxExplorerLink";

import { useSettingsStore } from "../store/settingsStore";
import { getZecPriceInFiat, formatFiatAmount } from "../utils/price";
import toast from "react-hot-toast";
interface HomePageProps {
  onNavigate: (tab: TabId) => void;
}

function TxDetailRow({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return (
    <div>
      <p className="text-xs font-medium text-gray-500 uppercase tracking-wide">{label}</p>
      <p className={`mt-0.5 text-gray-900 break-all ${mono ? "font-mono text-sm" : ""}`}>{value}</p>
    </div>
  );
}

export function HomePage({ onNavigate }: HomePageProps) {
  const { balance, address } = useWalletStore();
  const { tokens, activeTokenId, getToken } = useTokenStore();
  const {
    hideBalance,
    accountLabels,
    activeAccountId,
    fiatCurrency,
    useLiveFiatPrice,
    customFiatPerZec,
  } = useSettingsStore();
  const activeAccountLabel = accountLabels[activeAccountId] ?? activeAccountId ?? "Default";
  const activeToken = activeTokenId ? getToken(activeTokenId) : tokens[0];
  const [showBalance, setShowBalance] = useState(!hideBalance);
  const [activeModal, setActiveModal] = useState<"receive" | null>(null);
  const [copied, setCopied] = useState(false);
  const [recentHistory, setRecentHistory] = useState<HistoryTx[]>([]);
  const [historyLoading, setHistoryLoading] = useState(true);
  const [historyError, setHistoryError] = useState<string | null>(null);
  const [selectedTx, setSelectedTx] = useState<HistoryTx | null>(null);
  const [detailLoading, setDetailLoading] = useState(false);
  const [detailExtra, setDetailExtra] = useState<{
    confirmations?: number;
    block_height?: number;
    fee_zec?: number;
    broadcast_at?: string;
  } | null>(null);
  const [fiatRate, setFiatRate] = useState<number | null>(null);

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
  const balanceFiatLine =
    showBalance && effectiveFiatRate != null && effectiveFiatRate > 0
      ? `≈ ${formatFiatAmount(balance * effectiveFiatRate, fiatCurrency)}`
      : null;

  // Fetch transaction history
  useEffect(() => {
    let cancelled = false;
    const fetchHistory = async () => {
      setHistoryLoading(true);
      setHistoryError(null);
      try {
        await walletApi.checkTransactionConfirmations().catch(() => undefined);
        const res = await walletApi.getTransactionHistory();
        if (cancelled) return;
        if (res?.data && Array.isArray(res.data)) {
          setRecentHistory(
            sortHistoryNewestFirst(
              res.data
                .map((row) => normalizeHistoryTx(row as Record<string, unknown>))
                .filter((tx) => tx.id)
            ).slice(0, 3)
          );
        } else {
          setRecentHistory([]);
        }
      } catch {
        if (!cancelled) {
          setHistoryError("Could not load recent activity");
          setRecentHistory([]);
        }
      } finally {
        if (!cancelled) setHistoryLoading(false);
      }
    };
    fetchHistory();
    return () => {
      cancelled = true;
    };
  }, []);

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

  const displayAddress = address || "No address available";

  const handleCopy = () => {
    navigator.clipboard.writeText(displayAddress);
    toast.success("Address copied to clipboard");
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <>
      <div className="flex flex-col gap-8 animate-fade-in h-full w-full">
        <div className="space-y-8">
          <header className="flex justify-between items-center">
            <div>
              <h2 className="text-3xl font-bold text-gray-900 dark:text-gray-100">
                Shielded Orchard Assets
              </h2>
              <p className="text-gray-500 dark:text-gray-400">
                {activeAccountLabel} · Your portfolio
              </p>
            </div>
            <div className="flex gap-3">
              <Tooltip content="Get your receive address">
                <Button
                  variant="outline"
                  onClick={() => setActiveModal("receive")}
                  className="gap-2 bg-white/60 backdrop-blur-sm border border-white/50 text-gray-700 hover:bg-white/90"
                >
                  <ArrowLeftDown size={20} /> Receive
                </Button>
              </Tooltip>
              <Tooltip content="Send ZEC to another address">
                <Button
                  onClick={() => onNavigate("send")}
                  className="gap-2 shadow-lg shadow-primary/20"
                >
                  <ArrowRightUp size={20} /> Send
                </Button>
              </Tooltip>
            </div>
          </header>

          <div className="relative overflow-hidden rounded-3xl p-8 shadow-2xl bg-gradient-to-br from-[#D4AF37] to-[#F8D775] text-black transform transition-transform hover:scale-[1.01] duration-300">
            <div className="relative z-10">
              <div className="flex items-center gap-3 mb-2">
                <span className="uppercase tracking-wider text-xs font-bold text-black/70">
                  Total Balance
                </span>

                <button
                  onClick={() => setShowBalance(!showBalance)}
                  className="hover:text-black transition-colors"
                >
                  {showBalance ? <Eye size={16} /> : <EyeClosed size={16} />}
                </button>
              </div>

              <div className="flex items-end gap-2">
                <span className="text-5xl font-extrabold tracking-tight">
                  {showBalance ? balance.toLocaleString() : "••••••"}
                </span>
                <span className="text-2xl font-medium mb-1 text-black/70 uppercase">
                  {activeToken ? activeToken.symbol : "ZEC"}
                </span>
              </div>
              {balanceFiatLine && (
                <p className="mt-1 text-lg font-medium text-black/70">{balanceFiatLine}</p>
              )}

              <div className="mt-8 flex items-center justify-between text-sm">
                <Tooltip content="Copy address">
                  <div
                    className="flex items-center gap-2 text-black/70 hover:text-black cursor-pointer transition-colors group"
                    onClick={handleCopy}
                  >
                    <span className="font-medium">{displayAddress}</span>
                    {copied ? (
                      <CheckCircle
                        size={14}
                        className="text-black"
                      />
                    ) : (
                      <Copy
                        size={14}
                        className="group-hover:text-black transition-colors opacity-60 group-hover:opacity-100"
                      />
                    )}
                  </div>
                </Tooltip>
              </div>
            </div>
            <div className="absolute -right-12 -bottom-12 w-64 h-64 bg-white/20 rounded-full blur-3xl pointer-events-none" />
            <div className="absolute -left-12 -top-12 w-48 h-48 bg-white/20 rounded-full blur-2xl pointer-events-none" />
          </div>

          <div className="space-y-4">
            <h3 className="text-lg font-semibold text-gray-800">Your Assets</h3>
            {tokens.map((token) => (
              <div
                key={token.id}
                className="p-4 rounded-2xl bg-white/60 backdrop-blur-sm border border-white/50 flex items-center justify-between hover:bg-white/80 transition-colors cursor-pointer group"
              >
                <div className="flex items-center gap-4">
                  <div className="w-12 h-12 rounded-full overflow-hidden shadow-md group-hover:scale-110 transition-transform flex-shrink-0">
                    {token.icon ? (
                      <img
                        src={token.icon}
                        alt={`${token.name} logo`}
                        className="w-full h-full object-cover"
                      />
                    ) : (
                      <div className="w-full h-full bg-[#FA6800] flex items-center justify-center text-white">
                        <span className="font-bold">{token.symbol[0]}</span>
                      </div>
                    )}
                  </div>
                  <div>
                    <p className="font-bold text-gray-900 uppercase">
                      {token.name}
                    </p>
                    <p className="text-sm text-gray-500 uppercase">
                      {token.symbol}
                    </p>
                  </div>
                </div>
                <div className="text-right">
                  <p className="font-bold text-gray-900">
                    {showBalance && activeTokenId === token.id
                      ? balance.toLocaleString()
                      : "0"}
                  </p>
                  <p className="text-sm text-gray-500">
                    {token.isNative ? "Privacy Coin" : "Token"}
                  </p>
                </div>
              </div>
            ))}
          </div>

          <RecentActivityPanel
            transactions={recentHistory}
            loading={historyLoading}
            error={historyError}
            tokenSymbol={activeToken?.symbol}
            fiatRate={effectiveFiatRate}
            fiatCurrency={fiatCurrency}
            onViewAll={() => onNavigate("history")}
            onSelect={setSelectedTx}
          />

          <BlockSyncPanel />
        </div>
      </div>

      <Modal
        isOpen={selectedTx !== null}
        onClose={() => setSelectedTx(null)}
        title="Transaction details"
      >
        {selectedTx && (
          <div className="space-y-4">
            <TransactionIdDetail txid={selectedTx.id} />
            <TxDetailRow label="Type" value={historyTypeLabel(selectedTx)} />
            <TxDetailRow
              label="Amount"
              value={`${historyAmountPrefix(selectedTx)}${selectedTx.amount.toFixed(8)} ZEC`}
            />
            <TxDetailRow label="Date" value={formatHistoryDetailDate(selectedTx)} />
            <TxDetailRow label="Address" value={selectedTx.address} mono />
            <TxDetailRow label="Status" value={selectedTx.status ? selectedTx.status.charAt(0).toUpperCase() + selectedTx.status.slice(1) : "—"} />
            {selectedTx.memo && <TxDetailRow label="Memo" value={selectedTx.memo} />}
            {detailLoading && <p className="text-sm text-gray-500">Loading extra details…</p>}
            {!detailLoading && detailExtra && (
              <div className="pt-3 mt-3 border-t border-gray-200 space-y-2">
                {detailExtra.confirmations != null && (
                  <TxDetailRow label="Confirmations" value={String(detailExtra.confirmations)} />
                )}
                {detailExtra.block_height != null && (
                  <TxDetailRow label="Block height" value={String(detailExtra.block_height)} />
                )}
                {detailExtra.fee_zec != null && (
                  <TxDetailRow label="Fee" value={`${detailExtra.fee_zec.toFixed(8)} ZEC`} />
                )}
                {detailExtra.broadcast_at && (
                  <TxDetailRow
                    label="Broadcast at"
                    value={formatHistoryDetailDate({ date: detailExtra.broadcast_at.slice(0, 10) })}
                  />
                )}
              </div>
            )}
          </div>
        )}
      </Modal>

      <Modal
        isOpen={activeModal === "receive"}
        onClose={() => setActiveModal(null)}
        title={`Receive ${activeToken?.symbol}`}
      >
        <ReceiveContent />
      </Modal>
    </>
  );
}
