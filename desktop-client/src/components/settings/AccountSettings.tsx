import { useState } from "react";
import { Button } from "../Button";
import { Input } from "../Input";
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

export function AccountSettings({ onBack }: AccountSettingsProps) {
  const { address } = useWalletStore();
  const [showSeed, setShowSeed] = useState(false);
  const [showPrivateKey, setShowPrivateKey] = useState(false);
  const [copied, setCopied] = useState<string | null>(null);
  const [seedPhrase, setSeedPhrase] = useState<string | null>(null);
  const [privateKey, setPrivateKey] = useState<string | null>(null);
  const [password, setPassword] = useState("");
  const [showPasswordDialog, setShowPasswordDialog] = useState<"mnemonic" | "privateKey" | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const handleCopy = (text: string, type: string) => {
    navigator.clipboard.writeText(text);
    toast.success(`${type} copied to clipboard`);
    setCopied(type);
    setTimeout(() => setCopied(null), 2000);
  };

  const handleShowMnemonic = () => {
    if (seedPhrase) {
      setShowSeed(!showSeed);
    } else {
      setShowPasswordDialog("mnemonic");
    }
  };

  const handleShowPrivateKey = () => {
    if (privateKey) {
      setShowPrivateKey(!showPrivateKey);
    } else {
      setShowPasswordDialog("privateKey");
    }
  };

  const handlePasswordSubmit = async () => {
    if (!password) {
      toast.error("Please enter your password");
      return;
    }

    setIsLoading(true);
    try {
      if (showPasswordDialog === "mnemonic") {
        const response = await walletApi.getMnemonic({ password });
        setSeedPhrase(response.data);
        setShowSeed(true);
        toast.success("Mnemonic phrase revealed");
      } else if (showPasswordDialog === "privateKey") {
        const response = await walletApi.getPrivateKey({ password });
        setPrivateKey(response.data);
        setShowPrivateKey(true);
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
        className="flex items-center gap-2 text-gray-500 hover:text-gray-900 mb-6 transition-colors"
      >
        <ArrowLeft className="w-5 h-5" />
        <span className="font-medium">Back to Settings</span>
      </button>

      <h2 className="text-3xl font-bold text-gray-900 mb-2">
        Account Information
      </h2>
      <p className="text-gray-500 mb-8">
        View and manage your wallet keys and recovery information.
      </p>

      <div className="space-y-6">
        <div className="bg-white/60 backdrop-blur-sm rounded-2xl p-6 border border-white/50 shadow-sm">
          <h3 className="text-sm font-semibold text-gray-500 uppercase tracking-widest mb-3">
            Wallet Address
          </h3>
          <div className="bg-gray-50 p-4 rounded-xl border border-gray-100 font-mono text-sm text-gray-700 break-all mb-3">
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

        <div className="bg-white/60 backdrop-blur-sm rounded-2xl p-6 border border-white/50 shadow-sm">
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-sm font-semibold text-gray-500 uppercase tracking-widest">
              Recovery Seed Phrase
            </h3>
            <button
              onClick={handleShowMnemonic}
              className="text-gray-400 hover:text-gray-600 transition-colors"
            >
              {showSeed ? <EyeClosed size={20} /> : <Eye size={20} />}
            </button>
          </div>

          {showSeed && seedPhrase ? (
            <>
              <div className="bg-gray-50 p-4 rounded-xl border border-gray-100 mb-3">
                <div className="grid grid-cols-3 gap-3">
                  {seedPhrase.split(" ").map((word, index) => (
                    <div
                      key={index}
                      className="flex items-center gap-2"
                    >
                      <span className="text-xs text-gray-400 font-medium w-6">
                        {index + 1}.
                      </span>
                      <span className="font-mono text-sm text-gray-700">
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
            <div className="bg-gray-50 p-4 rounded-xl border border-gray-100 text-center">
              <p className="text-sm text-gray-500">
                Click the eye icon to reveal (password required)
              </p>
            </div>
          )}
        </div>

        <div className="bg-white/60 backdrop-blur-sm rounded-2xl p-6 border border-white/50 shadow-sm">
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-sm font-semibold text-gray-500 uppercase tracking-widest">
              Private Key
            </h3>
            <button
              onClick={handleShowPrivateKey}
              className="text-gray-400 hover:text-gray-600 transition-colors"
            >
              {showPrivateKey ? <EyeClosed size={20} /> : <Eye size={20} />}
            </button>
          </div>

          {showPrivateKey && privateKey ? (
            <>
              <div className="bg-gray-50 p-4 rounded-xl border border-gray-100 font-mono text-sm text-gray-700 break-all mb-3">
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
            <div className="bg-gray-50 p-4 rounded-xl border border-gray-100 text-center">
              <p className="text-sm text-gray-500">
                Click the eye icon to reveal (password required)
              </p>
            </div>
          )}
        </div>
      </div>

      {/* Password Dialog */}
      {showPasswordDialog && (
        <div className="fixed inset-0 bg-black/50 dark:bg-black/70 flex items-center justify-center z-50">
          <div className="bg-white dark:bg-gray-800 rounded-2xl p-6 max-w-md w-full mx-4 shadow-xl">
            <div className="flex items-center gap-3 mb-4">
              <div className="w-10 h-10 rounded-full bg-amber-100 dark:bg-amber-900/30 flex items-center justify-center">
                <Lock size={20} className="text-amber-600 dark:text-amber-400" />
              </div>
              <div>
                <h3 className="font-semibold text-gray-900 dark:text-gray-100">
                  Enter Password
                </h3>
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  {showPasswordDialog === "mnemonic"
                    ? "Enter your password to view the recovery seed phrase"
                    : "Enter your password to view the private key"}
                </p>
              </div>
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
                    handlePasswordSubmit();
                  } else if (e.key === "Escape") {
                    handleCancelPassword();
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
                  onClick={handlePasswordSubmit}
                  className="flex-1"
                  disabled={!password || isLoading}
                >
                  {isLoading ? "Verifying..." : "Verify"}
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
