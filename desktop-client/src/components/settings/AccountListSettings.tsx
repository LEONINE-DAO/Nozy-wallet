import { useState } from "react";
import { Button } from "../Button";
import { Input } from "../Input";
import { CheckCircle, User } from "@solar-icons/react";
import { useSettingsStore } from "../../store/settingsStore";
import { Modal } from "../Modal";
import toast from "react-hot-toast";
import { NetworkWalletSwitcher } from "../NetworkWalletSwitcher";
import { SettingsBackButton } from "./SettingsBackButton";

const DEFAULT_ACCOUNT_IDS = ["0"];

interface AccountListSettingsProps {
  onBack: () => void;
}

export function AccountListSettings({ onBack }: AccountListSettingsProps) {
  const { accountLabels, activeAccountId, setAccountLabel, setActiveAccountId } = useSettingsStore();
  const [renameId, setRenameId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState("");

  const accountIds = DEFAULT_ACCOUNT_IDS;

  const startRename = (id: string) => {
    setRenameId(id);
    setRenameValue(accountLabels[id] ?? id);
  };

  const saveRename = () => {
    if (renameId != null && renameValue.trim()) {
      setAccountLabel(renameId, renameValue.trim());
      toast.success("Account renamed");
      setRenameId(null);
      setRenameValue("");
    }
  };

  const cancelRename = () => {
    setRenameId(null);
    setRenameValue("");
  };

  return (
    <div className="max-w-2xl mx-auto animate-fade-in">
      <SettingsBackButton onClick={onBack} />

      <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2">
        Wallets & Accounts
      </h2>
      <p className="text-sm text-gray-500 dark:text-gray-400 mb-6">
        Add wallets, change the active wallet profile, and manage accounts after you unlock.
      </p>

      <div className="space-y-6">
        <NetworkWalletSwitcher />

        <div className="space-y-3">
          <div>
            <h3 className="text-sm font-semibold uppercase tracking-widest text-gray-500 dark:text-gray-400">
              In-wallet accounts
            </h3>
            <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
              These account labels stay inside the active wallet profile.
            </p>
          </div>

        {accountIds.map((id) => {
          const label = accountLabels[id] ?? id;
          const isActive = activeAccountId === id;
          return (
            <div
              key={id}
              className={`p-4 rounded-xl border flex items-center justify-between gap-4 ${
                isActive
                  ? "bg-primary/10 border-primary/30 dark:bg-primary/20 dark:border-primary/40"
                  : "bg-white/60 dark:bg-gray-800/60 border-white/50 dark:border-gray-700/50"
              }`}
            >
              <div className="flex items-center gap-3 min-w-0">
                <div className="w-10 h-10 rounded-full bg-primary/20 dark:bg-primary/30 flex items-center justify-center shrink-0">
                  <User size={20} className="text-primary-600 dark:text-primary-400" />
                </div>
                <div className="min-w-0">
                  <p className="font-medium text-gray-900 dark:text-gray-100 truncate">
                    {label}
                  </p>
                  <p className="text-xs text-gray-500 dark:text-gray-400">
                    Account {id}
                    {isActive && (
                      <span className="ml-2 text-primary-600 dark:text-primary-400">Active</span>
                    )}
                  </p>
                </div>
              </div>
              <div className="flex items-center gap-2 shrink-0">
                {!isActive && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setActiveAccountId(id)}
                    className="gap-1"
                  >
                    <CheckCircle size={16} />
                    Switch
                  </Button>
                )}
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => startRename(id)}
                  className="text-gray-600 dark:text-gray-400"
                >
                  Rename
                </Button>
              </div>
            </div>
          );
        })}

        <p className="text-sm text-gray-400 px-1">
          Additional accounts under one wallet profile are not available in v1.0.0.
          Use <span className="font-medium text-gray-300">Wallets &amp; Accounts</span> network
          profiles to manage separate mainnet/testnet wallets.
        </p>
        </div>
      </div>

      <Modal
        isOpen={renameId != null}
        onClose={cancelRename}
        title="Rename account"
      >
        <div className="space-y-4">
          <Input
            label="Account name"
            placeholder="e.g. Savings, Spending"
            value={renameValue}
            onChange={(e) => setRenameValue(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") saveRename();
              if (e.key === "Escape") cancelRename();
            }}
          />
          <div className="flex gap-2 justify-end">
            <Button variant="outline" onClick={cancelRename}>
              Cancel
            </Button>
            <Button onClick={saveRename} disabled={!renameValue.trim()}>
              Save
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
