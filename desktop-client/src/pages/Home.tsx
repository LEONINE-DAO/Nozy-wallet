import { useState, useEffect, useMemo } from "react";
import { useWalletStore } from "../store/walletStore";
import { useTokenStore } from "../store/tokenStore";
import { Button } from "../components/Button";
import { PageHeader } from "../components/PageHeader";
import { FeatureShell, MetricTile } from "../components/FeatureSurface";
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
import { walletApi } from "../lib/api";
import toast from "react-hot-toast";

interface HomePageProps {
  onNavigate: (tab: TabId) => void;
}

type PoolId = "total" | "orchard" | "ironwood" | "sapling";

function formatZec(amount: number, show: boolean): string {
  if (!show) return "••••••";
  return amount.toLocaleString(undefined, {
    minimumFractionDigits: 2,
    maximumFractionDigits: 8,
  });
}

function zatToZec(zat: number): number {
  return zat / 100_000_000;
}

export function HomePage({ onNavigate }: HomePageProps) {
  const { balance, address } = useWalletStore();
  const { tokens, activeTokenId, getToken } = useTokenStore();
  const {
    hideBalance,
    fiatCurrency,
    useLiveFiatPrice,
    customFiatPerZec,
  } = useSettingsStore();
  const activeToken = activeTokenId ? getToken(activeTokenId) : tokens[0];
  const [showBalance, setShowBalance] = useState(!hideBalance);
  const [activeModal, setActiveModal] = useState<"receive" | null>(null);
  const [copied, setCopied] = useState(false);
  const [fiatRate, setFiatRate] = useState<number | null>(null);
  const [poolTab, setPoolTab] = useState<PoolId>("total");
  const [orchardZec, setOrchardZec] = useState(0);
  const [ironwoodZec, setIronwoodZec] = useState(0);
  const [saplingZec, setSaplingZec] = useState(0);
  const [saplingFeeZec, setSaplingFeeZec] = useState(0);
  const [hasSapling, setHasSapling] = useState(false);
  const [ironwoodSendEnabled, setIronwoodSendEnabled] = useState(true);
  const [ironwoodActive, setIronwoodActive] = useState(false);
  const [legacyBusy, setLegacyBusy] = useState(false);

  const refreshPools = async () => {
    try {
      const [bal, iw, sap] = await Promise.all([
        walletApi.getBalance().catch(() => null),
        walletApi.getIronwoodStatus().catch(() => null),
        walletApi.getSaplingStatus().catch(() => null),
      ]);

      if (bal?.data) {
        const available = bal.data.available_zec ?? bal.data.balance ?? 0;
        useWalletStore.getState().setBalance(available);

        let orchard = bal.data.orchard_zec ?? 0;
        let ironwood = bal.data.ironwood_zec ?? 0;
        const saplingFromBal = bal.data.sapling_zec ?? 0;

        if (orchard + ironwood === 0 && available > 0) {
          orchard = available;
        }

        setOrchardZec(orchard);
        setIronwoodZec(ironwood);

        if (saplingFromBal > 0) {
          setHasSapling(true);
          setSaplingZec(saplingFromBal);
        }
      }

      if (iw?.data) {
        setIronwoodSendEnabled(iw.data.ironwood_send_enabled);
        setIronwoodActive(iw.data.ironwood_active);
        if (!bal?.data) {
          setOrchardZec(zatToZec(iw.data.orchard_wallet_zat));
          setIronwoodZec(zatToZec(iw.data.ironwood_wallet_zat));
        }
      }

      if (sap?.data) {
        setHasSapling(sap.data.has_legacy_balance);
        setSaplingZec(sap.data.unspent_zec);
        setSaplingFeeZec(sap.data.fee_zec);
      } else if (!bal?.data?.sapling_zec) {
        setHasSapling(false);
        setSaplingZec(0);
      }
    } catch {
      /* best-effort */
    }
  };

  useEffect(() => {
    void refreshPools();
  }, []);

  useEffect(() => {
    if (balance > 0) {
      void refreshPools();
    }
  }, [balance]);

  useEffect(() => {
    if (poolTab === "sapling" && !hasSapling) {
      setPoolTab("total");
    }
  }, [hasSapling, poolTab]);

  const handleMoveLegacy = async () => {
    if (legacyBusy) return;
    setLegacyBusy(true);
    const toastId = toast.loading("Moving Sapling funds into shielded balance…");
    try {
      await walletApi.scanSapling({ full: false });
      const res = await walletApi.shieldSapling({});
      toast.success(res.data.message, { id: toastId });
      await refreshPools();
      try {
        const bal = await walletApi.getBalance();
        useWalletStore.getState().setBalance(bal.data.balance);
      } catch {
        /* balance refresh best-effort */
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      toast.error(msg || "Could not move Sapling funds", { id: toastId });
    } finally {
      setLegacyBusy(false);
    }
  };

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

  const poolTabs = useMemo(() => {
    const tabs: { id: PoolId; label: string }[] = [
      { id: "total", label: "Total" },
      { id: "orchard", label: "Orchard" },
      { id: "ironwood", label: "Ironwood" },
    ];
    if (hasSapling) {
      tabs.push({ id: "sapling", label: "Sapling" });
    }
    return tabs;
  }, [hasSapling]);

  const totalBalanceZec = useMemo(
    () => orchardZec + ironwoodZec + saplingZec,
    [orchardZec, ironwoodZec, saplingZec],
  );

  const selectedPoolZec = useMemo(() => {
    switch (poolTab) {
      case "orchard":
        return orchardZec;
      case "ironwood":
        return ironwoodZec;
      case "sapling":
        return saplingZec;
      default:
        return totalBalanceZec;
    }
  }, [poolTab, orchardZec, ironwoodZec, saplingZec, totalBalanceZec]);

  const poolHint = useMemo(() => {
    switch (poolTab) {
      case "orchard":
        return ironwoodActive
          ? "Migrate via turnstile before Ironwood-only sends"
          : "Current shielded pool";
      case "ironwood":
        return ironwoodSendEnabled
          ? "Spendable after Ironwood"
          : "Waiting for Ironwood send readiness";
      case "sapling":
        return "Cannot send out — shield into Orchard/Ironwood";
      default:
        return "Nozy wallet";
    }
  }, [poolTab, ironwoodActive, ironwoodSendEnabled]);

  const canSendFromPool = poolTab !== "sapling";
  const sendBlockedReason =
    poolTab === "sapling"
      ? "Sapling funds cannot be sent out. Move them to shielded with Shield, then use Ironwood/Orchard."
      : poolTab === "orchard" && ironwoodActive && !ironwoodSendEnabled
        ? "Orchard notes remain — migrate via turnstile before sending."
        : null;

  const effectiveFiatRate = useLiveFiatPrice ? fiatRate : customFiatPerZec;
  const displayAddress = address || "No address available";
  const symbol = activeToken ? activeToken.symbol : "ZEC";
  const balanceDisplay = formatZec(selectedPoolZec, showBalance);
  const balanceFiatLine =
    showBalance && effectiveFiatRate != null && effectiveFiatRate > 0
      ? `≈ ${formatFiatAmount(selectedPoolZec * effectiveFiatRate, fiatCurrency)}`
      : null;
  const unitPriceLine =
    effectiveFiatRate != null && effectiveFiatRate > 0
      ? `1 ${symbol} ≈ ${formatFiatAmount(effectiveFiatRate, fiatCurrency)}`
      : null;

  const handleCopy = () => {
    navigator.clipboard.writeText(displayAddress);
    toast.success("Address copied to clipboard");
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const onSendClick = () => {
    if (!canSendFromPool || sendBlockedReason) {
      toast.error(sendBlockedReason ?? "Cannot send from this pool");
      return;
    }
    onNavigate("send");
  };

  return (
    <>
      <div className="relative flex flex-col gap-8 animate-fade-in w-full pb-4">
        <div
          aria-hidden
          className="pointer-events-none absolute inset-x-0 -top-6 bottom-0 z-0 flex items-start justify-center overflow-hidden pt-2 sm:items-start sm:justify-end sm:pr-4 sm:pt-0"
        >
          <img
            src="/nozy-logo.png"
            alt=""
            className="h-[min(70vh,520px)] w-auto max-w-[min(92%,420px)] object-contain opacity-[0.2] mix-blend-screen select-none"
          />
        </div>

        <div className="relative z-10">
        <PageHeader
          title="Home"
          description="Shielded pools · Orchard, Ironwood, Sapling"
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
              <Tooltip
                content={
                  canSendFromPool
                    ? "Send ZEC to another address"
                    : "Sapling cannot send — shield first"
                }
              >
                <Button
                  onClick={onSendClick}
                  className="gap-2"
                  disabled={!canSendFromPool}
                >
                  <ArrowRightUp size={18} /> Send
                </Button>
              </Tooltip>
            </>
          }
        />
        </div>

        <div className="relative z-10">
        <FeatureShell tone="platinum">
          <div className="flex flex-wrap gap-2">
            {poolTabs.map((tab) => {
              const active = poolTab === tab.id;
              return (
                <button
                  key={tab.id}
                  type="button"
                  onClick={() => setPoolTab(tab.id)}
                  className={`rounded-full border px-3.5 py-1.5 text-[0.7rem] font-bold uppercase tracking-[0.16em] transition ${
                    active
                      ? "border-emerald-400/60 bg-emerald-500/20 text-emerald-200"
                      : "border-emerald-500/25 bg-black/30 text-emerald-300/70 hover:border-emerald-400/45 hover:text-emerald-100"
                  }`}
                >
                  {tab.label}
                </button>
              );
            })}
          </div>

          <div className="mt-5 flex flex-col gap-5 lg:flex-row lg:items-start lg:justify-between">
            <div className="min-w-0 space-y-3">
              <div className="flex flex-wrap items-center gap-2">
                <span className="rounded-full border border-emerald-400/45 bg-emerald-500/15 px-3 py-1 text-[0.65rem] font-bold uppercase tracking-[0.22em] text-emerald-200">
                  {poolTab === "total" ? "Shielded" : poolTab}
                </span>
                <span className="rounded-full border border-emerald-500/30 bg-black/35 px-3 py-1 text-[0.65rem] font-bold uppercase tracking-[0.16em] text-emerald-300/80">
                  {symbol}
                </span>
              </div>
              <div>
                <div className="flex items-center gap-3">
                  <h3 className="text-2xl font-extrabold tracking-tight text-primary-100">
                    {poolTab === "total"
                      ? "Total balance"
                      : `${poolTab.charAt(0).toUpperCase()}${poolTab.slice(1)} balance`}
                  </h3>
                  <button
                    type="button"
                    onClick={() => setShowBalance(!showBalance)}
                    className="text-emerald-300/70 transition-colors hover:text-emerald-100"
                    aria-label={showBalance ? "Hide balance" : "Show balance"}
                  >
                    {showBalance ? <Eye size={16} /> : <EyeClosed size={16} />}
                  </button>
                </div>
                <div className="mt-3 flex items-end gap-2">
                  <span className="text-4xl font-extrabold tracking-tight text-primary-100 tabular-nums sm:text-5xl">
                    {balanceDisplay}
                  </span>
                  <span className="mb-1 text-xl font-medium uppercase text-emerald-300/70">
                    {symbol}
                  </span>
                </div>
                {balanceFiatLine ? (
                  <p className="mt-1 text-sm font-medium text-emerald-200/65">{balanceFiatLine}</p>
                ) : null}
                <p className="mt-2 text-xs text-emerald-200/55">{poolHint}</p>
              </div>
            </div>
          </div>

          {/* Zcash logo + live unit price */}
          <div className="mt-4 flex items-center justify-between gap-4 rounded-2xl border border-emerald-500/25 bg-black/35 p-4 shadow-lg shadow-emerald-950/30 backdrop-blur-md">
            <div className="flex min-w-0 items-center gap-4">
              <div className="h-12 w-12 shrink-0 overflow-hidden rounded-full border border-emerald-400/30 shadow-md shadow-emerald-950/40">
                <img
                  src={activeToken?.icon ?? "/zec.svg"}
                  alt="Zcash logo"
                  className="h-full w-full object-cover"
                />
              </div>
              <div className="min-w-0">
                <p className="font-bold uppercase tracking-wide text-primary-100">
                  {activeToken?.name ?? "Zcash"}
                </p>
                <p className="text-sm font-medium text-emerald-200/70">
                  {unitPriceLine ?? `${symbol} · price unavailable`}
                </p>
              </div>
            </div>
            <div className="shrink-0 text-right">
              <p className="font-bold tabular-nums text-primary-100">
                {showBalance
                  ? `${formatZec(selectedPoolZec, true)} ${symbol}`
                  : "••••••"}
              </p>
              <p className="mt-0.5 text-sm font-semibold tabular-nums text-emerald-200/70">
                {showBalance ? balanceFiatLine ?? "Price unavailable" : "••••"}
              </p>
            </div>
          </div>

          <div className="mt-4 grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
            <MetricTile
              tone="platinum"
              label="Orchard"
              value={showBalance ? `${formatZec(orchardZec, true)} ${symbol}` : "••••"}
              accent={orchardZec > 0 ? "emerald" : "default"}
              hint={ironwoodActive ? "Migrate to Ironwood" : "Shielded"}
            />
            <MetricTile
              tone="platinum"
              label="Ironwood"
              value={showBalance ? `${formatZec(ironwoodZec, true)} ${symbol}` : "••••"}
              accent={ironwoodZec > 0 ? "emerald" : "default"}
              hint={ironwoodSendEnabled ? "Send-ready pool" : "After activation"}
            />
            {hasSapling ? (
              <MetricTile
                tone="platinum"
                label="Sapling"
                value={showBalance ? `${formatZec(saplingZec, true)} ${symbol}` : "••••"}
                accent={saplingZec > 0 ? "amber" : "default"}
                hint="Shield into Orchard/Ironwood"
              />
            ) : null}
          </div>

          {sendBlockedReason && poolTab !== "total" ? (
            <div className="mt-4 rounded-2xl border border-amber-400/40 bg-amber-500/10 px-4 py-3 text-sm text-amber-100/90">
              {sendBlockedReason}
            </div>
          ) : null}

          {hasSapling && poolTab === "sapling" ? (
            <div className="mt-4 rounded-2xl border border-emerald-400/40 bg-emerald-500/10 px-4 py-3 text-sm text-emerald-100/90">
              <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                <p className="leading-relaxed">
                  <span className="font-semibold">Sapling · </span>
                  {showBalance
                    ? `${saplingZec.toFixed(8)} ZEC — cannot send out. Shield into Orchard/Ironwood (fee ~${saplingFeeZec.toFixed(8)} ZEC).`
                    : "Funds in Sapling cannot be sent out — shield them first."}
                </p>
                <Button
                  variant="outline"
                  className="shrink-0"
                  disabled={legacyBusy}
                  onClick={() => void handleMoveLegacy()}
                >
                  {legacyBusy ? "Shielding…" : "Shield Sapling"}
                </Button>
              </div>
            </div>
          ) : null}

          <div className="mt-4 grid gap-3 sm:grid-cols-2">
            <div className="flex min-h-[7.5rem] flex-col rounded-2xl border border-emerald-500/25 bg-black/35 p-4 shadow-lg shadow-emerald-950/30 backdrop-blur-md">
              <p className="text-[0.65rem] font-bold uppercase tracking-[0.2em] text-emerald-300/90">
                Receive address
              </p>
              <p className="mt-1 text-xs text-emerald-200/55">Shielded unified · tap to copy</p>
              <Tooltip content="Copy address">
                <button
                  type="button"
                  onClick={handleCopy}
                  className="mt-3 flex min-h-[2.75rem] w-full flex-1 items-center gap-2 rounded-xl border border-emerald-500/30 bg-black/40 px-3 py-2.5 text-left text-sm text-primary-100 transition hover:border-emerald-400/50"
                >
                  <span className="min-w-0 flex-1 break-all font-mono text-xs leading-relaxed">
                    {displayAddress}
                  </span>
                  {copied ? (
                    <CheckCircle size={14} className="shrink-0 text-emerald-300" />
                  ) : (
                    <Copy size={14} className="shrink-0 opacity-60" />
                  )}
                </button>
              </Tooltip>
            </div>
            <div className="flex min-h-[7.5rem] flex-col rounded-2xl border border-emerald-500/25 bg-black/35 p-4 shadow-lg shadow-emerald-950/30 backdrop-blur-md">
              <p className="text-[0.65rem] font-bold uppercase tracking-[0.2em] text-emerald-300/90">
                Chain sync
              </p>
              <p className="mt-1 text-xs text-emerald-200/55">Node tip and scan progress</p>
              <div className="mt-3 flex-1 [&_.mt-3]:mt-0 [&_.mb-2]:mb-0">
                <BlockSyncPanel />
              </div>
            </div>
          </div>
        </FeatureShell>
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
