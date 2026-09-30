import { useState, useEffect } from "react";
import { Header, TabId } from "../components/Header";
import { SyncStatusBanner } from "../components/SyncStatusBanner";
import { HomePage } from "../pages/Home";
import { SendPage } from "../pages/Send";
import { SettingsPage } from "../pages/Settings";
import { HistoryPage } from "../pages/History";
import { IronwoodPage } from "../pages/Ironwood";
import { VotePage } from "../pages/Vote";
import { CrosslinkPage } from "../pages/Crosslink";
import { BrowserPage } from "../pages/Browser";
import { ContactsPage } from "../pages/Contacts";
import { WebWalletWatchOnlyPage } from "../pages/WebWalletWatchOnly";
import { BrowserSubscriptionGate } from "../components/BrowserSubscriptionGate";
import { walletApi } from "../lib/api";
import { useWalletStore } from "../store/walletStore";
import { useSettingsStore } from "../store/settingsStore";
import { useSubscriptionStore } from "../store/subscriptionStore";
import { Button } from "../components/Button";
import toast from "react-hot-toast";
import { formatErrorForDisplay } from "../utils/errors";
import { Refresh, CloseCircle, Download } from "@solar-icons/react";
import { useWalletAutoSync } from "../hooks/useWalletAutoSync";
import { balanceFromResponse } from "../lib/syncHelpers";
import { dappBrowserEnabled, webWatchOnlyEnabled, nu7VoteEnabled, crosslinkEnabled } from "../lib/featureFlags";
import { SyncControlButton } from "../components/SyncControlButton";

