import { useState } from "react";
import { Button } from "../Button";
import { Input } from "../Input";
import { ArrowLeft, CheckCircle, AddCircle, User } from "@solar-icons/react";
import { useSettingsStore } from "../../store/settingsStore";
import { Modal } from "../Modal";
import toast from "react-hot-toast";
import { Tooltip } from "../Tooltip";

const DEFAULT_ACCOUNT_IDS = ["0"];

interface AccountListSettingsProps {
  onBack: () => void;
}

export function AccountListSettings({ onBack }: AccountListSettingsProps) {
  const { accountLabels, activeAccountId, setAccountLabel, setActiveAccountId } = useSettingsStore();
  const [renameId, setRenameId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState("");

  const accountIds = DEFAULT_ACCOUNT_IDS;
  const activeLabel = accountLabels[activeAccountId] ?? activeAccountId ?? "Default";

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
      <button
        onClick={onBack}
        className="flex items-center gap-2 text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 mb-6 transition-colors"
      >
        <ArrowLeft className="w-5 h-5" />
        <span className="font-medium">Back to Settings</span>
      </button>

      <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2">
        Accounts
      </h2>
      <p className="text-sm text-gray-500 dark:text-gray-400 mb-6">
        Manage accounts and switch between them. Each account has its own receive address and balance.
      </p>

      <div className="space-y-3">
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

        <Tooltip content="Backend support for multiple accounts is coming soon. You can rename the default account above.">
          <div className="flex">
            <button
              disabled
              className="w-full p-4 rounded-xl border border-dashed border-gray-300 dark:border-gray-600 text-gray-400 dark:text-gray-500 flex items-center justify-center gap-2 cursor-not-allowed"
            >
              <AddCircle size={20} />
              <span>Add account</span>
              <span className="text-xs">(coming soon)</span>
            </button>
          </div>
        </Tooltip>
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
