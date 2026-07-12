import { useState, useEffect, useRef } from "react";
import toast from "react-hot-toast";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { logger } from "../utils/logger";
import { formatErrorForDisplay } from "../utils/errors";
import { Button } from "../components/Button";
import { Input } from "../components/Input";
import { Textarea, textareaClassName } from "../components/Textarea";
import { Modal } from "../components/Modal";
import { Tooltip } from "../components/Tooltip";
import {
  InfoCircle,
  CheckCircle,
  User,
  Shield,
  Refresh,
} from "@solar-icons/react";
import QRCode from "react-qr-code";
import { useWalletStore } from "../store/walletStore";
import { useTokenStore } from "../store/tokenStore";
import { walletApi } from "../lib/api";
import { isWalletReadyForSend } from "../lib/syncHelpers";
import type { AddressBookEntry, IronwoodDesktopStatusResponse } from "../lib/types";
import { TransactionIdDetail } from "./TxExplorerLink";

type BackendSendProgress = {
  stage: string;
  percent: number;
  message: string;
};

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
const UA_MIN_LEN = 78;
const UA_MAX_LEN = 256;

function normalizeUnifiedAddress(value: string): string {
  return value.replace(/\s+/g, "");
}

function isUnifiedZcashAddress(value: string): boolean {
  const normalized = normalizeUnifiedAddress(value);
  if (!normalized.startsWith("u1") && !normalized.startsWith("utest1")) return false;
  return normalized.length >= UA_MIN_LEN && normalized.length <= UA_MAX_LEN;
}

function ironwoodSendBlockReason(
  status: IronwoodDesktopStatusResponse | null,
): string | null {
  if (!status?.ironwood_active) return null;
  if (status.ironwood_send_enabled) return null;
  const blocked = status.blockers.find((blocker) =>
    /orchard notes remain|no unspent shielded/i.test(blocker),
  );
  return blocked ?? null;
}

async function loadIronwoodSendBlockReason(): Promise<string | null> {
  try {
    const res = await walletApi.getIronwoodStatus();
    return ironwoodSendBlockReason(res.data);
  } catch {
    return null;
  }
}

function recipientNetworkError(
  recipient: string,
  network: "mainnet" | "testnet" | null,
): string | null {
  const normalized = normalizeUnifiedAddress(recipient);
  if (!normalized) return null;
  if (!isUnifiedZcashAddress(normalized)) {
    return "Address must be an Orchard unified address (u1… or utest1…) at least 78 characters.";
  }
  if (network === "testnet" && !normalized.startsWith("utest1")) {
    return "This wallet is on testnet. Recipients must use utest1… addresses. Switch network in Settings → Wallets & Accounts if you meant mainnet.";
  }
  if (network === "mainnet" && !normalized.startsWith("u1")) {
    return "This wallet is on mainnet. Recipients must use u1… addresses. Use Settings → Wallets & Accounts to switch to testnet for utest1… recipients.";
  }
  return null;
}

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

type SendProgressState = {
  percent: number;
  label: string;
  detail: string;
};

/** Timed stages while the backend builds/proves (no streamed proving %). */
const SEND_PROGRESS_STAGES: Array<{ atMs: number; percent: number; label: string; detail: string }> = [
  {
    atMs: 0,
    percent: 6,
    label: "Preparing transaction",
    detail: "Checking notes, fees, and sync status…",
  },
  {
    atMs: 3_000,
    percent: 18,
    label: "Selecting Orchard notes",
    detail: "Gathering shielded notes for this send…",
  },
  {
    atMs: 10_000,
    percent: 35,
    label: "Building zero-knowledge proof",
    detail: "First send can take several minutes — keep this window open.",
  },
  {
    atMs: 45_000,
    percent: 55,
    label: "Proving in progress",
    detail: "Still working on the Orchard proof — this is normal.",
  },
  {
    atMs: 120_000,
    percent: 72,
    label: "Proving taking longer",
    detail: "Large circuits are slow the first time; please wait.",
  },
  {
    atMs: 240_000,
    percent: 85,
    label: "Almost finished proving",
    detail: "Finalizing the shielded circuit…",
  },
];