export function AuthenticatedLayout() {
  const [activeTab, setActiveTab] = useState<TabId>("home");
  const { showNavigationLabels, onboardingFirstSyncDismissed, setOnboardingFirstSyncDismissed } = useSettingsStore();
  const { hasNymSubscription } = useSubscriptionStore();
  const { setAddress, isSyncing, setBalanceFromAvailable } =
    useWalletStore();
  const [syncBannerToken, setSyncBannerToken] = useState(0);
  const [provingDownloaded, setProvingDownloaded] = useState<boolean | null>(null);
  const [provingDownloading, setProvingDownloading] = useState(false);
  const [provingBannerDismissed, setProvingBannerDismissed] = useState(false);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const statusRes = await walletApi.getWalletStatus();
        if (!statusRes?.data?.unlocked) {
          return;
        }

        try {
          const addressRes = await walletApi.generateAddress();
          if (addressRes?.data?.address) {
            setAddress(addressRes.data.address);
          }
        } catch (e) {
        }

        try {
          const balanceRes = await walletApi.getBalance();
          setBalanceFromAvailable(balanceFromResponse(balanceRes.data).available);
        } catch (e) {
        }
      } catch (error) {
      }
    };

    fetchData();
    const interval = setInterval(fetchData, 30000);
    return () => clearInterval(interval);
  }, [setBalanceFromAvailable, setAddress]);

  useEffect(() => {
    let cancelled = false;
    walletApi
      .getProvingStatus()
      .then((res) => {
        if (!cancelled && res?.data) setProvingDownloaded(res.data.downloaded);
      })
      .catch(() => {
        if (!cancelled) setProvingDownloaded(null);
      });
    return () => {
      cancelled = true;
    };
  }, [provingDownloading]);

  const handleDownloadProving = async () => {
    setProvingDownloading(true);
    const toastId = toast.loading("Downloading proving parameters…");
    try {
      await walletApi.downloadProvingParams();
      const res = await walletApi.getProvingStatus();
      setProvingDownloaded(res?.data?.downloaded ?? true);
      toast.success("Proving parameters ready. You can send transactions.", { id: toastId });
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Failed to download proving parameters."), { id: toastId });
    } finally {
      setProvingDownloading(false);
    }
  };

  useWalletAutoSync({
    onSyncComplete: () => {
      setSyncBannerToken((t) => t + 1);
      void walletApi.getBalance().then((res) => {
        setBalanceFromAvailable(balanceFromResponse(res.data).available);
      });
    },
  });

  return (
    <div className="flex flex-col h-screen bg-[#020b07] text-primary-100 font-sans overflow-hidden">
      <Header
        activeTab={activeTab}
        onTabChange={setActiveTab}
        showLabels={showNavigationLabels}
      />

      <SyncStatusBanner
        isSyncing={isSyncing}
        refreshToken={syncBannerToken}
      />

      {!onboardingFirstSyncDismissed && (
        <div className="shrink-0 px-4 py-3 bg-emerald-500/10 border-b border-emerald-400/25 flex items-center justify-between gap-4 flex-wrap">
          <p className="text-sm text-emerald-100 font-medium">
            Sync your wallet to see your balance and transaction history.
          </p>
          <div className="flex items-center gap-2">
            <SyncControlButton />
            <button
              type="button"
              onClick={() => setOnboardingFirstSyncDismissed(true)}
              className="p-2 rounded-lg text-gray-500 hover:bg-white/50 hover:text-gray-700 dark:hover:text-gray-300 transition-colors"
              title="Dismiss"
              aria-label="Dismiss"
            >
              <CloseCircle size={20} />
            </button>
          </div>
        </div>
      )}

      {provingDownloaded === false && !provingBannerDismissed && (
        <div className="shrink-0 px-4 py-3 bg-amber-50 dark:bg-amber-900/20 border-b border-amber-200 dark:border-amber-800 flex items-center justify-between gap-4 flex-wrap">
          <p className="text-sm text-amber-900 dark:text-amber-200 font-medium">
            Proving parameters required for sending. Download once to enable transactions.
          </p>
          <div className="flex items-center gap-2">
            <Button
              size="sm"
              onClick={handleDownloadProving}
              disabled={provingDownloading}
              className="gap-2 bg-amber-600 hover:bg-amber-700 text-primary-100 border-amber-700"
            >
              {provingDownloading ? (
                <>
                  <Refresh size={16} className="animate-spin shrink-0" />
                  <span>Downloading…</span>
                </>
              ) : (
                <>
                  <Download size={16} />
                  Download
                </>
              )}
            </Button>
            <button
              type="button"
              onClick={() => setProvingBannerDismissed(true)}
              className="p-2 rounded-lg text-amber-700 dark:text-amber-300 hover:bg-amber-100 dark:hover:bg-amber-800/50 transition-colors"
              title="Dismiss"
              aria-label="Dismiss"
            >
              <CloseCircle size={20} />
            </button>
          </div>
        </div>
      )}

      <main className="flex-1 min-h-0 overflow-hidden bg-[#020b07] relative flex flex-col">
        {activeTab === "browser" && dappBrowserEnabled ? (
          hasNymSubscription ? (
            <BrowserPage />
          ) : (
            <BrowserSubscriptionGate onGoToSettings={() => setActiveTab("settings")} />
          )
        ) : (
          <>
            <div
              aria-hidden
              className="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_at_20%_0%,_rgba(0,255,140,0.16),_transparent_50%),radial-gradient(ellipse_at_100%_80%,_rgba(0,180,90,0.12),_transparent_45%),linear-gradient(180deg,_rgba(0,40,24,0.55)_0%,_transparent_42%)]"
            />
            <div className="relative z-10 flex-1 min-h-0 overflow-y-auto">
              <div className="container mx-auto px-8 pt-8 pb-24 max-w-6xl">
                {activeTab === "home" && <HomePage onNavigate={setActiveTab} />}
                {activeTab === "history" && <HistoryPage />}
                {activeTab === "ironwood" && <IronwoodPage />}
                {activeTab === "vote" &&
                  (nu7VoteEnabled ? (
                    <VotePage onNavigate={setActiveTab} />
                  ) : (
                    <div className="max-w-2xl mx-auto py-8">
                      <h2 className="text-2xl font-bold text-gray-100 mb-2">
                        Vote tab disabled
                      </h2>
                      <p className="text-sm text-gray-400">
                        Set `VITE_ENABLE_NU7_VOTE` unset or not `false` to enable.
                      </p>
                    </div>
                  ))}
                {activeTab === "crosslink" &&
                  (crosslinkEnabled ? (
                    <CrosslinkPage />
                  ) : (
                    <div className="max-w-2xl mx-auto py-8">
                      <h2 className="text-2xl font-bold text-gray-100 mb-2">
                        Crosslink tab disabled
                      </h2>
                      <p className="text-sm text-gray-400">
                        Set `VITE_ENABLE_CROSSLINK` unset or not `false` to enable.
                      </p>
                    </div>
                  ))}
                {activeTab === "send" && <SendPage />}
                {activeTab === "settings" && <SettingsPage />}
                {activeTab === "contacts" && <ContactsPage />}
                {activeTab === "web" &&
                  (webWatchOnlyEnabled ? (
                    <WebWalletWatchOnlyPage />
                  ) : (
                    <div className="max-w-2xl mx-auto py-8">
                      <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2">
                        Web Watch-Only Disabled
                      </h2>
                      <p className="text-sm text-gray-600 dark:text-gray-400">
                        Set `VITE_ENABLE_WEB_WATCH_ONLY=true` to enable the Phase 1 web watch-only panel.
                      </p>
                    </div>
                  ))}
              </div>
            </div>
          </>
        )}
      </main>
    </div>
  );
}
