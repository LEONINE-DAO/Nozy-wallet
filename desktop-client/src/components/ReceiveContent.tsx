import { useState } from "react";
import QRCode from "react-qr-code";
import { useWalletStore } from "../store/walletStore";
import { Copy, CheckCircle } from "@solar-icons/react";
import { Button } from "./Button";
import toast from "react-hot-toast";

export function ReceiveContent() {
  const { address } = useWalletStore();
  const [copied, setCopied] = useState(false);

  const displayAddress = address || "No address available";
  const hasAddress = Boolean(address);

  const handleCopy = () => {
    if (address) {
      navigator.clipboard.writeText(address);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
      toast.success("Address copied to clipboard");
    }
  };

  return (
    <div className="space-y-8 animate-fade-in text-center">
      <div className="space-y-2">
        <p className="text-gray-500 dark:text-gray-400 text-sm">
          Scan this QR code or copy the address below to receive funds.
        </p>
      </div>

      <div className="flex justify-center my-6">
        <div className="w-48 h-48 bg-white dark:bg-gray-100 rounded-xl border-2 border-primary/20 p-3 shadow-inner flex items-center justify-center">
          {hasAddress ? (
            <QRCode
              value={address!}
              size={168}
              bgColor="#ffffff"
              fgColor="#111827"
              level="M"
              aria-label="Wallet receive address QR code"
            />
          ) : (
            <div className="w-full h-full bg-gray-100 dark:bg-gray-200 flex items-center justify-center text-gray-400 text-xs text-center border border-dashed border-gray-300 rounded-lg px-3">
              Unlock your wallet to show a receive QR code
            </div>
          )}
        </div>
      </div>

      <div className="bg-gray-50 dark:bg-gray-800/60 rounded-2xl p-4 border border-gray-100 dark:border-gray-700/50 text-left relative group">
        <p className="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-widest mb-1 ml-1">
          Your Address
        </p>
        <div className="font-mono text-xs md:text-sm text-gray-800 dark:text-gray-200 break-all leading-relaxed pr-10">
          {displayAddress}
        </div>
        <button
          onClick={handleCopy}
          className="absolute top-2 right-2 p-2 rounded-xl bg-white dark:bg-gray-700 shadow-sm border border-gray-100 dark:border-gray-600 text-gray-500 dark:text-gray-400 hover:text-primary hover:border-primary/50 transition-all"
        >
          {copied ? (
            <CheckCircle size={16} className="text-green-500" />
          ) : (
            <Copy size={16} />
          )}
        </button>
      </div>

      <Button onClick={handleCopy} className="w-full" disabled={!hasAddress}>
        {copied ? "Copied to Clipboard" : "Copy Address"}
      </Button>
    </div>
  );
}
