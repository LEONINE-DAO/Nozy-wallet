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
import { SendForm } from "../components/SendForm";
import { ReceiveContent } from "../components/ReceiveContent";
import { Tooltip } from "../components/Tooltip";
import { walletApi } from "../lib/api";

import { useSettingsStore } from "../store/settingsStore";
import toast from "react-hot-toast";
interface HomePageProps {
  onNavigate: (tab: TabId) => void;
}

export function HomePage({ onNavigate }: HomePageProps) {
  const { balance, address } = useWalletStore();
  const { tokens, activeTokenId, getToken } = useTokenStore();
  const { hideBalance, accountLabels, activeAccountId } = useSettingsStore();
  const activeAccountLabel = accountLabels[activeAccountId] ?? activeAccountId ?? "Default";
  const activeToken = activeTokenId ? getToken(activeTokenId) : tokens[0];
  const [showBalance, setShowBalance] = useState(!hideBalance);
  const [activeModal, setActiveModal] = useState<"send" | "receive" | null>(
    null
  );
  const [copied, setCopied] = useState(false);
  const [recentHistory, setRecentHistory] = useState<any[]>([]);

  // Fetch transaction history
  useEffect(() => {
    const fetchHistory = async () => {
      try {
        const res = await walletApi.getTransactionHistory();
        if (res?.data && Array.isArray(res.data)) {
          // Get most recent 3 transactions
          setRecentHistory(res.data.slice(0, 3));
        }
      } catch (e) {
        // No transactions or error - show empty
        setRecentHistory([]);
      }
    };
    fetchHistory();
  }, []);

  const displayAddress = address || "No address available";

  const handleCopy = () => {
    navigator.clipboard.writeText(displayAddress);
    toast.success("Address copied to clipboard");
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <>
      <div className="flex flex-col lg:flex-row gap-8 animate-fade-in h-full">
        <div className="flex-1 space-y-8">
          <header className="flex justify-between items-center">
            <div>
              <h2 className="text-3xl font-bold text-gray-900 dark:text-gray-100">Assets</h2>
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
                  onClick={() => setActiveModal("send")}
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
                  <div className="w-12 h-12 rounded-full bg-[#FA6800] flex items-center justify-center text-white shadow-md group-hover:scale-110 transition-transform">
                    <span className="font-bold">{token.symbol[0]}</span>
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
        </div>

        <div className="lg:w-96 flex flex-col h-full overflow-hidden">
          <div className="flex items-center justify-between mb-6">
            <h3 className="text-xl font-bold text-gray-900">Recent Activity</h3>
            <button
              onClick={() => onNavigate("history")}
              className="text-sm text-primary hover:text-primary-700 font-medium hover:underline"
            >
              View All
            </button>
          </div>

          <div className="flex-1 overflow-y-auto space-y-3 pr-2 custom-scrollbar">
            {recentHistory.length === 0 ? (
              <div className="p-8 text-center text-gray-500">
                <p className="text-sm">No recent transactions</p>
                <p className="text-xs mt-1">Your transaction history will appear here</p>
              </div>
            ) : (
              recentHistory.map((tx) => (
              <div
                key={tx.id}
                className="p-4 rounded-xl bg-white/60 backdrop-blur-sm border border-white/50 hover:bg-white transition-all cursor-pointer group"
              >
                <div className="flex justify-between items-start mb-2">
                  <div
                    className={`p-2 rounded-full ${
                      tx.type === "received"
                        ? "bg-green-100 text-green-600"
                        : "bg-red-100 text-red-600"
                    }`}
                  >
                    {tx.type === "received" ? (
                      <ArrowLeftDown size={16} />
                    ) : (
                      <ArrowRightUp size={16} />
                    )}
                  </div>
                  <span
                    className={`font-bold uppercase ${
                      tx.type === "received"
                        ? "text-green-600"
                        : "text-gray-900"
                    }`}
                  >
                    {tx.type === "received" ? "+" : "-"}
                    {tx.amount} {activeToken?.symbol}
                  </span>
                </div>
                <div className="flex justify-between items-center text-xs text-gray-500">
                  <span className="font-mono bg-gray-100 px-2 py-0.5 rounded text-gray-600">
                    {tx.address || tx.txid || "Unknown"}
                  </span>
                  <span>{tx.date || new Date(tx.timestamp * 1000).toLocaleDateString()}</span>
                </div>
              </div>
              ))
            )}
          </div>
        </div>
      </div>

      <Modal
        isOpen={activeModal === "send"}
        onClose={() => setActiveModal(null)}
        title={`Send ${activeToken?.symbol}`}
      >
        <SendForm
          onSuccess={() => setActiveModal(null)}
          onCancel={() => setActiveModal(null)}
        />
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
