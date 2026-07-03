import { useState, useEffect, useCallback } from "react";
import { Button } from "../Button";
import { Input } from "../Input";
import { Modal } from "../Modal";
import {
  ArrowLeft,
  Copy,
  Eye,
  EyeClosed,
  CheckCircle,
  Lock,
} from "@solar-icons/react";
import { useWalletStore } from "../../store/walletStore";
import { walletApi } from "../../lib/api";
import toast from "react-hot-toast";
import { formatErrorForDisplay } from "../../utils/errors";
import { logger } from "../../utils/logger";

interface AccountSettingsProps {
  onBack: () => void;
}

function RevealControl({
  revealed,
  label,
  loading,
  onClick,
}: {
  revealed: boolean;
  label: string;
  loading: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={loading}
      aria-label={revealed ? `Hide ${label}` : `Reveal ${label}`}
      className="inline-flex shrink-0 items-center gap-2 rounded-lg border border-gray-200 bg-white px-3 py-2 text-sm font-medium text-gray-700 shadow-sm transition-colors hover:border-primary/40 hover:bg-primary-50 hover:text-primary-700 disabled:opacity-50 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-100 dark:hover:border-primary/50 dark:hover:bg-primary/10 dark:hover:text-primary-300"
    >
      {revealed ? (
        <EyeClosed size={18} weight="Bold" className="text-current" />
      ) : (
        <Eye size={18} weight="Bold" className="text-current" />
      )}
      <span>{revealed ? "Hide" : "Reveal"}</span>
    </button>
  );
}

