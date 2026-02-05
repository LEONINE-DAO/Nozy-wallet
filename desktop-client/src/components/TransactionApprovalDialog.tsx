import { useState, useEffect } from "react";
import { Button } from "./Button";
import { CloseCircle, DangerTriangle, CheckCircle } from "@solar-icons/react";
import { useWalletStore } from "../store/walletStore";

interface TransactionApprovalDialogProps {
  isOpen: boolean;
  transaction: {
    to: string;
    value: string;
    data?: string;
    gas?: string;
    gasPrice?: string;
  };
  origin: string;
  onApprove: () => void;
  onReject: () => void;
}

export function TransactionApprovalDialog({
  isOpen,
  transaction,
  origin,
  onApprove,
  onReject,
}: TransactionApprovalDialogProps) {
  const { address } = useWalletStore();
  const [isProcessing, setIsProcessing] = useState(false);

  // Keyboard shortcuts
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && !isProcessing) {
        onReject();
      }
      if (e.key === 'Enter' && !isProcessing && !e.shiftKey) {
        handleApprove();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, isProcessing]);

  if (!isOpen) return null;

  // Convert value from hex to ZEC (Zcash uses zatoshis: 1 ZEC = 10^8 zatoshis)
  const parseValue = (value: string): number => {
    try {
      // Remove 0x prefix if present
      const hexValue = value.startsWith("0x") ? value.slice(2) : value;
      // Convert hex to decimal, then to ZEC (1 ZEC = 10^8 zatoshis)
      const zatoshis = parseInt(hexValue, 16);
      return zatoshis / 100_000_000;
    } catch {
      return 0;
    }
  };

  const amount = parseValue(transaction.value || "0x0");
  const recipient = transaction.to || "Unknown";

  const handleApprove = async () => {
    setIsProcessing(true);
    try {
      await onApprove();
    } finally {
      setIsProcessing(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/60 dark:bg-black/80 flex items-center justify-center z-50 p-4">
      <div className="bg-white dark:bg-gray-800 rounded-2xl p-6 max-w-md w-full shadow-2xl border border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-full bg-amber-100 dark:bg-amber-900/30 flex items-center justify-center">
              <DangerTriangle size={20} className="text-amber-600 dark:text-amber-400" />
            </div>
            <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
              Transaction Request
            </h3>
          </div>
          <button
            onClick={onReject}
            className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
            disabled={isProcessing}
            aria-label="Close dialog"
            title="Close (Esc)"
          >
            <CloseCircle size={20} />
          </button>
        </div>

        <div className="space-y-4 mb-6">
          <div className="bg-gray-50 dark:bg-gray-900/50 rounded-lg p-3">
            <p className="text-xs text-gray-500 dark:text-gray-400 mb-1">From</p>
            <p className="text-sm font-mono text-gray-900 dark:text-gray-100 break-all">
              {address || "Not connected"}
            </p>
          </div>

          <div className="bg-gray-50 dark:bg-gray-900/50 rounded-lg p-3">
            <p className="text-xs text-gray-500 dark:text-gray-400 mb-1">To</p>
            <p className="text-sm font-mono text-gray-900 dark:text-gray-100 break-all">
              {recipient}
            </p>
          </div>

          <div className="bg-gray-50 dark:bg-gray-900/50 rounded-lg p-3">
            <p className="text-xs text-gray-500 dark:text-gray-400 mb-1">Amount</p>
            <p className="text-lg font-semibold text-gray-900 dark:text-gray-100">
              {amount.toFixed(8)} ZEC
            </p>
          </div>

          <div className="bg-gray-50 dark:bg-gray-900/50 rounded-lg p-3">
            <p className="text-xs text-gray-500 dark:text-gray-400 mb-1">Requested by</p>
            <p className="text-sm text-gray-900 dark:text-gray-100 break-all">
              {origin}
            </p>
          </div>

          {transaction.data && transaction.data !== "0x" && (
            <div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-lg p-3">
              <p className="text-xs text-amber-800 dark:text-amber-200">
                ⚠️ This transaction includes additional data. Review carefully before approving.
              </p>
            </div>
          )}
        </div>

        <div className="flex gap-3">
          <Button
            variant="secondary"
            onClick={onReject}
            className="flex-1"
            disabled={isProcessing}
          >
            Reject
          </Button>
          <Button
            onClick={handleApprove}
            className="flex-1"
            disabled={isProcessing || !address}
          >
            {isProcessing ? (
              <>
                <span className="animate-spin mr-2">⏳</span>
                Processing...
              </>
            ) : (
              <>
                <CheckCircle size={16} className="mr-2" />
                Approve
              </>
            )}
          </Button>
        </div>
      </div>
    </div>
  );
}