function sendProgressAtElapsed(elapsedMs: number): SendProgressState {
  let stage = SEND_PROGRESS_STAGES[0];
  for (const candidate of SEND_PROGRESS_STAGES) {
    if (elapsedMs >= candidate.atMs) stage = candidate;
  }
  // Ease toward ~92% so the bar never looks stuck at a flat number forever.
  const softCap = 92;
  const overshoot = Math.max(0, elapsedMs - stage.atMs);
  const eased = Math.min(
    softCap,
    stage.percent + Math.floor(Math.log10(10 + overshoot / 1000) * 4),
  );
  return { percent: eased, label: stage.label, detail: stage.detail };
}

function SendProgressPanel({ progress }: { progress: SendProgressState }) {
  return (
    <div
      className="rounded-2xl border border-primary/30 bg-primary/5 dark:bg-primary/10 p-4 space-y-3"
      role="status"
      aria-live="polite"
    >
      <div className="flex items-start gap-3">
        <Refresh size={20} className="animate-spin text-primary shrink-0 mt-0.5" />
        <div className="min-w-0 flex-1">
          <div className="flex items-center justify-between gap-2">
            <p className="font-semibold text-gray-900 dark:text-gray-100">{progress.label}</p>
            <span className="text-sm font-bold tabular-nums text-primary">{progress.percent}%</span>
          </div>
          <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">{progress.detail}</p>
          <p className="text-xs text-gray-500 dark:text-gray-500 mt-2">
            Please don’t close or navigate away while this finishes.
          </p>
        </div>
      </div>
      <div
        className="h-2 rounded-full bg-black/10 dark:bg-white/10 overflow-hidden"
        role="progressbar"
        aria-valuenow={progress.percent}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-label={`Send progress ${progress.percent} percent`}
      >
        <div
          className="h-full rounded-full bg-primary transition-all duration-700"
          style={{ width: `${progress.percent}%` }}
        />
      </div>
    </div>
  );
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
  const [sendProgress, setSendProgress] = useState<SendProgressState | null>(null);
  const sendProgressTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const sendProgressUnlistenRef = useRef<UnlistenFn | null>(null);
  const gotBackendProgressRef = useRef(false);
  const backendProveStartedAtRef = useRef<number | null>(null);
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
  const [activeNetwork, setActiveNetwork] = useState<"mainnet" | "testnet" | null>(null);
  const [ironwoodSendBlocked, setIronwoodSendBlocked] = useState<string | null>(null);
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
    let cancelled = false;
    void loadIronwoodSendBlockReason().then((reason) => {
      if (!cancelled) setIronwoodSendBlocked(reason);
    });
    return () => {
      cancelled = true;
    };
  }, [showReview, activeNetwork]);

  useEffect(() => {
    walletApi.getWalletStatus().then((r) => {
      setWalletUnlocked(r.data?.unlocked ?? false);
      setWalletHasPassword(r.data?.has_password ?? false);
    }).catch(() => {});
    walletApi.getNetworkWalletStatus().then((r) => {
      const network = r.data?.network;
      if (network === "testnet" || network === "mainnet") {
        setActiveNetwork(network);
      }
    }).catch(() => {});
    walletApi.getKeystoneStatus().then((r) => {
      const onMainnet = r.data?.network !== "testnet";
      setKeystoneEnabled((r.data?.enabled ?? false) && onMainnet);
      if (r.data?.network === "testnet" || r.data?.network === "mainnet") {
        setActiveNetwork(r.data.network);
      }
    }).catch(() => {});
  }, [showReview]);

  const amountValue = parseAmount(amount);
  const normalizedAddress = normalizeUnifiedAddress(address);
  const addressValidationError = recipientNetworkError(address, activeNetwork)
    ?? (normalizedAddress && !isUnifiedZcashAddress(normalizedAddress)
      ? "Address must be an Orchard unified address (u1… or utest1…) at least 78 characters."
      : null);

  const reviewDisabledReason = (() => {
    if (!normalizedAddress) return "Enter a recipient address.";
    if (addressValidationError) return addressValidationError;
    if (!Number.isFinite(amountValue) || amountValue <= 0) {
      return "Enter an amount greater than 0.";
    }
    if (amountValue + feeZec > balance) {
      return `Insufficient balance: need ${(amountValue + feeZec).toFixed(8)} ${tokenSymbol} including fee (${balance.toFixed(8)} ${tokenSymbol} available). Sync if balance looks wrong.`;
    }
    return null;
  })();

  const isValid = reviewDisabledReason === null;

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
    const networkError = recipientNetworkError(address, activeNetwork);
    if (networkError) {
      toast.error(networkError);
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
    const networkError = recipientNetworkError(address, activeNetwork);
    if (networkError) {
      toast.error(networkError);
      return;
    }
    const ironwoodBlock = await loadIronwoodSendBlockReason();
    if (ironwoodBlock) {
      setIronwoodSendBlocked(ironwoodBlock);
      toast.error(ironwoodBlock);
      return;
    }
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

  const stopSendProgress = () => {
    if (sendProgressTimerRef.current) {
      clearInterval(sendProgressTimerRef.current);
      sendProgressTimerRef.current = null;
    }
    if (sendProgressUnlistenRef.current) {
      void sendProgressUnlistenRef.current();
      sendProgressUnlistenRef.current = null;
    }
    gotBackendProgressRef.current = false;
    backendProveStartedAtRef.current = null;
    setSendProgress(null);
  };

  const applyProgress = (next: SendProgressState, toastId: string) => {
    setSendProgress(next);
    toast.loading(`${next.label} (${next.percent}%)`, { id: toastId });
  };

  const startSendProgress = (toastId: string) => {
    stopSendProgress();
    const startedAt = Date.now();
    gotBackendProgressRef.current = false;
    backendProveStartedAtRef.current = null;

    void listen<BackendSendProgress>("send-progress", (event) => {
      const payload = event.payload;
      if (!payload || typeof payload.percent !== "number") return;
      gotBackendProgressRef.current = true;
      if (/proof|prove/i.test(payload.stage) && backendProveStartedAtRef.current == null) {
        backendProveStartedAtRef.current = Date.now();
      }
      applyProgress(
        {
          percent: Math.min(100, Math.max(0, payload.percent)),
          label: payload.stage || "Sending",
          detail: payload.message || "Working…",
        },
        toastId
      );
    })
      .then((unlisten) => {
        sendProgressUnlistenRef.current = unlisten;
      })
      .catch(() => {
        // Not running under Tauri / event API unavailable — timer fallback only.
      });

    const tick = () => {
      // Prefer real Tauri stages once they arrive. During the long proving wait,
      // gently advance between the last backend percent and 90% so the UI doesn't freeze.
      if (gotBackendProgressRef.current) {
        setSendProgress((prev) => {
          if (!prev || prev.percent >= 90 || prev.percent >= 100) return prev;
          if (backendProveStartedAtRef.current == null) return prev;
          const elapsed = Date.now() - backendProveStartedAtRef.current;
          const eased = Math.min(
            90,
            Math.max(prev.percent, 58 + Math.floor(Math.log10(10 + elapsed / 1000) * 8))
          );
          if (eased <= prev.percent) return prev;
          const next = {
            ...prev,
            percent: eased,
            detail:
              eased < 90
                ? "Still generating the zero-knowledge proof — keep this window open."
                : prev.detail,
          };
          toast.loading(`${next.label} (${next.percent}%)`, { id: toastId });
          return next;
        });
        return;
      }
      applyProgress(sendProgressAtElapsed(Date.now() - startedAt), toastId);
    };
    tick();
    sendProgressTimerRef.current = setInterval(tick, 1000);
  };

  useEffect(() => {
    return () => {
      if (sendProgressTimerRef.current) {
        clearInterval(sendProgressTimerRef.current);
      }
      if (sendProgressUnlistenRef.current) {
        void sendProgressUnlistenRef.current();
      }
    };
  }, []);

  const handleKeystonePrepare = async () => {
    setIsSending(true);
    const toastId = toast.loading("Preparing Keystone PCZT…");
    startSendProgress(toastId);
    const trimmedPassword = password.trim();
    const recipient = normalizeUnifiedAddress(address);
    try {
      const { data } = await walletApi.keystonePrepareSend({
        recipient,
        amount: amountValue,
        memo: memo || undefined,
        password: trimmedPassword || undefined,
      });
      stopSendProgress();
      if (!data.success) {
        toast.error(data.message ?? "Failed to prepare Keystone transaction", { id: toastId });
        return;
      }
      setKeystonePrepared(true);
      setKeystoneSummary(data.summary ?? "Transaction prepared");
      setKeystoneUrFrames(data.ur_frames ?? (data.pczt_hex ? [data.pczt_hex] : []));
      toast.success("Scan QR on Keystone, sign, then paste signed data below", { id: toastId });
    } catch (error: unknown) {
      stopSendProgress();
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
    setSendProgress({
      percent: 90,
      label: "Broadcasting transaction",
      detail: "Submitting the signed transaction to the network…",
    });
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
      stopSendProgress();
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
      stopSendProgress();
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
    const sendToast = toast.loading("Preparing shielded transaction…");
    startSendProgress(sendToast);
    const trimmedPassword = password.trim();
    const recipient = normalizeUnifiedAddress(address);
    try {
      const { data: sent } = await walletApi.sendTransaction({
        recipient,
        amount: amountValue,
        memo: memo || undefined,
        password: trimmedPassword || undefined,
      });
      setSendProgress({
        percent: 100,
        label: "Transaction sent",
        detail: "Broadcast complete.",
      });
      const txidMsg = sent.txid ? ` TXID: ${sent.txid}` : "";
      toast.success(`Transaction sent successfully!${txidMsg}`, { id: sendToast });
      stopSendProgress();
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
      stopSendProgress();
      logger.error("Transaction send failed", error as Error, {
        recipient,
        amount: amountValue,
        hasMemo: !!memo
      });
      toast.error(
        formatErrorForDisplay(
          error,
          "Send failed. Check sync status, balance, and whether Ironwood is blocking Orchard sends on this network."
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
        <div className="w-20 h-20 rounded-full bg-green-100 dark:bg-green-900/30 text-green-500 flex items-center justify-center mx-auto mb-6 animate-scale-up">
          <CheckCircle size={48} weight="Bold" />
        </div>
        <h3 className="text-2xl font-bold text-gray-900 dark:text-gray-100">Transaction Sent!</h3>
        <p className="text-gray-500 dark:text-gray-400">Your funds are on their way.</p>
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
          <h3 className="text-xl font-bold text-gray-900 dark:text-gray-100">
            {keystoneEnabled
              ? keystonePrepared
                ? "Sign on Keystone"
                : "Prepare for Keystone"
              : "Confirm Transfer"}
          </h3>
          <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
            {keystoneEnabled
              ? keystonePrepared
                ? "Scan the QR on your Keystone device, sign, then paste the signed PCZT below."
                : "Review details, then build a PCZT for Keystone to sign."
              : "Please review the transaction details carefully."}
          </p>
        </div>

        {ironwoodSendBlocked && (
          <div className="flex items-start gap-3 p-3 rounded-xl border border-amber-200/80 dark:border-amber-800/50 bg-amber-50/70 dark:bg-amber-900/20">
            <Shield size={18} className="text-amber-700 dark:text-amber-400 mt-0.5 shrink-0" />
            <p className="text-sm text-amber-900/90 dark:text-amber-200">{ironwoodSendBlocked}</p>
          </div>
        )}

        <div className="bg-gray-50 dark:bg-gray-800/50 rounded-2xl p-6 space-y-4 border border-gray-100 dark:border-gray-700/50">
          <div className="flex justify-between items-center">
            <span className="text-gray-500 dark:text-gray-400 text-sm">Amount</span>
            <span className="font-bold text-lg text-gray-900 dark:text-gray-100">
              {amount} {tokenSymbol}
            </span>
          </div>
          <div className="flex justify-between items-center">
            <span className="text-gray-500 dark:text-gray-400 text-sm">Fee</span>
            <span className="text-gray-900 dark:text-gray-100 font-medium text-sm">
              {feeZec.toFixed(8)} {tokenSymbol} (priority ×4)
            </span>
          </div>
          <div className="h-px bg-gray-200 dark:bg-gray-700 my-2" />
          <div className="flex justify-between items-center text-base">
            <span className="font-bold text-gray-900 dark:text-gray-100">Total</span>
            <span className="font-bold text-primary-700">
              {((Number.isFinite(amountValue) ? amountValue : 0) + feeZec).toFixed(8)}{" "}
              {tokenSymbol}
            </span>
          </div>
        </div>

        <div className="space-y-1">
          <p className="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-widest pl-1">
            Recipient
          </p>
          <div className="bg-gray-50 dark:bg-gray-800/50 p-3 rounded-xl border border-gray-100 dark:border-gray-700/50 font-mono text-xs text-gray-600 dark:text-gray-300 break-all leading-relaxed">
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
              <p className="text-xs text-gray-500 dark:text-gray-400 pl-1">
                Wallet is unlocked. Leave blank to use your unlock session, or re-enter to confirm.
              </p>
            )}
          </div>
        ) : (
          <p className="text-xs text-gray-500 dark:text-gray-400">
            This wallet has no encryption password (same as CLI with empty password).
          </p>
        )}

        {keystoneEnabled && keystonePrepared && (
          <div className="space-y-3">
            {keystoneSummary && (
              <p className="text-sm text-gray-600 dark:text-gray-300 whitespace-pre-wrap rounded-lg bg-amber-50/60 dark:bg-amber-900/20 border border-amber-200/60 dark:border-amber-800/50 p-3">
                {keystoneSummary}
              </p>
            )}
            {keystoneUrFrames.length > 0 && (
              <div className="space-y-2">
                <p className="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-widest pl-1">
                  PCZT for Keystone {keystoneUrFrames.length > 1 ? `(frame 1 of ${keystoneUrFrames.length})` : ""}
                </p>
                <div className="flex justify-center p-4 bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700">
                  <QRCode value={keystoneUrFrames[0]} size={180} level="M" />
                </div>
                <textarea
                  readOnly
                  value={keystoneUrFrames.join("\n")}
                  rows={4}
                  className={textareaClassName + " border-amber-200 dark:border-amber-800/50 bg-amber-50/50 dark:bg-amber-900/20 text-xs font-mono"}
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
            <Textarea
              value={keystoneSignedInput}
              onChange={(e) => setKeystoneSignedInput(e.target.value)}
              placeholder="Paste signed PCZT hex or UR frames from Keystone"
              rows={4}
              className="text-xs font-mono"
            />
          </div>
        )}

        {/* FUTURE: group file co-sign export UI */}

        {sendProgress && <SendProgressPanel progress={sendProgress} />}

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
                {isBroadcasting
                  ? sendProgress
                    ? `${sendProgress.percent}%`
                    : "Broadcasting…"
                  : "Broadcast signed tx"}
              </Button>
            ) : (
              <Button
                onClick={() => void handleKeystonePrepare()}
                disabled={isSending || sendPasswordRequired || Boolean(ironwoodSendBlocked)}
                className="flex-1 rounded-xl shadow-lg shadow-primary/20 gap-2"
              >
                <Shield size={18} />
                {isSending
                  ? sendProgress
                    ? `${sendProgress.percent}%`
                    : "Building…"
                  : "Prepare for Keystone"}
              </Button>
            )
          ) : (
            <Button
              onClick={handleSend}
              disabled={isSending || sendPasswordRequired || Boolean(ironwoodSendBlocked)}
              className="flex-1 rounded-xl shadow-lg shadow-primary/20"
            >
              {isSending
                ? sendProgress
                  ? `Sending ${sendProgress.percent}%`
                  : "Sending…"
                : "Confirm Send"}
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
          <label className="text-sm font-medium text-gray-500 dark:text-gray-400 group-focus-within:text-primary transition-colors">
            Amount
          </label>
          <div className="text-xs text-gray-400 dark:text-gray-500 font-medium">
            Available:{" "}
            <Tooltip content="Total Orchard shielded balance after sync. Use max after fee.">
              <span
                className="text-gray-700 dark:text-gray-300 cursor-pointer hover:text-primary transition-colors uppercase"
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
            className="w-full text-4xl bg-transparent border-none focus:ring-0 text-center font-bold text-gray-900 dark:text-gray-100 placeholder:text-gray-300 dark:placeholder:text-gray-600 p-2 drop-shadow-sm [appearance:textfield]"
          />
          <span className="absolute right-8 top-1/2 -translate-y-1/2 text-primary font-bold text-xl pointer-events-none uppercase">
            {tokenSymbol}
          </span>
        </div>
      </div>

      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300 ml-1">
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
                className="text-xs text-gray-500 dark:text-gray-400 hover:text-primary hover:underline"
              >
                Save to contacts
              </button>
            )}
          </div>
        </div>
        <div className="relative group">
          <Textarea
            value={address}
            onChange={(e) => setAddress(e.target.value)}
            placeholder="u1… or utest1… Orchard unified address"
            rows={2}
            spellCheck={false}
            autoComplete="off"
            className="min-h-[3.5rem] font-mono"
          />
        </div>
        {activeNetwork && (
          <p className="text-xs text-gray-500 dark:text-gray-400 ml-1">
            Active network:{" "}
            <span className="font-medium text-gray-700 dark:text-gray-300">
              {activeNetwork === "testnet" ? "Testnet (utest1…)" : "Mainnet (u1…)"}
            </span>
          </p>
        )}
        {addressValidationError && (
          <p className="text-sm text-amber-800 dark:text-amber-300 ml-1">{addressValidationError}</p>
        )}

        {pickContactOpen && (
          <div className="rounded-xl border border-gray-200/60 dark:border-gray-700/60 bg-white/80 dark:bg-gray-800/80 p-3 space-y-1 max-h-48 overflow-y-auto">
            <div className="flex items-center justify-between mb-2">
              <p className="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wide">
                Choose contact
              </p>
              <button
                type="button"
                onClick={() => setPickContactOpen(false)}
                className="text-xs text-gray-500 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200"
              >
                Close
              </button>
            </div>
            {contacts.length === 0 ? (
              <p className="text-sm text-gray-500 dark:text-gray-400 py-2">No contacts yet. Add some from Contacts.</p>
            ) : (
              contacts.map((c) => (
                <button
                  key={c.name}
                  type="button"
                  onClick={() => {
                    setAddress(c.address);
                    setPickContactOpen(false);
                  }}
                  className="w-full text-left px-3 py-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700/50 transition-colors"
                >
                  <p className="font-medium text-gray-900 dark:text-gray-100">{c.name}</p>
                  <p className="text-xs font-mono text-gray-500 dark:text-gray-400 truncate">{c.address}</p>
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
          className="text-sm text-gray-500 dark:text-gray-400 hover:text-primary flex items-center gap-1.5 transition-colors ml-1"
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

      <p className="text-sm text-gray-500 dark:text-gray-400 ml-1">Network fee (ZIP-317 × 4)</p>

      {ironwoodSendBlocked && (
        <div className="flex items-start gap-3 p-3 rounded-xl border border-amber-200/80 dark:border-amber-800/50 bg-amber-50/70 dark:bg-amber-900/20">
          <Shield size={18} className="text-amber-700 dark:text-amber-400 mt-0.5 shrink-0" />
          <p className="text-sm text-amber-900/90 dark:text-amber-200">{ironwoodSendBlocked}</p>
        </div>
      )}

      {keystoneEnabled && (
        <div className="flex items-start gap-3 p-3 rounded-xl border border-amber-200/60 dark:border-amber-800/40 bg-amber-50/40 dark:bg-amber-900/15">
          <Shield size={18} className="text-amber-700 dark:text-amber-400 mt-0.5 shrink-0" />
          <span className="text-sm text-gray-700 dark:text-gray-300">
            <span className="font-medium">Keystone signing enabled (mainnet)</span>
            <span className="block text-xs text-gray-500 dark:text-gray-400 mt-1">
              Sends require approval on your Keystone device (Zcash mainnet). Pair in Settings → Keystone.
            </span>
          </span>
        </div>
      )}

      {/* FUTURE: group file co-sign — checkbox + broadcast panel
      <label>...</label>
      <div className="rounded-xl border...">...</div>
      */}

      <div className="flex flex-col gap-2 pt-4">
        {reviewDisabledReason && (
          <p className="text-sm text-amber-800 text-center px-2">{reviewDisabledReason}</p>
        )}
        {!reviewDisabledReason && ironwoodSendBlocked && (
          <p className="text-sm text-amber-800 text-center px-2">
            You can open Review, but Orchard-only sends are blocked while Ironwood (NU6.3) is
            active. Migrate Orchard notes on the Ironwood tab (Split → Plan → Migrate → Broadcast),
            then send from Ironwood balance.
          </p>
        )}
        <div className="flex gap-3">
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
            reviewDisabledReason
              ? reviewDisabledReason
              : ironwoodSendBlocked
                ? "Open review — migrate on the Ironwood tab before Orchard-only sends work on this network"
                : keystoneEnabled
                  ? "Review and prepare a Keystone PCZT transaction"
                  : "Review transaction details before sending"
          }
        >
          <span className="inline-flex flex-1">
            <Button
              size="lg"
              disabled={!isValid}
              onClick={() => void handleProceedToReview()}
              className="w-full"
            >
              {keystoneEnabled ? "Review & prepare" : "Review"}
            </Button>
          </span>
        </Tooltip>
        </div>
      </div>
    </div>
  );
}
