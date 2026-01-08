import { useState } from "react";
import { Button } from "../Button";
import {
  ArrowLeft,
  Copy,
  Eye,
  EyeClosed,
  CheckCircle,
} from "@solar-icons/react";
import { useWalletStore } from "../../store/walletStore";
import toast from "react-hot-toast";

interface AccountSettingsProps {
  onBack: () => void;
}

export function AccountSettings({ onBack }: AccountSettingsProps) {
  const { address } = useWalletStore();
  const [showSeed, setShowSeed] = useState(false);
  const [showPrivateKey, setShowPrivateKey] = useState(false);
  const [copied, setCopied] = useState<string | null>(null);

  // Mock data - replace with actual wallet data
  const seedPhrase =
    "word1 word2 word3 word4 word5 word6 word7 word8 word9 word10 word11 word12";
  const privateKey = "5KQwrPbwdL6PhXujxW37FSSQZ1JiwsST4cqQzDeyXtP79zkvFD3";

  const handleCopy = (text: string, type: string) => {
    navigator.clipboard.writeText(text);
    toast.success(`${type} copied to clipboard`);
    setCopied(type);
    setTimeout(() => setCopied(null), 2000);
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
              onClick={() => setShowSeed(!showSeed)}
              className="text-gray-400 hover:text-gray-600 transition-colors"
            >
              {showSeed ? <EyeClosed size={20} /> : <Eye size={20} />}
            </button>
          </div>

          {showSeed ? (
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
                Click the eye icon to reveal
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
              onClick={() => setShowPrivateKey(!showPrivateKey)}
              className="text-gray-400 hover:text-gray-600 transition-colors"
            >
              {showPrivateKey ? <EyeClosed size={20} /> : <Eye size={20} />}
            </button>
          </div>

          {showPrivateKey ? (
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
                Click the eye icon to reveal
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
