import { useState, useEffect } from "react";
import { Header, TabId } from "../components/Header";
import { HomePage } from "../pages/Home";
import { SendPage } from "../pages/Send";
import { SettingsPage } from "../pages/Settings";
import { HistoryPage } from "../pages/History";
import { walletApi } from "../lib/api";
import { useWalletStore } from "../store/walletStore";
import { useSettingsStore } from "../store/settingsStore";

export function AuthenticatedLayout() {
  const [activeTab, setActiveTab] = useState<TabId>("home");
  const { showNavigationLabels } = useSettingsStore();
  const { setBalance, setAddress } = useWalletStore();

  // Sync wallet data - only if wallet is unlocked
  useEffect(() => {
    const fetchData = async () => {
      try {
        // Check wallet status first
        const statusRes = await walletApi.getWalletStatus();
        if (!statusRes?.data?.unlocked) {
          // Wallet not unlocked, don't try to fetch data
          return;
        }

        // Only fetch address and balance if wallet is unlocked
        try {
          const addressRes = await walletApi.generateAddress();
          if (addressRes?.data?.address) {
            setAddress(addressRes.data.address);
          }
        } catch (e) {
          // Address generation failed, but continue with balance
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
          // Balance fetch failed
        }
      } catch (error) {
        // Silently fail - wallet might not be unlocked yet
      }
    };

    fetchData();
    const interval = setInterval(fetchData, 30000); // Sync every 30s
    return () => clearInterval(interval);
  }, [setBalance, setAddress]);

  return (
    <div className="flex flex-col h-screen bg-gray-50 text-gray-900 font-sans overflow-hidden">
      <Header
        activeTab={activeTab}
        onTabChange={setActiveTab}
        showLabels={showNavigationLabels}
      />

      <main className="flex-1 overflow-y-auto bg-gray-50 relative">
        <div className="absolute top-0 left-0 w-full h-64 bg-linear-to-b from-primary-50/70 to-transparent pointer-events-none" />
        <div className="container mx-auto px-10 py-10 relative z-10">
          {activeTab === "home" && <HomePage onNavigate={setActiveTab} />}
          {activeTab === "history" && <HistoryPage />}
          {activeTab === "send" && <SendPage />}
          {activeTab === "settings" && <SettingsPage />}
        </div>
      </main>
    </div>
  );
}
