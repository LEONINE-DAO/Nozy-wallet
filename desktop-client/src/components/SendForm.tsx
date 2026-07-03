import { useState, useEffect } from "react";
import toast from "react-hot-toast";
import { logger } from "../utils/logger";
import { formatErrorForDisplay } from "../utils/errors";
import { Button } from "../components/Button";
import { Input } from "../components/Input";
import { Modal } from "../components/Modal";
import { Tooltip } from "../components/Tooltip";
import {
  Scanner,
  InfoCircle,
  CheckCircle,
  User,
  Shield,
} from "@solar-icons/react";
import QRCode from "react-qr-code";
import { useWalletStore } from "../store/walletStore";
import { useTokenStore } from "../store/tokenStore";
import { walletApi } from "../lib/api";
import { isWalletReadyForSend } from "../lib/syncHelpers";
import type { AddressBookEntry } from "../lib/types";
import { TransactionIdDetail } from "./TxExplorerLink";

/* FUTURE: group file co-sign — re-enable when treasury multi-party approval ships
import {
  cosignToJson,
  downloadJsonFile,
  parseCosignJson,
  zecFromZatoshis,
} from "../lib/cosignHelpers";
*/

interface SendFormProps {
  onSuccess?: () => void;
  onCancel?: () => void;
}

const DEFAULT_AMOUNT = "0.00";
const MAX_ZEC_DECIMALS = 8;

function sanitizeAmountInput(raw: string): string {
  const cleaned = raw.replace(/-/g, "").replace(/[^0-9.]/g, "");
  const dotIndex = cleaned.indexOf(".");
  if (dotIndex === -1) return cleaned;
  const intPart = cleaned.slice(0, dotIndex);
  const fracPart = cleaned.slice(dotIndex + 1).replace(/\./g, "").slice(0, MAX_ZEC_DECIMALS);
  return `${intPart}.${fracPart}`;
}

function parseAmount(value: string): number {
  const parsed = Number.parseFloat(value);
  return Number.isFinite(parsed) ? parsed : Number.NaN;
}

