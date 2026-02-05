import { useState, useEffect } from "react";
import { Header, TabId } from "../components/Header";
import { HomePage } from "../pages/Home";
import { SendPage } from "../pages/Send";
import { SettingsPage } from "../pages/Settings";
import { HistoryPage } from "../pages/History";
import { BrowserPage } from "../pages/Browser";
import { ContactsPage } from "../pages/Contacts";
import { walletApi } from "../lib/api";
import { useWalletStore } from "../store/walletStore";
import { useSettingsStore } from "../store/settingsStore";
import { Button } from "../components/Button";
import toast from "react-hot-toast";
import { formatErrorForDisplay } from "../utils/errors";
import { Refresh, CloseCircle, Download } from "@solar-icons/react";

export function AuthenticatedLayout() {
  const [activeTab, setActiveTab] = useState<TabId>("home");
  const { showNavigationLabels, onboardingFirstSyncDismissed, setOnboardingFirstSyncDismissed } = useSettingsStore();
  const { setBalance, setAddress, isSyncing, setIsSyncing } = useWalletStore();
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
          const balanceVal =
            typeof balanceRes?.data === "number"
              ? balanceRes.data
              : balanceRes?.data?.balance;

          if (typeof balanceVal === "number") {
            setBalance(balanceVal);
          }
        } catch (e) {
        }
      } catch (error) {
      }
    };

    fetchData();
    const interval = setInterval(fetchData, 30000);
    return () => clearInterval(interval);
  }, [setBalance, setAddress]);

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

  const handleFirstSync = async () => {
    setIsSyncing(true);
    const syncToast = toast.loading("Syncing wallet...");
    try {
      await walletApi.syncWallet();
      toast.success("Wallet synced. Your balance and history are up to date.", { id: syncToast });
      setOnboardingFirstSyncDismissed(true);
      try {
        const balanceRes = await walletApi.getBalance();
        const balanceVal = typeof balanceRes?.data === "number" ? balanceRes.data : balanceRes?.data?.balance;
        if (typeof balanceVal === "number") setBalance(balanceVal);
      } catch {
        // ignore
      }
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Sync failed. You can try again from the header."), { id: syncToast });
    } finally {
      setIsSyncing(false);
    }
  };

  return (
    <div className="flex flex-col h-screen bg-gray-50 text-gray-900 font-sans overflow-hidden">
      <Header
        activeTab={activeTab}
        onTabChange={setActiveTab}
        showLabels={showNavigationLabels}
      />

      {!onboardingFirstSyncDismissed && (
        <div className="shrink-0 px-4 py-3 bg-primary/10 border-b border-primary/20 flex items-center justify-between gap-4 flex-wrap">
          <p className="text-sm text-gray-800 dark:text-gray-200 font-medium">
            Sync your wallet to see your balance and transaction history.
          </p>
          <div className="flex items-center gap-2">
            <Button
              size="sm"
              onClick={handleFirstSync}
              disabled={isSyncing}
              className="gap-2"
            >
              {isSyncing ? (
                <>
                  <Refresh size={16} className="animate-spin shrink-0" />
                  <span>Syncing…</span>
                </>
              ) : (
                <>
                  <Refresh size={16} />
                  Sync now
                </>
              )}
            </Button>
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
              className="gap-2 bg-amber-600 hover:bg-amber-700 text-white border-amber-700"
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

      <main className="flex-1 overflow-hidden bg-gray-50 dark:bg-gray-900 relative">
        {activeTab === "browser" ? (
          <BrowserPage />
        ) : (
          <>
            <div className="absolute top-0 left-0 w-full h-64 bg-linear-to-b from-primary-50/70 to-transparent pointer-events-none" />
            <div className="container mx-auto px-10 py-10 relative z-10 overflow-y-auto h-full">
              {activeTab === "home" && <HomePage onNavigate={setActiveTab} />}
              {activeTab === "history" && <HistoryPage />}
              {activeTab === "send" && <SendPage />}
              {activeTab === "settings" && <SettingsPage />}
              {activeTab === "contacts" && <ContactsPage />}
            </div>
          </>
        )}
      </main>
    </div>
  );
}
