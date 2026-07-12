import React from "react";
import { Home, History, Refresh, Shield } from "@solar-icons/react";
import { ProfileDropdown } from "./ProfileDropdown";
import { Tooltip } from "./Tooltip";
import { runWalletSyncWithFeedback } from "../lib/walletSyncUi";
import { useWalletStore } from "../store/walletStore";
import { dappBrowserEnabled, webWatchOnlyEnabled } from "../lib/featureFlags";
import { cn } from "../lib/cn";

export type TabId =
  | "home"
  | "history"
  | "ironwood"
  | "settings"
  | "send"
  | "browser"
  | "contacts"
  | "web";

interface HeaderProps {
  activeTab: TabId;
  onTabChange: (tab: TabId) => void;
  showLabels: boolean;
}

export function Header({ activeTab, onTabChange, showLabels }: HeaderProps) {
  const { isSyncing, setIsSyncing, setBalanceFromAvailable, syncProgressPercent } =
    useWalletStore();

  const handleManualSync = async () => {
    if (isSyncing) return;
    await runWalletSyncWithFeedback({
      setIsSyncing,
      onBalance: (available) => setBalanceFromAvailable(available),
    });
  };

  const syncButtonLabel = isSyncing
    ? syncProgressPercent != null
      ? `${syncProgressPercent}%`
      : "Syncing…"
    : "Sync";

  return (
    <header className="w-full shrink-0 z-20 border-b border-gray-800 bg-gray-950 shadow-sm">
      <div className="container mx-auto px-8 py-3 flex items-center justify-between gap-6">
        <div className="flex items-center gap-6 min-w-0">
          <div className="flex items-center shrink-0">
            <img
              src="/logo.png"
              alt="Nozy Wallet"
              className="w-auto h-10 object-contain"
              onError={(e) => {
                e.currentTarget.style.display = "none";
              }}
            />
          </div>
          <nav className="flex items-center gap-2" aria-label="Primary">
            <Tooltip content="View balance and recent activity" placement="bottom">
              <div>
                <HeaderItem
                  icon={<Home weight="Bold" />}
                  label="Home"
                  showLabel
                  active={activeTab === "home"}
                  onClick={() => onTabChange("home")}
                />
              </div>
            </Tooltip>
            <Tooltip content="View and search transaction history" placement="bottom">
              <div>
                <HeaderItem
                  icon={<History weight="Bold" />}
                  label="History"
                  showLabel
                  active={activeTab === "history"}
                  onClick={() => onTabChange("history")}
                />
              </div>
            </Tooltip>
            <Tooltip content="Ironwood / NU6.3 migration readiness" placement="bottom">
              <div>
                <HeaderItem
                  icon={<Shield weight="Bold" />}
                  label="Ironwood"
                  showLabel
                  active={activeTab === "ironwood"}
                  onClick={() => onTabChange("ironwood")}
                />
              </div>
            </Tooltip>
            {dappBrowserEnabled && (
              <Tooltip content="Browse Zcash dApps" placement="bottom">
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
            )}
            {webWatchOnlyEnabled && (
              <Tooltip content="Watch-only web wallet status" placement="bottom">
                <div>
                  <HeaderItem
                    icon={<Shield weight="Bold" />}
                    label="Web"
                    showLabel={showLabels}
                    active={activeTab === "web"}
                    onClick={() => onTabChange("web")}
                  />
                </div>
              </Tooltip>
            )}
          </nav>
        </div>

        <div className="flex items-center gap-2 shrink-0">
          <Tooltip
            placement="bottom"
            content={
              isSyncing
                ? syncProgressPercent != null
                  ? `Syncing — ${syncProgressPercent}% complete`
                  : "Syncing wallet with the network…"
                : "Sync wallet with the network"
            }
          >
            <button
              className={cn(
                "flex items-center gap-2 px-3.5 py-2.5 rounded-xl font-semibold text-sm border transition-all active:scale-95 disabled:opacity-50 disabled:cursor-wait min-w-[5rem] justify-center",
                isSyncing
                  ? "bg-primary/15 text-primary border-primary/40"
                  : "text-gray-100 bg-gray-800 border-gray-600 hover:bg-primary/15 hover:border-primary/40"
              )}
              title="Sync Wallet"
              onClick={handleManualSync}
              disabled={isSyncing}
            >
              <Refresh
                size={18}
                className={cn("shrink-0 transition-transform", isSyncing && "animate-spin")}
              />
              <span className="tabular-nums">{syncButtonLabel}</span>
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
      aria-current={active ? "page" : undefined}
      className={cn(
        "flex items-center gap-2 px-4 py-2.5 rounded-xl transition-all duration-200 font-semibold text-sm border",
        active
          ? "bg-primary text-gray-950 border-primary shadow-md shadow-primary/25"
          : "text-gray-100 bg-gray-800 border-gray-600 hover:bg-primary/20 hover:border-primary/50"
      )}
    >
      {React.cloneElement(icon as React.ReactElement, {
        size: 20,
        weight: "Bold",
      })}
      {showLabel && <span>{label}</span>}
    </button>
  );
}
