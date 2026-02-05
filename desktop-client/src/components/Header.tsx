import React from "react";
import { Home, History, Refresh, Shield } from "@solar-icons/react";
import toast from "react-hot-toast";
import { formatErrorForDisplay } from "../utils/errors";
import { ProfileDropdown } from "./ProfileDropdown";
import { Tooltip } from "./Tooltip";
import { walletApi } from "../lib/api";
import { useWalletStore } from "../store/walletStore";

export type TabId = "home" | "history" | "settings" | "send" | "browser" | "contacts";

interface HeaderProps {
  activeTab: TabId;
  onTabChange: (tab: TabId) => void;
  showLabels: boolean;
}

function cn(...classes: (string | boolean | undefined)[]) {
  return classes.filter(Boolean).join(" ");
}

export function Header({ activeTab, onTabChange, showLabels }: HeaderProps) {
  const { isSyncing, setIsSyncing } = useWalletStore();

  const handleManualSync = async () => {
    if (isSyncing) return;
    const syncToast = toast.loading("Syncing wallet...");
    try {
      setIsSyncing(true);
      await walletApi.syncWallet();
      toast.success("Wallet synced successfully!", { id: syncToast });
    } catch (error) {
      toast.error(formatErrorForDisplay(error, "Sync failed. Please try again."), { id: syncToast });
    } finally {
      setIsSyncing(false);
    }
  };

  return (
    <header className="w-full backdrop-blur-xl z-20">
      <div className="container mx-auto px-6 py-4 flex items-center justify-between">
        <div className="flex items-center gap-10">
          <div className="flex items-center gap-3">
            <img
              src="/logo.png"
              alt="Nozy Wallet"
              className="w-auto h-24 object-contain drop-shadow-md"
              onError={(e) => {
                e.currentTarget.style.display = "none";
              }}
            />
          </div>
          <nav className="flex items-center gap-2">
            <Tooltip content="View balance and recent activity">
              <div>
                <HeaderItem
                  icon={<Home weight="Bold" />}
                  label="Home"
                  showLabel={showLabels}
                  active={activeTab === "home"}
                  onClick={() => onTabChange("home")}
                />
              </div>
            </Tooltip>
            <Tooltip content="View and search transaction history">
              <div>
                <HeaderItem
                  icon={<History weight="Bold" />}
                  label="History"
                  showLabel={showLabels}
                  active={activeTab === "history"}
                  onClick={() => onTabChange("history")}
                />
              </div>
            </Tooltip>
            <Tooltip content="Browse Zcash dApps">
              <div>
                <HeaderItem
                  icon={<Shield weight="Bold" />}
                  label="Browser"
                  showLabel={showLabels}
                  active={activeTab === "browser"}
                  onClick={() => onTabChange("browser")}
                />
              </div>
            </Tooltip>
          </nav>
        </div>

        <div className="flex items-center gap-3">
          <Tooltip content={isSyncing ? "Syncing wallet with the network…" : "Sync wallet with the network"}>
            <button
              className={cn(
                "flex items-center gap-2 px-3 py-2.5 rounded-xl text-gray-500 hover:text-primary hover:bg-white/60 hover:shadow-sm border border-transparent hover:border-white/50 transition-all active:scale-95 disabled:opacity-50 disabled:cursor-wait",
                isSyncing && "bg-primary/10 text-primary"
              )}
              title="Sync Wallet"
              onClick={handleManualSync}
              disabled={isSyncing}
            >
              <Refresh
                size={20}
                className={cn("shrink-0 transition-transform", isSyncing && "animate-spin")}
              />
              {(showLabels || isSyncing) && (
                <span className="text-sm font-medium">
                  {isSyncing ? "Syncing…" : "Sync"}
                </span>
              )}
            </button>
          </Tooltip>
          <ProfileDropdown onNavigate={(path) => onTabChange(path)} />
        </div>
      </div>
    </header>
  );
}

function HeaderItem({
  icon,
  label,
  showLabel,
  active,
  onClick,
}: {
  icon: React.ReactNode;
  label: string;
  showLabel: boolean;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "flex items-center gap-2 px-4 py-2.5 rounded-xl transition-all duration-200 font-medium text-sm group",
        active
          ? "bg-[#f0a113] text-black shadow-lg shadow-gray-200/25"
          : "text-gray-700 hover:bg-white/60 hover:shadow-sm hover:text-primary-700"
      )}
    >
      <div
        className={cn(
          "transition-transform duration-200",
          active ? "scale-110" : "group-hover:scale-110"
        )}
      >
        {React.cloneElement(icon as React.ReactElement, {
          size: 20,
          weight: active ? "Bold" : "Linear",
        })}
      </div>
      {showLabel && <span>{label}</span>}
    </button>
  );
}
