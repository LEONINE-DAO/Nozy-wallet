import { useState, useEffect } from "react";
import { useWalletStore } from "../store/walletStore";
import { useTokenStore } from "../store/tokenStore";
import { Button } from "../components/Button";
import { PageHeader } from "../components/PageHeader";
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
import { BlockSyncPanel } from "../components/BlockSyncPanel";
import { useSettingsStore } from "../store/settingsStore";
import { getZecPriceInFiat, formatFiatAmount } from "../utils/price";
import toast from "react-hot-toast";

interface HomePageProps {
  onNavigate: (tab: TabId) => void;
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

  const displayAddress = address || "No address available";

  const handleCopy = () => {
    navigator.clipboard.writeText(displayAddress);
    toast.success("Address copied to clipboard");
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <>
      <div className="flex flex-col gap-8 animate-fade-in w-full pb-4">
        <div className="space-y-6">
          <PageHeader
            title="Shielded Orchard Assets"
            description={`${activeAccountLabel} · Your portfolio`}
            actions={
              <>
                <Tooltip content="Get your receive address">
                  <Button
                    variant="outline"
                    onClick={() => setActiveModal("receive")}
                    className="gap-2"
                  >
                    <ArrowLeftDown size={18} /> Receive
                  </Button>
                </Tooltip>
                <Tooltip content="Send ZEC to another address">
                  <Button onClick={() => onNavigate("send")} className="gap-2">
                    <ArrowRightUp size={18} /> Send
                  </Button>
                </Tooltip>
              </>
            }
          />

          <div className="relative overflow-hidden rounded-2xl p-6 sm:p-8 shadow-xl bg-gradient-to-br from-primary to-primary-200 text-gray-900 transition-transform hover:scale-[1.005] duration-300">
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
                  {showBalance
                    ? balance.toLocaleString(undefined, {
                        minimumFractionDigits: 2,
                        maximumFractionDigits: 8,
                      })
                    : "••••••"}
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
                      <CheckCircle size={14} className="text-black" />
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

          <div className="space-y-3">
            <h3 className="text-base font-semibold text-gray-800 dark:text-gray-200">Your Assets</h3>
            {tokens.map((token) => {
              const isActive = activeTokenId === token.id;
              const amount = showBalance && isActive ? balance : 0;
              const holdingFiat =
                showBalance &&
                isActive &&
                effectiveFiatRate != null &&
                effectiveFiatRate > 0
                  ? formatFiatAmount(amount * effectiveFiatRate, fiatCurrency)
                  : null;
              const unitPrice =
                token.isNative && effectiveFiatRate != null && effectiveFiatRate > 0
                  ? formatFiatAmount(effectiveFiatRate, fiatCurrency)
                  : null;

              return (
                <div
                  key={token.id}
                  className="p-4 rounded-2xl bg-white/60 dark:bg-gray-800/60 backdrop-blur-sm border border-white/50 dark:border-gray-700/50 flex items-center justify-between hover:bg-white/80 dark:hover:bg-gray-800/80 transition-colors group"
                >
                  <div className="flex items-center gap-4 min-w-0">
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
                    <div className="min-w-0">
                      <p className="font-bold text-gray-900 dark:text-gray-100 uppercase">
                        {token.name}
                      </p>
                      <p className="text-sm text-gray-600 dark:text-gray-300 font-medium">
                        {unitPrice
                          ? `1 ${token.symbol} ≈ ${unitPrice}`
                          : token.symbol}
                      </p>
                    </div>
                  </div>
                  <div className="text-right shrink-0 pl-3">
                    <p className="font-bold text-gray-900 dark:text-gray-100 tabular-nums">
                      {showBalance
                        ? `${amount.toLocaleString(undefined, {
                            minimumFractionDigits: 2,
                            maximumFractionDigits: 8,
                          })} ${token.symbol}`
                        : "••••••"}
                    </p>
                    <p className="text-sm font-semibold text-gray-700 dark:text-gray-200 tabular-nums mt-0.5">
                      {showBalance
                        ? holdingFiat
                          ? `≈ ${holdingFiat}`
                          : "Price unavailable"
                        : "••••"}
                    </p>
                  </div>
                </div>
              );
            })}
          </div>

          <BlockSyncPanel />
        </div>
      </div>

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
