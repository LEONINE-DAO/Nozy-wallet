import { useState, useEffect } from "react";
import { Button } from "./Button";
import { Input } from "./Input";
import { CloseCircle, Lock, CheckCircle } from "@solar-icons/react";
import { useWalletStore } from "../store/walletStore";

interface MessageSigningDialogProps {
  isOpen: boolean;
  message: string;
  origin: string;
  onApprove: (password: string) => void;
  onReject: () => void;
}

export function MessageSigningDialog({
  isOpen,
  message,
  origin,
  onApprove,
  onReject,
}: MessageSigningDialogProps) {
  const { address } = useWalletStore();
  const [isProcessing, setIsProcessing] = useState(false);
  const [password, setPassword] = useState("");

  // Keyboard shortcuts
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && !isProcessing) {
        onReject();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, isProcessing, onReject]);

  if (!isOpen) return null;

  // Decode message if it's hex
  const decodeMessage = (msg: string): string => {
    try {
      if (msg.startsWith("0x")) {
        const hex = msg.slice(2);
        let decoded = "";
        for (let i = 0; i < hex.length; i += 2) {
          const charCode = parseInt(hex.substr(i, 2), 16);
          if (charCode === 0) break;
          decoded += String.fromCharCode(charCode);
        }
        return decoded || msg;
      }
      return msg;
    } catch {
      return msg;
    }
  };

  const decodedMessage = decodeMessage(message);

  const handleApprove = async () => {
    if (!password) {
      return;
    }
    setIsProcessing(true);
    try {
      await onApprove(password);
      setPassword("");
    } finally {
      setIsProcessing(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/60 dark:bg-black/80 flex items-center justify-center z-50 p-4">
      <div className="bg-white dark:bg-gray-800 rounded-2xl p-6 max-w-md w-full shadow-2xl border border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-full bg-blue-100 dark:bg-blue-900/30 flex items-center justify-center">
              <Lock size={20} className="text-blue-600 dark:text-blue-400" />
            </div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
              Sign Message
            </h3>
          </div>
          <button
            onClick={onReject}
            className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
            disabled={isProcessing}
          >
            <CloseCircle size={20} />
          </button>
        </div>

        <div className="space-y-4 mb-6">
          <div className="bg-gray-50 dark:bg-gray-900/50 rounded-lg p-3">
            <p className="text-xs text-gray-500 dark:text-gray-400 mb-1">Signing with</p>
            <p className="text-sm font-mono text-gray-900 dark:text-gray-100 break-all">
              {address || "Not connected"}
            </p>
          </div>

          <div className="bg-gray-50 dark:bg-gray-900/50 rounded-lg p-3">
            <p className="text-xs text-gray-500 dark:text-gray-400 mb-2">Message</p>
            <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded p-3 max-h-40 overflow-y-auto">
              <p className="text-sm text-gray-900 dark:text-gray-100 whitespace-pre-wrap break-words">
                {decodedMessage}
              </p>
            </div>
          </div>

          <div className="bg-gray-50 dark:bg-gray-900/50 rounded-lg p-3">
            <p className="text-xs text-gray-500 dark:text-gray-400 mb-1">Requested by</p>
            <p className="text-sm text-gray-900 dark:text-gray-100 break-all">
              {origin}
            </p>
          </div>

          <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-3">
            <p className="text-xs text-blue-800 dark:text-blue-200">
              ℹ️ Signing a message does not send any funds. This is used for authentication and verification.
            </p>
          </div>

          <div className="bg-gray-50 dark:bg-gray-900/50 rounded-lg p-3">
            <p className="text-xs text-gray-500 dark:text-gray-400 mb-2">Wallet Password</p>
            <Input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Enter your wallet password"
              className="w-full"
              autoFocus
              onKeyDown={(e) => {
                if (e.key === "Enter" && password && !isProcessing) {
                  handleApprove();
                }
              }}
            />
          </div>
        </div>

        <div className="flex gap-3">
          <Button
            variant="secondary"
            onClick={onReject}
            className="flex-1"
            disabled={isProcessing}
          >
            Cancel
          </Button>
          <Button
            onClick={handleApprove}
            className="flex-1"
            disabled={isProcessing || !address || !password}
          >
            {isProcessing ? (
              <>
                <span className="animate-spin mr-2">⏳</span>
                Signing...
              </>
            ) : (
              <>
                <CheckCircle size={16} className="mr-2" />
                Sign Message
              </>
            )}
          </Button>
        </div>
      </div>
    </div>
  );
}
