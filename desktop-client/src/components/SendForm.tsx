import { useState } from "react";
import toast from "react-hot-toast";
import { Button } from "../components/Button";
import { Input } from "../components/Input";
import {
  Scanner,
  InfoCircle,
  BoltCircle,
  CheckCircle,
} from "@solar-icons/react";
import { useWalletStore } from "../store/walletStore";
import { useTokenStore } from "../store/tokenStore";
import { walletApi } from "../lib/api";
import { cn } from "../components/Button";

type Priority = "slow" | "normal" | "fast";

const FEES = {
  slow: 0.00004,
  normal: 0.0002,
  fast: 0.001,
};

interface SendFormProps {
  onSuccess?: () => void;
  onCancel?: () => void;
}

export function SendForm({ onSuccess, onCancel }: SendFormProps) {
  const { balance } = useWalletStore();
  const { activeTokenId, getToken } = useTokenStore();

  const activeToken = activeTokenId ? getToken(activeTokenId) : null;
  const tokenSymbol = activeToken?.symbol || "ZEC";

  const [amount, setAmount] = useState("");
  const [address, setAddress] = useState("");
  const [password, setPassword] = useState("");
  const [memo, setMemo] = useState("");
  const [priority, setPriority] = useState<Priority>("normal");
  const [showMemo, setShowMemo] = useState(false);
  const [showReview, setShowReview] = useState(false);
  const [isSending, setIsSending] = useState(false);
  const [success, setSuccess] = useState(false);

  const isValid =
    amount &&
    address &&
    parseFloat(amount) > 0 &&
    parseFloat(amount) + FEES[priority] <= balance;

  const handleMax = () => {
    const maxAmount = Math.max(0, balance - FEES[priority]);
    setAmount(maxAmount >= 0 ? maxAmount.toString() : "0");
  };

  const handleSend = async () => {
    setIsSending(true);
    const sendToast = toast.loading("Sending transaction...");
    try {
      await walletApi.sendTransaction({
        recipient: address,
        amount: parseFloat(amount),
        memo: memo || undefined,
        password: password,
      });
      toast.success("Transaction sent successfully!", { id: sendToast });
      setSuccess(true);

      setTimeout(() => {
        setSuccess(false);
        setShowReview(false);
        setAmount("");
        setAddress("");
        setMemo("");
        setPassword("");
        onSuccess?.();
      }, 2000);
    } catch (error: any) {
      // console.error("Send failed", error);
      toast.error("Send failed. Please check your password and balance.", {
        id: sendToast,
      });
      toast.error(
        error?.message ||
          "Send failed. Please check your password and balance.",
        { id: sendToast }
      );
    } finally {
      setIsSending(false);
    }
  };

  if (showReview) {
    return (
      <div className="space-y-6 animate-fade-in">
        {!success ? (
          <>
            <div className="text-center">
              <h3 className="text-xl font-bold text-gray-900">
                Confirm Transfer
              </h3>
              <p className="text-sm text-gray-500 mt-1">
                Please review the transaction details carefully.
              </p>
            </div>

            <div className="bg-gray-50 rounded-2xl p-6 space-y-4">
              <div className="flex justify-between items-center">
                <span className="text-gray-500 text-sm">Amount</span>
                <span className="font-bold text-lg text-gray-900">
                  {amount} {tokenSymbol}
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-gray-500 text-sm">Fee</span>
                <span className="text-gray-900 font-medium text-sm">
                  {FEES[priority]} {tokenSymbol}
                </span>
              </div>
              <div className="h-px bg-gray-200 my-2" />
              <div className="flex justify-between items-center text-base">
                <span className="font-bold text-gray-900">Total</span>
                <span className="font-bold text-primary-700">
                  {(parseFloat(amount || "0") + FEES[priority]).toFixed(5)}{" "}
                  {tokenSymbol}
                </span>
              </div>
            </div>

            <div className="space-y-1">
              <p className="text-xs font-semibold text-gray-500 uppercase tracking-widest pl-1">
                Recipient
              </p>
              <div className="bg-gray-50 p-3 rounded-xl border border-gray-100 font-mono text-xs text-gray-600 break-all leading-relaxed">
                {address}
              </div>
            </div>

            <div className="space-y-1">
              <Input
                type="password"
                label="Wallet Password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="Enter your password to confirm"
                className="bg-white/50 focus:bg-white transition-all"
              />
            </div>

            <div className="flex gap-3 pt-2">
              <Button
                variant="ghost"
                onClick={() => setShowReview(false)}
                disabled={isSending}
                className="flex-1"
              >
                Back
              </Button>
              <Button
                onClick={handleSend}
                disabled={isSending || !password}
                className="flex-1 rounded-xl shadow-lg shadow-primary/20"
              >
                {isSending ? "Sending..." : "Confirm Send"}
              </Button>
            </div>
          </>
        ) : (
          <div className="text-center py-8 space-y-4">
            <div className="w-20 h-20 rounded-full bg-green-100 text-green-500 flex items-center justify-center mx-auto mb-6 animate-scale-up">
              <CheckCircle
                size={48}
                weight="Bold"
              />
            </div>
            <h3 className="text-2xl font-bold text-gray-900">
              Transaction Sent!
            </h3>
            <p className="text-gray-500">Your funds are on their way.</p>
          </div>
        )}
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="space-y-2 group">
        <div className="flex justify-between items-end px-2">
          <label className="text-sm font-medium text-gray-500 group-focus-within:text-primary transition-colors">
            Amount
          </label>
          <div className="text-xs text-gray-400 font-medium">
            Available:{" "}
            <span
              className="text-gray-700 cursor-pointer hover:text-primary transition-colors uppercase"
              onClick={handleMax}
            >
              {balance.toLocaleString()} {tokenSymbol}
            </span>
          </div>
        </div>

        <div className="relative">
          <input
            type="number"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            placeholder="0.00"
            className="w-full text-4xl bg-transparent border-none focus:ring-0 text-center font-bold text-gray-900 placeholder:text-gray-300 p-2 drop-shadow-sm"
          />
          <span className="absolute right-8 top-1/2 -translate-y-1/2 text-primary font-bold text-xl pointer-events-none uppercase">
            {tokenSymbol}
          </span>
        </div>
      </div>

      <div className="space-y-3">
        <label className="text-sm font-medium text-gray-700 ml-1">
          Recipient Address
        </label>
        <div className="relative group">
          <Input
            value={address}
            onChange={(e) => setAddress(e.target.value)}
            placeholder="44AFFq5kSiGBoZ... (ZCash Address)"
            className="pr-12 py-3.5 bg-white/50 focus:bg-white ring-none transition-all font-mono text-sm"
          />
          <button className="absolute right-4 top-1/2 -translate-y-1/2 text-gray-400 hover:text-primary transition-colors">
            <Scanner size={20} />
          </button>
        </div>
      </div>

      <div>
        <button
          onClick={() => setShowMemo(!showMemo)}
          className="text-sm text-gray-500 hover:text-primary flex items-center gap-1.5 transition-colors ml-1"
        >
          <InfoCircle size={16} />
          {showMemo ? "Hide Memo" : "Add Memo (Optional)"}
        </button>

        {showMemo && (
          <div className="mt-3 animate-slide-up">
            <Input
              value={memo}
              onChange={(e) => setMemo(e.target.value)}
              placeholder="Memo (optional)"
              className="bg-white/50 border-gray-200 focus:bg-white focus:border-primary focus:ring-4 focus:ring-primary/10 transition-all font-mono text-sm"
            />
          </div>
        )}
      </div>

      <div className="space-y-3">
        <label className="text-sm font-medium text-gray-700 ml-1">
          Transaction Priority
        </label>
        <div className="grid grid-cols-3 gap-3">
          {(["slow", "normal", "fast"] as Priority[]).map((p) => (
            <button
              key={p}
              onClick={() => setPriority(p)}
              className={cn(
                "flex flex-col items-center justify-center py-2.5 rounded-xl border transition-all duration-200 relative overflow-hidden",
                priority === p
                  ? "bg-primary text-black border-primary shadow-md shadow-primary/20"
                  : "bg-white/40 border-gray-200 text-gray-600 hover:border-primary/30 hover:bg-white/80"
              )}
            >
              <span className="capitalize font-bold text-sm tracking-wide">
                {p}
              </span>
              <span
                className={cn(
                  "text-[10px] mt-0.5 opacity-80",
                  priority === p ? "text-black" : "text-gray-400"
                )}
              >
                {p === "slow" ? "~20m" : p === "normal" ? "~2m" : "~0m"}
              </span>
              {priority === p && p === "fast" && (
                <div className="absolute top-1 right-1">
                  <BoltCircle
                    size={10}
                    weight="Bold"
                    className="text-white"
                  />
                </div>
              )}
            </button>
          ))}
        </div>
      </div>

      <div className="flex gap-3 pt-4">
        {onCancel && (
          <Button
            variant="ghost"
            className="flex-1"
            onClick={onCancel}
          >
            Cancel
          </Button>
        )}
        <Button
          size="lg"
          disabled={!isValid}
          onClick={() => setShowReview(true)}
          className="flex-1 rounded-xl py-4 text-lg shadow-xl bg-black text-white shadow-primary/30 -hover:translate-y-[2px] transition-all duration-300 disabled:cursor-not-allowed disabled:transform-none disabled:shadow-none"
        >
          Review
        </Button>
      </div>
    </div>
  );
}
