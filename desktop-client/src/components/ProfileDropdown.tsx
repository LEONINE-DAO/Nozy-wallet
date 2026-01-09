import { useState, useRef, useEffect } from "react";
import { Settings, User } from "@solar-icons/react";
import { useQuery } from "@tanstack/react-query";
import { useWalletStore } from "../store/walletStore";
import { walletApi } from "../lib/api";

interface ProfileDropdownProps {
  onNavigate: (path: "settings") => void;
}

export function ProfileDropdown({ onNavigate }: ProfileDropdownProps) {
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const { address } = useWalletStore();

  const displayAddress = address
    ? `${address.slice(0, 6)}...${address.slice(-4)}`
    : "No Wallet";

  // Poll health every 2 minutes
  const { data: healthData, isError } = useQuery({
    queryKey: ["walletHealth"],
    queryFn: async () => {
      const res = await walletApi.checkHealth();
      return res.data;
    },
    refetchInterval: 120000, // 2 minutes
    staleTime: 60000,
    retry: false,
  });

  const isSynced = !isError && healthData;

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
      }
    }

    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, []);

  return (
    <div
      className="relative"
      ref={dropdownRef}
    >
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="flex items-center gap-3 px-3 py-2 rounded-xl bg-white/60 border border-white/50 shadow-sm backdrop-blur-sm hover:bg-white/80 transition-all cursor-pointer text-left group"
      >
        <div className="w-9 h-9 rounded-full bg-linear-to-br from-primary-100 to-white flex items-center justify-center text-primary-600 shadow-inner border border-white">
          <User
            size={20}
            weight="Bold"
          />
        </div>
        <div className="flex flex-col">
          <span className="text-xs font-bold text-gray-800 group-hover:text-primary transition-colors uppercase tracking-wide">
            My Wallet
          </span>
          <span className="text-xs text-gray-500 font-mono">
            {displayAddress}
          </span>
        </div>
      </button>

      {isOpen && (
        <div className="absolute right-0 top-full mt-2 w-48 rounded-xl bg-white/90 backdrop-blur-md border border-white/50 shadow-xl py-2 z-50 animate-fade-in">
          <div className="px-4 py-2 border-b border-gray-100 mb-1">
            <p className="text-xs font-semibold text-gray-500 uppercase tracking-wider">
              Profile
            </p>
          </div>
          <button
            onClick={() => {
              onNavigate("settings");
              setIsOpen(false);
            }}
            className="w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-primary-50 hover:text-primary transition-colors flex items-center gap-2"
          >
            <Settings size={18} />
            Settings
          </button>
          <div className="w-full px-4 py-2 flex items-center gap-3 bg-gray-50/50 mt-1">
            <div
              className={`w-2.5 h-2.5 rounded-full shadow-lg transition-colors duration-500 ${
                isSynced
                  ? "bg-green-500 shadow-green-500/50 animate-pulse"
                  : "bg-red-500 shadow-red-500/50"
              }`}
            />
            <div className="flex flex-col">
              <span className="text-sm font-medium text-gray-900">Mainnet</span>
              <span className="text-xs text-gray-500">
                {isSynced ? "Synced" : "Disconnected"}
              </span>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
