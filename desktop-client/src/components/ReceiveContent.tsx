import { useState } from "react";
import { useWalletStore } from "../store/walletStore";
import { Copy, CheckCircle } from "@solar-icons/react";
import { Button } from "./Button";
import toast from "react-hot-toast";

export function ReceiveContent() {
  const { address } = useWalletStore();
  const [copied, setCopied] = useState(false);

  const displayAddress = address || "no address";

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
        <p className="text-gray-500 text-sm">
          Scan this QR code or copy the address below to receive funds.
        </p>
      </div>

      <div className="flex justify-center my-6">
        <div className="w-48 h-48 bg-white rounded-xl border-2 border-primary/20 p-2 shadow-inner flex items-center justify-center">
          <div className="w-full h-full bg-gray-100 flex items-center justify-center text-gray-400 text-xs text-center border border-dashed border-gray-300 rounded-lg">
            QR Code Placeholder
          </div>
        </div>
      </div>

      <div className="bg-gray-50 rounded-2xl p-4 border border-gray-100 text-left relative group">
        <p className="text-xs font-semibold text-gray-500 uppercase tracking-widest mb-1 ml-1">
          Your Address
        </p>
        <div className="font-mono text-xs md:text-sm text-gray-800 break-all leading-relaxed">
          {displayAddress}
        </div>
        <button
          onClick={handleCopy}
          className="absolute top-2 right-2 p-2 rounded-lg bg-white shadow-sm border border-gray-100 text-gray-500 hover:text-primary hover:border-primary/50 transition-all"
        >
          {copied ? (
            <CheckCircle
              size={16}
              className="text-green-500"
            />
          ) : (
            <Copy size={16} />
          )}
        </button>
      </div>

      <Button
        onClick={handleCopy}
        className="w-full rounded-xl py-3 shadow-lg shadow-primary/20"
      >
        {copied ? "Copied to Clipboard" : "Copy Address"}
      </Button>
    </div>
  );
}
