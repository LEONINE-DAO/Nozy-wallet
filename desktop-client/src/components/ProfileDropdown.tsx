import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Settings, Shield, User } from "@solar-icons/react";
import { useQuery } from "@tanstack/react-query";
import { useWalletStore } from "../store/walletStore";
import { useSettingsStore } from "../store/settingsStore";
import { walletApi } from "../lib/api";
import type { WalletProfileInfo } from "../lib/types";
import { cn } from "../lib/cn";

interface ProfileDropdownProps {
  onNavigate: (path: "settings" | "contacts") => void;
}

function shortAddress(address: string | null | undefined): string {
  if (!address) return "No address";
  if (address.length <= 16) return address;
  return `${address.slice(0, 8)}…${address.slice(-6)}`;
}

/**
 * Profile menu: show the unlocked wallet only.
 * Never one-click switch profiles (that locks the session and looks like funds vanished).
 * Create / switch wallets happens in Settings → Wallets & Accounts.
 */
export function ProfileDropdown({ onNavigate }: ProfileDropdownProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [profiles, setProfiles] = useState<WalletProfileInfo[]>([]);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const { address } = useWalletStore();
  const { accountLabels, activeAccountId, requestSettingsSection } = useSettingsStore();

  const activeAccountLabel =
    accountLabels[activeAccountId] ?? (activeAccountId === "0" ? "Default" : `Account ${activeAccountId}`);

  /** Only wallets that actually exist on disk (created/restored in Nozy). */
  const createdWallets = useMemo(
    () => profiles.filter((profile) => profile.has_wallet),
    [profiles]
  );

  const activeProfile = useMemo(
    () => createdWallets.find((p) => p.is_active) ?? profiles.find((p) => p.is_active) ?? null,
    [createdWallets, profiles]
  );

  const otherCreatedWallets = useMemo(
    () => createdWallets.filter((p) => !p.is_active),
    [createdWallets]
  );

  const { data: healthData, isError } = useQuery({
    queryKey: ["walletHealth"],
    queryFn: async () => {
      const res = await walletApi.checkHealth();
      return res.data;
    },
    refetchInterval: 120000,
    staleTime: 60000,
    retry: false,
  });

  const isSynced = !isError && healthData;
  const networkLabel = activeProfile?.network ?? "mainnet";

  const loadProfiles = useCallback(async () => {
    try {
      const res = await walletApi.getNetworkWalletStatus();
      setProfiles(res.data.profiles ?? []);
    } catch {
      setProfiles([]);
    }
  }, []);

  useEffect(() => {
    if (isOpen) void loadProfiles();
  }, [isOpen, loadProfiles]);

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const openWalletsSettings = () => {
    requestSettingsSection("accounts");
    onNavigate("settings");
    setIsOpen(false);
  };

  return (
    <div className="relative" ref={dropdownRef}>
      <button
        type="button"
        onClick={() => setIsOpen(!isOpen)}
        aria-expanded={isOpen}
        aria-haspopup="menu"
        title={`${activeAccountLabel} · ${shortAddress(address)}`}
        className={cn(
          "flex h-10 w-10 items-center justify-center rounded-xl border transition-all",
          "bg-gray-800 border-gray-600 text-gray-100 hover:bg-primary/15 hover:border-primary/40",
          isOpen && "border-primary/50 bg-primary/15 text-primary"
        )}
      >
        <User size={20} weight="Bold" />
      </button>

      {isOpen && (
        <div
          role="menu"
          className="absolute right-0 top-full z-50 mt-2 w-72 animate-fade-in overflow-hidden rounded-xl border border-gray-700/60 bg-gray-900/95 py-2 shadow-xl backdrop-blur-md"
        >
          <div className="border-b border-gray-800 px-4 py-3">
            <p className="text-[0.65rem] font-bold uppercase tracking-[0.18em] text-gray-500">
              This session
            </p>
            <p className="mt-1 truncate text-sm font-semibold text-primary-100">
              {activeProfile?.name ?? activeAccountLabel}
            </p>
            <p className="mt-0.5 truncate font-mono text-xs text-gray-400" title={address ?? undefined}>
              {shortAddress(address)}
            </p>
            <p className="mt-1 text-[0.65rem] uppercase tracking-wide text-gray-500">
              {networkLabel} · unlocked
            </p>
          </div>

          {otherCreatedWallets.length > 0 && (
            <div className="border-b border-gray-800 px-4 py-3">
              <p className="text-[0.65rem] font-bold uppercase tracking-[0.16em] text-gray-500">
                Other wallets on this device
              </p>
              <ul className="mt-2 space-y-1.5">
                {otherCreatedWallets.map((profile) => (
                  <li key={profile.id} className="text-sm text-gray-300">
                    <span className="font-medium text-gray-100">{profile.name}</span>
                    <span className="ml-1.5 text-[0.65rem] uppercase text-gray-500">
                      {profile.network}
                    </span>
                  </li>
                ))}
              </ul>
              <p className="mt-2 text-[11px] leading-relaxed text-gray-500">
                Switching wallets needs Settings — your current funds stay on this device; you unlock
                the other wallet with its password.
              </p>
            </div>
          )}

          <button
            type="button"
            role="menuitem"
            onClick={openWalletsSettings}
            className="flex w-full items-center gap-2 px-4 py-2.5 text-left text-sm text-gray-300 transition-colors hover:bg-primary/10 hover:text-primary"
          >
            <Shield size={18} />
            <span className="min-w-0">
              <span className="block font-medium">Create or manage wallets</span>
              <span className="block text-[11px] text-gray-500">
                Only lists wallets you created in Nozy
              </span>
            </span>
          </button>

          <button
            type="button"
            role="menuitem"
            onClick={() => {
              onNavigate("contacts");
              setIsOpen(false);
            }}
            className="flex w-full items-center gap-2 px-4 py-2 text-left text-sm text-gray-300 transition-colors hover:bg-primary/10 hover:text-primary"
          >
            <User size={18} />
            Contacts
          </button>
          <button
            type="button"
            role="menuitem"
            onClick={() => {
              onNavigate("settings");
              setIsOpen(false);
            }}
            className="flex w-full items-center gap-2 px-4 py-2 text-left text-sm text-gray-300 transition-colors hover:bg-primary/10 hover:text-primary"
          >
            <Settings size={18} />
            Settings
          </button>

          <div className="mt-1 flex items-center gap-3 bg-gray-800/50 px-4 py-2.5">
            <div
              className={cn(
                "h-2.5 w-2.5 rounded-full shadow-lg transition-colors duration-500",
                isSynced
                  ? "bg-green-500 shadow-green-500/50 animate-pulse"
                  : "bg-red-500 shadow-red-500/50"
              )}
            />
            <div className="flex flex-col">
              <span className="text-sm font-medium capitalize text-gray-100">{networkLabel}</span>
              <span className="text-xs text-gray-500">{isSynced ? "Synced" : "Disconnected"}</span>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