export function SendForm({ onSuccess, onCancel }: SendFormProps) {
  const { balance } = useWalletStore();
  const { activeTokenId, getToken } = useTokenStore();

  const activeToken = activeTokenId ? getToken(activeTokenId) : null;
  const tokenSymbol = activeToken?.symbol || "ZEC";

  const [amount, setAmount] = useState(DEFAULT_AMOUNT);
  const [address, setAddress] = useState("");
  const [password, setPassword] = useState("");
  const [memo, setMemo] = useState("");
  const [feeZec, setFeeZec] = useState(0.00015);
  const [showMemo, setShowMemo] = useState(false);
  const [showReview, setShowReview] = useState(false);
  const [isSending, setIsSending] = useState(false);
  const [success, setSuccess] = useState(false);
  const [pickContactOpen, setPickContactOpen] = useState(false);
  const [contacts, setContacts] = useState<AddressBookEntry[]>([]);
  const [saveContactOpen, setSaveContactOpen] = useState(false);
  const [saveContactName, setSaveContactName] = useState("");
  const [saveContactNotes, setSaveContactNotes] = useState("");
  const [saveContactSaving, setSaveContactSaving] = useState(false);
  const [sentTxid, setSentTxid] = useState<string | null>(null);
  const [walletUnlocked, setWalletUnlocked] = useState(false);
  const [walletHasPassword, setWalletHasPassword] = useState(true);
  const [keystoneEnabled, setKeystoneEnabled] = useState(false);
  const [keystonePrepared, setKeystonePrepared] = useState(false);
  const [keystoneSummary, setKeystoneSummary] = useState("");
  const [keystoneUrFrames, setKeystoneUrFrames] = useState<string[]>([]);
  const [keystoneSignedInput, setKeystoneSignedInput] = useState("");
  const [isBroadcasting, setIsBroadcasting] = useState(false);
  /* FUTURE: group file co-sign
  const [requireCosigner, setRequireCosigner] = useState(false);
  const [showBroadcastPanel, setShowBroadcastPanel] = useState(false);
  const [cosignExportJson, setCosignExportJson] = useState<string | null>(null);
  const [broadcastPayload, setBroadcastPayload] = useState("");
  const [broadcastPassword, setBroadcastPassword] = useState("");
  */

  useEffect(() => {
    walletApi.getWalletStatus().then((r) => {
      setWalletUnlocked(r.data?.unlocked ?? false);
      setWalletHasPassword(r.data?.has_password ?? true);
    }).catch(() => {});
    walletApi.getKeystoneStatus().then((r) => {
      const onMainnet = r.data?.network !== "testnet";
      setKeystoneEnabled((r.data?.enabled ?? false) && onMainnet);
    }).catch(() => {});
  }, [showReview]);

  const amountValue = parseAmount(amount);
  const isValid =
    address &&
    Number.isFinite(amountValue) &&
    amountValue > 0 &&
    amountValue + feeZec <= balance;

  useEffect(() => {
    if (pickContactOpen) {
      walletApi.listAddressBook().then((r) => setContacts(Array.isArray(r?.data) ? r.data : []));
    }
  }, [pickContactOpen]);

  useEffect(() => {
    let cancelled = false;
    walletApi
      .estimateFee()
      .then((r) => {
        if (!cancelled && typeof r?.data === "number") setFeeZec(r.data);
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  }, [memo]);

  const handleSaveToContacts = async () => {
    const name = saveContactName.trim();
    if (!name || !address.trim()) {
      toast.error("Name is required");
      return;
    }
    if (!address.startsWith("u1")) {
      toast.error("Address must be a mainnet Orchard unified address (u1…)");
      return;
    }
    setSaveContactSaving(true);
    try {
      await walletApi.addAddressBookEntry({
        name,
        address: address.trim(),
        notes: saveContactNotes.trim() || undefined,
      });
      toast.success("Saved to contacts");
      setSaveContactOpen(false);
      setSaveContactName("");
      setSaveContactNotes("");
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Failed to save contact"));
    } finally {
      setSaveContactSaving(false);
    }
  };

  const handleMax = () => {
    const maxAmount = Math.max(0, balance - feeZec);
    setAmount(maxAmount > 0 ? maxAmount.toFixed(MAX_ZEC_DECIMALS) : DEFAULT_AMOUNT);
  };

  const handleAmountChange = (raw: string) => {
    setAmount(sanitizeAmountInput(raw));
  };

  const handleAmountFocus = (event: React.FocusEvent<HTMLInputElement>) => {
    if (amount === DEFAULT_AMOUNT || amount === "0" || amount === "0.0") {
      event.target.select();
    }
  };

  const handleAmountBlur = () => {
    const parsed = parseAmount(amount);
    if (!amount || amount === "." || !Number.isFinite(parsed) || parsed <= 0) {
      setAmount(DEFAULT_AMOUNT);
    }
  };

  const preventAmountWheel = (event: React.WheelEvent<HTMLInputElement>) => {
    event.currentTarget.blur();
  };

  const handleProceedToReview = async () => {
    try {
      const statusRes = await walletApi.getSyncStatus();
      const { ready, reason } = isWalletReadyForSend(statusRes.data);
      if (!ready) {
        toast.error(reason ?? "Wallet not ready for send. Sync to tip first.");
        return;
      }
    } catch (e) {
      toast.error(formatErrorForDisplay(e, "Could not verify wallet sync status."));
      return;
    }
    setShowReview(true);
  };

  const sendPasswordRequired =
    walletHasPassword && !walletUnlocked && password.trim().length === 0;

  const resetKeystoneSendState = () => {
    setKeystonePrepared(false);
    setKeystoneSummary("");
    setKeystoneUrFrames([]);
    setKeystoneSignedInput("");
  };

  const handleKeystonePrepare = async () => {
    setIsSending(true);
    const toastId = toast.loading(
      "Building Keystone PCZT (proving may take several minutes)…"
    );
    const trimmedPassword = password.trim();
    try {
      const { data } = await walletApi.keystonePrepareSend({
        recipient: address,
        amount: amountValue,
        memo: memo || undefined,
        password: trimmedPassword || undefined,
      });
      if (!data.success) {
        toast.error(data.message ?? "Failed to prepare Keystone transaction", { id: toastId });
        return;
      }
      setKeystonePrepared(true);
      setKeystoneSummary(data.summary ?? "Transaction prepared");
      setKeystoneUrFrames(data.ur_frames ?? (data.pczt_hex ? [data.pczt_hex] : []));
      toast.success("Scan QR on Keystone, sign, then paste signed data below", { id: toastId });
    } catch (error: unknown) {
      toast.error(formatErrorForDisplay(error, "Failed to prepare Keystone transaction."), {
        id: toastId,
      });
    } finally {
      setIsSending(false);
    }
  };

  const handleKeystoneBroadcast = async () => {
    if (!keystoneSignedInput.trim()) {
      toast.error("Paste signed PCZT hex or UR frames from Keystone");
      return;
    }
    setIsBroadcasting(true);
    const toastId = toast.loading("Broadcasting Keystone-signed transaction…");
    try {
      const lines = keystoneSignedInput
        .split(/\n/)
        .map((l) => l.trim())
        .filter(Boolean);
      const isHex = /^[0-9a-fA-F]+$/.test(lines[0] ?? "");
      const { data } = await walletApi.keystoneCompleteSend(
        isHex
          ? { pcztHex: lines[0], broadcast: true }
          : { urFrames: lines, broadcast: true }
      );
      if (!data.success || !data.txid) {
        toast.error("Broadcast failed — check signed PCZT and sync status", { id: toastId });
        return;
      }
      toast.success(`Transaction broadcast! TXID: ${data.txid}`, { id: toastId });
      setSentTxid(data.txid);
      setSuccess(true);
      resetKeystoneSendState();
      setTimeout(() => {
        setSuccess(false);
        setSentTxid(null);
        setShowReview(false);
        setAmount(DEFAULT_AMOUNT);
        setAddress("");
        setMemo("");
        setPassword("");
        onSuccess?.();
      }, 3000);
    } catch (error: unknown) {
      toast.error(
        formatErrorForDisplay(error, "Broadcast failed. Check signed PCZT and sync status."),
        { id: toastId }
      );
    } finally {
      setIsBroadcasting(false);
    }
  };

  /* FUTURE: group file co-sign
  const handleExportCosignRequest = async () => { ... };
  const handleBroadcastSigned = async () => { ... };
  */

  const handleSend = async () => {
    setIsSending(true);
    const sendToast = toast.loading(
      "Building shielded transaction (first send may take several minutes while proving)…"
    );
    const trimmedPassword = password.trim();
    try {
      const { data: sent } = await walletApi.sendTransaction({
        recipient: address,
        amount: amountValue,
        memo: memo || undefined,
        password: trimmedPassword || undefined,
      });
      const txidMsg = sent.txid ? ` TXID: ${sent.txid}` : "";
      toast.success(`Transaction sent successfully!${txidMsg}`, { id: sendToast });
      setSentTxid(sent.txid ?? null);
      setSuccess(true);

      setTimeout(() => {
        setSuccess(false);
        setSentTxid(null);
        setShowReview(false);
        setAmount(DEFAULT_AMOUNT);
        setAddress("");
        setMemo("");
        setPassword("");
        onSuccess?.();
      }, 2000);
    } catch (error: unknown) {
      logger.error("Transaction send failed", error as Error, {
        recipient: address,
        amount: amountValue,
        hasMemo: !!memo
      });
      toast.error(
        formatErrorForDisplay(
          error,
          "Send failed. Check password (same as Unlock), sync status, and balance."
        ),
        {
        id: sendToast,
      }
      );
    } finally {
      setIsSending(false);
    }
  };

  if (success && sentTxid) {
    return (
      <div className="text-center py-8 space-y-4 animate-fade-in">
        <div className="w-20 h-20 rounded-full bg-green-100 text-green-500 flex items-center justify-center mx-auto mb-6 animate-scale-up">
          <CheckCircle size={48} weight="Bold" />
        </div>
        <h3 className="text-2xl font-bold text-gray-900">Transaction Sent!</h3>
        <p className="text-gray-500">Your funds are on their way.</p>
        <div className="mt-4 text-left">
          <TransactionIdDetail txid={sentTxid} />
        </div>
      </div>
    );
  }

  if (showReview) {
    return (
      <div className="space-y-6 animate-fade-in">
        <div className="text-center">
          <h3 className="text-xl font-bold text-gray-900">
            {keystoneEnabled
              ? keystonePrepared
                ? "Sign on Keystone"
                : "Prepare for Keystone"
              : "Confirm Transfer"}
          </h3>
          <p className="text-sm text-gray-500 mt-1">
            {keystoneEnabled
              ? keystonePrepared
                ? "Scan the QR on your Keystone device, sign, then paste the signed PCZT below."
                : "Review details, then build a PCZT for Keystone to sign."
              : "Please review the transaction details carefully."}
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
              {feeZec.toFixed(8)} {tokenSymbol} (priority ×4)
            </span>
          </div>
          <div className="h-px bg-gray-200 my-2" />
          <div className="flex justify-between items-center text-base">
            <span className="font-bold text-gray-900">Total</span>
            <span className="font-bold text-primary-700">
              {((Number.isFinite(amountValue) ? amountValue : 0) + feeZec).toFixed(8)}{" "}
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

        {walletHasPassword ? (
          <div className="space-y-1">
            <Input
              type="password"
              label="Wallet Password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder={
                walletUnlocked
                  ? "Optional — already unlocked"
                  : "Enter your wallet password"
              }
              className="bg-white/50 focus:bg-white transition-all"
            />
            {walletUnlocked && (
              <p className="text-xs text-gray-500 pl-1">
                Wallet is unlocked. Leave blank to use your unlock session, or re-enter to confirm.
              </p>
            )}
          </div>
        ) : (
          <p className="text-xs text-gray-500">
            This wallet has no encryption password (same as CLI with empty password).
          </p>
        )}

        {keystoneEnabled && keystonePrepared && (
          <div className="space-y-3">
            {keystoneSummary && (
              <p className="text-sm text-gray-600 whitespace-pre-wrap rounded-lg bg-amber-50/60 border border-amber-200/60 p-3">
                {keystoneSummary}
              </p>
            )}
            {keystoneUrFrames.length > 0 && (
              <div className="space-y-2">
                <p className="text-xs font-semibold text-gray-500 uppercase tracking-widest pl-1">
                  PCZT for Keystone {keystoneUrFrames.length > 1 ? `(frame 1 of ${keystoneUrFrames.length})` : ""}
                </p>
                <div className="flex justify-center p-4 bg-white rounded-xl border border-gray-200">
                  <QRCode value={keystoneUrFrames[0]} size={180} level="M" />
                </div>
                <textarea
                  readOnly
                  value={keystoneUrFrames.join("\n")}
                  rows={4}
                  className="w-full rounded-lg border border-amber-200 bg-amber-50/50 px-3 py-2 text-xs font-mono"
                />
                <Button
                  variant="outline"
                  size="sm"
                  onClick={async () => {
                    try {
                      await navigator.clipboard.writeText(keystoneUrFrames.join("\n"));
                      toast.success("PCZT data copied");
                    } catch {
                      toast.error("Could not copy to clipboard");
                    }
                  }}
                >
                  Copy PCZT data
                </Button>
              </div>
            )}
            <textarea
              value={keystoneSignedInput}
              onChange={(e) => setKeystoneSignedInput(e.target.value)}
              placeholder="Paste signed PCZT hex or UR frames from Keystone"
              rows={4}
              className="w-full rounded-lg border border-gray-200 px-3 py-2 text-xs font-mono"
            />
          </div>
        )}

        {/* FUTURE: group file co-sign export UI */}

        <div className="flex gap-3 pt-2">
          <Button
            variant="ghost"
            onClick={() => {
              setShowReview(false);
              resetKeystoneSendState();
            }}
            disabled={isSending || isBroadcasting}
            className="flex-1"
          >
            Back
          </Button>
          {keystoneEnabled ? (
            keystonePrepared ? (
              <Button
                onClick={() => void handleKeystoneBroadcast()}
                disabled={isBroadcasting || !keystoneSignedInput.trim()}
                className="flex-1 rounded-xl shadow-lg shadow-primary/20 gap-2"
              >
                <Shield size={18} />
                {isBroadcasting ? "Broadcasting…" : "Broadcast signed tx"}
              </Button>
            ) : (
              <Button
                onClick={() => void handleKeystonePrepare()}
                disabled={isSending || sendPasswordRequired}
                className="flex-1 rounded-xl shadow-lg shadow-primary/20 gap-2"
              >
                <Shield size={18} />
                {isSending ? "Building…" : "Prepare for Keystone"}
              </Button>
            )
          ) : (
            <Button
              onClick={handleSend}
              disabled={isSending || sendPasswordRequired}
              className="flex-1 rounded-xl shadow-lg shadow-primary/20"
            >
              {isSending ? "Sending..." : "Confirm Send"}
            </Button>
          )}
        </div>
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
            <Tooltip content="Total Orchard shielded balance after sync. Use max after fee.">
              <span
                className="text-gray-700 cursor-pointer hover:text-primary transition-colors uppercase"
                onClick={handleMax}
              >
                {balance.toLocaleString()} {tokenSymbol}
              </span>
            </Tooltip>
          </div>
        </div>

        <div className="relative">
          <input
            type="text"
            inputMode="decimal"
            autoComplete="off"
            value={amount}
            onChange={(e) => handleAmountChange(e.target.value)}
            onFocus={handleAmountFocus}
            onBlur={handleAmountBlur}
            onWheel={preventAmountWheel}
            placeholder={DEFAULT_AMOUNT}
            className="w-full text-4xl bg-transparent border-none focus:ring-0 text-center font-bold text-gray-900 placeholder:text-gray-300 p-2 drop-shadow-sm [appearance:textfield]"
          />
          <span className="absolute right-8 top-1/2 -translate-y-1/2 text-primary font-bold text-xl pointer-events-none uppercase">
            {tokenSymbol}
          </span>
        </div>
      </div>

      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <label className="text-sm font-medium text-gray-700 ml-1">
            Recipient Address
          </label>
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={() => setPickContactOpen((open) => !open)}
              className="text-xs text-primary hover:underline flex items-center gap-1"
            >
              <User size={14} />
              Choose from contacts
            </button>
            {address.trim() && (
              <button
                type="button"
                onClick={() => setSaveContactOpen(true)}
                className="text-xs text-gray-500 hover:text-primary hover:underline"
              >
                Save to contacts
              </button>
            )}
          </div>
        </div>
        <div className="relative group">
          <textarea
            value={address}
            onChange={(e) => setAddress(e.target.value)}
            placeholder="u1… (mainnet Orchard unified address)"
            rows={2}
            spellCheck={false}
            autoComplete="off"
            className="flex w-full min-h-[3.5rem] rounded-lg border border-gray-200/60 bg-white/60 px-3 py-2.5 pr-12 text-sm ring-offset-transparent placeholder:text-gray-400 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2 transition-all hover:border-primary/30 hover:bg-white/80 backdrop-blur-sm font-mono resize-y"
          />
          <button
            type="button"
            title="QR scan coming soon"
            className="absolute right-4 top-4 text-gray-400 hover:text-primary transition-colors"
            onClick={() => toast("QR scanning is not available yet. Paste the address manually.")}
          >
            <Scanner size={20} />
          </button>
        </div>

        {pickContactOpen && (
          <div className="rounded-xl border border-gray-200/60 bg-white/80 p-3 space-y-1 max-h-48 overflow-y-auto">
            <div className="flex items-center justify-between mb-2">
              <p className="text-xs font-semibold text-gray-500 uppercase tracking-wide">
                Choose contact
              </p>
              <button
                type="button"
                onClick={() => setPickContactOpen(false)}
                className="text-xs text-gray-500 hover:text-gray-800"
              >
                Close
              </button>
            </div>
            {contacts.length === 0 ? (
              <p className="text-sm text-gray-500 py-2">No contacts yet. Add some from Contacts.</p>
            ) : (
              contacts.map((c) => (
                <button
                  key={c.name}
                  type="button"
                  onClick={() => {
                    setAddress(c.address);
                    setPickContactOpen(false);
                  }}
                  className="w-full text-left px-3 py-2 rounded-lg hover:bg-gray-100 transition-colors"
                >
                  <p className="font-medium text-gray-900">{c.name}</p>
                  <p className="text-xs font-mono text-gray-500 truncate">{c.address}</p>
                </button>
              ))
            )}
          </div>
        )}
      </div>

      <Modal
        isOpen={saveContactOpen}
        onClose={() => !saveContactSaving && setSaveContactOpen(false)}
        title="Save to contacts"
      >
        <div className="space-y-4">
          <Input
            label="Name"
            placeholder="e.g. Exchange"
            value={saveContactName}
            onChange={(e) => setSaveContactName(e.target.value)}
          />
          <Input
            label="Notes (optional)"
            placeholder="e.g. Withdrawal"
            value={saveContactNotes}
            onChange={(e) => setSaveContactNotes(e.target.value)}
          />
          <div className="flex gap-2 justify-end pt-2">
            <Button variant="outline" onClick={() => setSaveContactOpen(false)} disabled={saveContactSaving}>
              Cancel
            </Button>
            <Button onClick={handleSaveToContacts} disabled={saveContactSaving}>
              {saveContactSaving ? "Saving…" : "Save"}
            </Button>
          </div>
        </div>
      </Modal>

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

      <p className="text-sm text-gray-500 ml-1">Network fee (ZIP-317 × 4)</p>

      {keystoneEnabled && (
        <div className="flex items-start gap-3 p-3 rounded-xl border border-amber-200/60 bg-amber-50/40">
          <Shield size={18} className="text-amber-700 mt-0.5 shrink-0" />
          <span className="text-sm text-gray-700">
            <span className="font-medium">Keystone signing enabled (mainnet)</span>
            <span className="block text-xs text-gray-500 mt-1">
              Sends require approval on your Keystone device (Zcash mainnet). Pair in Settings → Keystone.
            </span>
          </span>
        </div>
      )}

      {/* FUTURE: group file co-sign — checkbox + broadcast panel
      <label>...</label>
      <div className="rounded-xl border...">...</div>
      */}

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
        <Tooltip
          content={
            keystoneEnabled
              ? "Review and prepare a Keystone PCZT transaction"
              : "Review transaction details before sending"
          }
        >
          <Button
            size="lg"
            disabled={!isValid}
            onClick={handleProceedToReview}
            className="flex-1 rounded-xl py-4 text-lg shadow-xl bg-black text-white shadow-primary/30 -hover:translate-y-[2px] transition-all duration-300 disabled:cursor-not-allowed disabled:transform-none disabled:shadow-none"
          >
            {keystoneEnabled ? "Review & prepare" : "Review"}
          </Button>
        </Tooltip>
      </div>
    </div>
  );
}