export function AccountSettings({ onBack }: AccountSettingsProps) {
  const { address } = useWalletStore();
  const [showSeed, setShowSeed] = useState(false);
  const [showPrivateKey, setShowPrivateKey] = useState(false);
  const [copied, setCopied] = useState<string | null>(null);
  const [seedPhrase, setSeedPhrase] = useState<string | null>(null);
  const [privateKey, setPrivateKey] = useState<string | null>(null);
  const [password, setPassword] = useState("");
  const [hasPassword, setHasPassword] = useState<boolean | null>(null);
  const [showPasswordDialog, setShowPasswordDialog] = useState<"mnemonic" | "privateKey" | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    walletApi
      .getWalletStatus()
      .then((res) => setHasPassword(res.data.has_password))
      .catch(() => setHasPassword(true));
  }, []);

  const revealMnemonic = useCallback(async (pwd: string) => {
    const response = await walletApi.getMnemonic({ password: pwd });
    setSeedPhrase(response.data);
    setShowSeed(true);
  }, []);

  const revealPrivateKey = useCallback(async (pwd: string) => {
    const response = await walletApi.getPrivateKey({ password: pwd });
    setPrivateKey(response.data);
    setShowPrivateKey(true);
  }, []);

  const revealHint =
    hasPassword === false
      ? "Click the eye icon to reveal"
      : "Click the eye icon to reveal (password required)";

  const handleCopy = (text: string, type: string) => {
    navigator.clipboard.writeText(text);
    toast.success(`${type} copied to clipboard`);
    setCopied(type);
    setTimeout(() => setCopied(null), 2000);
  };

  const handleShowMnemonic = async () => {
    if (seedPhrase) {
      setShowSeed(!showSeed);
      return;
    }
    if (hasPassword === false) {
      setIsLoading(true);
      try {
        await revealMnemonic("");
        toast.success("Recovery phrase revealed");
      } catch (error: unknown) {
        toast.error(formatErrorForDisplay(error, "Failed to reveal recovery phrase"));
      } finally {
        setIsLoading(false);
      }
      return;
    }
    setShowPasswordDialog("mnemonic");
  };

  const handleShowPrivateKey = async () => {
    if (privateKey) {
      setShowPrivateKey(!showPrivateKey);
      return;
    }
    if (hasPassword === false) {
      setIsLoading(true);
      try {
        await revealPrivateKey("");
        toast.success("Private key revealed");
      } catch (error: unknown) {
        toast.error(formatErrorForDisplay(error, "Failed to reveal private key"));
      } finally {
        setIsLoading(false);
      }
      return;
    }
    setShowPasswordDialog("privateKey");
  };

  const handlePasswordSubmit = async () => {
    if (hasPassword !== false && !password) {
      toast.error("Please enter your password");
      return;
    }

    setIsLoading(true);
    try {
      if (showPasswordDialog === "mnemonic") {
        await revealMnemonic(password);
        toast.success("Recovery phrase revealed");
      } else if (showPasswordDialog === "privateKey") {
        await revealPrivateKey(password);
        toast.success("Private key revealed");
      }
      setPassword("");
      setShowPasswordDialog(null);
    } catch (error: unknown) {
      logger.error("Password verification failed", error as Error, { 
        action: showPasswordDialog 
      });
      toast.error(formatErrorForDisplay(error, "Failed to verify password"));
    } finally {
      setIsLoading(false);
    }
  };

  const handleCancelPassword = () => {
    setPassword("");
    setShowPasswordDialog(null);
  };

  return (
    <div className="max-w-2xl mx-auto animate-fade-in">
      <button
        onClick={onBack}
        className="mb-6 flex items-center gap-2 text-gray-500 transition-colors hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-100"
      >
        <ArrowLeft className="h-5 w-5" />
        <span className="font-medium">Back to Settings</span>
      </button>

      <h2 className="mb-2 text-3xl font-bold text-gray-900 dark:text-gray-100">
        Account Information
      </h2>
      <p className="mb-8 text-gray-500 dark:text-gray-400">
        View and manage your wallet keys and recovery information.
      </p>

      <div className="space-y-6">
        <div className="rounded-2xl border border-white/50 bg-white/60 p-6 shadow-sm backdrop-blur-sm dark:border-gray-700/50 dark:bg-gray-800/60">
          <h3 className="mb-3 text-sm font-semibold uppercase tracking-widest text-gray-500 dark:text-gray-400">
            Wallet Address
          </h3>
          <div className="mb-3 break-all rounded-xl border border-gray-100 bg-gray-50 p-4 font-mono text-sm text-gray-700 dark:border-gray-700 dark:bg-gray-900/50 dark:text-gray-200">
            {address || "Loading..."}
          </div>
          <Button
            variant="secondary"
            size="sm"
            onClick={() => handleCopy(address || "", "Address")}
            className="w-full"
          >
            {copied === "Address" ? (
              <>
                <CheckCircle
                  size={16}
                  className="mr-2 text-green-500"
                />
                Copied!
              </>
            ) : (
              <>
                <Copy
                  size={16}
                  className="mr-2"
                />
                Copy Address
              </>
            )}
          </Button>
        </div>

        <div className="rounded-2xl border border-white/50 bg-white/60 p-6 shadow-sm backdrop-blur-sm dark:border-gray-700/50 dark:bg-gray-800/60">
          <div className="mb-3 flex items-start justify-between gap-3">
            <h3 className="min-w-0 flex-1 text-sm font-semibold uppercase tracking-widest text-gray-500 dark:text-gray-400">
              Recovery Seed Phrase
            </h3>
            <RevealControl
              revealed={showSeed}
              label="recovery seed phrase"
              loading={isLoading}
              onClick={() => void handleShowMnemonic()}
            />
          </div>

          {showSeed && seedPhrase ? (
            <>
              <div className="mb-3 rounded-xl border border-gray-100 bg-gray-50 p-4 dark:border-gray-700 dark:bg-gray-900/50">
                <div className="grid grid-cols-3 gap-3">
                  {seedPhrase.split(" ").map((word, index) => (
                    <div
                      key={index}
                      className="flex items-center gap-2"
                    >
                      <span className="text-xs text-gray-400 font-medium w-6">
                        {index + 1}.
                      </span>
                      <span className="font-mono text-sm text-gray-700 dark:text-gray-200">
                        {word}
                      </span>
                    </div>
                  ))}
                </div>
              </div>
              <div className="bg-amber-50 border border-amber-200 rounded-xl p-3 mb-3">
                <p className="text-xs text-amber-800">
                  ⚠️ Never share your seed phrase. Anyone with this phrase can
                  access your funds.
                </p>
              </div>
              <Button
                variant="secondary"
                size="sm"
                onClick={() => handleCopy(seedPhrase, "Seed phrase")}
                className="w-full"
              >
                {copied === "Seed phrase" ? (
                  <>
                    <CheckCircle
                      size={16}
                      className="mr-2 text-green-500"
                    />
                    Copied!
                  </>
                ) : (
                  <>
                    <Copy
                      size={16}
                      className="mr-2"
                    />
                    Copy Seed Phrase
                  </>
                )}
              </Button>
            </>
          ) : (
            <div className="rounded-xl border border-gray-100 bg-gray-50 p-4 text-center dark:border-gray-700 dark:bg-gray-900/50">
              <p className="mb-4 text-sm text-gray-500 dark:text-gray-400">{revealHint}</p>
              <RevealControl
                revealed={false}
                label="recovery seed phrase"
                loading={isLoading}
                onClick={() => void handleShowMnemonic()}
              />
            </div>
          )}
        </div>

        <div className="rounded-2xl border border-white/50 bg-white/60 p-6 shadow-sm backdrop-blur-sm dark:border-gray-700/50 dark:bg-gray-800/60">
          <div className="mb-3 flex items-start justify-between gap-3">
            <h3 className="min-w-0 flex-1 text-sm font-semibold uppercase tracking-widest text-gray-500 dark:text-gray-400">
              Private Key
            </h3>
            <RevealControl
              revealed={showPrivateKey}
              label="private key"
              loading={isLoading}
              onClick={() => void handleShowPrivateKey()}
            />
          </div>

          {showPrivateKey && privateKey ? (
            <>
              <div className="mb-3 break-all rounded-xl border border-gray-100 bg-gray-50 p-4 font-mono text-sm text-gray-700 dark:border-gray-700 dark:bg-gray-900/50 dark:text-gray-200">
                {privateKey}
              </div>
              <div className="bg-amber-50 border border-amber-200 rounded-xl p-3 mb-3">
                <p className="text-xs text-amber-800">
                  ⚠️ Never share your private key. Anyone with this key can
                  access your funds.
                </p>
              </div>
              <Button
                variant="secondary"
                size="sm"
                onClick={() => handleCopy(privateKey, "Private key")}
                className="w-full"
              >
                {copied === "Private key" ? (
                  <>
                    <CheckCircle
                      size={16}
                      className="mr-2 text-green-500"
                    />
                    Copied!
                  </>
                ) : (
                  <>
                    <Copy
                      size={16}
                      className="mr-2"
                    />
                    Copy Private Key
                  </>
                )}
              </Button>
            </>
          ) : (
            <div className="rounded-xl border border-gray-100 bg-gray-50 p-4 text-center dark:border-gray-700 dark:bg-gray-900/50">
              <p className="mb-4 text-sm text-gray-500 dark:text-gray-400">{revealHint}</p>
              <RevealControl
                revealed={false}
                label="private key"
                loading={isLoading}
                onClick={() => void handleShowPrivateKey()}
              />
            </div>
          )}
        </div>
      </div>

      <Modal
        isOpen={showPasswordDialog !== null}
        onClose={handleCancelPassword}
        title="Enter password"
      >
        <div className="flex items-center gap-3 mb-4">
          <div className="w-10 h-10 rounded-full bg-amber-100 flex items-center justify-center">
            <Lock size={20} className="text-amber-600" />
          </div>
          <p className="text-sm text-gray-500">
            {showPasswordDialog === "mnemonic"
              ? "Enter your wallet password to view the recovery seed phrase."
              : "Enter your wallet password to view the private key."}
          </p>
        </div>

        <div className="space-y-4">
          <Input
            type="password"
            label="Password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            placeholder="Enter your wallet password"
            autoFocus
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                void handlePasswordSubmit();
              }
            }}
          />

          <div className="flex gap-3">
            <Button
              variant="secondary"
              onClick={handleCancelPassword}
              className="flex-1"
              disabled={isLoading}
            >
              Cancel
            </Button>
            <Button
              onClick={() => void handlePasswordSubmit()}
              className="flex-1"
              disabled={(hasPassword !== false && !password) || isLoading}
            >
              {isLoading ? "Verifying..." : "Verify"}
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
